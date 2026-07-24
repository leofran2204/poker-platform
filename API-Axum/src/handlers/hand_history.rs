// Hand history handlers — Endpoints REST HTTPS para Replay de Mãos
use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::state::AppState;

// ─── Response DTOs ───

#[derive(Debug, Serialize)]
pub struct HandHistoryResponse {
    pub hand_id: String,
    pub replay: Value,
}

#[derive(Debug, Serialize)]
pub struct HandHistorySummary {
    pub hand_id: String,
    pub pot_total: i64,
    pub rake_collected: i64,
    pub end_reason: Option<String>,
    pub created_at: i64,
}

// ─── Handlers ───

/// GET /api/hand-history/{hand_id}
/// Response: replay completo da mão
pub async fn get_hand_history(
    State(state): State<AppState>,
    Path(hand_id): Path<String>,
) -> Result<Json<HandHistoryResponse>, ApiError> {
    let row: Option<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT
            id::TEXT,
            COALESCE(
                jsonb_build_object(
                    'hand_number', hand_number,
                    'game_type', game_type,
                    'small_blind', small_blind,
                    'big_blind', big_blind,
                    'actions', actions_json,
                    'community_cards', community_cards_json,
                    'pot_total', pot_total,
                    'rake_collected', rake_collected,
                    'end_reason', end_reason,
                    'winner', winner_player_id,
                    'created_at', created_at
                ),
                '{}'::JSONB
            ) AS replay
        FROM hand_history
        WHERE id::TEXT = $1
        "#,
    )
    .bind(&hand_id)
    .fetch_optional(&state.db)
    .await?;

    let (id, replay) =
        row.ok_or_else(|| ApiError::NotFound(format!("Hand history {hand_id} not found")))?;

    Ok(Json(HandHistoryResponse {
        hand_id: id,
        replay,
    }))
}

/// GET /api/tables/{table_id}/history
/// Response: Lista das últimas 50 mãos finalizadas da mesa
pub async fn list_table_hand_histories(
    State(state): State<AppState>,
    Path(table_id): Path<String>,
) -> Result<Json<Vec<HandHistorySummary>>, ApiError> {
    let rows: Vec<(String, i64, i64, Option<String>, i64)> = sqlx::query_as(
        r#"
        SELECT
            id::TEXT,
            pot_total,
            rake_collected,
            end_reason,
            created_at
        FROM hand_history
        WHERE table_id::TEXT = $1 OR table_id IS NULL
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&table_id)
    .fetch_all(&state.db)
    .await?;

    let summaries = rows
        .into_iter()
        .map(|(id, pot, rake, reason, created)| HandHistorySummary {
            hand_id: id,
            pot_total: pot,
            rake_collected: rake,
            end_reason: reason,
            created_at: created,
        })
        .collect();

    Ok(Json(summaries))
}
