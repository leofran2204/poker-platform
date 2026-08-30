// Unified error type for all API handlers.
// Converts to API response status codes via `IntoResponse`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    /// 400 — malformed request, validation failure
    BadRequest(String),
    /// 401 — missing or invalid token
    Unauthorized(String),
    /// 403 — authenticated but not allowed
    Forbidden(String),
    /// 404 — resource not found
    NotFound(String),
    /// 409 — conflict (duplicate, already exists)
    Conflict(String),
    /// 429 — rate limit exceeded
    TooManyRequests(String),
    /// 500 — internal server error (DB, panic, etc.)
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ApiError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone()),
            // Do not expose SQL, Redis, or infrastructure details to clients.
            ApiError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        if status.is_server_error() {
            tracing::error!(error = ?self, "API error: {} — {}", status, message);
        } else if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::FORBIDDEN {
            tracing::warn!(error = ?self, "API request rejected: {} — {}", status, message);
        } else {
            // Invalid input, expired credentials and not-found responses are
            // expected outcomes and must not trigger operational error alerts.
            tracing::info!(error = ?self, "API request rejected: {} — {}", status, message);
        }

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                ApiError::Conflict("Resource already exists".to_string())
            }
            _ => ApiError::Internal(format!("Database error: {err}")),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON error: {err}"))
    }
}
