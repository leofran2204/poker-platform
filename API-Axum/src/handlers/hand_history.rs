// Hand history handlers — Endpoints REST HTTPS para Replay de Mãos
use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::game_actor::settlement_signature_valid;
use crate::middleware::auth::RequireAuth;
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
    RequireAuth(auth_user): RequireAuth,
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
                    'settlement', settlement_json,
                    'settlement_signature', settlement_signature,
                    'created_at', created_at
                ),
                '{}'::JSONB
            ) AS replay
        FROM hand_history
        WHERE id::TEXT = $1
          AND (
              $2
              OR EXISTS (
                  SELECT 1 FROM hand_participants participant
                  WHERE participant.hand_id = hand_history.id
                    AND participant.user_id = $3::uuid
              )
          )
        "#,
    )
    .bind(&hand_id)
    .bind(auth_user.role == "admin")
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db)
    .await?;

    let (id, mut replay) =
        row.ok_or_else(|| ApiError::NotFound(format!("Hand history {hand_id} not found")))?;

    let settlement = replay
        .get("settlement")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    if settlement
        .as_object()
        .is_some_and(|value| !value.is_empty())
    {
        let signature = replay
            .get("settlement_signature")
            .and_then(Value::as_str)
            .ok_or_else(|| ApiError::Internal("Settlement signature is missing".to_string()))?;
        if !settlement_signature_valid(&settlement, signature, state.jwt_secret.as_bytes()) {
            return Err(ApiError::Internal(
                "Settlement signature verification failed".to_string(),
            ));
        }
        replay["settlement_verified"] = Value::Bool(true);
    } else {
        replay["settlement_verified"] = Value::Null;
    }

    Ok(Json(HandHistoryResponse {
        hand_id: id,
        replay,
    }))
}

/// GET /api/tables/{table_id}/history
/// Response: Lista das últimas 50 mãos finalizadas da mesa
pub async fn list_table_hand_histories(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
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
        WHERE table_id::TEXT = $1
          AND (
              $2
              OR EXISTS (
                  SELECT 1 FROM hand_participants participant
                  WHERE participant.hand_id = hand_history.id
                    AND participant.user_id = $3::uuid
              )
          )
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&table_id)
    .bind(auth_user.role == "admin")
    .bind(&auth_user.user_id)
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
