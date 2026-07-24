// red_team_simulation_tests.rs — Simulação Autônoma de Red Team Massivo (Vetores de Ataque em Larga Escala)
// Valida a resiliência de produção da API Axum contra 1.000 ataques concorrentes em paralelo de Força Bruta, 1.000 JWT Tamperings e 1.000 WS Injections.

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
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
    }
}

// ─── Vetor 1: Simulação Massiva de Força Bruta (1.000 Requisições Concorrentes em Lote) ───
#[tokio::test]
async fn test_red_team_attack_vector_1_brute_force_auth() {
    let state = make_test_state();
    const TOTAL_ATTACKS: usize = 1_000;
    const CONCURRENT_TASKS: usize = 50;
    const ATTACKS_PER_TASK: usize = TOTAL_ATTACKS / CONCURRENT_TASKS;

    let mut handles = Vec::with_capacity(CONCURRENT_TASKS);

    for task_idx in 0..CONCURRENT_TASKS {
        let state_clone = state.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..ATTACKS_PER_TASK {
                let app = build_router(state_clone.clone());
                let attack_payload = serde_json::json!({
                    "email": format!("hacker_{}_{}@malicious.com", task_idx, i),
                    "password": format!("wrong_pass_{}_{}", task_idx, i)
                })
                .to_string();

                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/api/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(attack_payload))
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                let status = response.status();
                assert!(
                    status == StatusCode::UNAUTHORIZED
                        || status == StatusCode::BAD_REQUEST
                        || status == StatusCode::INTERNAL_SERVER_ERROR,
                    "Rejeição de força bruta falhou no lote {} índice {}: {}",
                    task_idx,
                    i,
                    status
                );
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

// ─── Vetor 2: Ataque Massivo de Forja de Token JWT (1.000 Mutadores & Signatures Spoofing) ───
#[tokio::test]
async fn test_red_team_attack_vector_2_jwt_tampering() {
    let state = make_test_state();
    const TOTAL_TAMPERED_TOKENS: usize = 1_000;
    const CONCURRENT_TASKS: usize = 50;
    const TOKENS_PER_TASK: usize = TOTAL_TAMPERED_TOKENS / CONCURRENT_TASKS;

    let mut handles = Vec::with_capacity(CONCURRENT_TASKS);

    for task_idx in 0..CONCURRENT_TASKS {
        let state_clone = state.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..TOKENS_PER_TASK {
                let app = build_router(state_clone.clone());
                let fake_signature = format!("fake_sig_{}_{}", task_idx, i);
                let forged_token = format!(
                    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbl9oYWNrZXIiLCJleHAiOjE5OTk5OTk5OTl9.{}",
                    fake_signature
                );

                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/api/lobby/join")
                    .header("Authorization", format!("Bearer {}", forged_token))
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"table_id":"table_1"}"#))
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

// ─── Vetor 3: Injeção Massiva de WebSocket (1.000 Handshakes Corrompidos) ───
#[tokio::test]
async fn test_red_team_attack_vector_3_websocket_fuzz_injection() {
    let state = make_test_state();
    const TOTAL_WS_INJECTIONS: usize = 1_000;
    const CONCURRENT_TASKS: usize = 50;
    const INJECTIONS_PER_TASK: usize = TOTAL_WS_INJECTIONS / CONCURRENT_TASKS;

    let mut handles = Vec::with_capacity(CONCURRENT_TASKS);

    for task_idx in 0..CONCURRENT_TASKS {
        let state_clone = state.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..INJECTIONS_PER_TASK {
                let app = build_router(state_clone.clone());
                let malformed_uri = format!(
                    "/ws/game/table-1?token=malicious_token_{}_{}&sqli=%27%20OR%201%3D1%20%2D%2D",
                    task_idx, i
                );

                let request = Request::builder()
                    .method(Method::GET)
                    .uri(malformed_uri)
                    .body(Body::empty())
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

// ─── Vetor 4: Verificação dos Endpoints de Métricas Prometheus & Health Security ───
#[tokio::test]
async fn test_red_team_prometheus_and_security_health_endpoint() {
    let state = make_test_state();

    let app = build_router(state.clone());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/health/security")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

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
