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

#[tokio::test]
#[ignore = "Requires local PostgreSQL with migration 060 — set DATABASE_URL to run"]
async fn agent_cycle_close_is_guarded_idempotent_and_separates_wallets() {
    use axum::extract::{Path, State};
    use axum::Json;
    use poker_api::handlers::estrutura::{close_agent_cycle, AgentCloseRequest};
    use poker_api::middleware::auth::{AuthUser, RequireAuth};

    let mut state = make_test_state();
    state.db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
        .await
        .unwrap();
    let agent = uuid::Uuid::new_v4();
    let player = uuid::Uuid::new_v4();
    for (id, sponsor) in [(agent, None), (player, Some(agent))] {
        let name = format!("ac_{}", &id.simple().to_string()[..12]);
        sqlx::query("INSERT INTO users (id, username, email, password_hash, agent_status, sponsored_by) VALUES ($1, $2, $3, 'test', 'active', $4)")
            .bind(id).bind(&name).bind(format!("{name}@test.invalid")).bind(sponsor)
            .execute(&state.db).await.unwrap();
    }
    let cycle: time::Date = sqlx::query_scalar("SELECT (date_trunc('month', timezone('America/Sao_Paulo', now())) - INTERVAL '3 months')::date")
        .fetch_one(&state.db).await.unwrap();
    let next = poker_api::clock::MonthsExt::checked_add_months(cycle, 1).unwrap();
    let before: (i64, i64, i64) = sqlx::query_as(
        "SELECT estrutura_points, agent_commission_balance_cents, balance_real FROM users WHERE id = $1",
    )
    .bind(agent)
    .fetch_one(&state.db)
    .await
    .unwrap();
    let mut tx = state.db.begin().await.unwrap();
    for (mode, amount, deduction, target) in [
        ("play", 1500_i64, 500_i64, 1000_i64),
        ("real", 2000, 2500, 0),
    ] {
        poker_api::estrutura::distribute_fee(&mut tx, player, amount, uuid::Uuid::new_v4(), mode)
            .await
            .unwrap();
        sqlx::query("INSERT INTO agent_monthly_cycles (agent_user_id, cycle_start, money_mode, target_ngr_cents) VALUES ($1, $2, $3, $4)")
            .bind(agent).bind(cycle).bind(mode).bind(target).execute(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO agent_ngr_adjustments (agent_user_id, cycle_start, money_mode, category, amount_cents, note) VALUES ($1, $2, $3, 'payment_cost', $4, 'Contract fixture')")
            .bind(agent).bind(cycle).bind(mode).bind(deduction).execute(&mut *tx).await.unwrap();
    }
    sqlx::query("UPDATE estrutura_ledger SET cycle_start = $2 WHERE beneficiary_user_id = $1 AND program_version = 2")
        .bind(agent).bind(cycle).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let admin = RequireAuth(AuthUser {
        user_id: agent.to_string(),
        username: "contract".into(),
        role: "admin".into(),
    });
    let request = |mode: &str, gross, deductions, target| AgentCloseRequest {
        cycle_start: cycle,
        money_mode: mode.into(),
        reconciled: true,
        expected_gross_revenue_cents: gross,
        expected_deductions_cents: deductions,
        expected_target_ngr_cents: target,
    };
    let mut ordinary = admin.clone();
    ordinary.0.role = "user".into();
    assert!(matches!(
        close_agent_cycle(
            ordinary,
            State(state.clone()),
            Path(agent.to_string()),
            Json(request("play", 1500, 500, 1000))
        )
        .await,
        Err(poker_api::error::ApiError::Forbidden(_))
    ));
    assert!(matches!(
        close_agent_cycle(
            admin.clone(),
            State(state.clone()),
            Path(agent.to_string()),
            Json(request("play", 1501, 500, 1000))
        )
        .await,
        Err(poker_api::error::ApiError::Conflict(_))
    ));
    let mut unreconciled = request("play", 1500, 500, 1000);
    unreconciled.reconciled = false;
    assert!(close_agent_cycle(
        admin.clone(),
        State(state.clone()),
        Path(agent.to_string()),
        Json(unreconciled)
    )
    .await
    .is_err());
    let (one, two) = tokio::join!(
        close_agent_cycle(
            admin.clone(),
            State(state.clone()),
            Path(agent.to_string()),
            Json(request("play", 1500, 500, 1000))
        ),
        close_agent_cycle(
            admin.clone(),
            State(state.clone()),
            Path(agent.to_string()),
            Json(request("play", 1500, 500, 1000))
        )
    );
    let one = one.unwrap().0;
    let two = two.unwrap().0;
    assert_ne!(one.already_closed, two.already_closed);
    assert_eq!(
        (one.ngr_cents, one.commission_percent, one.commission_cents),
        (1000, 35, 350)
    );
    assert_eq!(two.commission_cents, 350);
    let real = close_agent_cycle(
        admin.clone(),
        State(state.clone()),
        Path(agent.to_string()),
        Json(request("real", 2000, 2500, 0)),
    )
    .await
    .unwrap()
    .0;
    assert_eq!((real.ngr_cents, real.commission_cents), (0, 0));
    let repeated = close_agent_cycle(
        admin.clone(),
        State(state.clone()),
        Path(agent.to_string()),
        Json(request("real", 2000, 2500, 0)),
    )
    .await
    .unwrap()
    .0;
    assert!(repeated.already_closed);
    let carried: (i64, i64) = sqlx::query_as("SELECT COUNT(*)::BIGINT, SUM(amount_cents)::BIGINT FROM agent_ngr_adjustments WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = 'real'")
        .bind(agent).bind(next).fetch_one(&state.db).await.unwrap();
    assert_eq!(carried, (1, 500));
    let reference = uuid::Uuid::new_v4();
    let mut tx = state.db.begin().await.unwrap();
    poker_api::estrutura::distribute_fee(&mut tx, player, 1500, reference, "real")
        .await
        .unwrap();
    sqlx::query("UPDATE estrutura_ledger SET cycle_start = $2 WHERE source_reference_id = $1")
        .bind(reference)
        .bind(next)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut next_request = request("real", 1500, 500, 0);
    next_request.cycle_start = next;
    let next_real = close_agent_cycle(
        admin,
        State(state.clone()),
        Path(agent.to_string()),
        Json(next_request),
    )
    .await
    .unwrap()
    .0;
    assert_eq!(
        (
            next_real.ngr_cents,
            next_real.commission_percent,
            next_real.commission_cents
        ),
        (1000, 30, 300)
    );
    let after: (i64, i64, i64) = sqlx::query_as(
        "SELECT estrutura_points, agent_commission_balance_cents, balance_real FROM users WHERE id = $1",
    )
    .bind(agent)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(after, (before.0 + 350, before.1 + 300, before.2));
    let mut cleanup = state.db.begin().await.unwrap();
    for statement in [
        "DELETE FROM estrutura_ledger WHERE beneficiary_user_id = $1",
        "DELETE FROM agent_ngr_adjustments WHERE agent_user_id = $1",
        "DELETE FROM agent_monthly_cycles WHERE agent_user_id = $1",
    ] {
        sqlx::query(statement)
            .bind(agent)
            .execute(&mut *cleanup)
            .await
            .unwrap();
    }
    sqlx::query("DELETE FROM audit_logs WHERE user_id = $1")
        .bind(agent.to_string())
        .execute(&mut *cleanup)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(player)
        .execute(&mut *cleanup)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(agent)
        .execute(&mut *cleanup)
        .await
        .unwrap();
    cleanup.commit().await.unwrap();
}

#[tokio::test]
async fn academy_public_route_works_without_database_or_wallet() {
    let app = poker_api::build_router(make_test_state());
    let payload = serde_json::json!({"version":7,"scenario":"pine-allin","seed":7,"actions":[]});
    let (status, body) = send_request(
        app.clone(),
        Method::POST,
        "/api/academy/play",
        Some(payload.to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let initial: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        initial["state"]["players"][0]["cards"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        initial["state"]["players"][1]["cards"],
        serde_json::json!([])
    );
    let played = serde_json::json!({"version":7,"scenario":"pine-allin","seed":7,"actions":[{"action":"all_in","answer":"2"}]});
    let (status, body) = send_request(
        app,
        Method::POST,
        "/api/academy/play",
        Some(played.to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let final_state: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(final_state["state"]["finished"], true);
    assert_eq!(
        final_state["state"]["players"][0]["cards"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(final_state["reviews"][0]["correct"], true);
    assert!(final_state.get("balance").is_none());
}

#[tokio::test]
async fn academy_rejects_invalid_payloads_and_oversized_bodies() {
    let app = poker_api::build_router(make_test_state());
    for (payload, expected) in [
        (
            serde_json::json!({"version":1,"scenario":"pine-allin","seed":1}),
            StatusCode::BAD_REQUEST,
        ),
        (
            serde_json::json!({"version":999,"scenario":"pine-allin","seed":1}),
            StatusCode::BAD_REQUEST,
        ),
        (
            serde_json::json!({"version":7,"scenario":"missing","seed":1}),
            StatusCode::BAD_REQUEST,
        ),
        (
            serde_json::json!({"version":7,"scenario":"pine-allin","seed":-1}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            serde_json::json!({"version":7,"scenario":"pine-allin","seed":1,"balance":100}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            serde_json::json!({"version":7,"scenario":"pine-allin","seed":1,"actions":[{"action":"raise","amount":100.5}]}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            serde_json::json!({"version":7,"scenario":"primeira-mao","seed":1,"actions":[{"action":"raise","amount":101}]}),
            StatusCode::BAD_REQUEST,
        ),
        (
            serde_json::json!({"version":7,"scenario":"pine-allin","seed":1,"actions":[{"action":"call","answer":"x".repeat(17000)}]}),
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
    ] {
        let (status, body) = send_request(
            app.clone(),
            Method::POST,
            "/api/academy/play",
            Some(payload.to_string()),
        )
        .await;
        assert_eq!(status, expected, "{body}");
    }
}

#[tokio::test]
async fn academy_rate_limit_does_not_exhaust_auth_quota() {
    let mut state = make_test_state();
    state.rate_limiter = poker_api::middleware::rate_limit::RateLimiter::new(2, 60);
    let app = poker_api::build_router(state.clone());
    let payload = serde_json::json!({"version":7,"scenario":"primeira-mao","seed":1});
    for expected in [
        StatusCode::OK,
        StatusCode::OK,
        StatusCode::TOO_MANY_REQUESTS,
    ] {
        let (status, body) = send_request(
            app.clone(),
            Method::POST,
            "/api/academy/play",
            Some(payload.to_string()),
        )
        .await;
        assert_eq!(status, expected, "{body}");
    }
    // Mesmo identificador fallback das requisições sem ConnectInfo.
    let (status, _) = send_request(app, Method::POST, "/api/auth/login", Some("{}".into())).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

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
        require_invite: false,
        presence: poker_api::presence::PresenceTracker::new(),
        bots: poker_api::bots::BotFleet::for_test(),
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

/// Vídeo de aula só sai com sessão: o Caddy pergunta a esta rota antes.
#[tokio::test]
async fn test_media_check_without_session_returns_401() {
    let app = poker_api::build_router(make_test_state());
    let (status, _) = send_request(app.clone(), Method::GET, "/api/auth/check", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send_request(
        app.clone(),
        Method::GET,
        "/api/course/material?lesson_id=m0l4",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/check")
        .header("Cookie", "zt_access=token-falso")
        .body(Body::empty())
        .unwrap();
    let status = app.oneshot(request).await.unwrap().status();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
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

/// Termos 2.3: na mesa cash, outra conta com assento ativo não pode ter o mesmo
/// IP público nem a mesma sub-rede /24 de quem entra.
#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 061 — set DATABASE_URL to run"]
async fn test_contract_cash_join_refuses_same_network_at_same_table() {
    use axum::extract::connect_info::ConnectInfo;
    use std::net::SocketAddr;

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let table_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO tables (id, name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, \
             max_players, current_players, visibility, status, rake_basis_points, rake_cap, \
             rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus, poker_variant, money_mode) \
         VALUES ($1, $2, 'cash', 25, 25, 2500, 2500, 6, 0, 'public', 'OPEN', 500, 500, 150, 300, 500, \
             'holdem', 'play')",
    )
    .bind(table_id)
    .bind(format!("Guarda de rede {}", &uid[..8]))
    .execute(&state.db)
    .await
    .expect("mesa de contrato");

    let mut usernames = Vec::new();
    let mut tokens = Vec::new();
    for index in 0..3 {
        let username = format!("netguard{index}_{}", &uid[..10]);
        let body = serde_json::json!({
            "email": format!("{username}@example.com"),
            "password": "StrongPass123!",
            "username": username,
        })
        .to_string();
        let (status, body) =
            send_request(app.clone(), Method::POST, "/api/auth/register", Some(body)).await;
        assert_eq!(status, StatusCode::OK, "cadastro: {body}");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        tokens.push(json["token"].as_str().unwrap().to_string());
        usernames.push(username);
    }

    let join = |token: &str, peer: [u8; 4]| {
        let mut request = Request::builder()
            .method(Method::POST)
            .uri("/api/lobby/join")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "table_id": table_id.to_string(),
                    "buy_in": 2500,
                    "wallet_mode": "play",
                })
                .to_string(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from((peer, 443))));
        request
    };

    let first = app
        .clone()
        .oneshot(join(&tokens[0], [203, 0, 113, 10]))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    // Mesma /24, outro endereço: recusado, sem débito nem assento.
    let blocked = app
        .clone()
        .oneshot(join(&tokens[1], [203, 0, 113, 77]))
        .await
        .unwrap();
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);
    let body = blocked.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&body).contains("mesma rede"));

    // Outra rede entra normalmente; repetir a entrada da primeira conta continua idempotente.
    let other = app
        .clone()
        .oneshot(join(&tokens[2], [198, 51, 100, 10]))
        .await
        .unwrap();
    assert_eq!(other.status(), StatusCode::OK);
    let again = app
        .clone()
        .oneshot(join(&tokens[0], [203, 0, 113, 10]))
        .await
        .unwrap();
    assert_eq!(again.status(), StatusCode::OK);

    let (seats, stored_ips): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE ip_fingerprint LIKE '203.%' OR ip_fingerprint LIKE '198.%') \
         FROM cash_game_seats WHERE table_id = $1 AND status = 'ACTIVE'",
    )
    .bind(table_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(seats, 2);
    assert_eq!(stored_ips, 0, "o IP em claro nunca é gravado");

    // Ao sair, as impressões são apagadas pelo gatilho da migration 061.
    sqlx::query("UPDATE cash_game_seats SET status = 'CASHED_OUT' WHERE table_id = $1")
        .bind(table_id)
        .execute(&state.db)
        .await
        .unwrap();
    let (left_with_fingerprint,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM cash_game_seats WHERE table_id = $1 \
         AND (ip_fingerprint IS NOT NULL OR subnet_fingerprint IS NOT NULL)",
    )
    .bind(table_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(left_with_fingerprint, 0);

    for statement in [
        "DELETE FROM cash_game_ledger WHERE table_id = $1",
        "DELETE FROM cash_game_seats WHERE table_id = $1",
        "DELETE FROM table_waitlist WHERE table_id = $1",
        "DELETE FROM tables WHERE id = $1",
    ] {
        sqlx::query(statement)
            .bind(table_id)
            .execute(&state.db)
            .await
            .unwrap();
    }
    for username in &usernames {
        cleanup_contract_user(&state, username).await;
    }
}

/// Termos 2.3 nos torneios: com uma conta da mesma rede em cada uma das 3 mesas,
/// a próxima inscrição da rede é recusada; a impressão some quando o torneio acaba.
#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 067 — set DATABASE_URL to run"]
async fn test_contract_tournament_register_refuses_fourth_account_of_same_network() {
    use axum::extract::connect_info::ConnectInfo;
    use std::net::SocketAddr;

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let tournament_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO tournaments (id, name, buy_in, starting_stack, max_players, status, money_mode, prize_pool) \
         VALUES ($1, $2, 0, 1000, 27, 'registering', 'play', 0)",
    )
    .bind(tournament_id)
    .bind(format!("Guarda de rede {}", &uid[..8]))
    .execute(&state.db)
    .await
    .expect("torneio de contrato");
    {
        let mut stores = state.tournaments.write().await;
        let store = TournamentStore::with_mode_and_variant(
            tournament_id.to_string(),
            TournamentConfig {
                buy_in: 0,
                max_players: 27,
                is_freeroll: true,
                ..Default::default()
            },
            "play".into(),
            "holdem".into(),
        );
        stores.insert(tournament_id.to_string(), store);
    }

    let mut usernames = Vec::new();
    let mut tokens = Vec::new();
    for index in 0..5 {
        let username = format!("mttnet{index}_{}", &uid[..10]);
        let body = serde_json::json!({
            "email": format!("{username}@example.com"),
            "password": "StrongPass123!",
            "username": username,
        })
        .to_string();
        let (status, body) =
            send_request(app.clone(), Method::POST, "/api/auth/register", Some(body)).await;
        assert_eq!(status, StatusCode::OK, "cadastro: {body}");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        tokens.push(json["token"].as_str().unwrap().to_string());
        usernames.push(username);
    }

    let register = |token: &str, peer: [u8; 4]| {
        let mut request = Request::builder()
            .method(Method::POST)
            .uri("/api/tournament/register")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "tournament_id": tournament_id.to_string(),
                    "wallet_mode": "play",
                })
                .to_string(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from((peer, 443))));
        request
    };

    // Três contas da mesma /24 cabem, uma por mesa.
    for (index, last) in [10u8, 11, 200].into_iter().enumerate() {
        let response = app
            .clone()
            .oneshot(register(&tokens[index], [203, 0, 113, last]))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "conta {index}");
    }
    // A quarta da mesma rede é recusada, sem inscrição.
    let blocked = app
        .clone()
        .oneshot(register(&tokens[3], [203, 0, 113, 77]))
        .await
        .unwrap();
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);
    let body = blocked.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&body).contains("mesma rede"));
    // Outra rede entra normalmente.
    let other = app
        .clone()
        .oneshot(register(&tokens[4], [198, 51, 100, 10]))
        .await
        .unwrap();
    assert_eq!(other.status(), StatusCode::OK);

    let (players, fingerprints, plain): (i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(subnet_fingerprint), \
                COUNT(*) FILTER (WHERE subnet_fingerprint LIKE '203.%' OR subnet_fingerprint LIKE '198.%') \
         FROM tournament_players WHERE tournament_id = $1",
    )
    .bind(tournament_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!((players, fingerprints, plain), (4, 4, 0));
    {
        let stores = state.tournaments.read().await;
        assert_eq!(stores[&tournament_id.to_string()].state.players.len(), 4);
    }

    // Fim do torneio: o gatilho da migration 067 apaga as impressões.
    sqlx::query("UPDATE tournaments SET status = 'finished' WHERE id = $1")
        .bind(tournament_id)
        .execute(&state.db)
        .await
        .unwrap();
    let (left,): (i64,) = sqlx::query_as(
        "SELECT COUNT(subnet_fingerprint) FROM tournament_players WHERE tournament_id = $1",
    )
    .bind(tournament_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(left, 0);

    sqlx::query("DELETE FROM tournaments WHERE id = $1")
        .bind(tournament_id)
        .execute(&state.db)
        .await
        .unwrap();
    for username in &usernames {
        cleanup_contract_user(&state, username).await;
    }
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

#[tokio::test]
#[ignore = "Requires PostgreSQL — set DATABASE_URL to run"]
async fn test_contract_wallet_credit_fails_when_no_account_receives_it() {
    use poker_api::wallet::{credit_wallet, LedgerReason, WalletKind};
    const WHY: LedgerReason = LedgerReason::AdminAdjustment;

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "credit").await;

    // Conta inexistente: o crédito não pode "dar certo" sem chegar a ninguém.
    let missing = uuid::Uuid::new_v4().to_string();
    for kind in [WalletKind::PmCash, WalletKind::PmMtt, WalletKind::Real] {
        let result = credit_wallet(&state.db, &missing, 500, kind, WHY, None).await;
        assert!(
            matches!(result, Err(poker_api::error::ApiError::Internal(_))),
            "{kind:?}: {result:?}"
        );
    }

    // Dentro de uma transação, o erro desfaz o que veio antes dele.
    let mut tx = state.db.begin().await.unwrap();
    credit_wallet(&mut *tx, &user.user_id, 700, WalletKind::Real, WHY, None)
        .await
        .unwrap();
    assert!(
        credit_wallet(&mut *tx, &missing, 700, WalletKind::Real, WHY, None)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();

    // Conta existente recebe exatamente o valor.
    credit_wallet(
        &state.db,
        &user.user_id,
        500,
        WalletKind::Real,
        WHY,
        Some("teste"),
    )
    .await
    .unwrap();
    let balance_real: i64 =
        sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1::uuid")
            .bind(&user.user_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(balance_real, 500);

    // Livro único (migration 066): só a linha do crédito que valeu, com motivo,
    // referência e saldo resultante; o crédito desfeito não deixou linha.
    let rows: Vec<(i64, i64, String, Option<String>)> = sqlx::query_as(
        "SELECT amount_cents, balance_after_cents, reason, reference FROM wallet_ledger \
         WHERE user_id = $1::uuid AND wallet = 'real' ORDER BY id",
    )
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![(
            500,
            500,
            "admin_adjustment".to_string(),
            Some("teste".to_string())
        )]
    );
    let mismatches: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM wallet_ledger_mismatches WHERE user_id = $1::uuid",
    )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(mismatches, 0, "saldo = soma do livro");
    let tampered = sqlx::query("DELETE FROM wallet_ledger WHERE user_id = $1::uuid")
        .bind(&user.user_id)
        .execute(&state.db)
        .await;
    assert!(tampered.is_err(), "o livro é só de inclusão");

    cleanup_contract_user(&state, &user.username).await;
}

// ─── Rotação do refresh token (migration 063) ───

struct RefreshContractUser {
    username: String,
    user_id: String,
    email: String,
    password: &'static str,
    refresh_token: String,
}

async fn register_refresh_contract_user(
    state: &AppState,
    app: &axum::Router,
    prefix: &str,
) -> RefreshContractUser {
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("{prefix}_{uid}@example.com");
    let username = format!("{prefix}_{}", &uid[..12]);
    let password = "StrongPass123!";
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "username": username,
    })
    .to_string();
    let (status, response) =
        send_request(app.clone(), Method::POST, "/api/auth/register", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    let refresh_token = serde_json::from_str::<serde_json::Value>(&response).unwrap()
        ["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();
    let user_id: String = sqlx::query_scalar("SELECT id::text FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db)
        .await
        .unwrap();
    RefreshContractUser {
        username,
        user_id,
        email,
        password,
        refresh_token,
    }
}

async fn refresh_with(app: &axum::Router, refresh_token: &str) -> (StatusCode, serde_json::Value) {
    let body = serde_json::json!({ "refresh_token": refresh_token }).to_string();
    let (status, response) =
        send_request(app.clone(), Method::POST, "/api/auth/refresh", Some(body)).await;
    let json = serde_json::from_str(&response).unwrap_or(serde_json::Value::Null);
    (status, json)
}

fn rotated_refresh_token(response: &serde_json::Value) -> String {
    response["refresh_token"]
        .as_str()
        .expect("response carries a refresh token")
        .to_string()
}

async fn login_refresh_contract_user(app: &axum::Router, user: &RefreshContractUser) -> String {
    let body = serde_json::json!({ "email": user.email, "password": user.password }).to_string();
    let (status, response) =
        send_request(app.clone(), Method::POST, "/api/auth/login", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    rotated_refresh_token(&serde_json::from_str(&response).unwrap())
}

/// Empurra o consumo do token para antes da tolerância de corrida, como se a
/// troca tivesse acontecido há uma hora.
async fn age_refresh_token_consumption(state: &AppState, refresh_token: &str) {
    let updated = sqlx::query(
        "UPDATE auth_refresh_tokens SET consumed_at = consumed_at - 3600 \
         WHERE token_hash = $1 AND consumed_at IS NOT NULL",
    )
    .bind(poker_api::refresh_tokens::hash_token(refresh_token))
    .execute(&state.db)
    .await
    .unwrap();
    assert_eq!(updated.rows_affected(), 1, "token should be consumed");
}

fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[n as usize & 63] as char);
        }
    }
    out
}

