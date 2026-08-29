// Library crate for Poker API — exposes modules for integration testing
//
// This lib.rs re-exports the public API surface so that integration tests
// in `tests/` can access `build_router`, `AppState`, and `TournamentStore`
// without needing to duplicate the router construction logic.

pub mod admin_panel;
pub mod admin_routes;
pub mod binary_codec;
pub mod email_service;
pub mod error;
pub mod game_actor;
pub mod handlers;
pub mod middleware;
pub mod payment_gateway;
pub mod payments_routes;
pub mod presence;
pub mod state;
pub mod telemetry;
pub mod tournament_catalog;
pub mod tournament_store;

use axum::extract::State;
use axum::routing::{get, patch, post, put};
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

use crate::handlers::{auth, hand_history, lobby, tournament, websocket};
use crate::handlers::presence as presence_handlers;
use crate::middleware::auth::RequireAuth;
use crate::middleware::rate_limit::EnforceRateLimit;
use crate::state::AppState;
use axum::middleware::from_extractor_with_state;

static API_STARTED_AT: OnceLock<Instant> = OnceLock::new();
static ACTIVE_WEBSOCKET_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

pub struct WebSocketConnectionGuard;

impl Drop for WebSocketConnectionGuard {
    fn drop(&mut self) {
        ACTIVE_WEBSOCKET_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
    }
}

pub fn track_websocket_connection() -> WebSocketConnectionGuard {
    ACTIVE_WEBSOCKET_CONNECTIONS.fetch_add(1, Ordering::Relaxed);
    WebSocketConnectionGuard
}

