// payments_tests.rs — Testes de Integração do Módulo de Pagamentos e Depósitos PIX Instantâneos

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use hmac::{Hmac, Mac};
use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::{AuthManager, LoginRequest, RegisterRequest};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

fn make_test_state() -> AppState {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://unused:unused@localhost:5432/unused".to_string());
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&url)
        .unwrap();

    let auth_mgr = AuthManager::new("payments-jwt-secret-key-32chars");

    AppState {
        db,
        auth: Arc::new(RwLock::new(auth_mgr)),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "payments-jwt-secret-key-32chars".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: None,
        ws_tickets: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        require_email_verification: false,
        presence: poker_api::presence::PresenceTracker::new(),
    }
}

async fn get_valid_access_token(state: &AppState, username: &str) -> String {
    let mut auth_mgr = state.auth.write().await;
    let _ = auth_mgr.register_user(&RegisterRequest {
        username: username.to_string(),
        email: format!("{}@test.com", username),
        password: "Password123".to_string(),
    });

    let token_pair = auth_mgr
        .login(&LoginRequest {
            username: username.to_string(),
            password: "Password123".to_string(),
            mfa_code: None,
        })
        .unwrap();

    token_pair.access_token
}

async fn make_persistent_state(username: &str) -> (AppState, String, String) {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is required for wallet contract tests");
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("wallet test database must be reachable");
    let mut auth = AuthManager::new("payments-jwt-secret-key-32chars");
    let user = auth
        .register_user(&RegisterRequest {
            username: username.to_string(),
            email: format!("{username}@test.invalid"),
            password: "Password123".to_string(),
        })
        .expect("test user must be valid");
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, status, balance, mfa_enabled, created_at) \
         VALUES ($1::uuid, $2, $3, $4, 'player', 'active', 0, false, $5)",
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(i64::try_from(user.created_at).expect("test timestamp fits i64"))
    .execute(&db)
    .await
    .expect("test user must persist");
    let state = AppState {
        db,
        auth: Arc::new(RwLock::new(auth)),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "payments-jwt-secret-key-32chars".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: None,
        ws_tickets: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        require_email_verification: false,
        presence: poker_api::presence::PresenceTracker::new(),
    };
    let token = get_valid_access_token(&state, username).await;
    (state, user.id, token)
}

async fn cleanup_persistent_user(state: &AppState, user_id: &str) {
    let _ = sqlx::query("DELETE FROM outbox_events WHERE payload ->> 'user_id' = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;
    sqlx::query("DELETE FROM users WHERE id = $1::uuid")
        .bind(user_id)
        .execute(&state.db)
        .await
        .expect("test user cleanup must succeed");
}

fn webhook_signature(body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(b"poker-pix-webhook-secret-key-32chars")
        .expect("test HMAC key is valid");
    mac.update(body);
    let signature = mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256={signature}")
}

fn unique_username(prefix: &str) -> String {
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    // AuthManager accepts user names with at most 30 characters. Keeping the
    // random suffix short also makes the PostgreSQL integration fixtures
    // independent when they run in parallel.
    let suffix_len = 30usize.saturating_sub(prefix.len() + 1).min(suffix.len());
    format!("{prefix}_{}", &suffix[..suffix_len])
}

// ─── Rejeição de Depósito Negativo ou Zero ───
#[tokio::test]
#[ignore = "Requires PostgreSQL authorization state — run with DATABASE_URL"]
async fn test_pix_deposit_invalid_amount_returns_400() {
    let username = unique_username("deposit_zero");
    let (state, user_id, valid_token) = make_persistent_state(&username).await;

    let app = build_router(state.clone());
    let payload = serde_json::json!({
        "amount": 0
    })
    .to_string();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/payments/pix/deposit")
        .header("Authorization", format!("Bearer {}", valid_token))
        .header("Content-Type", "application/json")
        .body(Body::from(payload))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    cleanup_persistent_user(&state, &user_id).await;
}

// ─── Rejeição de Webhook com Assinatura Inválida ───
#[tokio::test]
async fn test_pix_webhook_invalid_secret_returns_401() {
    let state = make_test_state();
    let app = build_router(state);

    let webhook_payload = serde_json::json!({
        "tx_id": "tx_dep_fake",
        "amount": 50000,
        "status": "APPROVED"
    })
    .to_string();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/webhooks/pix")
        .header("X-Webhook-Secret", "wrong-secret-key")
        .header("Content-Type", "application/json")
        .body(Body::from(webhook_payload))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "Requires PostgreSQL ledger — run with DATABASE_URL"]
async fn wallet_deposit_webhook_is_atomic_and_idempotent() {
    let username = unique_username("wallet_dep");
    let (state, user_id, token) = make_persistent_state(&username).await;
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/payments/pix/deposit")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"amount":5000}"#))
        .unwrap();
    let response = poker_api::build_router(state.clone())
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 10_000)
        .await
        .unwrap();
    let deposit: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let tx_id = deposit["tx_id"].as_str().unwrap().to_string();
    let external_tx_id: String = sqlx::query_scalar(
        "SELECT external_tx_id FROM wallet_transactions WHERE idempotency_key = $1",
    )
    .bind(&tx_id)
    .fetch_one(&state.db)
    .await
    .unwrap();

    let webhook = serde_json::json!({
        "tx_id": tx_id,
        "external_tx_id": external_tx_id,
        "amount": 5000,
        "status": "COMPLETED"
    })
    .to_string();
    for expected_status in ["COMPLETED", "IGNORED"] {
        let signature = webhook_signature(webhook.as_bytes());
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/webhooks/pix")
            .header("X-Signature", signature)
            .header("Content-Type", "application/json")
            .body(Body::from(webhook.clone()))
            .unwrap();
        let response = poker_api::build_router(state.clone())
            .oneshot(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 10_000)
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"],
            expected_status
        );
    }
    let balance: i64 = sqlx::query_scalar("SELECT balance FROM users WHERE id = $1::uuid")
        .bind(&user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(balance, 5_000);
    cleanup_persistent_user(&state, &user_id).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL ledger — run with DATABASE_URL"]
async fn concurrent_withdrawals_cannot_reserve_the_same_balance_twice() {
    let username = unique_username("wallet_wdr");
    let (state, user_id, token) = make_persistent_state(&username).await;
    sqlx::query("UPDATE users SET balance = 10000 WHERE id = $1::uuid")
        .bind(&user_id)
        .execute(&state.db)
        .await
        .unwrap();
    let request = || {
        Request::builder()
            .method(Method::POST)
            .uri("/api/payments/pix/withdraw")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"amount":8000,"pix_key_type":"cpf","pix_key":"12345678900"}"#,
            ))
            .unwrap()
    };
    let (first, second) = tokio::join!(
        poker_api::build_router(state.clone()).oneshot(request()),
        poker_api::build_router(state.clone()).oneshot(request())
    );
    let statuses = [first.unwrap().status(), second.unwrap().status()];
    assert!(statuses.contains(&StatusCode::ACCEPTED));
    assert!(statuses.contains(&StatusCode::BAD_REQUEST));
    let balance: i64 = sqlx::query_scalar("SELECT balance FROM users WHERE id = $1::uuid")
        .bind(&user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(balance, 2_000);
    let reservations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wallet_transactions WHERE user_id = $1::uuid AND transaction_type = 'WITHDRAW'",
    )
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(reservations, 1);
    cleanup_persistent_user(&state, &user_id).await;
}
