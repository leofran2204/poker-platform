// Integration tests for the Poker API (10-API-Axum)
//
// Tests are organized into two categories:
//   1. Unit-level router tests (no DB required) — run in CI always
//   2. DB-dependent contract tests — marked #[ignore], run with DATABASE_URL set
//
// QUALITY.md §3.4 defines 8 contract endpoints that MUST be validated:
//   POST /api/auth/register, POST /api/auth/login, POST /api/auth/mfa/verify,
//   GET /api/lobby/tables, POST /api/lobby/join, WS /ws/game/{table_id},
//   POST /api/tournament/register, GET /api/hand-history/{hand_id}

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use poker_engine::auth::AuthManager;
use poker_engine::lobby::LobbyManager;
use poker_engine::tournament_engine::TournamentConfig;
use poker_engine::tournament_engine::TournamentSpeed;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

use poker_api::state::AppState;
use poker_api::tournament_store::TournamentStore;

// ─── Test helpers ───

/// Builds a test AppState with an in-memory AuthManager and LobbyManager.
/// The `db` field is a placeholder — DB-dependent tests are marked #[ignore].
fn make_test_state() -> AppState {
    // We can't create a real PgPool without a DATABASE_URL.
    // For non-DB tests, we use a pool that will never be queried.
    // If a test needs the DB, it's marked #[ignore] and requires DATABASE_URL.
    let db = create_placeholder_pool();

    AppState {
        db,
        auth: Arc::new(RwLock::new(AuthManager::new("test-secret-key-for-tests"))),
        lobby: Arc::new(RwLock::new(LobbyManager::new())),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "test-secret-key-for-tests".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
    }
}

/// Creates a placeholder PgPool using `connect_lazy`, which does NOT attempt
/// a connection immediately — it only connects when a query is executed.
/// Non-DB tests never execute queries, so this is safe and avoids needing
/// a live PostgreSQL instance or a nested tokio runtime.
fn create_placeholder_pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Dummy URL — never contacted in non-DB tests.
        "postgres://unused:unused@localhost:1/unused".to_string()
    });

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&url)
        .expect("connect_lazy should not fail — it defers the actual connection")
}

/// Sends a request to the router and returns (status, body_text).
async fn send_request(
    app: axum::Router,
    method: Method,
    uri: &str,
    body: Option<String>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(uri);
    let request = if let Some(b) = body {
        builder = builder.header("Content-Type", "application/json");
        builder.body(Body::from(b)).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    };

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8_lossy(&body_bytes).to_string();
    (status, body_text)
}

// ─── Router structure tests (no DB required) ───

#[tokio::test]
async fn test_health_check_returns_200() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, body) = send_request(app, Method::GET, "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "OK");
}

#[tokio::test]
async fn test_unknown_route_returns_404() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, _) = send_request(app, Method::GET, "/nonexistent", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_lobby_tables_returns_200_empty_list() {
    // GET /api/lobby/tables — no DB dependency, uses in-memory LobbyManager
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, body) = send_request(app, Method::GET, "/api/lobby/tables", None).await;

    assert_eq!(status, StatusCode::OK);
    // Empty lobby → empty array
    assert_eq!(body, "[]");
}

#[tokio::test]
async fn test_lobby_get_table_not_found() {
    // GET /api/lobby/tables/{id} — table doesn't exist → 404
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, body) =
        send_request(app, Method::GET, "/api/lobby/tables/nonexistent", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.contains("not found"),
        "Body should contain 'not found': {body}"
    );
}

