// e2e_player_journey_tests.rs — Teste E2E da Jornada Completa do Jogador
// Simula a experiência completa: Registro -> Login -> Depósito PIX -> Entrada no Lobby -> Split Pot Odd Cent -> Saque PIX.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::AuthManager;
use poker_engine::lobby::LobbyManager;

fn make_test_state() -> AppState {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://unused:unused@localhost:5432/unused".to_string()
    });
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&url)
        .unwrap();

    let auth_mgr = AuthManager::new("e2e-jwt-secret-key-32-chars-long");

    AppState {
        db,
        auth: Arc::new(RwLock::new(auth_mgr)),
        lobby: Arc::new(RwLock::new(LobbyManager::new())),
        tournaments: Arc::new(RwLock::new(std::collections::HashMap::new())),
        active_tables: Arc::new(RwLock::new(std::collections::HashMap::new())),
        jwt_secret: "e2e-jwt-secret-key-32-chars-long".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
    }
}

#[tokio::test]
async fn test_e2e_full_player_journey_deposit_lobby_splitpot_withdraw() {
    let state = make_test_state();
    let app = build_router(state);

    // 1. Registro de Jogador
    let reg_payload = serde_json::json!({
        "username": "e2e_player",
        "email": "e2e@poker.com",
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

    // 2. Login e Emissão de Token JWT
    let login_payload = serde_json::json!({
        "username": "e2e_player",
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
    let access_token = login_json["tokens"]["access_token"].as_str().unwrap();

    // 3. Depósito PIX
    let deposit_payload = serde_json::json!({
        "amount": 100.00
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
    let payouts = poker_engine::utils::dividir_pote_empatado(10.05, &vencedores, &assentos);
    assert_eq!(*payouts.get("alice").unwrap(), 5.03);
    assert_eq!(*payouts.get("bob").unwrap(), 5.02);

    // 6. Saque PIX
    let withdraw_payload = serde_json::json!({
        "amount": 50.00,
        "pix_key_type": "CPF",
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
    assert_eq!(res.status(), StatusCode::OK);
}
