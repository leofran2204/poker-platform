// Hand history handlers — GET /api/hand-history/{hand_id}

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

// ─── Handlers ───

/// GET /api/hand-history/{hand_id}
/// Response: replay completo da mão
pub async fn get_hand_history(
    State(state): State<AppState>,
    Path(hand_id): Path<String>,
) -> Result<Json<HandHistoryResponse>, ApiError> {
    // Fetch hand history from PostgreSQL
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