/// Assina um JWT HS256 com o segredo dos testes. Serve para montar refresh
/// tokens que a API não emite mais (formato anterior à rotação, sem `jti`) ou
/// que ela nunca emitiu (com `jti`, mas sem registro).
fn sign_test_refresh_token(user: &RefreshContractUser, jti: Option<&str>) -> String {
    let now = test_epoch_now();
    let mut claims = serde_json::json!({
        "sub": user.user_id,
        "username": user.username,
        "role": "player",
        "token_version": 0,
        "iat": now - 3600,
        "exp": now + 3600,
        "type": "refresh",
    });
    if let Some(jti) = jti {
        claims["jti"] = serde_json::Value::String(jti.to_string());
    }
    let header = base64url(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = base64url(claims.to_string().as_bytes());
    let mut mac = Hmac::<Sha256>::new_from_slice(b"test-secret-key-for-tests").unwrap();
    mac.update(format!("{header}.{payload}").as_bytes());
    let signature = base64url(&mac.finalize().into_bytes());
    format!("{header}.{payload}.{signature}")
}

#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 063 — set DATABASE_URL to run"]
async fn test_contract_refresh_token_rotates_and_reuse_revokes_the_session() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "rotate").await;

    let (status, first) = refresh_with(&app, &user.refresh_token).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert!(first["token"].is_string());
    let second_token = rotated_refresh_token(&first);
    assert_ne!(second_token, user.refresh_token);

    // O substituto renova normalmente e também é trocado.
    let (status, second) = refresh_with(&app, &second_token).await;
    assert_eq!(status, StatusCode::OK, "{second}");
    let third_token = rotated_refresh_token(&second);
    assert_ne!(third_token, second_token);

    // Só os hashes ficam no banco, nunca o JWT.
    let stored: Vec<String> =
        sqlx::query_scalar("SELECT token_hash FROM auth_refresh_tokens WHERE user_id = $1::uuid")
            .bind(&user.user_id)
            .fetch_all(&state.db)
            .await
            .unwrap();
    assert_eq!(stored.len(), 3);
    assert!(stored.iter().all(|hash| hash.len() == 64));
    assert!(stored.contains(&poker_api::refresh_tokens::hash_token(&third_token)));

    // Passada a tolerância, o primeiro token (já trocado) reaparece: é cópia.
    age_refresh_token_consumption(&state, &user.refresh_token).await;
    let (status, _) = refresh_with(&app, &user.refresh_token).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // A família inteira caiu, inclusive o token mais novo, nunca usado.
    let (status, _) = refresh_with(&app, &third_token).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM auth_refresh_tokens WHERE user_id = $1::uuid AND revoked_at IS NULL",
    )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(live, 0);
    let reuse_events: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE user_id = $1 AND action = 'REFRESH_TOKEN_REUSE'",
    )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(reuse_events, 1);

    // Entrar de novo abre outra família, que funciona.
    let fresh_login = login_refresh_contract_user(&app, &user).await;
    let (status, _) = refresh_with(&app, &fresh_login).await;
    assert_eq!(status, StatusCode::OK);

    sqlx::query("DELETE FROM audit_logs WHERE user_id = $1")
        .bind(&user.user_id)
        .execute(&state.db)
        .await
        .unwrap();
    cleanup_contract_user(&state, &user.username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 063 — set DATABASE_URL to run"]
async fn test_contract_refresh_token_tolerates_two_tabs_renewing_together() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "twotabs").await;

    // Duas abas leem o mesmo token do navegador e renovam ao mesmo tempo.
    let ((status_a, tab_a), (status_b, tab_b)) = tokio::join!(
        refresh_with(&app, &user.refresh_token),
        refresh_with(&app, &user.refresh_token)
    );
    assert_eq!(status_a, StatusCode::OK, "{tab_a}");
    assert_eq!(status_b, StatusCode::OK, "{tab_b}");
    let token_a = rotated_refresh_token(&tab_a);
    let token_b = rotated_refresh_token(&tab_b);
    assert_ne!(token_a, token_b);

    // Uma nova tentativa logo depois (resposta perdida na rede) também passa.
    let (status, retry) = refresh_with(&app, &user.refresh_token).await;
    assert_eq!(status, StatusCode::OK, "{retry}");

    // Nenhuma das abas perdeu a sessão.
    let (status, _) = refresh_with(&app, &token_a).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = refresh_with(&app, &token_b).await;
    assert_eq!(status, StatusCode::OK);
    let (revoked, families): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*) FILTER (WHERE revoked_at IS NOT NULL), COUNT(DISTINCT family_id) \
         FROM auth_refresh_tokens WHERE user_id = $1::uuid",
    )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(revoked, 0);
    assert_eq!(families, 1);

    cleanup_contract_user(&state, &user.username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 063 — set DATABASE_URL to run"]
async fn test_contract_refresh_token_issued_before_rotation_is_adopted_once() {
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "legacy").await;

    // Sessão aberta antes da rotação: token válido, sem `jti` e sem registro.
    let legacy = sign_test_refresh_token(&user, None);
    let (status, adopted) = refresh_with(&app, &legacy).await;
    assert_eq!(status, StatusCode::OK, "{adopted}");
    let replacement = rotated_refresh_token(&adopted);
    let consumed_at: Option<i64> =
        sqlx::query_scalar("SELECT consumed_at FROM auth_refresh_tokens WHERE token_hash = $1")
            .bind(poker_api::refresh_tokens::hash_token(&legacy))
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert!(consumed_at.is_some());

    // Daí em diante ele segue a regra dos demais: reapresentado, derruba a sessão.
    age_refresh_token_consumption(&state, &legacy).await;
    let (status, _) = refresh_with(&app, &legacy).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = refresh_with(&app, &replacement).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Token no formato novo que a API nunca registrou não renova nada.
    let unknown = sign_test_refresh_token(&user, Some(&uuid::Uuid::new_v4().to_string()));
    let (status, _) = refresh_with(&app, &unknown).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let unknown_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM auth_refresh_tokens WHERE token_hash = $1")
            .bind(poker_api::refresh_tokens::hash_token(&unknown))
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(unknown_rows, 0);

    // O token do cadastro pertence a outra família e não foi afetado.
    let (status, _) = refresh_with(&app, &user.refresh_token).await;
    assert_eq!(status, StatusCode::OK);

    sqlx::query("DELETE FROM audit_logs WHERE user_id = $1")
        .bind(&user.user_id)
        .execute(&state.db)
        .await
        .unwrap();
    cleanup_contract_user(&state, &user.username).await;
}

