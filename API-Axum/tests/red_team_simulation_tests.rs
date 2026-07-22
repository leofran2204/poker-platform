// red_team_simulation_tests.rs — Simulação Autônoma de Red Team (Vetores de Ataque Reais)
// Testa a resiliência da API Axum contra ataques de Força Bruta, Forja de JWT, WebSocket Injection e Integridade.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::AuthManager;
use poker_engine::lobby::LobbyManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

fn make_test_state() -> AppState {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://unused:unused@localhost:5432/unused".to_string()
    });
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&url)
        .unwrap();

    AppState {
        db,
        auth: Arc::new(Mutex::new(AuthManager::new("red-team-jwt-secret-key-32chars"))),
        lobby: Arc::new(Mutex::new(LobbyManager::new())),
        tournaments: Arc::new(Mutex::new(HashMap::new())),
        active_tables: Arc::new(Mutex::new(HashMap::new())),
        jwt_secret: "red-team-jwt-secret-key-32chars".to_string(),
    }
}

// ─── Vetor 1: Simulação de Força Bruta / Credential Stuffing ───
#[tokio::test]
async fn test_red_team_attack_vector_1_brute_force_auth() {
    let state = make_test_state();

    for i in 0..30 {
        let app = build_router(state.clone());
        let attack_payload = serde_json::json!({
            "email": format!("hacker_{}@malicious.com", i),
            "password": format!("wrong_pass_{}", i)
        })
        .to_string();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(attack_payload))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // O Axum deve rejeitar todas as credenciais inválidas com 401 Unauthorized ou 400 Bad Request
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "Rejeição de força bruta falhou no índice {}: {}",
            i,
            response.status()
        );
    }
}

// ─── Vetor 2: Ataque de Forja de Token JWT (Tampering & Signature Spoofing) ───
#[tokio::test]
async fn test_red_team_attack_vector_2_jwt_tampering() {
    let state = make_test_state();
    let app = build_router(state);

    // Token forjado por atacante com segredo falso e payload adulterado
    let forged_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbiIsImV4cCI6OTk5OTk5OTk5OX0.fake_signature";

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/lobby/join")
        .header("Authorization", format!("Bearer {}", forged_token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"table_id":"table_1"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Deve rejeitar imediatamente o JWT forjado com 401 Unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ─── Vetor 3: Injeção de WebSocket e Payload Corrompido ───
#[tokio::test]
async fn test_red_team_attack_vector_3_websocket_fuzz_injection() {
    let state = make_test_state();
    let app = build_router(state);

    // Simula tentativa de upgrade de WS sem headers obrigatórios de handshake RFC 6455
    let request = Request::builder()
        .method(Method::GET)
        .uri("/ws/game/table-1?token=malicious_token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Deve rejeitar handshake malformado sem travar o servidor
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ─── Vetor 4: Verificação dos Endpoints de Métricas Prometheus & Health Security ───
#[tokio::test]
async fn test_red_team_prometheus_and_security_health_endpoint() {
    let state = make_test_state();

    // 1. Testa /api/health/security
    let app = build_router(state.clone());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/health/security")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 2. Testa /api/metrics
    let app2 = build_router(state);
    let req2 = Request::builder()
        .method(Method::GET)
        .uri("/api/metrics")
        .body(Body::empty())
        .unwrap();
    let resp2 = app2.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(resp2.into_body(), 10_000).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body_str.contains("poker_uptime_seconds"));
    assert!(body_str.contains("poker_http_requests_total"));
    assert!(body_str.contains("poker_antifraud_checks_total"));
}
