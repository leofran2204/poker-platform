// Auth handlers — POST /api/auth/register, /login, /mfa/verify, /refresh

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::ApiError;
use crate::state::AppState;

type PersistedUserRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    bool,
    Option<String>,
    i32,
    Option<i64>,
    i64,
    Option<i64>,
    i64,
);

// ─── Request / Response DTOs ───

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MfaVerifyBody {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshBody {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub message: String,
}

// ─── Helpers ───

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn persisted_user_from_row(row: PersistedUserRow) -> Result<poker_engine::auth::User, ApiError> {
    let (
        id,
        username,
        email,
        password_hash,
        role,
        status,
        balance,
        mfa_enabled,
        mfa_secret,
        failed_login_attempts,
        locked_until,
        created_at,
        last_login,
        token_version,
    ) = row;

    let role = match role.as_str() {
        "player" => poker_engine::auth::UserRole::Player,
        "admin" => poker_engine::auth::UserRole::Admin,
        "moderator" => poker_engine::auth::UserRole::Moderator,
        _ => {
            return Err(ApiError::Internal(
                "Persisted user has an invalid role".to_string(),
            ))
        }
    };
    let status = match status.as_str() {
        "active" => poker_engine::auth::AccountStatus::Active,
        "suspended" => poker_engine::auth::AccountStatus::Suspended,
        "banned" => poker_engine::auth::AccountStatus::Banned,
        "pending_email_verification" => poker_engine::auth::AccountStatus::PendingEmailVerification,
        _ => {
            return Err(ApiError::Internal(
                "Persisted user has an invalid account status".to_string(),
            ))
        }
    };

    Ok(poker_engine::auth::User {
        id,
        username,
        email,
        password_hash,
        role,
        status,
        balance,
        mfa_enabled,
        mfa_secret,
        failed_login_attempts: u32::try_from(failed_login_attempts)
            .map_err(|_| ApiError::Internal("Persisted login attempts are invalid".to_string()))?,
        locked_until: locked_until
            .map(u64::try_from)
            .transpose()
            .map_err(|_| ApiError::Internal("Persisted lock timestamp is invalid".to_string()))?,
        created_at: u64::try_from(created_at).map_err(|_| {
            ApiError::Internal("Persisted creation timestamp is invalid".to_string())
        })?,
        last_login: last_login
            .map(u64::try_from)
            .transpose()
            .map_err(|_| ApiError::Internal("Persisted login timestamp is invalid".to_string()))?,
        token_version,
    })
}

async fn load_persisted_user(
    state: &AppState,
    predicate: &str,
    value: &str,
) -> Result<Option<poker_engine::auth::User>, ApiError> {
    let query = format!(
        "SELECT id::text, username, email, password_hash, role, status, balance, \
         mfa_enabled, mfa_secret, failed_login_attempts, locked_until, created_at, last_login, token_version \
         FROM users WHERE {predicate} = $1"
    );
    let row: Option<PersistedUserRow> = sqlx::query_as(&query)
        .bind(value)
        .fetch_optional(&state.db)
        .await?;
    row.map(persisted_user_from_row).transpose()
}

async fn persist_login_state(
    state: &AppState,
    user: &poker_engine::auth::User,
) -> Result<(), ApiError> {
    let failed_login_attempts = i32::try_from(user.failed_login_attempts).map_err(|_| {
        ApiError::Internal("Login attempt counter exceeds database range".to_string())
    })?;
    let locked_until = user
        .locked_until
        .map(i64::try_from)
        .transpose()
        .map_err(|_| ApiError::Internal("Lock timestamp exceeds database range".to_string()))?;
    let last_login = user
        .last_login
        .map(i64::try_from)
        .transpose()
        .map_err(|_| ApiError::Internal("Login timestamp exceeds database range".to_string()))?;

    sqlx::query(
        "UPDATE users SET failed_login_attempts = $1, locked_until = $2, last_login = $3 \
         WHERE id = $4::uuid",
    )
    .bind(failed_login_attempts)
    .bind(locked_until)
    .bind(last_login)
    .bind(&user.id)
    .execute(&state.db)
    .await?;
    Ok(())
}

// ─── Handlers ───

/// POST /api/auth/register
/// Request: `{email, password, username}` → Response: `{token, expires_in}`
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let request = poker_engine::auth::RegisterRequest {
        username: body.username.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
    };

    let user = {
        let mut auth = state.auth.write().await;
        auth.register_user(&request).map_err(|e| match e {
            poker_engine::auth::AuthResult::UsernameAlreadyExists => {
                ApiError::Conflict("Username already exists".to_string())
            }
            poker_engine::auth::AuthResult::EmailAlreadyExists => {
                ApiError::Conflict("Email already exists".to_string())
            }
            poker_engine::auth::AuthResult::PasswordTooWeak => {
                ApiError::BadRequest("Password too weak".to_string())
            }
            poker_engine::auth::AuthResult::InvalidEmail => {
                ApiError::BadRequest("Invalid email".to_string())
            }
            _ => ApiError::Internal(format!("Auth error: {e:?}")),
        })?
    };

    // Persist user to PostgreSQL
    let persist_result = sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, role, status, balance, mfa_enabled, created_at)
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(format!("{:?}", user.role).to_lowercase())
    .bind(format!("{:?}", user.status).to_lowercase())
    .bind(user.balance)
    .bind(user.mfa_enabled)
    .bind(user.created_at as i64)
    .execute(&state.db)
    .await;
    if let Err(error) = persist_result {
        state.auth.write().await.remove_user(&user.username);
        return Err(error.into());
    }

    // Generate token pair
    let login_req = poker_engine::auth::LoginRequest {
        username: body.username,
        password: body.password,
        mfa_code: None,
    };

    let tokens = state
        .auth
        .write()
        .await
        .login(&login_req)
        .map_err(|e| ApiError::Internal(format!("Login after register failed: {e:?}")))?;

    let expires_in = tokens.expires_at.saturating_sub(now_epoch());

    Ok(Json(TokenResponse {
        token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in,
    }))
}