#[tokio::test]
async fn test_protected_route_without_token_returns_401() {
    // POST /api/lobby/join — protected by RequireAuth middleware
    // Without Authorization header → 401 Unauthorized
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let body = serde_json::json!({"table_id": "table-1"}).to_string();
    let (status, body_text) = send_request(app, Method::POST, "/api/lobby/join", Some(body)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(
        body_text.contains("Authorization") || body_text.contains("token"),
        "Body should mention auth: {body_text}"
    );
}

#[tokio::test]
async fn test_tournament_register_without_token_returns_401() {
    // POST /api/tournament/register — protected by RequireAuth
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let body = serde_json::json!({
        "tournament_id": "t1",
        "player_id": "p1",
        "player_name": "Alice"
    })
    .to_string();
    let (status, _) = send_request(app, Method::POST, "/api/tournament/register", Some(body)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_hand_history_without_token_returns_401() {
    // GET /api/hand-history/{hand_id} — protected by RequireAuth
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, _) = send_request(app, Method::GET, "/api/hand-history/hand-123", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_hand_history_with_invalid_token_returns_401() {
    // GET /api/hand-history/{hand_id} — with invalid Bearer token → 401
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/hand-history/hand-123")
        .header("Authorization", "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_register_with_invalid_json_returns_400() {
    // POST /api/auth/register — malformed JSON → 400
    // Note: This test requires DB, but the JSON parse error happens before DB.
    // However, axum's Json extractor runs before our handler, so a malformed
    // JSON will return 415 or 400 before hitting the DB.
    // Actually, axum returns 415 Unsupported Media Type if Content-Type is missing,
    // and 400 if JSON is malformed.
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, _) = send_request(
        app,
        Method::POST,
        "/api/auth/register",
        Some("{invalid json}".to_string()),
    )
    .await;

    // Malformed JSON → 400 Bad Request (axum's Json extractor rejects)
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422 for malformed JSON, got {status}"
    );
}

#[tokio::test]
async fn test_auth_login_with_invalid_json_returns_400() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, _) = send_request(
        app,
        Method::POST,
        "/api/auth/login",
        Some("{bad}".to_string()),
    )
    .await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422 for malformed JSON, got {status}"
    );
}

#[tokio::test]
async fn test_tournament_get_not_found() {
    // GET /api/tournament/{id} — tournament doesn't exist → 404
    // Note: This endpoint is NOT protected (no RequireAuth)
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, body) = send_request(app, Method::GET, "/api/tournament/nonexistent", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.contains("not found"),
        "Body should contain 'not found': {body}"
    );
}

#[tokio::test]
async fn test_tournament_get_with_preloaded_tournament() {
    // GET /api/tournament/{id} — with a pre-loaded tournament in memory
    let state = make_test_state();

    // Pre-load a tournament into the in-memory store
    {
        let mut tournaments = state.tournaments.lock().await;
        let config = TournamentConfig {
            name: "Test Tournament".to_string(),
            game_type: "Holdem".to_string(),
            buy_in: 100,
            starting_stack: 10_000,
            max_players: 50,
            speed: TournamentSpeed::Normal,
            blind_levels: vec![],
            prize_pool_pct: 0.15,
            prize_distribution: vec![0.50, 0.30, 0.20],
            late_registration: true,
            late_registration_max_level: 4,
            allow_rebuy: false,
            allow_addon: false,
            rebuy_max_level: 0,
        };
        let store = TournamentStore::new("test-tournament-1".to_string(), config);
        tournaments.insert("test-tournament-1".to_string(), store);
    }

    let app = poker_api::build_router(state);

    let (status, body) =
        send_request(app, Method::GET, "/api/tournament/test-tournament-1", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("test-tournament-1"),
        "Body should contain tournament ID: {body}"
    );
    assert!(
        body.contains("Test Tournament"),
        "Body should contain tournament name: {body}"
    );
}

