// Shared application state — holds DB pool, AuthManager, LobbyManager, tournaments.

use poker_engine::auth::AuthManager;
use poker_engine::lobby::LobbyManager;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::tournament_store::TournamentStore;
use crate::game_actor::PlayerCommand;

#[derive(Clone)]
pub struct TableActorHandle {
    pub tx_cmd: mpsc::Sender<PlayerCommand>,
    pub tx_broadcast: broadcast::Sender<serde_json::Value>,
}
// Shared application state — holds DB pool, AuthManager, LobbyManager, tournaments.

use poker_engine::auth::AuthManager;
use poker_engine::lobby::LobbyManager;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::tournament_store::TournamentStore;
use crate::game_actor::PlayerCommand;

#[derive(Clone)]
pub struct TableActorHandle {
    pub tx_cmd: mpsc::Sender<PlayerCommand>,
    pub tx_broadcast: broadcast::Sender<serde_json::Value>,
}

/// AppState is wrapped in `Arc` and shared across all Axum handlers.
/// Interior mutability is handled via `Mutex` (tokio async-aware).
///
/// `FromRef<AppState>` is auto-implemented by axum's blanket impl
/// `impl<T: Clone> FromRef<T> for T`, so no manual impl is needed.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<Mutex<AuthManager>>,
    pub lobby: Arc<Mutex<LobbyManager>>,
    pub tournaments: Arc<Mutex<HashMap<String, TournamentStore>>>,
    pub active_tables: Arc<Mutex<HashMap<String, TableActorHandle>>>,
    pub jwt_secret: String,
    pub rate_limiter: crate::middleware::rate_limit::RateLimiter,
}