/// Builds the Axum router with all routes wired.
///
/// Public routes (no auth):
///   - POST /api/auth/register
///   - POST /api/auth/login
///   - POST /api/auth/mfa/verify
///   - POST /api/auth/refresh
///   - GET  /api/lobby/tables
///   - GET  /api/lobby/tables/:id
///   - GET  /api/tournament/:id
///   - GET  /health
///   - POST /api/webhooks/pix (PIX Payment Webhook)
///
/// Protected routes (RequireAuth middleware):
///   - POST /api/lobby/join
///   - POST /api/lobby/leave
///   - POST /api/lobby/tables/:id/ws-ticket (short-lived WebSocket ticket)
///   - POST /api/tournament/register
///   - GET  /api/hand-history/:hand_id
///   - POST /api/payments/pix/deposit
///   - POST /api/payments/pix/withdraw
///   - WS   /ws/game/:table_id (JWT + funded active seat required)
///   - POST /api/admin/tables and PATCH /api/admin/tables/:id/status (admin role)
pub fn build_router(state: AppState) -> Router {
    API_STARTED_AT.get_or_init(Instant::now);
    Router::new()
        // ─── Auth routes (public + rate limited) ───
        .route(
            "/api/auth/register",
            post(auth::register).route_layer(
                from_extractor_with_state::<EnforceRateLimit, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/auth/login",
            post(auth::login).route_layer(from_extractor_with_state::<EnforceRateLimit, AppState>(
                state.clone(),
            )),
        )
        .route(
            "/api/auth/mfa/verify",
            post(auth::mfa_verify)
                .route_layer(from_extractor_with_state::<EnforceRateLimit, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/auth/refresh",
            post(auth::refresh).route_layer(
                from_extractor_with_state::<EnforceRateLimit, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/auth/verify-email",
            post(auth::verify_email)
                .route_layer(from_extractor_with_state::<EnforceRateLimit, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/auth/resend-verification",
            post(auth::resend_verification)
                .route_layer(from_extractor_with_state::<EnforceRateLimit, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/auth/me",
            get(auth::me).route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                state.clone(),
            )),
        )
        // ─── Payment routes (PIX Deposit, Webhook & Withdraw + rate limited) ───
        .route(
            "/api/payments/pix/deposit",
            post(payments_routes::create_pix_deposit_handler).route_layer(
                from_extractor_with_state::<EnforceRateLimit, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/webhooks/pix",
            post(payments_routes::pix_webhook_handler),
        )
        .route(
            "/api/payments/pix/withdraw",
            post(payments_routes::create_pix_withdraw_handler).route_layer(
                from_extractor_with_state::<EnforceRateLimit, AppState>(state.clone()),
            ),
        )
        // ─── Presence (online counter) ───
        .route("/api/presence/online", get(presence_handlers::online_count))
        .route(
            "/api/presence/heartbeat",
            post(presence_handlers::heartbeat).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        // ─── Lobby routes ───
        .route("/api/lobby/tables", get(lobby::list_tables))
        .route(
            "/api/lobby/join",
            post(lobby::join_table).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/lobby/leave",
            post(lobby::leave_table).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/lobby/tables/:id/ws-ticket",
            post(websocket::create_ws_ticket)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                ))
                .route_layer(from_extractor_with_state::<EnforceRateLimit, AppState>(
                    state.clone(),
                )),
        )
        .route("/api/lobby/tables/:id", get(lobby::get_table))
        // ─── Tournament routes ───
        .route(
            "/api/lobby/tournaments",
            get(tournament::list_tournaments),
        )
        .route(
            "/api/tournament/register",
            post(tournament::register_player)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        .route("/api/tournament/:id", get(tournament::get_tournament))
        // ─── Hand history routes (protected) ───
        .route(
            "/api/hand-history/:hand_id",
            get(hand_history::get_hand_history)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/tables/:id/history",
            get(hand_history::list_table_hand_histories)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        // ─── Admin & Antifraud routes (protected admin role) ───
        .route(
            "/api/admin/stats",
            get(admin_panel::admin_stats).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/users",
            get(admin_panel::list_users).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/users/:id",
            patch(admin_panel::patch_user).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/users/:id/adjust-balance",
            post(admin_panel::adjust_balance).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/tournaments",
            get(admin_panel::list_admin_tournaments).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/tournaments/:id",
            patch(admin_panel::patch_tournament).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/tournaments/:id/players",
            get(admin_panel::list_tournament_players).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/presence",
            get(admin_panel::admin_presence).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/audit-logs",
            get(admin_panel::list_audit_logs).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/antifraud/alerts",
            get(admin_routes::get_antifraud_alerts_handler).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/tables",
            get(admin_panel::list_admin_tables)
                .post(admin_routes::create_cash_table_handler)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/admin/tables/:id/recovery/abort",
            post(admin_routes::abort_table_recovery_handler).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .route(
            "/api/admin/tables/:id/status",
            patch(admin_routes::update_table_status_handler).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        // ─── B2B SaaS Admin routes ───
        .route(
            "/api/admin/clubs",
            post(admin_routes::create_club)
                .get(admin_routes::list_clubs)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/admin/clubs/:id/financials",
            get(admin_routes::get_club_financials)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/admin/clubs/:id/withdraw",
            post(admin_routes::withdraw_club_balance)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/admin/clubs/:id/theme",
            put(admin_routes::update_club_theme)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        .route(
            "/api/admin/clubs/:id/agents",
            post(admin_routes::create_club_agent)
                .get(admin_routes::list_club_agents)
                .route_layer(from_extractor_with_state::<RequireAuth, AppState>(
                    state.clone(),
                )),
        )
        // ─── WebSocket route ───
        .route("/ws/game/:table_id", get(websocket::game_websocket))
        // ─── Health & Security Metrics check ───
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        .route("/api/health/security", get(security_health_check))
        .route(
            "/api/metrics",
            get(prometheus_metrics).route_layer(
                from_extractor_with_state::<RequireAuth, AppState>(state.clone()),
            ),
        )
        .with_state(state)
}

/// Readiness endpoint. A process is healthy only while its authoritative
/// database is reachable; Redis is also checked whenever configured.
async fn health_check(
    State(state): State<AppState>,
) -> Result<&'static str, crate::error::ApiError> {
    let _: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&state.db).await?;
    if let Some(redis) = &state.redis {
        let mut connection = redis.clone();
        let _: String = redis::cmd("PING")
            .query_async(&mut connection)
            .await
            .map_err(|error| {
                crate::error::ApiError::Internal(format!("Redis health check failed: {error}"))
            })?;
    }
    Ok("OK")
}

/// Public security-boundary metadata.
///
/// This endpoint intentionally does not claim that headers, TLS certificates,
/// container flags, or anti-fraud rules have been verified at runtime: those
/// controls are owned by the deployment gateway and operational checks. It
/// only exposes facts that this process can truthfully assert.
async fn security_health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "INFORMATIONAL",
        "transport": "HTTPS is terminated by the deployment gateway",
        "metrics_access": "administrator authentication required",
        "runtime_security_attestation": "not available from the application process"
    }))
}

/// Prometheus metrics endpoint (format: text/plain; version=0.0.4)
async fn prometheus_metrics(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
) -> Result<(axum::http::HeaderMap, String), crate::error::ApiError> {
    if auth_user.role != "admin" {
        return Err(crate::error::ApiError::Forbidden(
            "Metrics access is restricted to administrators".to_string(),
        ));
    }
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; version=0.0.4"),
    );

    let started_at = API_STARTED_AT.get_or_init(Instant::now);
    let uptime_seconds = started_at.elapsed().as_secs();
    let active_tables = state.active_tables.read().await.len();
    let active_websockets = ACTIVE_WEBSOCKET_CONNECTIONS.load(Ordering::Relaxed);
    let metrics_text = format!(
        "# HELP poker_uptime_seconds Total server uptime in seconds.\n\
         # TYPE poker_uptime_seconds counter\n\
         poker_uptime_seconds {uptime_seconds}\n\
         # HELP poker_active_websocket_connections Current active WebSockets.\n\
         # TYPE poker_active_websocket_connections gauge\n\
         poker_active_websocket_connections {active_websockets}\n\
         # HELP poker_active_table_actors Current in-process table actors.\n\
         # TYPE poker_active_table_actors gauge\n\
         poker_active_table_actors {active_tables}\n"
    );

    Ok((headers, metrics_text))
}