#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 063 — set DATABASE_URL to run"]
async fn test_contract_logout_revokes_only_that_session() {
    async fn logout_with(app: &axum::Router, refresh_token: &str) -> (StatusCode, String) {
        let body = serde_json::json!({ "refresh_token": refresh_token }).to_string();
        send_request(app.clone(), Method::POST, "/api/auth/logout", Some(body)).await
    }

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "logout").await;

    // Segunda sessão da mesma conta (outro aparelho).
    let other_device = login_refresh_contract_user(&app, &user).await;

    let (status, renewed) = refresh_with(&app, &user.refresh_token).await;
    assert_eq!(status, StatusCode::OK, "{renewed}");
    let current = rotated_refresh_token(&renewed);

    let (status, response) = logout_with(&app, &current).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).unwrap()["ok"],
        true
    );

    // A sessão encerrada não renova mais.
    let (status, _) = refresh_with(&app, &current).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Sair de novo, ou com um token qualquer, responde igual e não revoga nada.
    let (status, _) = logout_with(&app, &current).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = logout_with(&app, "not-a-token").await;
    assert_eq!(status, StatusCode::OK);

    // O outro aparelho continua logado.
    let (status, _) = refresh_with(&app, &other_device).await;
    assert_eq!(status, StatusCode::OK);

    cleanup_contract_user(&state, &user.username).await;
}

