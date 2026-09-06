//! Endpoints admin da frota de bots (base do futuro coach).
//!
//! POST /api/admin/bots/ensure-pool — cria as 72 contas bot_*
//! POST /api/admin/bots/start — liga N bots numa mesa play
//! POST /api/admin/bots/stop — desliga os bots da mesa (cashout)
//! GET  /api/admin/bots/status — elenco + deploys ativos

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::bots::{BotError, BOT_POOL_SIZE, STRATEGY_LAG_V1};
use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

fn bot_err(e: BotError) -> ApiError {
    ApiError::BadRequest(e.0)
}

#[derive(Debug, Serialize)]
pub struct PoolResponse {
    pub created: i64,
    pub total: i64,
    pub pool_size: i64,
}

/// POST /api/admin/bots/ensure-pool
pub async fn ensure_pool(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
) -> Result<Json<PoolResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let (created, total) = state.bots.ensure_pool().await.map_err(bot_err)?;
    Ok(Json(PoolResponse {
        created,
        total,
        pool_size: BOT_POOL_SIZE,
    }))
}

#[derive(Debug, Deserialize)]
pub struct StartBotsBody {
    pub table_id: String,
    pub count: usize,
    pub strategy: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StartBotsResponse {
    pub table_id: String,
    pub table_name: String,
    pub strategy: String,
    pub bots: Vec<String>,
}

/// POST /api/admin/bots/start
pub async fn start_bots(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<StartBotsBody>,
) -> Result<Json<StartBotsResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let strategy = body.strategy.unwrap_or_else(|| STRATEGY_LAG_V1.to_string());
    let dep = state
        .bots
        .start(&body.table_id, body.count, &strategy)
        .await
        .map_err(bot_err)?;
    Ok(Json(StartBotsResponse {
        table_id: dep.table_id,
        table_name: dep.table_name,
        strategy: dep.strategy,
        bots: dep.bot_ids,
    }))
}

#[derive(Debug, Deserialize)]
pub struct StopBotsBody {
    pub table_id: String,
}

#[derive(Debug, Serialize)]
pub struct StopBotsResponse {
    pub table_id: String,
    pub refunded_chips: i64,
}

/// POST /api/admin/bots/stop
pub async fn stop_bots(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<StopBotsBody>,
) -> Result<Json<StopBotsResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let refunded = state.bots.stop(&body.table_id).await.map_err(bot_err)?;
    Ok(Json(StopBotsResponse {
        table_id: body.table_id,
        refunded_chips: refunded,
    }))
}

#[derive(Debug, Serialize)]
pub struct BotTableStatus {
    pub table_id: String,
    pub table_name: String,
    pub strategy: String,
    pub started_at: i64,
    pub bots_total: i64,
    pub bots_alive: i64,
    pub hands_played: i64,
    pub leader: Option<BotSeatInfo>,
    pub seats: Vec<BotSeatInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BotSeatInfo {
    pub username: String,
    pub chips: i64,
}

#[derive(Debug, Serialize)]
pub struct BotsStatusResponse {
    pub pool_total: i64,
    pub pool_free: i64,
    pub strategies: Vec<String>,
    pub tables: Vec<BotTableStatus>,
}

/// GET /api/admin/bots/status
pub async fn bots_status(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
) -> Result<Json<BotsStatusResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let pool_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_bot")
        .fetch_one(&state.db)
        .await?;
    let deployments = state.bots.status().await;
    let busy: i64 = deployments.iter().map(|d| d.bot_ids.len() as i64).sum();
    let mut tables = Vec::new();
    for dep in deployments {
        let table_uuid = uuid::Uuid::parse_str(&dep.table_id)
            .map_err(|_| ApiError::BadRequest("deploy com table_id invalido".to_string()))?;
        let seats: Vec<(String, i64)> = sqlx::query_as(
            "SELECT u.username, s.chips FROM cash_game_seats s \
             JOIN users u ON u.id = s.user_id \
             WHERE s.table_id = $1 AND s.status = 'ACTIVE' AND u.is_bot \
             ORDER BY s.chips DESC",
        )
        .bind(table_uuid)
        .fetch_all(&state.db)
        .await?;
        let hands_played: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM hand_history WHERE table_id = $1",
        )
        .bind(table_uuid)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0)
            - dep.hands_at_start;
        let seat_infos: Vec<BotSeatInfo> = seats
            .into_iter()
            .map(|(username, chips)| BotSeatInfo { username, chips })
            .collect();
        let alive = seat_infos.iter().filter(|s| s.chips > 0).count() as i64;
        tables.push(BotTableStatus {
            table_id: dep.table_id,
            table_name: dep.table_name,
            strategy: dep.strategy,
            started_at: dep.started_at,
            bots_total: dep.bot_ids.len() as i64,
            bots_alive: alive,
            hands_played: hands_played.max(0),
            leader: seat_infos.first().cloned(),
            seats: seat_infos,
        });
    }
    Ok(Json(BotsStatusResponse {
        pool_total,
        pool_free: pool_total - busy,
        strategies: vec![STRATEGY_LAG_V1.to_string()],
        tables,
    }))
}