/// POST /api/auth/login
/// Request: `{email, password}` → Response: `{token, mfa_required?}`
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // PostgreSQL is authoritative across API restarts; the AuthManager is a
    // short-lived in-memory verifier/cache.
    let user = load_persisted_user(&state, "email", &body.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;
    let username = user.username.clone();

    let request = poker_engine::auth::LoginRequest {
        username: username.clone(),
        password: body.password,
        mfa_code: None,
    };
    let (login_result, updated_user) = {
        let mut auth = state.auth.write().await;
        auth.upsert_persisted_user(user);
        let login_result = auth.login(&request);
        let updated_user = auth.get_user(&username).cloned();
        (login_result, updated_user)
    };
    if let Some(updated_user) = updated_user {
        persist_login_state(&state, &updated_user).await?;
    }

    // First attempt — may return MfaRequired
    match login_result {
        Ok(tokens) => {
            let expires_in = tokens.expires_at.saturating_sub(now_epoch());
            Ok(Json(json!({
                "token": tokens.access_token,
                "refresh_token": tokens.refresh_token,
                "expires_in": expires_in,
                "mfa_required": false,
            })))
        }
        Err(poker_engine::auth::AuthResult::MfaRequired) => Ok(Json(json!({
            "mfa_required": true,
            "message": "MFA code required. POST /api/auth/mfa/verify with your code.",
        }))),
        Err(poker_engine::auth::AuthResult::InvalidCredentials) => {
            Err(ApiError::Unauthorized("Invalid credentials".to_string()))
        }
        Err(poker_engine::auth::AuthResult::AccountSuspended) => {
            Err(ApiError::Forbidden("Account suspended".to_string()))
        }
        Err(poker_engine::auth::AuthResult::AccountBanned) => {
            Err(ApiError::Forbidden("Account banned".to_string()))
        }
        Err(e) => Err(ApiError::Internal(format!("Login error: {e:?}"))),
    }
}

#[allow(dead_code)]
/// POST /api/auth/mfa/verify
/// Request: `{code}` → Response: `{token}` or 401 Unauthorized
pub async fn mfa_verify(
    State(_state): State<AppState>,
    Json(_body): Json<MfaVerifyBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // We need the username from the pending MFA state.
    // In a real system, this would come from a pending-MFA token.
    // For now, we require the username in the body extension.
    // This endpoint is typically called right after login returns MfaRequired.
    //
    // The AuthManager stores the last attempted login user internally,
    // but since it doesn't expose that, we use verify_mfa_for_user.
    // The frontend should send the username alongside the code.
    //
    // For the contract test, we accept a `username` field too.
    tracing::warn!("MFA verify endpoint requires username in body — see contract");

    Err(ApiError::BadRequest(
        "MFA verify requires username field. Use POST /api/auth/mfa/verify with {username, code}."
            .to_string(),
    ))
}

/// POST /api/auth/mfa/verify (extended with username)
/// Request: `{username, code}` → Response: `{token}` or 401
pub async fn mfa_verify_with_username(
    State(state): State<AppState>,
    Json(body): Json<MfaVerifyBodyWithUsername>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = load_persisted_user(&state, "username", &body.username)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid MFA credentials".to_string()))?;
    let mut auth = state.auth.write().await;
    auth.upsert_persisted_user(user);

    let valid = auth
        .verify_mfa_for_user(&body.username, &body.code)
        .map_err(|e| match e {
            poker_engine::auth::AuthResult::MfaFailed => {
                ApiError::Unauthorized("Invalid MFA code".to_string())
            }
            _ => ApiError::Internal(format!("MFA verify error: {e:?}")),
        })?;

    if !valid {
        return Err(ApiError::Unauthorized("Invalid MFA code".to_string()));
    }

    Ok(Json(json!({
        "mfa_verified": true,
        "message": "MFA verified. Please complete login with password.",
    })))
}

#[derive(Debug, Deserialize)]
pub struct MfaVerifyBodyWithUsername {
    pub username: String,
    pub code: String,
}

/// POST /api/auth/refresh
/// Request: `{refresh_token}` → Response: `{token, expires_in}`
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let request = poker_engine::auth::RefreshRequest {
        refresh_token: body.refresh_token,
    };
    let claims = state
        .auth
        .read()
        .await
        .validate_token(&request.refresh_token, "refresh")
        .map_err(|e| match e {
            poker_engine::auth::AuthResult::TokenExpired => {
                ApiError::Unauthorized("Refresh token expired".to_string())
            }
            _ => ApiError::Unauthorized("Invalid refresh token".to_string()),
        })?;
    let user = load_persisted_user(&state, "id::text", &claims.sub)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid refresh token".to_string()))?;
    let tokens = {
        let mut auth = state.auth.write().await;
        auth.upsert_persisted_user(user);
        auth.refresh_access_token(&request)
    }
    .map_err(|e| match e {
        poker_engine::auth::AuthResult::TokenExpired => {
            ApiError::Unauthorized("Refresh token expired".to_string())
        }
        poker_engine::auth::AuthResult::TokenInvalid => {
            ApiError::Unauthorized("Invalid refresh token".to_string())
        }
        _ => ApiError::Internal(format!("Refresh error: {e:?}")),
    })?;

    let expires_in = tokens.expires_at.saturating_sub(now_epoch());

    Ok(Json(TokenResponse {
        token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in,
    }))
}
