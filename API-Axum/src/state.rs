// Shared application state — holds DB pool, AuthManager, tournaments and live actors.

use poker_engine::auth::AuthManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

use crate::game_actor::PlayerCommand;
use crate::presence::PresenceTracker;
use crate::tournament_store::TournamentStore;

#[derive(Clone)]
pub struct TableActorHandle {
    pub tx_cmd: mpsc::Sender<PlayerCommand>,
    pub tx_broadcast: broadcast::Sender<serde_json::Value>,
}

/// Opaque ticket accepted by the WebSocket upgrade endpoint. It is short lived
/// and consumed atomically, so the long-lived JWT never appears in a URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketTicket {
    pub user_id: String,
    pub username: String,
    pub table_id: String,
    pub expires_at: i64,
}

/// AppState is wrapped in `Arc` and shared across all Axum handlers.
/// High-concurrency interior mutability is handled via `RwLock` (allowing parallel reads).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<RwLock<AuthManager>>,
    pub tournaments: Arc<RwLock<HashMap<String, TournamentStore>>>,
    pub active_tables: Arc<RwLock<HashMap<String, TableActorHandle>>>,
    pub jwt_secret: String,
    pub rate_limiter: crate::middleware::rate_limit::RateLimiter,
    pub redis: Option<redis::aio::ConnectionManager>,
    /// Development fallback when Redis is not configured. Production instances
    /// should provide Redis so ticket consumption is shared across replicas.
    pub ws_tickets: Arc<Mutex<HashMap<String, WebSocketTicket>>>,
    /// Quando true: registro exige confirmação de senha + código por e-mail
    /// antes de liberar tokens e join em mesa.
    pub require_email_verification: bool,
    /// Contador de usuários autenticados com heartbeat recente (Redis ou memória).
    pub presence: PresenceTracker,
}
