// payments_tests.rs — Testes de Integração do Módulo de Pagamentos e Depósitos PIX Instantâneos

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::{AuthManager, LoginRequest, RegisterRequest};
use poker_engine::lobby::LobbyManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

fn make_test_state() -> AppState {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://unused:unused@localhost:5432/unused".to_string()
    });
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&url)
        .unwrap();

    let auth_mgr = AuthManager::new("payments-jwt-secret-key-32chars");

    AppState {
        db,
        auth: Arc::new(RwLock::new(auth_mgr)),
        lobby: Arc::new(RwLock::new(LobbyManager::new())),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "payments-jwt-secret-key-32chars".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
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

// ─── 1. Teste de Geração de Depósito PIX ───
#[tokio::test]
async fn test_pix_deposit_generation_success() {
    let state = make_test_state();
    let valid_token = get_valid_access_token(&state, "user_deposit_1").await;

    let app = build_router(state);
    let payload = serde_json::json!({
        "amount": 5000
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
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 10_000).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body_json.get("tx_id").is_some());
    assert_eq!(body_json["amount"], 5000);
    assert!(body_json["pix_copy_paste"].as_str().unwrap().contains("BR.GOV.BCB.PIX"));
    assert!(body_json["qr_code_base64"].as_str().unwrap().contains("data:image"));
}

// ─── 2. Teste de Rejeição de Depósito Negativo ou Zero ───
#[tokio::test]
async fn test_pix_deposit_invalid_amount_returns_400() {
    let state = make_test_state();
    let valid_token = get_valid_access_token(&state, "user_deposit_2").await;

    let app = build_router(state);
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
}

// ─── 3. Teste de Recebimento de Webhook PIX Válido ───
#[tokio::test]
async fn test_pix_webhook_success() {
    let state = make_test_state();
    let app = build_router(state);

    let webhook_payload = serde_json::json!({
        "tx_id": "tx_dep_123456",
        "external_tx_id": "asaas_pay_123456",
        "amount": 5000,
        "status": "APPROVED"
    })
    .to_string();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/webhooks/pix")
        .header("X-Webhook-Secret", "poker-pix-webhook-secret-key-32chars")
        .header("Content-Type", "application/json")
        .body(Body::from(webhook_payload))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 10_000).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["status"], "PROCESSED");
}

// ─── 4. Teste de Rejeição de Webhook com Secret Inválido ───
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

// ─── 5. Teste de Saque PIX Válido ───
#[tokio::test]
async fn test_pix_withdraw_success() {
    let state = make_test_state();
    let valid_token = get_valid_access_token(&state, "user_withdraw_1").await;

    let app = build_router(state);
    let payload = serde_json::json!({
        "amount": 10000,
        "pix_key_type": "cpf",
        "pix_key": "12345678900"
    })
    .to_string();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/payments/pix/withdraw")
        .header("Authorization", format!("Bearer {}", valid_token))
        .header("Content-Type", "application/json")
        .body(Body::from(payload))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 10_000).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.get("tx_id").is_some());
    assert_eq!(body_json["amount"], 10000);
    assert_eq!(body_json["status"], "PROCESSING");
}
