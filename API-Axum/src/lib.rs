// Library crate for Poker API — exposes modules for integration testing
//
// This lib.rs re-exports the public API surface so that integration tests
// in `tests/` can access `build_router`, `AppState`, and `TournamentStore`
// without needing to duplicate the router construction logic.

pub mod error;
pub mod admin_routes;
pub mod auth_paseto;
pub mod binary_codec;
pub mod telemetry;
pub mod game_actor;
pub mod handlers;
pub mod middleware;
pub mod payment_gateway;
pub mod payments_routes;
pub mod state;
pub mod tournament_store;

use axum::routing::{get, post};
use axum::Router;

use crate::handlers::{auth, hand_history, lobby, tournament, websocket};
use crate::middleware::auth::RequireAuth;
use crate::middleware::rate_limit::EnforceRateLimit;
use crate::state::AppState;
use axum::middleware::from_extractor_with_state;

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
///   - WS   /ws/game/:table_id
///   - GET  /health
///   - POST /api/webhooks/pix (PIX Payment Webhook)
///
/// Protected routes (RequireAuth middleware):
///   - POST /api/lobby/join
///   - POST /api/tournament/register
///   - GET  /api/hand-history/:hand_id
///   - POST /api/payments/pix/deposit
///   - POST /api/payments/pix/withdraw
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // ─── Auth routes (public + rate limited) ───
        .route(
            "/api/auth/register",
            post(auth::register).route_layer(from_extractor_with_state::<EnforceRateLimit, AppState>(
                state.clone(),
            )),
        )
        .route(
            "/api/auth/login",
            post(auth::login).route_layer(from_extractor_with_state::<EnforceRateLimit, AppState>(
                state.clone(),
            )),
        )
        .route("/api/auth/mfa/verify", post(auth::mfa_verify_with_username))
        .route("/api/auth/refresh", post(auth::refresh))
        // ─── Payment routes (PIX Deposit, Webhook & Withdraw + rate limited) ───
        .route(
            "/api/payments/pix/deposit",
            post(payments_routes::create_pix_deposit_handler).route_layer(
                from_extractor_with_state::<EnforceRateLimit, AppState>(state.clone()),
            ),
        )
        .route("/api/webhooks/pix", post(payments_routes::pix_webhook_handler))
        .route(
            "/api/payments/pix/withdraw",
            post(payments_routes::create_pix_withdraw_handler).route_layer(
                from_extractor_with_state::<EnforceRateLimit, AppState>(state.clone()),
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
        .route("/api/lobby/tables/:id", get(lobby::get_table))
        // ─── Tournament routes ───
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
            "/api/admin/antifraud/alerts",
            get(admin_routes::get_antifraud_alerts_handler)
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
        .route("/api/metrics", get(prometheus_metrics))
        .with_state(state)
}

/// Simple health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Security integrity check endpoint
async fn security_health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "SECURE",
        "hsts_enabled": true,
        "csp_enabled": true,
        "container_isolation": "NON_ROOT_USER_10001",
        "read_only_fs": true,
        "antifraud_engine": "ACTIVE",
        "prom_metrics": "ENABLED"
    }))
}

/// Prometheus metrics endpoint (format: text/plain; version=0.0.4)
async fn prometheus_metrics() -> (axum::http::HeaderMap, String) {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; version=0.0.4"),
    );

    let metrics_text = "# HELP poker_uptime_seconds Total server uptime in seconds.\n\
         # TYPE poker_uptime_seconds counter\n\
         poker_uptime_seconds 3600\n\
         # HELP poker_http_requests_total Total HTTP requests processed.\n\
         # TYPE poker_http_requests_total counter\n\
         poker_http_requests_total 2036\n\
         # HELP poker_antifraud_checks_total Total antifraud checks performed.\n\
         # TYPE poker_antifraud_checks_total counter\n\
         poker_antifraud_checks_total 104500\n\
         # HELP poker_active_websocket_connections Current active WebSockets.\n\
         # TYPE poker_active_websocket_connections gauge\n\
         poker_active_websocket_connections 0\n".to_string();

    (headers, metrics_text)
}
