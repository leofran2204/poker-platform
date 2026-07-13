// Tournament handlers — POST /api/tournament/register

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

// ─── Request / Response DTOs ───

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub tournament_id: String,
    pub player_id: String,
    pub player_name: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub tournament_id: String,
    pub player_id: String,
    pub stack: u64,
    pub registered: bool,
}

#[derive(Debug, Serialize)]
pub struct TournamentInfoResponse {
    pub id: String,
    pub name: String,
    pub buy_in: u64,
    pub starting_stack: u64,
    pub max_players: u32,
    pub status: String,
    pub players_remaining: u32,
    pub prize_pool: u64,
}

// ─── Handlers ───

/// POST /api/tournament/register
/// Request: `{tournament_id, player_id, player_name}` → Response: `{tournament_id, player_id, stack, registered}`
pub async fn register_player(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<RegisterResponse>, ApiError> {
    let mut tournaments = state.tournaments.lock().await;

    // Get or create the tournament store entry
    let store = tournaments
        .entry(body.tournament_id.clone())
        .or_insert_with(|| {
            // If tournament doesn't exist in memory, create a default
            let config = poker_engine::tournament_engine::TournamentConfig {
                name: "Loaded Tournament".to_string(),
                game_type: "Holdem".to_string(),
                buy_in: 100,
                starting_stack: 10_000,
                max_players: 100,
                speed: poker_engine::tournament_engine::TournamentSpeed::Normal,
                blind_levels: vec![],
                prize_pool_pct: 0.15,
                prize_distribution: vec![0.50, 0.30, 0.20],
                late_registration: true,
                late_registration_max_level: 4,
                allow_rebuy: false,
                allow_addon: false,
                rebuy_max_level: 0,
            };
            crate::tournament_store::TournamentStore::new(body.tournament_id.clone(), config)
        });

    // Register the player in the tournament
    poker_engine::tournament_engine::register_player(
        &mut store.state,
        &body.player_id,
        &body.player_name,
    )
    .map_err(ApiError::BadRequest)?;

    // Persist to PostgreSQL
    sqlx::query(
        r#"
        INSERT INTO tournament_players
            (tournament_id, player_id, player_name, stack, registered_at)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (tournament_id, player_id) DO NOTHING
        "#,
    )
    .bind(&body.tournament_id)
    .bind(&body.player_id)
    .bind(&body.player_name)
    .bind(store.state.config.starting_stack as i64)
    .execute(&state.db)
    .await?;

    Ok(Json(RegisterResponse {
        tournament_id: body.tournament_id,
        player_id: body.player_id,
        stack: store.state.config.starting_stack,
        registered: true,
    }))
}

/// GET /api/tournament/{id}
/// Get tournament info
pub async fn get_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> Result<Json<TournamentInfoResponse>, ApiError> {
    let tournaments = state.tournaments.lock().await;

    let store = tournaments
        .get(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;

    Ok(Json(TournamentInfoResponse {
        id: store.id.clone(),
        name: store.state.config.name.clone(),
        buy_in: store.state.config.buy_in,
        starting_stack: store.state.config.starting_stack,
        max_players: store.state.config.max_players,
        status: format!("{:?}", store.state.status).to_lowercase(),
        players_remaining: store.state.players_remaining,
        prize_pool: store.state.prize_pool,
    }))
}
