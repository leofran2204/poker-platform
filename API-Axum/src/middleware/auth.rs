// Auth middleware — extracts and validates JWT from Authorization header

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::HeaderMap;

use crate::error::ApiError;
use crate::state::AppState;

/// Authenticated user info extracted from JWT
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: String,
}

/// Extractor that validates the Bearer token and returns AuthUser
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthUser);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract token from Authorization header
        let token = extract_bearer_token(&parts.headers).ok_or_else(|| {
            ApiError::Unauthorized("Missing or invalid Authorization header".to_string())
        })?;

        // Validate token (parallel read lock)
        let auth = app_state.auth.read().await;
        let claims = auth.validate_token(&token, "access").map_err(|e| match e {
            poker_engine::auth::AuthResult::TokenExpired => {
                ApiError::Unauthorized("Token expired".to_string())
            }
            poker_engine::auth::AuthResult::TokenInvalid => {
                ApiError::Unauthorized("Invalid token".to_string())
            }
            _ => ApiError::Unauthorized("Authentication failed".to_string()),
        })?;

        Ok(RequireAuth(AuthUser {
            user_id: claims.sub,
            username: claims.username,
            role: format!("{:?}", claims.role).to_lowercase(),
        }))
    }
}

/// Extracts Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}
