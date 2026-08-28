//! Tournament handlers — list / get / register (MVP; gameplay MTT = fase 2).

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub tournament_id: String,
    /// Optional; ignored when it does not match the authenticated user.
    pub player_id: Option<String>,
    pub player_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub tournament_id: String,
    pub player_id: String,
    pub stack: u64,
    pub registered: bool,
    pub gameplay_ready: bool,
}

#[derive(Debug, Serialize)]
pub struct BlindLevelDto {
    pub level: u32,
    pub small_blind: u64,
    pub big_blind: u64,
    pub ante: u64,
    pub duration_minutes: u32,
}

#[derive(Debug, Serialize)]
pub struct TournamentInfoResponse {
    pub id: String,
    pub name: String,
    pub buy_in: u64,
    pub starting_stack: u64,
    pub max_players: u32,
    pub registered_players: u32,
    pub status: String,
    pub players_remaining: u32,
    pub prize_pool: u64,
    pub guaranteed_prize: u64,
    pub is_freeroll: bool,
    pub allow_rebuy: bool,
    pub rebuy_cost: u64,
    pub rebuy_chips: u64,
    pub rebuy_max_count: u32,
    pub rebuy_stack_threshold: u64,
    pub rebuy_max_level: u32,
    pub blind_levels: Vec<BlindLevelDto>,
    pub gameplay_ready: bool,
}

fn status_string(status: &poker_engine::tournament_engine::TournamentStatus) -> String {
    match status {
        poker_engine::tournament_engine::TournamentStatus::Registering => "registering".into(),
        poker_engine::tournament_engine::TournamentStatus::Running => "running".into(),
        poker_engine::tournament_engine::TournamentStatus::Paused => "paused".into(),
        poker_engine::tournament_engine::TournamentStatus::Finished => "finished".into(),
        poker_engine::tournament_engine::TournamentStatus::Cancelled => "cancelled".into(),
    }
}

fn to_info(store: &crate::tournament_store::TournamentStore) -> TournamentInfoResponse {
    let cfg = &store.state.config;
    TournamentInfoResponse {
        id: store.id.clone(),
        name: cfg.name.clone(),
        buy_in: cfg.buy_in,
        starting_stack: cfg.starting_stack,
        max_players: cfg.max_players,
        registered_players: store.state.players.len() as u32,
        status: status_string(&store.state.status),
        players_remaining: store.state.players_remaining,
        prize_pool: store.state.prize_pool.max(cfg.guaranteed_prize),
        guaranteed_prize: cfg.guaranteed_prize,
        is_freeroll: cfg.is_freeroll,
        allow_rebuy: cfg.allow_rebuy,
        rebuy_cost: if cfg.rebuy_cost > 0 {
            cfg.rebuy_cost
        } else {
            cfg.buy_in
        },
        rebuy_chips: if cfg.rebuy_chips > 0 {
            cfg.rebuy_chips
        } else {
            cfg.starting_stack
        },
        rebuy_max_count: cfg.rebuy_max_count,
        rebuy_stack_threshold: cfg.rebuy_stack_threshold,
        rebuy_max_level: cfg.rebuy_max_level,
        blind_levels: cfg
            .blind_levels
            .iter()
            .map(|b| BlindLevelDto {
                level: b.level,
                small_blind: b.small_blind,
                big_blind: b.big_blind,
                ante: b.ante,
                duration_minutes: b.duration_minutes,
            })
            .collect(),
        // MTT hands not wired to TableActor yet.
        gameplay_ready: false,
    }
}

/// GET /api/lobby/tournaments
pub async fn list_tournaments(
    State(state): State<AppState>,
) -> Result<Json<Vec<TournamentInfoResponse>>, ApiError> {
    let tournaments = state.tournaments.read().await;
    let mut list: Vec<_> = tournaments.values().map(to_info).collect();
    list.sort_by(|a, b| {
        b.is_freeroll
            .cmp(&a.is_freeroll)
            .then(a.buy_in.cmp(&b.buy_in))
            .then(a.name.cmp(&b.name))
    });
    Ok(Json(list))
}

/// GET /api/tournament/{id}
pub async fn get_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> Result<Json<TournamentInfoResponse>, ApiError> {
    let tournaments = state.tournaments.read().await;
    let store = tournaments
        .get(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;
    Ok(Json(to_info(store)))
}

/// POST /api/tournament/register
pub async fn register_player(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<RegisterBody>,
) -> Result<Json<RegisterResponse>, ApiError> {
    if let Some(ref pid) = body.player_id {
        if pid != &auth_user.user_id {
            return Err(ApiError::Forbidden(
                "Tournament registration must use the authenticated player identity".to_string(),
            ));
        }
    }

    let tournament_id = body.tournament_id.clone();

    let mut tournaments = state.tournaments.write().await;
    let store = tournaments
        .get_mut(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;

    let buy_in = store.state.config.buy_in;
    let starting_stack = store.state.config.starting_stack;

    poker_engine::tournament_engine::register_player(
        &mut store.state,
        &auth_user.user_id,
        &auth_user.username,
    )
    .map_err(ApiError::BadRequest)?;

    let mut tx = state.db.begin().await.map_err(|e| {
        store.state.players.remove(&auth_user.user_id);
        e
    })?;

    if buy_in > 0 {
        let buy_in_i = i64::try_from(buy_in).map_err(|_| {
            store.state.players.remove(&auth_user.user_id);
            ApiError::BadRequest("Invalid buy-in".into())
        })?;
        let updated: Option<(i64,)> = sqlx::query_as(
            "UPDATE users SET balance = balance - $1 \
             WHERE id = $2::uuid AND balance >= $1 \
             RETURNING balance",
        )
        .bind(buy_in_i)
        .bind(&auth_user.user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            store.state.players.remove(&auth_user.user_id);
            e
        })?;
        if updated.is_none() {
            store.state.players.remove(&auth_user.user_id);
            return Err(ApiError::BadRequest(
                "Insufficient wallet balance".to_string(),
            ));
        }
    }

    sqlx::query(
        r#"
        INSERT INTO tournament_players
            (tournament_id, player_id, player_name, stack, registered_at)
        VALUES ($1::uuid, $2, $3, $4, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ON CONFLICT (tournament_id, player_id) DO NOTHING
        "#,
    )
    .bind(&tournament_id)
    .bind(&auth_user.user_id)
    .bind(&auth_user.username)
    .bind(starting_stack as i64)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        store.state.players.remove(&auth_user.user_id);
        e
    })?;

    sqlx::query(
        r#"
        UPDATE tournaments
        SET prize_pool = $2,
            players_remaining = $3,
            total_buyins = $4
        WHERE id = $1::uuid
        "#,
    )
    .bind(&tournament_id)
    .bind(store.state.prize_pool as i64)
    .bind(store.state.players_remaining as i32)
    .bind(store.state.players.len() as i32)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        store.state.players.remove(&auth_user.user_id);
        e
    })?;

    tx.commit().await.map_err(|e| {
        store.state.players.remove(&auth_user.user_id);
        e
    })?;

    Ok(Json(RegisterResponse {
        tournament_id,
        player_id: auth_user.user_id,
        stack: starting_stack,
        registered: true,
        gameplay_ready: false,
    }))
}