/// Sessão por cookie (item 2 de segurança, 23/11/2026): o site entra com
/// `X-ZT-Session: cookie`, recebe os tokens só em cookies HttpOnly, usa a conta
/// pelo cookie, precisa do cabeçalho anti-CSRF nas escritas, renova pelo cookie
/// e sai apagando os cookies. O Bearer continua valendo para scripts.
#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 063 — set DATABASE_URL to run"]
async fn test_contract_cookie_session_login_me_csrf_refresh_logout() {
    async fn call(
        app: &axum::Router,
        method: Method,
        uri: &str,
        cookies: &str,
        extra: &[(&str, &str)],
        body: Option<&str>,
    ) -> (StatusCode, Vec<String>, String) {
        let mut builder = Request::builder().method(method).uri(uri);
        if !cookies.is_empty() {
            builder = builder.header("Cookie", cookies);
        }
        for (key, value) in extra {
            builder = builder.header(*key, *value);
        }
        let request = match body {
            Some(text) => builder
                .header("Content-Type", "application/json")
                .body(Body::from(text.to_string()))
                .unwrap(),
            None => builder.body(Body::empty()).unwrap(),
        };
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let set_cookies = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|value| value.to_str().unwrap().to_string())
            .collect();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            set_cookies,
            String::from_utf8_lossy(&bytes).to_string(),
        )
    }
    fn jar(set_cookies: &[String]) -> String {
        set_cookies
            .iter()
            .map(|cookie| cookie.split(';').next().unwrap().to_string())
            .collect::<Vec<_>>()
            .join("; ")
    }

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "cookie").await;
    let credentials =
        serde_json::json!({ "email": user.email, "password": user.password }).to_string();

    let (status, set, body) = call(
        &app,
        Method::POST,
        "/api/auth/login",
        "",
        &[("X-ZT-Session", "cookie")],
        Some(&credentials),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json.get("token").is_none(), "token fora do corpo: {body}");
    assert!(
        json.get("refresh_token").is_none(),
        "refresh fora do corpo: {body}"
    );
    assert_eq!(json["session"], "cookie");
    assert_eq!(set.len(), 2, "{set:?}");
    assert!(set
        .iter()
        .all(|c| c.contains("HttpOnly") && c.contains("SameSite=Strict") && c.contains("Secure")));
    let cookies = jar(&set);

    let (status, _, body) = call(&app, Method::GET, "/api/auth/me", &cookies, &[], None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(&user.username), "{body}");

    // Escrita pelo cookie sem o cabeçalho anti-CSRF: recusada antes do handler.
    let mode = r#"{"mode":"play"}"#;
    let (status, _, _) = call(
        &app,
        Method::POST,
        "/api/wallet/mode",
        &cookies,
        &[],
        Some(mode),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, body) = call(
        &app,
        Method::POST,
        "/api/wallet/mode",
        &cookies,
        &[("X-ZT-CSRF", "1")],
        Some(mode),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Renovação pelo cookie, sem token no corpo; sem o cabeçalho, recusada.
    let (status, _, _) = call(
        &app,
        Method::POST,
        "/api/auth/refresh",
        &cookies,
        &[("X-ZT-Session", "cookie")],
        Some("{}"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, set, body) = call(
        &app,
        Method::POST,
        "/api/auth/refresh",
        &cookies,
        &[("X-ZT-Session", "cookie"), ("X-ZT-CSRF", "1")],
        Some("{}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(!body.contains("refresh_token"), "{body}");
    let renewed = jar(&set);
    assert_ne!(renewed, cookies);
    let (status, _, _) = call(&app, Method::GET, "/api/auth/me", &renewed, &[], None).await;
    assert_eq!(status, StatusCode::OK);

    // Logout pelo cookie apaga os cookies e encerra a sessão no servidor.
    let (status, set, _) = call(
        &app,
        Method::POST,
        "/api/auth/logout",
        &renewed,
        &[("X-ZT-CSRF", "1")],
        Some("{}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(set.len(), 2);
    assert!(set.iter().all(|c| c.contains("Max-Age=0")), "{set:?}");
    let (status, set, _) = call(
        &app,
        Method::POST,
        "/api/auth/refresh",
        &renewed,
        &[("X-ZT-CSRF", "1")],
        Some("{}"),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "sessão encerrada não renova"
    );
    assert!(set.iter().all(|c| c.contains("Max-Age=0")));

    // Scripts seguem com o Bearer: tokens no corpo e escrita sem cabeçalho extra.
    let (status, _, body) = call(
        &app,
        Method::POST,
        "/api/auth/login",
        "",
        &[],
        Some(&credentials),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let token = serde_json::from_str::<serde_json::Value>(&body).unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let bearer = format!("Bearer {token}");
    let (status, _, body) = call(
        &app,
        Method::POST,
        "/api/wallet/mode",
        "",
        &[("Authorization", bearer.as_str())],
        Some(mode),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    cleanup_contract_user(&state, &user.username).await;
}

/// Livro único (migration 066): o extrato da carteira lê o livro, filtra por
/// carteira e a conferência saldo = soma do livro é só do admin.
#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 066 — set DATABASE_URL to run"]
async fn test_contract_wallet_ledger_statement_and_reconcile() {
    use poker_api::wallet::{credit_wallet, LedgerReason, WalletKind};

    async fn get_json(
        app: &axum::Router,
        uri: &str,
        token: &str,
    ) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }
    async fn login_token(app: &axum::Router, email: &str, password: &str) -> String {
        let body = serde_json::json!({ "email": email, "password": password }).to_string();
        let (status, text) =
            send_request(app.clone(), Method::POST, "/api/auth/login", Some(body)).await;
        assert_eq!(status, StatusCode::OK, "{text}");
        serde_json::from_str::<serde_json::Value>(&text).unwrap()["token"]
            .as_str()
            .unwrap()
            .to_string()
    }

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "ledger").await;
    credit_wallet(
        &state.db,
        &user.user_id,
        900,
        WalletKind::Real,
        LedgerReason::ManualDeposit,
        Some("req-ledger"),
    )
    .await
    .unwrap();
    let token = login_token(&app, &user.email, user.password).await;

    let (status, rows) = get_json(&app, "/api/wallet/ledger?wallet=real", &token).await;
    assert_eq!(status, StatusCode::OK, "{rows}");
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["amount_cents"], 900);
    assert_eq!(rows[0]["balance_after_cents"], 900);
    assert_eq!(rows[0]["reason"], "manual_deposit");
    assert_eq!(rows[0]["reference"], "req-ledger");
    assert!(rows[0]["created_at"].as_str().unwrap().ends_with('Z'));

    let (status, _) = get_json(&app, "/api/wallet/ledger?wallet=bogus", &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = get_json(&app, "/api/admin/ledger/reconcile", &token).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "conferência só do admin");

    sqlx::query("UPDATE users SET role = 'admin' WHERE id = $1::uuid")
        .bind(&user.user_id)
        .execute(&state.db)
        .await
        .unwrap();
    // Mudar o papel revoga os tokens já emitidos (token_version): entra de novo.
    let token = login_token(&app, &user.email, user.password).await;
    let (status, report) = get_json(&app, "/api/admin/ledger/reconcile", &token).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert!(report["entries"].as_i64().unwrap() >= 1);
    assert!(report["mismatches"]
        .as_array()
        .unwrap()
        .iter()
        .all(|row| row["user_id"] != user.user_id.as_str()));

    cleanup_contract_user(&state, &user.username).await;
}

// ─── Nota do quiz corrigida no servidor ───

/// O servidor ignora nota pronta, corrige pelas respostas, confere o
/// pré-requisito e guarda a melhor nota.
#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 054 — set DATABASE_URL to run"]
async fn test_contract_course_quiz_is_graded_by_the_server() {
    async fn post(
        app: &axum::Router,
        token: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/course/progress")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }
    let course: serde_json::Value = serde_json::from_str(include_str!(
        "../../Frontend-Web/src/data/courseContent.json"
    ))
    .unwrap();
    let lesson = |id: &str| -> serde_json::Value {
        course["modules"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|m| m["lessons"].as_array().unwrap().iter())
            .find(|l| l["id"] == id)
            .cloned()
            .unwrap()
    };
    let answers = |id: &str, right: bool| -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for q in lesson(id)["quiz"].as_array().unwrap() {
            let correct = if q["kind"] == "engine" {
                q["expected"].as_str().unwrap().to_string()
            } else {
                let i = q["expectedIndex"].as_u64().unwrap() as usize;
                q["options"][i].as_str().unwrap().to_string()
            };
            let pick = if right {
                correct
            } else {
                format!("não {correct}")
            };
            map.insert(q["id"].as_str().unwrap().to_string(), pick.into());
        }
        serde_json::Value::Object(map)
    };
    assert_eq!(lesson("m0l5")["prerequisites"], serde_json::json!(["m0l4"]));

    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "quiz").await;
    let login = serde_json::json!({ "email": user.email, "password": user.password }).to_string();
    let (status, text) =
        send_request(app.clone(), Method::POST, "/api/auth/login", Some(login)).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    let token = serde_json::from_str::<serde_json::Value>(&text).unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // Nota pronta do navegador (formato antigo) não vale mais.
    let (status, _) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l4", "completed": true, "score": 100 }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m9l9", "answers": {} }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // Pré-requisito (m0l4) ainda não concluído.
    let (status, _) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l5", "answers": answers("m0l5", true) }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Sem o vídeo assistido, o quiz não abre.
    let (status, _) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l4", "answers": answers("m0l4", true) }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let watched_all = |lesson: &'static str| {
        let db = state.db.clone();
        let user_id = user.user_id.clone();
        async move {
            sqlx::query(
                "INSERT INTO course_watch (user_id, lesson_id, watched, checkpoints) \
                 VALUES ($1::uuid, $2, decode(repeat('ff', 64), 'hex'), '{}') \
                 ON CONFLICT (user_id, lesson_id) DO UPDATE SET watched = EXCLUDED.watched, \
                   checkpoints = '{}', check_id = NULL",
            )
            .bind(&user_id)
            .bind(lesson)
            .execute(&db)
            .await
            .unwrap();
        }
    };
    watched_all("m0l4").await;

    let (status, body) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l4", "answers": answers("m0l4", false) }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        (body["status"].as_str(), body["best_score"].as_i64()),
        (Some("started"), Some(0))
    );
    // Reprovado: só a nota e a hora da nova tentativa, sem gabarito.
    assert!(body["retry_at"].is_string(), "{body}");
    assert!(body.get("review").is_none(), "{body}");
    let (status, _) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l4", "answers": answers("m0l4", true) }),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    sqlx::query(
        "UPDATE course_progress SET last_failed_at = NOW() - INTERVAL '16 minutes' \
         WHERE user_id = $1::uuid AND lesson_id = 'm0l4'",
    )
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .unwrap();
    let (status, body) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l4", "answers": answers("m0l4", true) }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "completed");
    assert_eq!(body["best_score"], 100);
    assert_eq!(body["attempts"], 2);
    assert!(body.get("retry_at").is_none(), "{body}");
    let review = body["review"].as_array().unwrap();
    assert_eq!(
        review.len(),
        lesson("m0l4")["quiz"].as_array().unwrap().len()
    );
    assert!(review
        .iter()
        .all(|r| r["explanation"].as_str().is_some_and(|e| !e.is_empty())));
    // Aula concluída não é corrigida de novo nem rebaixada.
    let (_, body) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l4", "answers": answers("m0l4", false) }),
    )
    .await;
    assert_eq!(
        (
            body["status"].as_str(),
            body["best_score"].as_i64(),
            body["attempts"].as_i64()
        ),
        (Some("completed"), Some(100), Some(2))
    );
    // Com o pré-requisito concluído, a aula seguinte pede o próprio vídeo.
    let (status, _) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l5", "answers": answers("m0l5", false) }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    watched_all("m0l5").await;
    let (status, body) = post(
        &app,
        &token,
        serde_json::json!({ "lesson_id": "m0l5", "answers": answers("m0l5", false) }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "started");

    cleanup_contract_user(&state, &user.username).await;
}

