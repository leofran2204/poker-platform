// Library crate for Poker API — exposes modules for integration testing
//
// This lib.rs re-exports the public API surface so that integration tests
// in `tests/` can access `build_router`, `AppState`, and `TournamentStore`
// without needing to duplicate the router construction logic.

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod state;
pub mod tournament_store;

use axum::routing::{get, post};
use axum::Router;

use crate::handlers::{auth, hand_history, lobby, tournament, websocket};
use crate::middleware::auth::RequireAuth;
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
///
/// Protected routes (RequireAuth middleware):
///   - POST /api/lobby/join
///   - POST /api/tournament/register
///   - GET  /api/hand-history/:hand_id
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // ─── Auth routes (public) ───
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/mfa/verify", post(auth::mfa_verify_with_username))
        .route("/api/auth/refresh", post(auth::refresh))
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
        // ─── WebSocket route ───
        .route("/ws/game/:table_id", get(websocket::game_websocket))
        // ─── Health check ───
        .route("/health", get(health_check))
        .with_state(state)
}

/// Simple health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
