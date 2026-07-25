// Shared application state — holds DB pool, AuthManager, LobbyManager, tournaments.

use poker_engine::auth::AuthManager;
use poker_engine::lobby::LobbyManager;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};

use crate::tournament_store::TournamentStore;
use crate::game_actor::PlayerCommand;

#[derive(Clone)]
pub struct TableActorHandle {
    pub tx_cmd: mpsc::Sender<PlayerCommand>,
    pub tx_broadcast: broadcast::Sender<serde_json::Value>,
}

/// AppState is wrapped in `Arc` and shared across all Axum handlers.
/// High-concurrency interior mutability is handled via `RwLock` (allowing parallel reads).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<RwLock<AuthManager>>,
    pub lobby: Arc<RwLock<LobbyManager>>,
    pub tournaments: Arc<RwLock<HashMap<String, TournamentStore>>>,
    pub active_tables: Arc<RwLock<HashMap<String, TableActorHandle>>>,
    pub jwt_secret: String,
    pub rate_limiter: crate::middleware::rate_limit::RateLimiter,
}
