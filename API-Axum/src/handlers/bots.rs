//! Endpoints admin da frota de bots (base do futuro coach).
//!
//! POST /api/admin/bots/ensure-pool — cria as 72 contas bot_*
//! POST /api/admin/bots/start — liga N bots numa mesa play
//! POST /api/admin/bots/stop — desliga os bots da mesa (cashout)
//! POST /api/admin/bots/start-tournament — inscreve N bots num torneio play
//! POST /api/admin/bots/stop-tournament — desliga os bots do torneio (sit-out)
//! GET  /api/admin/bots/status — elenco + deploys ativos (mesas e torneios)

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::bots::{all_strategies, BotError, BOT_POOL_SIZE};
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
pub struct BotGroupBody {
    pub personality: String,
    pub count: usize,
}

fn parse_body_groups(
    groups: &Option<Vec<BotGroupBody>>,
) -> Result<Vec<(crate::bots::Personality, usize)>, ApiError> {
    let groups = groups.as_deref().unwrap_or(&[]);
    groups
        .iter()
        .map(|g| {
            crate::bots::Personality::parse(&g.personality)
                .map(|p| (p, g.count))
                .ok_or_else(|| {
                    ApiError::BadRequest(format!("personalidade desconhecida: {}", g.personality))
                })
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct StartBotsBody {
    pub table_id: String,
    pub count: usize,
    pub strategy: Option<String>,
    /// Mix opcional: [{"personality":"tag","count":2},...] somando `count`.
    /// Ausente = tudo Lag (compatível).
    pub groups: Option<Vec<BotGroupBody>>,
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
    let strategy = body
        .strategy
        .unwrap_or_else(|| crate::bots::STRATEGY_LAG_V2.to_string());
    let groups = parse_body_groups(&body.groups)?;
    let dep = state
        .bots
        .start(&body.table_id, body.count, &strategy, &groups)
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
    pub variant: String,
    pub started_at: i64,
    pub bots_total: i64,
    pub bots_alive: i64,
    pub hands_played: i64,
    pub leader: Option<BotSeatInfo>,
    pub seats: Vec<BotSeatInfo>,
    pub personalities: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BotSeatInfo {
    pub username: String,
    pub chips: i64,
    /// Personalidade do bot neste deploy (ex.: "tag"). None p/ humanos.
    pub personality: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BotTournamentPlayer {
    pub username: String,
    pub personality: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BotTournamentStatus {
    pub tournament_id: String,
    pub tournament_name: String,
    pub strategy: String,
    pub bots_total: usize,
    pub personalities: std::collections::HashMap<String, String>,
    pub players: Vec<BotTournamentPlayer>,
}

#[derive(Debug, Serialize)]
pub struct BotsStatusResponse {
    pub pool_total: i64,
    pub pool_free: i64,
    pub strategies: Vec<String>,
    pub tables: Vec<BotTableStatus>,
    pub tournaments: Vec<BotTournamentStatus>,
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
        let seats: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT u.id::text, u.username, s.chips FROM cash_game_seats s \
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
            .map(|(user_id, username, chips)| BotSeatInfo {
                username,
                chips,
                personality: dep.personalities.get(&user_id).cloned(),
            })
            .collect();
        let alive = seat_infos.iter().filter(|s| s.chips > 0).count() as i64;
        tables.push(BotTableStatus {
            table_id: dep.table_id,
            table_name: dep.table_name,
            strategy: dep.strategy,
            variant: dep.variant,
            started_at: dep.started_at,
            bots_total: dep.bot_ids.len() as i64,
            bots_alive: alive,
            hands_played: hands_played.max(0),
            leader: seat_infos.first().cloned(),
            seats: seat_infos,
            personalities: dep.personalities,
        });
    }
    Ok(Json(BotsStatusResponse {
        pool_total,
        pool_free: pool_total - busy,
        strategies: all_strategies(),
        tables,
        tournaments: {
            let mut out = Vec::new();
            for d in state.bots.tournament_status().await {
                let names: Vec<(String, String)> =
                    sqlx::query_as("SELECT id::text, username FROM users WHERE is_bot")
                        .fetch_all(&state.db)
                        .await
                        .unwrap_or_default();
                let by_id: std::collections::HashMap<_, _> = names.into_iter().collect();
                out.push(BotTournamentStatus {
                    tournament_id: d.tournament_id,
                    tournament_name: d.tournament_name,
                    strategy: d.strategy,
                    bots_total: d.bot_ids.len(),
                    players: d
                        .bot_ids
                        .iter()
                        .map(|id| BotTournamentPlayer {
                            username: by_id.get(id).cloned().unwrap_or_else(|| id.clone()),
                            personality: d.personalities.get(id).cloned(),
                        })
                        .collect(),
                    personalities: d.personalities,
                });
            }
            out
        },
    }))
}

/// POST /api/admin/bots/start-tournament — inscreve N bots (lag_v2) no torneio play.
pub async fn start_tournament_bots(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<StartTournamentBotsBody>,
) -> Result<Json<StartTournamentBotsResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let strategy = body
        .strategy
        .unwrap_or_else(|| crate::bots::STRATEGY_LAG_V2.to_string());
    let groups = parse_body_groups(&body.groups)?;
    let dep = state
        .bots
        .start_tournament(&body.tournament_id, body.count, &strategy, &groups)
        .await
        .map_err(bot_err)?;
    Ok(Json(StartTournamentBotsResponse {
        tournament_id: dep.tournament_id,
        tournament_name: dep.tournament_name,
        strategy: dep.strategy,
        bots: dep.bot_ids,
    }))
}

/// POST /api/admin/bots/stop-tournament — desliga os bots do torneio (sit-out).
pub async fn stop_tournament_bots(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<StopTournamentBotsBody>,
) -> Result<Json<StopTournamentBotsResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let stopped = state
        .bots
        .stop_tournament(&body.tournament_id)
        .await
        .map_err(bot_err)?;
    Ok(Json(StopTournamentBotsResponse {
        tournament_id: body.tournament_id,
        bots_stopped: stopped,
    }))
}

#[derive(Debug, Deserialize)]
pub struct StartTournamentBotsBody {
    pub tournament_id: String,
    pub count: usize,
    pub strategy: Option<String>,
    pub groups: Option<Vec<BotGroupBody>>,
}

#[derive(Debug, Serialize)]
pub struct StartTournamentBotsResponse {
    pub tournament_id: String,
    pub tournament_name: String,
    pub strategy: String,
    pub bots: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct StopTournamentBotsBody {
    pub tournament_id: String,
}

#[derive(Debug, Serialize)]
pub struct StopTournamentBotsResponse {
    pub tournament_id: String,
    pub bots_stopped: usize,
}
