//! Presence endpoints — contador de usuários online (heartbeat + leitura pública).

use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::presence::PRESENCE_TTL_SECS;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct OnlinePresenceResponse {
    pub online_count: u64,
    pub ttl_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub ok: bool,
    pub online_count: u64,
    pub ttl_seconds: u64,
}

/// GET /api/presence/online — público (amigos veem quantos estão logados).
pub async fn online_count(
    State(state): State<AppState>,
) -> Result<Json<OnlinePresenceResponse>, ApiError> {
    let online_count = state
        .presence
        .online_count(&state.redis)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(OnlinePresenceResponse {
        online_count,
        ttl_seconds: PRESENCE_TTL_SECS,
    }))
}

/// POST /api/presence/heartbeat — autenticado; renova presença do usuário.
pub async fn heartbeat(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    let _ = crate::wallet::ensure_pm_daily_reset_pool(&state.db, &user.user_id).await;
    state
        .presence
        .heartbeat(&state.redis, &user.user_id)
        .await
        .map_err(ApiError::Internal)?;
    let online_count = state
        .presence
        .online_count(&state.redis)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(HeartbeatResponse {
        ok: true,
        online_count,
        ttl_seconds: PRESENCE_TTL_SECS,
    }))
}

/// POST /api/presence/offline — remove imediatamente o usuário no logout.
pub async fn offline(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    state
        .presence
        .remove(&state.redis, &user.user_id)
        .await
        .map_err(ApiError::Internal)?;
    let online_count = state
        .presence
        .online_count(&state.redis)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(HeartbeatResponse {
        ok: true,
        online_count,
        ttl_seconds: PRESENCE_TTL_SECS,
    }))
}