/// O tempo de vídeo é contado pelo servidor (até 2x, dentro do relógio) e a
/// pergunta surpresa errada ou sem resposta apaga o trecho desde a anterior.
#[tokio::test]
#[ignore = "Requires PostgreSQL with migration 068 — set DATABASE_URL to run"]
async fn test_contract_course_video_is_counted_by_the_server() {
    async fn call(
        app: &axum::Router,
        token: &str,
        method: Method,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("Authorization", format!("Bearer {token}"));
        if body.is_some() {
            request = request.header("Content-Type", "application/json");
        }
        let request = request
            .body(Body::from(body.map(|b| b.to_string()).unwrap_or_default()))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }
    let state = make_test_state();
    let app = poker_api::build_router(state.clone());
    let user = register_refresh_contract_user(&state, &app, "watch").await;
    let login = serde_json::json!({ "email": user.email, "password": user.password }).to_string();
    let (status, text) =
        send_request(app.clone(), Method::POST, "/api/auth/login", Some(login)).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    let token = serde_json::from_str::<serde_json::Value>(&text).unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let watch = |from: f64, to: f64, rate: f64| serde_json::json!({ "lesson_id": "m0l4", "from": from, "to": to, "rate": rate });
    let reply = |id: &serde_json::Value, answer: serde_json::Value| serde_json::json!({ "lesson_id": "m0l4", "check_id": id, "answer": answer });
    let db = state.db.clone();
    let user_id = user.user_id.clone();
    let sql = |statement: &'static str| {
        let db = db.clone();
        let user_id = user_id.clone();
        async move {
            sqlx::query(statement)
                .bind(&user_id)
                .execute(&db)
                .await
                .unwrap();
        }
    };
    let rewind_clock = "UPDATE course_watch SET last_beat_at = NOW() - INTERVAL '20 seconds' \
                        WHERE user_id = $1::uuid AND lesson_id = 'm0l4'";
    let answer = || {
        let db = db.clone();
        let user_id = user_id.clone();
        async move {
            sqlx::query_scalar::<_, String>(
                "SELECT check_answer FROM course_watch \
                 WHERE user_id = $1::uuid AND lesson_id = 'm0l4'",
            )
            .bind(&user_id)
            .fetch_one(&db)
            .await
            .unwrap()
        }
    };
    let get_state = "/api/course/lesson?lesson_id=m0l4";
    // Com sessão, o Caddy libera os vídeos das aulas.
    let (status, _) = call(&app, &token, Method::GET, "/api/auth/check", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    // Texto da aula (fora do pacote do site) para quem entrou na conta.
    let (status, body) = call(
        &app,
        &token,
        Method::GET,
        "/api/course/material?lesson_id=m0l4",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body["body"].as_str().is_some_and(|t| t.len() > 200),
        "{body}"
    );
    assert!(
        body["transcript"].as_str().is_some_and(|t| t.len() > 200),
        "{body}"
    );

    let (status, body) = call(&app, &token, Method::GET, get_state, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        (body["has_video"].as_bool(), body["quiz_open"].as_bool()),
        (Some(true), Some(false))
    );
    assert_eq!(body["watched_percent"], 0);
    assert!(body["checks_left"].as_u64().unwrap() >= 1);

    // Início da reprodução e, logo em seguida, 20 s de vídeo: rápido demais para o relógio.
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/watch",
        Some(watch(0.0, 0.0, 1.0)),
    )
    .await;
    assert_eq!(body["accepted"], false);
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/watch",
        Some(watch(0.0, 20.0, 1.0)),
    )
    .await;
    assert_eq!(
        (body["accepted"].as_bool(), body["watched_percent"].as_i64()),
        (Some(false), Some(0))
    );
    // Acima de 2x não conta, mesmo com relógio de sobra.
    sql(
        "UPDATE course_watch SET last_beat_at = NOW() - INTERVAL '20 seconds', \
           checkpoints = '{15}' WHERE user_id = $1::uuid AND lesson_id = 'm0l4'",
    )
    .await;
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/watch",
        Some(watch(0.0, 20.0, 3.0)),
    )
    .await;
    assert_eq!(body["accepted"], false);

    // Trecho válido que passa pelo momento sorteado: abre a pergunta surpresa.
    sql(rewind_clock).await;
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/watch",
        Some(watch(0.0, 20.0, 1.0)),
    )
    .await;
    assert_eq!(body["accepted"], true, "{body}");
    assert!(body["watched_percent"].as_i64().unwrap() > 0);
    let check = body["pending_check"].clone();
    let options = check["options"].as_array().unwrap();
    assert_eq!(options.len(), 4, "{check}");
    assert!(check["seconds_left"].as_i64().unwrap() > 50);
    let right = answer().await;
    let wrong = options
        .iter()
        .find(|o| **o != right.as_str())
        .unwrap()
        .clone();

    // Errou: o trecho deixa de contar e o vídeo volta ao início do trecho.
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/check",
        Some(reply(&check["id"], wrong)),
    )
    .await;
    assert_eq!(body["check_result"], "wrong", "{body}");
    assert_eq!(
        (body["rewind_to"].as_f64(), body["watched_percent"].as_i64()),
        (Some(0.0), Some(0))
    );
    assert_eq!(body["checks_left"], 1);
    assert!(body["pending_check"].is_null());

    // Reviu o trecho: a pergunta volta; sem resposta em 60 s, também não conta.
    sql(rewind_clock).await;
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/watch",
        Some(watch(0.0, 20.0, 1.0)),
    )
    .await;
    assert!(body["pending_check"].is_object(), "{body}");
    sql(
        "UPDATE course_watch SET check_issued_at = NOW() - INTERVAL '2 minutes', \
           last_beat_at = NOW() - INTERVAL '20 seconds' \
         WHERE user_id = $1::uuid AND lesson_id = 'm0l4'",
    )
    .await;
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/watch",
        Some(watch(20.0, 30.0, 1.0)),
    )
    .await;
    assert_eq!(
        (body["check_result"].as_str(), body["accepted"].as_bool()),
        (Some("expired"), Some(false))
    );
    assert_eq!(body["watched_percent"], 0);

    // Terceira passagem com a resposta certa: a pergunta sai da lista.
    sql(rewind_clock).await;
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/watch",
        Some(watch(0.0, 20.0, 1.0)),
    )
    .await;
    let check = body["pending_check"].clone();
    let right = answer().await;
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/check",
        Some(reply(&check["id"], right.clone().into())),
    )
    .await;
    assert_eq!(body["check_result"], "right", "{body}");
    assert_eq!(body["checks_left"], 0);
    assert!(body["watched_percent"].as_i64().unwrap() > 0);
    // Resposta repetida para pergunta já encerrada não muda nada.
    let (_, body) = call(
        &app,
        &token,
        Method::POST,
        "/api/course/check",
        Some(reply(&check["id"], right.into())),
    )
    .await;
    assert_eq!(body["check_result"], "stale");
    assert_eq!(body["quiz_open"], false, "vídeo ainda abaixo de 90%");

    // Com 90% do vídeo marcado, o quiz abre.
    sql(
        "UPDATE course_watch SET watched = decode(repeat('ff', 64), 'hex') \
         WHERE user_id = $1::uuid AND lesson_id = 'm0l4'",
    )
    .await;
    let (_, body) = call(&app, &token, Method::GET, get_state, None).await;
    assert_eq!(
        (
            body["watched_percent"].as_i64(),
            body["quiz_open"].as_bool()
        ),
        (Some(100), Some(true))
    );

    cleanup_contract_user(&state, &user.username).await;
}