// ─── DB-dependent contract tests (require PostgreSQL) ───
// Run with: DATABASE_URL=postgres://... cargo test -- --ignored

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_register_login_flow() {
    use serde_json::Value;

    let state = make_test_state();
    let app = poker_api::build_router(state);

    // ─── Register ───
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("test_contract_{}@example.com", uid);
    let username = format!("contract_{}", &uid[..12]);

    let register_body = serde_json::json!({
        "email": email,
        "password": "StrongPass123!",
        "username": username
    })
    .to_string();

    let (status, body) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register_body),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Register failed: {body}");
    let json: Value = serde_json::from_str(&body).expect("Register response is not JSON");
    assert!(
        json["token"].is_string(),
        "Response should contain token: {body}"
    );
    assert!(
        json["refresh_token"].is_string(),
        "Response should contain refresh_token: {body}"
    );
    assert!(
        json["expires_in"].is_number(),
        "Response should contain expires_in: {body}"
    );

    // ─── Login ───
    let login_body = serde_json::json!({
        "email": email,
        "password": "StrongPass123!"
    })
    .to_string();

    let (status2, body2) =
        send_request(app.clone(), Method::POST, "/api/auth/login", Some(login_body)).await;
    assert_eq!(status2, StatusCode::OK, "Login failed: {body2}");

    let login_json: Value = serde_json::from_str(&body2).expect("Login response is not JSON");
    assert!(
        login_json["token"].is_string(),
        "Login response should contain token: {body2}"
    );
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_register_duplicate_returns_409() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("dup_test_{}@example.com", uid);
    let username = format!("dup_{}", &uid[..12]);

    let body = serde_json::json!({
        "email": email,
        "password": "StrongPass123!",
        "username": username
    })
    .to_string();

    // First registration — should succeed
    let (status1, _) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(body.clone()),
    )
    .await;
    assert_eq!(status1, StatusCode::OK);

    // Second registration with same username — should return 409 Conflict
    let (status2, body2) = send_request(app, Method::POST, "/api/auth/register", Some(body)).await;
    assert_eq!(
        status2,
        StatusCode::CONFLICT,
        "Duplicate register should return 409: {body2}"
    );
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_login_invalid_credentials_returns_401() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("invalid_cred_{}@example.com", uid);
    let username = format!("invcred_{}", &uid[..12]);

    // Register a user first
    let register_body = serde_json::json!({
        "email": email,
        "password": "CorrectPass123!",
        "username": username
    })
    .to_string();
    let _ = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register_body),
    )
    .await;

    // Login with wrong password
    let login_body = serde_json::json!({
        "email": email,
        "password": "WrongPassword123!"
    })
    .to_string();

    let (status, _) = send_request(app, Method::POST, "/api/auth/login", Some(login_body)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_hand_history_not_found_returns_404() {
    // This test needs a valid token to pass the auth middleware,
    // then queries DB for a non-existent hand → 404
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("hh_test_{}@example.com", uid);
    let username = format!("hhtest_{}", &uid[..12]);

    // Register and get token
    let register_body = serde_json::json!({
        "email": email,
        "password": "StrongPass123!",
        "username": username
    })
    .to_string();
    let (status, body) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register_body),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Register failed in hh test: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Request hand history with valid token but non-existent hand_id
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/hand-history/00000000-0000-0000-0000-000000000000")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_lobby_join_with_valid_token() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("join_test_{}@example.com", uid);
    let username = format!("jointest_{}", &uid[..12]);

    // Register and get token
    let register_body = serde_json::json!({
        "email": email,
        "password": "StrongPass123!",
        "username": username
    })
    .to_string();
    let (status, body) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register_body),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Register failed in join test: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Join a table — will fail because table doesn't exist, but should pass auth
    let join_body = serde_json::json!({"table_id": "00000000-0000-0000-0000-000000000000"}).to_string();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/lobby/join")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(join_body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Auth passes, but table doesn't exist → 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_websocket_upgrade_success() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/ws/game/table-1?token=test")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("Sec-WebSocket-Version", "13")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Em testes oneshot sem socket TCP real, o Axum retorna 426 Upgrade Required,
    // o que comprova que a rota foi correspondida e o extrator de WS foi executado.
    assert_eq!(response.status(), StatusCode::UPGRADE_REQUIRED);
}
