// e2e_player_journey_tests.rs — Teste E2E da Jornada Completa do Jogador
// Simula a experiência completa: Registro -> Login -> Depósito PIX -> Entrada no Lobby -> Split Pot Odd Cent -> Saque PIX.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use hmac::{Hmac, Mac};
use http_body_util::BodyExt;
use sha2::Sha256;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::AuthManager;

fn make_test_state() -> AppState {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://unused:unused@localhost:5432/unused".to_string());
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&url)
        .unwrap();

    let auth_mgr = AuthManager::new("e2e-jwt-secret-key-32-chars-long");

    AppState {
        db,
        auth: Arc::new(RwLock::new(auth_mgr)),
        tournaments: Arc::new(RwLock::new(std::collections::HashMap::new())),
        active_tables: Arc::new(RwLock::new(std::collections::HashMap::new())),
        jwt_secret: "e2e-jwt-secret-key-32-chars-long".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: None,
        ws_tickets: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        require_email_verification: false,
        presence: poker_api::presence::PresenceTracker::new(),
    }
}

type HmacSha256 = Hmac<Sha256>;

fn webhook_signature(body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(b"poker-pix-webhook-secret-key-32chars")
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

async fn cleanup_e2e_user(state: &AppState, user_id: &str) {
    let _ = sqlx::query("DELETE FROM outbox_events WHERE payload ->> 'user_id' = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;
    sqlx::query("DELETE FROM users WHERE id = $1::uuid")
        .bind(user_id)
        .execute(&state.db)
        .await
        .expect("E2E user cleanup must succeed");
}

#[tokio::test]
#[ignore]
async fn test_e2e_full_player_journey_deposit_lobby_splitpot_withdraw() {
    let state = make_test_state();
    let app = build_router(state.clone());
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let username = format!("e2e_{}", &uid[..12]);
    let email = format!("{username}@example.com");

    // 1. Registro de Jogador
    let reg_payload = serde_json::json!({
        "username": username.clone(),
        "email": email.clone(),
        "password": "Password123!"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&reg_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let user_id: String = sqlx::query_scalar("SELECT id::text FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db)
        .await
        .expect("registered E2E user must be persisted");
    let initial_balance: i64 =
        sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1::uuid")
            .bind(&user_id)
            .fetch_one(&state.db)
            .await
            .expect("registered E2E user must have a balance");

    // 2. Login e Emissão de Token JWT
    let login_payload = serde_json::json!({
        "email": email,
        "password": "Password123!"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let login_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let access_token = login_json["token"].as_str().unwrap().to_string();

    // 3. Depósito PIX
    let deposit_payload = serde_json::json!({
        "amount": 10_000
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/payments/pix/deposit")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&deposit_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let deposit_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let tx_id = deposit_json["tx_id"]
        .as_str()
        .expect("deposit response must contain tx_id")
        .to_string();
    let external_tx_id: String = sqlx::query_scalar(
        "SELECT external_tx_id FROM wallet_transactions WHERE idempotency_key = $1",
    )
    .bind(&tx_id)
    .fetch_one(&state.db)
    .await
    .expect("deposit must persist its provider transaction id");

    let webhook = serde_json::json!({
        "tx_id": tx_id,
        "external_tx_id": external_tx_id,
        "amount": 10_000,
        "status": "COMPLETED"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/webhooks/pix")
        .header("X-Signature", webhook_signature(webhook.as_bytes()))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(webhook))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let balance: i64 = sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1::uuid")
        .bind(&user_id)
        .fetch_one(&state.db)
        .await
        .expect("settled deposit must update the wallet balance");
    assert_eq!(balance, initial_balance + 10_000);

    // 4. Listagem do Lobby
    let req = Request::builder()
        .method("GET")
        .uri("/api/lobby/tables")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Teste da Regra do Centavo Ímpar no Motor (TDA Rule 68 Split Pot)
    let vencedores = vec!["alice".to_string(), "bob".to_string()];
    let assentos = vec!["alice".to_string(), "bob".to_string()];
    let payouts = poker_engine::utils::dividir_pote_empatado(1005, &vencedores, &assentos);
    assert_eq!(*payouts.get("alice").unwrap(), 503);
    assert_eq!(*payouts.get("bob").unwrap(), 502);

    // 6. Saque PIX
    let withdraw_payload = serde_json::json!({
        "amount": 5_000,
        "pix_key_type": "cpf",
        "pix_key": "123.456.789-00"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/payments/pix/withdraw")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&withdraw_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    cleanup_e2e_user(&state, &user_id).await;
}
