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
use hmac::{Hmac, Mac};
use http_body_util::BodyExt;
use poker_engine::auth::AuthManager;
use poker_engine::tournament_engine::TournamentConfig;
use poker_engine::tournament_engine::TournamentSpeed;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tower::ServiceExt;

use poker_api::state::AppState;
use poker_api::tournament_store::TournamentStore;

// ─── Test helpers ───

/// Builds a test AppState with an in-memory AuthManager.
/// The `db` field is a placeholder — DB-dependent tests are marked #[ignore].
fn make_test_state() -> AppState {
    // We can't create a real PgPool without a DATABASE_URL.
    // For non-DB tests, we use a pool that will never be queried.
    // If a test needs the DB, it's marked #[ignore] and requires DATABASE_URL.
    let db = create_placeholder_pool();

    AppState {
        db,
        auth: Arc::new(RwLock::new(AuthManager::new("test-secret-key-for-tests"))),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "test-secret-key-for-tests".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: None,
        ws_tickets: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        require_email_verification: false,
        presence: poker_api::presence::PresenceTracker::new(),
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

/// Remove o único usuário criado por um contrato PostgreSQL. Os testes de
/// integração podem rodar repetidamente no banco local sem acumular dados.
async fn cleanup_contract_user(state: &AppState, username: &str) {
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&state.db)
        .await
        .expect("Falha ao remover usuário criado pelo teste de contrato");
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
#[ignore = "Readiness checks PostgreSQL — set DATABASE_URL to run"]
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
async fn presence_online_starts_at_zero() {
    let app = poker_api::build_router(make_test_state());
    let (status, body) = send_request(app, Method::GET, "/api/presence/online", None).await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["online_count"], 0);
    assert_eq!(json["ttl_seconds"], 90);
}

#[tokio::test]
#[ignore = "Lobby tables are PostgreSQL-authoritative — set DATABASE_URL to run"]
async fn test_lobby_tables_returns_200_json_list() {
    // GET /api/lobby/tables is backed by PostgreSQL.
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let (status, body) = send_request(app, Method::GET, "/api/lobby/tables", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(serde_json::from_str::<Vec<serde_json::Value>>(&body).is_ok());
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
async fn test_websocket_ticket_without_token_returns_401() {
    let app = poker_api::build_router(make_test_state());
    let (status, _) = send_request(
        app,
        Method::POST,
        "/api/lobby/tables/00000000-0000-0000-0000-000000000000/ws-ticket",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_table_routes_without_token_return_401() {
    let create_body = serde_json::json!({
        "name": "R$ 1/2",
        "small_blind": 100,
        "big_blind": 200,
        "min_buy_in": 4_000,
        "max_buy_in": 20_000,
        "max_players": 6,
        "rake_basis_points": 500,
        "rake_cap": 10_000
    })
    .to_string();
    let create_app = poker_api::build_router(make_test_state());
    let (create_status, _) = send_request(
        create_app,
        Method::POST,
        "/api/admin/tables",
        Some(create_body),
    )
    .await;
    assert_eq!(create_status, StatusCode::UNAUTHORIZED);

    let update_app = poker_api::build_router(make_test_state());
    let (update_status, _) = send_request(
        update_app,
        Method::PATCH,
        "/api/admin/tables/00000000-0000-0000-0000-000000000000/status",
        Some(serde_json::json!({"status": "PAUSED"}).to_string()),
    )
    .await;
    assert_eq!(update_status, StatusCode::UNAUTHORIZED);
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
        let mut tournaments = state.tournaments.write().await;
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
            guaranteed_prize: 0,
            is_freeroll: false,
            rebuy_cost: 0,
            rebuy_chips: 0,
            rebuy_max_count: 0,
            rebuy_stack_threshold: 0,
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
    let app = poker_api::build_router(state.clone());

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

    let (status2, body2) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/login",
        Some(login_body),
    )
    .await;
    assert_eq!(status2, StatusCode::OK, "Login failed: {body2}");

    let login_json: Value = serde_json::from_str(&body2).expect("Login response is not JSON");
    assert!(
        login_json["token"].is_string(),
        "Login response should contain token: {body2}"
    );

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_auth_survives_in_memory_cache_reset() {
    use serde_json::Value;

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("restart_contract_{uid}@example.com");
    let username = format!("restart_{}", &uid[..12]);
    let password = "StrongPass123!";

    let register_body = serde_json::json!({
        "email": email,
        "password": password,
        "username": username,
    })
    .to_string();
    let (register_status, register_response) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register_body),
    )
    .await;
    assert_eq!(register_status, StatusCode::OK, "{register_response}");
    let refresh_token = serde_json::from_str::<Value>(&register_response).unwrap()["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Simulates a process restart: no account is left in AuthManager memory.
    *state.auth.write().await = AuthManager::new("test-secret-key-for-tests");
    let login_body = serde_json::json!({ "email": email, "password": password }).to_string();
    let (login_status, login_response) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/login",
        Some(login_body),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "{login_response}");

    *state.auth.write().await = AuthManager::new("test-secret-key-for-tests");
    let refresh_body = serde_json::json!({ "refresh_token": refresh_token }).to_string();
    let (refresh_status, refresh_response) =
        send_request(app, Method::POST, "/api/auth/refresh", Some(refresh_body)).await;
    assert_eq!(refresh_status, StatusCode::OK, "{refresh_response}");

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL token-version migration — set DATABASE_URL to run"]
async fn test_contract_security_change_revokes_an_already_issued_access_token() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("token_version_{uid}@example.com");
    let username = format!("tokver_{}", &uid[..12]);

    let register_body = serde_json::json!({
        "email": email,
        "password": "StrongPass123!",
        "username": username,
    })
    .to_string();
    let (register_status, register_response) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register_body),
    )
    .await;
    assert_eq!(register_status, StatusCode::OK, "{register_response}");
    let token = serde_json::from_str::<serde_json::Value>(&register_response).unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // Migration 009 increments token_version for a security-sensitive account
    // change, making every access token issued under the old version unusable
    // in every API replica.
    sqlx::query("UPDATE users SET mfa_enabled = true WHERE username = $1")
        .bind(&username)
        .execute(&state.db)
        .await
        .unwrap();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/hand-history/00000000-0000-0000-0000-000000000000")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_register_duplicate_returns_409() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());

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

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_login_invalid_credentials_returns_401() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());

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

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_hand_history_not_found_returns_404() {
    // This test needs a valid token to pass the auth middleware,
    // then queries DB for a non-existent hand → 404
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());

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

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_lobby_join_with_valid_token() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());

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
    assert_eq!(
        status,
        StatusCode::OK,
        "Register failed in join test: {body}"
    );
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Join a table — authentication and buy-in validation pass, then the
    // unknown table is rejected without debiting the wallet.
    let join_body = serde_json::json!({
        "table_id": "00000000-0000-0000-0000-000000000000",
        "buy_in": 10_000
    })
    .to_string();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/lobby/join")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(join_body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Auth passes, but the requested table does not exist.
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
async fn test_websocket_upgrade_success() {
    let state = make_test_state();
    let app = poker_api::build_router(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/ws/game/table-1?ticket=test")
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

type HmacSha1 = Hmac<Sha1>;

fn test_epoch_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock must be after the Unix epoch")
        .as_secs() as i64
}

fn totp_code_for_counter(secret: &[u8], counter: u64) -> String {
    let mut mac = HmacSha1::new_from_slice(secret).expect("test TOTP key is valid");
    mac.update(&counter.to_be_bytes());
    let result = mac.finalize().into_bytes();
    let offset = usize::from(result[result.len() - 1] & 0x0f);
    let binary = (u32::from(result[offset] & 0x7f) << 24)
        | (u32::from(result[offset + 1]) << 16)
        | (u32::from(result[offset + 2]) << 8)
        | u32::from(result[offset + 3]);
    format!("{:06}", binary % 1_000_000)
}

fn current_totp_code(secret: &[u8]) -> String {
    totp_code_for_counter(secret, test_epoch_now() as u64 / 30)
}

fn definitely_invalid_totp_code(secret: &[u8]) -> String {
    let counter = test_epoch_now() as u64 / 30;
    let valid_window: Vec<String> = (-2i64..=2)
        .map(|offset| totp_code_for_counter(secret, (counter as i64 + offset) as u64))
        .collect();
    (0..1_000_000)
        .map(|candidate| format!("{candidate:06}"))
        .find(|candidate| !valid_window.contains(candidate))
        .expect("a code outside the five-step TOTP window must exist")
}

fn mfa_challenge_hash(challenge: &str) -> String {
    format!("{:x}", Sha256::digest(challenge.as_bytes()))
}

async fn request_mfa_challenge(app: axum::Router, email: &str, password: &str) -> String {
    let body = serde_json::json!({ "email": email, "password": password }).to_string();
    let (status, response) = send_request(app, Method::POST, "/api/auth/login", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["mfa_required"], true);
    response["mfa_challenge"]
        .as_str()
        .expect("MFA login must return a challenge")
        .to_string()
}

#[tokio::test]
#[ignore = "Requires PostgreSQL atomic login state — set DATABASE_URL to run"]
async fn test_contract_concurrent_login_failures_are_atomic() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("atomic_login_{uid}@example.com");
    let username = format!("atomic_{}", &uid[..12]);
    let password = "CorrectPass123!";
    let register = serde_json::json!({
        "email": email,
        "password": password,
        "username": username,
    })
    .to_string();
    let (status, response) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{response}");

    let wrong_body = serde_json::json!({ "email": email, "password": "WrongPass123!" }).to_string();
    let attempt = || {
        send_request(
            app.clone(),
            Method::POST,
            "/api/auth/login",
            Some(wrong_body.clone()),
        )
    };
    let (one, two, three, four, five) =
        tokio::join!(attempt(), attempt(), attempt(), attempt(), attempt());
    for (status, _) in [one, two, three, four, five] {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    let (attempts, locked_until): (i32, Option<i64>) =
        sqlx::query_as("SELECT failed_login_attempts, locked_until FROM users WHERE username = $1")
            .bind(&username)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(attempts, poker_engine::auth::MAX_LOGIN_ATTEMPTS as i32);
    assert!(locked_until.is_some_and(|until| until > test_epoch_now()));

    let correct_body = serde_json::json!({ "email": email, "password": password }).to_string();
    let (locked_status, _) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/login",
        Some(correct_body.clone()),
    )
    .await;
    assert_eq!(locked_status, StatusCode::UNAUTHORIZED);

    sqlx::query("UPDATE users SET locked_until = $1 WHERE username = $2")
        .bind(test_epoch_now() - 1)
        .bind(&username)
        .execute(&state.db)
        .await
        .unwrap();
    let (unlocked_status, response) =
        send_request(app, Method::POST, "/api/auth/login", Some(correct_body)).await;
    assert_eq!(unlocked_status, StatusCode::OK, "{response}");
    let attempts_after_success: i32 =
        sqlx::query_scalar("SELECT failed_login_attempts FROM users WHERE username = $1")
            .bind(&username)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(attempts_after_success, 0);

    cleanup_contract_user(&state, &username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL MFA challenges — set DATABASE_URL to run"]
async fn test_contract_mfa_challenge_lifecycle_is_durable_and_single_use() {
    const SECRET_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
    const SECRET_BYTES: &[u8] = b"12345678901234567890";

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("mfa_contract_{uid}@example.com");
    let username = format!("mfa_{}", &uid[..12]);
    let password = "StrongPass123!";
    let register = serde_json::json!({
        "email": email,
        "password": password,
        "username": username,
    })
    .to_string();
    let (status, response) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        Some(register),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    let user_id: String = sqlx::query_scalar("SELECT id::text FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db)
        .await
        .unwrap();
    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = $1 WHERE id = $2::uuid")
        .bind(SECRET_BASE32)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .unwrap();

    let now = test_epoch_now();
    let stale_hash = format!("{:x}", Sha256::digest(uuid::Uuid::new_v4().as_bytes()));
    sqlx::query("INSERT INTO auth_mfa_challenges (user_id, token_hash, created_at, expires_at) VALUES ($1::uuid, $2, $3, $4)")
        .bind(&user_id)
        .bind(&stale_hash)
        .bind(now - 90_000)
        .bind(now - 89_700)
        .execute(&state.db)
        .await
        .unwrap();

    let challenge = request_mfa_challenge(app.clone(), &email, password).await;
    let stale_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM auth_mfa_challenges WHERE token_hash = $1")
            .bind(stale_hash)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(stale_count, 0);

    let valid_code = current_totp_code(SECRET_BYTES);
    let invalid_code = definitely_invalid_totp_code(SECRET_BYTES);
    let wrong_body =
        serde_json::json!({ "challenge": challenge, "code": invalid_code }).to_string();
    let (wrong_status, _) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/mfa/verify",
        Some(wrong_body),
    )
    .await;
    assert_eq!(wrong_status, StatusCode::UNAUTHORIZED);
    let challenge_hash = mfa_challenge_hash(&challenge);
    let attempts: i16 =
        sqlx::query_scalar("SELECT attempts FROM auth_mfa_challenges WHERE token_hash = $1")
            .bind(&challenge_hash)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(attempts, 1);

    let valid_body = serde_json::json!({ "challenge": challenge, "code": valid_code }).to_string();
    let (valid_status, valid_response) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/mfa/verify",
        Some(valid_body.clone()),
    )
    .await;
    assert_eq!(valid_status, StatusCode::OK, "{valid_response}");
    assert!(
        serde_json::from_str::<serde_json::Value>(&valid_response).unwrap()["token"].is_string()
    );
    let (replay_status, _) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/mfa/verify",
        Some(valid_body),
    )
    .await;
    assert_eq!(replay_status, StatusCode::UNAUTHORIZED);

    let expired_challenge = request_mfa_challenge(app.clone(), &email, password).await;
    let expired_hash = mfa_challenge_hash(&expired_challenge);
    sqlx::query(
        "UPDATE auth_mfa_challenges SET created_at = $1, expires_at = $2 WHERE token_hash = $3",
    )
    .bind(now - 600)
    .bind(now - 1)
    .bind(&expired_hash)
    .execute(&state.db)
    .await
    .unwrap();
    let expired_body =
        serde_json::json!({ "challenge": expired_challenge, "code": valid_code }).to_string();
    let (expired_status, _) = send_request(
        app.clone(),
        Method::POST,
        "/api/auth/mfa/verify",
        Some(expired_body),
    )
    .await;
    assert_eq!(expired_status, StatusCode::UNAUTHORIZED);

    let limited_challenge = request_mfa_challenge(app.clone(), &email, password).await;
    let limited_hash = mfa_challenge_hash(&limited_challenge);
    for _ in 0..5 {
        let body = serde_json::json!({
            "challenge": limited_challenge,
            "code": invalid_code
        })
        .to_string();
        let (status, _) = send_request(
            app.clone(),
            Method::POST,
            "/api/auth/mfa/verify",
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (attempts, consumed_at): (i16, Option<i64>) = sqlx::query_as(
        "SELECT attempts, consumed_at FROM auth_mfa_challenges WHERE token_hash = $1",
    )
    .bind(&limited_hash)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(attempts, 5);
    assert!(consumed_at.is_some());
    let after_limit_body =
        serde_json::json!({ "challenge": limited_challenge, "code": valid_code }).to_string();
    let (after_limit_status, _) = send_request(
        app,
        Method::POST,
        "/api/auth/mfa/verify",
        Some(after_limit_body),
    )
    .await;
    assert_eq!(after_limit_status, StatusCode::UNAUTHORIZED);

    cleanup_contract_user(&state, &username).await;
}
