// Lobby handlers — GET /api/lobby/tables, POST /api/lobby/join

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

// ─── Response DTOs ───

#[derive(Debug, Serialize)]
pub struct TableResponse {
    pub id: String,
    pub name: String,
    pub players: u8,
    pub max_players: u8,
    pub small_blind: u64,
    pub big_blind: u64,
    pub game_type: String,
}

#[derive(Debug, Serialize)]
pub struct JoinResponse {
    pub seat: u8,
    pub chips: u64,
}

#[derive(Debug, Deserialize)]
pub struct JoinBody {
    pub table_id: String,
}

// ─── Handlers ───

/// GET /api/lobby/tables
/// Response: `[{id, name, players, max_players, blinds, type}]`
pub async fn list_tables(
    State(state): State<AppState>,
) -> Result<Json<Vec<TableResponse>>, ApiError> {
    let lobby = state.lobby.read().await;
    let tables = lobby.list_available_tables();

    let response: Vec<TableResponse> = tables
        .iter()
        .map(|t| TableResponse {
            id: t.id.clone(),
            name: t.name.clone(),
            players: t.current_players,
            max_players: t.max_players,
            small_blind: t.small_blind,
            big_blind: t.big_blind,
            game_type: format!("{:?}", t.game_type).to_lowercase(),
        })
        .collect();

    Ok(Json(response))
}

/// POST /api/lobby/join
/// Request: `{table_id}` → Response: `{seat, chips}`
pub async fn join_table(
    State(state): State<AppState>,
    Json(body): Json<JoinBody>,
) -> Result<Json<JoinResponse>, ApiError> {
    let mut lobby = state.lobby.write().await;

    // player_balance = 1000 (default), no password (public table)
    let result = lobby.join_table(&body.table_id, 1000, None);

    if !result.success {
        return Err(ApiError::BadRequest(result.message));
    }

    // Find the table to get seat info
    let table = lobby
        .find_table(&body.table_id)
        .ok_or_else(|| ApiError::NotFound("Table not found after join".to_string()))?;

    Ok(Json(JoinResponse {
        seat: table.current_players, // seat = current count after join
        chips: table.min_buy_in,     // default buy-in
    }))
}

/// GET /api/lobby/tables/{id}
/// Get a specific table by ID
pub async fn get_table(
    State(state): State<AppState>,
    Path(table_id): Path<String>,
) -> Result<Json<TableResponse>, ApiError> {
    let lobby = state.lobby.read().await;
    let table = lobby
        .find_table(&table_id)
        .ok_or_else(|| ApiError::NotFound(format!("Table {table_id} not found")))?;

    Ok(Json(TableResponse {
        id: table.id.clone(),
        name: table.name.clone(),
        players: table.current_players,
        max_players: table.max_players,
        small_blind: table.small_blind,
        big_blind: table.big_blind,
        game_type: format!("{:?}", table.game_type).to_lowercase(),
    }))
}
