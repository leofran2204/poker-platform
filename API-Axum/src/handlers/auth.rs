// Auth handlers — POST /api/auth/register, /login, /mfa/verify, /refresh,
//                 /verify-email, /resend-verification

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;

use crate::email_service::{
    codes_equal_hash, generate_numeric_code, hash_code, send_verification_email, CODE_TTL_SECS,
};
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
    /// Confirmação de senha (obrigatória quando require_email_verification;
    /// se enviada, deve sempre coincidir).
    #[serde(default)]
    pub password_confirm: Option<String>,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailBody {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ResendVerificationBody {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct MfaVerifyBody {
    pub challenge: String,
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

fn account_status_db(status: &poker_engine::auth::AccountStatus) -> &'static str {
    match status {
        poker_engine::auth::AccountStatus::Active => "active",
        poker_engine::auth::AccountStatus::Suspended => "suspended",
        poker_engine::auth::AccountStatus::Banned => "banned",
        poker_engine::auth::AccountStatus::PendingEmailVerification => "pending_email_verification",
    }
}

fn role_db(role: &poker_engine::auth::UserRole) -> &'static str {
    match role {
        poker_engine::auth::UserRole::Player => "player",
        poker_engine::auth::UserRole::Admin => "admin",
        poker_engine::auth::UserRole::Moderator => "moderator",
    }
}

fn passwords_match(password: &str, confirm: &Option<String>) -> bool {
    match confirm {
        Some(c) => password == c,
        None => true, // campo omitido: aceito se flag de verificação desligada
    }
}

async fn issue_verification_code(
    state: &AppState,
    user_id: &str,
    email: &str,
    username: &str,
) -> Result<(), ApiError> {
    let code = generate_numeric_code();
    let code_hash = hash_code(&code);
    let now = now_epoch() as i64;
    let expires_at = now + CODE_TTL_SECS as i64;

    // Invalida códigos anteriores não consumidos
    sqlx::query(
        "UPDATE email_verification_codes SET consumed_at = $1 \
         WHERE user_id = $2::uuid AND consumed_at IS NULL",
    )
    .bind(now)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO email_verification_codes (user_id, code_hash, expires_at, created_at) \
         VALUES ($1::uuid, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(&code_hash)
    .bind(expires_at)
    .bind(now)
    .execute(&state.db)
    .await?;

    send_verification_email(email, username, &code)
        .await
        .map_err(ApiError::Internal)?;
    Ok(())
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

fn password_work_slots() -> Arc<Semaphore> {
    static SLOTS: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let permits = std::env::var("AUTH_PASSWORD_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(4);
    Arc::clone(SLOTS.get_or_init(|| Arc::new(Semaphore::new(permits))))
}

async fn register_user_off_runtime(
    request: poker_engine::auth::RegisterRequest,
    jwt_secret: String,
) -> Result<Result<poker_engine::auth::User, poker_engine::auth::AuthResult>, ApiError> {
    let _permit = password_work_slots()
        .acquire_owned()
        .await
        .map_err(|_| ApiError::Internal("Password worker pool is unavailable".to_string()))?;
    tokio::task::spawn_blocking(move || {
        let mut auth = poker_engine::auth::AuthManager::new(&jwt_secret);
        auth.register_user(&request)
    })
    .await
    .map_err(|error| ApiError::Internal(format!("Password worker failed: {error}")))
}

async fn verify_password_off_runtime(
    password: String,
    password_hash: String,
) -> Result<bool, ApiError> {
    let _permit = password_work_slots()
        .acquire_owned()
        .await
        .map_err(|_| ApiError::Internal("Password worker pool is unavailable".to_string()))?;
    tokio::task::spawn_blocking(move || {
        poker_engine::auth::verify_password_hash(&password, &password_hash)
    })
    .await
    .map_err(|error| ApiError::Internal(format!("Password worker failed: {error}")))
}
const MFA_CHALLENGE_TTL_SECS: i64 = 5 * 60;
const MFA_CHALLENGE_MAX_ATTEMPTS: i16 = 5;
const MFA_CHALLENGE_RETENTION_SECS: i64 = 24 * 60 * 60;

fn hash_mfa_challenge(challenge: &str) -> String {
    format!("{:x}", Sha256::digest(challenge.as_bytes()))
}

async fn create_mfa_challenge(state: &AppState, user_id: &str) -> Result<String, ApiError> {
    let challenge = format!("{}.{}", uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
    let token_hash = hash_mfa_challenge(&challenge);
    let now = now_epoch() as i64;
    let expires_at = now + MFA_CHALLENGE_TTL_SECS;
    let mut tx = state.db.begin().await?;
    // Limpeza oportunista, limitada e indexada: mantém o request path curto
    // mesmo após longos períodos de operação.
    sqlx::query("DELETE FROM auth_mfa_challenges WHERE id IN (SELECT id FROM auth_mfa_challenges WHERE expires_at < $1 ORDER BY expires_at LIMIT 1000)")
        .bind(now - MFA_CHALLENGE_RETENTION_SECS)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE auth_mfa_challenges SET consumed_at = $1 \
         WHERE user_id = $2::uuid AND consumed_at IS NULL",
    )
    .bind(now)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO auth_mfa_challenges (user_id, token_hash, expires_at, created_at) \
         VALUES ($1::uuid, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(challenge)
}
// ─── Handlers ───

/// POST /api/auth/register
///
/// Com `REQUIRE_EMAIL_VERIFICATION=true` (padrão em runtime de produto):
/// `{email, password, password_confirm, username}` →
/// `{email_verification_required, email, username, message}` (sem JWT).
///
/// Com flag desligada (testes): devolve tokens como antes.
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if state.require_email_verification {
        match &body.password_confirm {
            None => {
                return Err(ApiError::BadRequest(
                    "password_confirm is required".to_string(),
                ));
            }
            Some(confirm) if confirm != &body.password => {
                return Err(ApiError::BadRequest(
                    "password and password_confirm do not match".to_string(),
                ));
            }
            Some(_) => {}
        }
    } else if !passwords_match(&body.password, &body.password_confirm) {
        return Err(ApiError::BadRequest(
            "password and password_confirm do not match".to_string(),
        ));
    }

    let request = poker_engine::auth::RegisterRequest {
        username: body.username.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
    };

    let mut user = register_user_off_runtime(request, state.jwt_secret.clone())
        .await?
        .map_err(|error| match error {
            poker_engine::auth::AuthResult::UsernameAlreadyExists => {
                ApiError::Conflict("Username already exists".to_string())
            }
            poker_engine::auth::AuthResult::EmailAlreadyExists => {
                ApiError::Conflict("Email already exists".to_string())
            }
            poker_engine::auth::AuthResult::PasswordTooWeak => ApiError::BadRequest(
                "Password too weak (min 8 chars, 1 upper, 1 lower, 1 digit)".to_string(),
            ),
            poker_engine::auth::AuthResult::InvalidEmail => {
                ApiError::BadRequest("Invalid email".to_string())
            }
            _ => ApiError::Internal(format!("Auth error: {error:?}")),
        })?;

    if state.require_email_verification {
        user.status = poker_engine::auth::AccountStatus::PendingEmailVerification;
    }

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
    .bind(role_db(&user.role))
    .bind(account_status_db(&user.status))
    .bind(user.balance)
    .bind(user.mfa_enabled)
    .bind(user.created_at as i64)
    .execute(&state.db)
    .await;
    if let Err(error) = persist_result {
        return Err(error.into());
    }

    state.auth.write().await.upsert_persisted_user(user.clone());

    if state.require_email_verification {
        if let Err(e) = issue_verification_code(&state, &user.id, &user.email, &user.username).await
        {
            tracing::error!(error = ?e, "failed to issue verification code after register");
            // Conta criada; usuário pode reenviar. Não desfaz o registro.
        }
        return Ok(Json(json!({
            "email_verification_required": true,
            "email": user.email,
            "username": user.username,
            "message": "Conta criada. Verifique seu e-mail e informe o código de 6 dígitos para ativar o acesso.",
        })));
    }

    // Generate token pair (modo legado / testes)
    let tokens = state
        .auth
        .read()
        .await
        .issue_tokens_for_user(&user)
        .map_err(|error| ApiError::Internal(format!("Token issue failed: {error:?}")))?;

    let expires_in = tokens.expires_at.saturating_sub(now_epoch());

    Ok(Json(json!({
        "token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "expires_in": expires_in,
        "email_verification_required": false,
    })))
}

/// POST /api/auth/login
/// Request: `{email, password}` → Response: `{token, mfa_required?}`
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // PostgreSQL is authoritative. Bcrypt runs before the row lock, while the
    // counter transition is serialized so independent replicas cannot lose
    // failed attempts through read-modify-write races.
    let snapshot = load_persisted_user(&state, "email", &body.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;
    let password_valid =
        verify_password_off_runtime(body.password, snapshot.password_hash.clone()).await?;
    let now = now_epoch() as i64;
    let mut tx = state.db.begin().await?;
    let current_row: Option<PersistedUserRow> = sqlx::query_as(
        concat!(
            "SELECT id::text, username, email, password_hash, role, status, balance, ",
            "mfa_enabled, mfa_secret, failed_login_attempts, locked_until, created_at, last_login, token_version ",
            "FROM users WHERE id = $1::uuid FOR UPDATE"
        ),
    )
    .bind(&snapshot.id)
    .fetch_optional(&mut *tx)
    .await?;
    let mut user = current_row
        .map(persisted_user_from_row)
        .transpose()?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    // A password change between the initial read and the row lock invalidates
    // this attempt. The client can retry against the new credential.
    if user.password_hash != snapshot.password_hash {
        tx.rollback().await?;
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    if user
        .locked_until
        .is_some_and(|locked_until| locked_until > now as u64)
    {
        tx.rollback().await?;
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    if !password_valid {
        let previous_lock_expired = user
            .locked_until
            .is_some_and(|locked_until| locked_until <= now as u64);
        let base_attempts = if previous_lock_expired {
            0
        } else {
            user.failed_login_attempts
        };
        let next_attempts = base_attempts.saturating_add(1);
        let next_locked_until = (next_attempts >= poker_engine::auth::MAX_LOGIN_ATTEMPTS)
            .then_some(
                now + i64::try_from(poker_engine::auth::LOCKOUT_DURATION_SECS)
                    .expect("lockout duration fits i64"),
            );
        sqlx::query(
            "UPDATE users SET failed_login_attempts = $1, locked_until = $2 WHERE id = $3::uuid",
        )
        .bind(i32::try_from(next_attempts).map_err(|_| {
            ApiError::Internal("Login attempt counter exceeds database range".to_string())
        })?)
        .bind(next_locked_until)
        .bind(&user.id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    match user.status {
        poker_engine::auth::AccountStatus::Suspended => {
            tx.rollback().await?;
            return Err(ApiError::Forbidden("Account suspended".to_string()));
        }
        poker_engine::auth::AccountStatus::Banned => {
            tx.rollback().await?;
            return Err(ApiError::Forbidden("Account banned".to_string()));
        }
        poker_engine::auth::AccountStatus::Active
        | poker_engine::auth::AccountStatus::PendingEmailVerification => {}
    }

    let username = user.username.clone();
    if user.mfa_enabled {
        tx.commit().await?;
        state.auth.write().await.upsert_persisted_user(user.clone());
        let challenge = create_mfa_challenge(&state, &user.id).await?;
        return Ok(Json(json!({
            "mfa_required": true,
            "mfa_challenge": challenge,
            "expires_in": MFA_CHALLENGE_TTL_SECS,
            "message": "MFA code required.",
        })));
    }

    sqlx::query(
        "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, last_login = $1 WHERE id = $2::uuid",
    )
    .bind(now)
    .bind(&user.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    user.failed_login_attempts = 0;
    user.locked_until = None;
    user.last_login = Some(now as u64);
    state.auth.write().await.upsert_persisted_user(user.clone());

    if state.require_email_verification
        && user.status == poker_engine::auth::AccountStatus::PendingEmailVerification
    {
        return Ok(Json(json!({
            "email_verification_required": true,
            "email": body.email.to_lowercase(),
            "username": username,
            "message": "Confirme o código enviado ao seu e-mail para ativar a conta.",
            "mfa_required": false,
        })));
    }

    let tokens = state
        .auth
        .read()
        .await
        .issue_tokens_for_user(&user)
        .map_err(|error| ApiError::Internal(format!("Token issue failed: {error:?}")))?;
    let expires_in = tokens.expires_at.saturating_sub(now_epoch());
    Ok(Json(json!({
        "token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "expires_in": expires_in,
        "mfa_required": false,
        "email_verification_required": false,
        "username": username,
    })))
}

/// POST /api/auth/verify-email
/// Request: `{email, code}` → tokens JWT se o código for válido.
pub async fn verify_email(
    State(state): State<AppState>,
    Json(body): Json<VerifyEmailBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let email = body.email.trim().to_lowercase();
    let code = body.code.trim();
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest(
            "Code must be a 6-digit number".to_string(),
        ));
    }

    let user = load_persisted_user(&state, "email", &email)
        .await?
        .ok_or_else(|| ApiError::BadRequest("Invalid verification request".to_string()))?;

    if user.status == poker_engine::auth::AccountStatus::Active {
        // Já ativo: devolve tokens se souber a senha? Não — pede login.
        return Ok(Json(json!({
            "already_verified": true,
            "message": "E-mail já verificado. Faça login.",
        })));
    }
    if user.status != poker_engine::auth::AccountStatus::PendingEmailVerification {
        return Err(ApiError::Forbidden(
            "Account cannot be verified in its current state".to_string(),
        ));
    }

    let now = now_epoch() as i64;
    let row: Option<(uuid::Uuid, String, i64)> = sqlx::query_as(
        "SELECT id, code_hash, expires_at FROM email_verification_codes \
         WHERE user_id = $1::uuid AND consumed_at IS NULL \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await?;

    let (code_id, code_hash, expires_at) =
        row.ok_or_else(|| ApiError::BadRequest("No active verification code".to_string()))?;

    if expires_at < now {
        return Err(ApiError::BadRequest(
            "Verification code expired. Request a new one.".to_string(),
        ));
    }
    if !codes_equal_hash(code, &code_hash) {
        return Err(ApiError::Unauthorized(
            "Invalid verification code".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    sqlx::query("UPDATE email_verification_codes SET consumed_at = $1 WHERE id = $2")
        .bind(now)
        .bind(code_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE users SET status = 'active', email_verified_at = $1 WHERE id = $2::uuid")
        .bind(now)
        .bind(&user.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let mut active_user = user;
    active_user.status = poker_engine::auth::AccountStatus::Active;
    {
        let mut auth = state.auth.write().await;
        auth.upsert_persisted_user(active_user.clone());
    }

    // Emite tokens sem revalidar senha (prova de posse do e-mail + código)
    let tokens = {
        let auth = state.auth.read().await;
        auth.issue_tokens_for_user(&active_user)
            .map_err(|e| ApiError::Internal(format!("Token issue failed: {e:?}")))?
    };

    let expires_in = tokens.expires_at.saturating_sub(now_epoch());
    Ok(Json(json!({
        "token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "expires_in": expires_in,
        "email_verification_required": false,
        "username": active_user.username,
        "message": "E-mail confirmado. Bem-vindo à Zero Tilt — o lobby é seu.",
    })))
}

/// POST /api/auth/resend-verification
/// Request: `{email}` — sempre responde generico (anti-enumeration).
pub async fn resend_verification(
    State(state): State<AppState>,
    Json(body): Json<ResendVerificationBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let email = body.email.trim().to_lowercase();
    let generic = Json(json!({
        "ok": true,
        "message": "Se o e-mail existir e estiver pendente, enviamos um novo código.",
    }));

    let Some(user) = load_persisted_user(&state, "email", &email).await? else {
        return Ok(generic);
    };
    if user.status != poker_engine::auth::AccountStatus::PendingEmailVerification {
        return Ok(generic);
    }

    // Rate limit simples: no máximo 1 reenvio a cada 60s
    let now = now_epoch() as i64;
    let last: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(created_at) FROM email_verification_codes WHERE user_id = $1::uuid",
    )
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await?
    .flatten();
    if let Some(created) = last {
        if now - created < 60 {
            return Err(ApiError::BadRequest(
                "Aguarde 60 segundos antes de reenviar o código".to_string(),
            ));
        }
    }

    let _ = issue_verification_code(&state, &user.id, &user.email, &user.username).await;
    Ok(generic)
}

/// Completes a password-authenticated login with a durable, single-use MFA challenge.
pub async fn mfa_verify(
    State(state): State<AppState>,
    Json(body): Json<MfaVerifyBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let challenge = body.challenge.trim();
    let code = body.code.trim();
    if challenge.len() != 73 || code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ApiError::Unauthorized(
            "Invalid MFA credentials".to_string(),
        ));
    }

    let now = now_epoch() as i64;
    let token_hash = hash_mfa_challenge(challenge);
    let mut tx = state.db.begin().await?;
    let challenge_row: Option<(uuid::Uuid, String, i64, i16)> = sqlx::query_as(
        "SELECT id, user_id::text, expires_at, attempts \
         FROM auth_mfa_challenges \
         WHERE token_hash = $1 AND consumed_at IS NULL \
         FOR UPDATE",
    )
    .bind(token_hash)
    .fetch_optional(&mut *tx)
    .await?;
    let (challenge_id, user_id, expires_at, attempts) = challenge_row
        .ok_or_else(|| ApiError::Unauthorized("Invalid MFA credentials".to_string()))?;

    if expires_at <= now || attempts >= MFA_CHALLENGE_MAX_ATTEMPTS {
        sqlx::query(
            "UPDATE auth_mfa_challenges SET consumed_at = COALESCE(consumed_at, $1) WHERE id = $2",
        )
        .bind(now)
        .bind(challenge_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Err(ApiError::Unauthorized(
            "Invalid MFA credentials".to_string(),
        ));
    }

    let user_row: Option<PersistedUserRow> = sqlx::query_as(
        "SELECT id::text, username, email, password_hash, role, status, balance, \
         mfa_enabled, mfa_secret, failed_login_attempts, locked_until, created_at, last_login, token_version \
         FROM users WHERE id = $1::uuid FOR UPDATE",
    )
    .bind(&user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let mut user = user_row
        .map(persisted_user_from_row)
        .transpose()?
        .ok_or_else(|| ApiError::Unauthorized("Invalid MFA credentials".to_string()))?;

    if user.status != poker_engine::auth::AccountStatus::Active || !user.mfa_enabled {
        sqlx::query("UPDATE auth_mfa_challenges SET consumed_at = $1 WHERE id = $2")
            .bind(now)
            .bind(challenge_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Err(ApiError::Unauthorized(
            "Invalid MFA credentials".to_string(),
        ));
    }

    let valid = {
        let mut auth = poker_engine::auth::AuthManager::new(&state.jwt_secret);
        auth.upsert_persisted_user(user.clone());
        auth.verify_mfa_for_user(&user.username, code)
            .unwrap_or(false)
    };
    if !valid {
        let next_attempt = attempts.saturating_add(1);
        sqlx::query(
            "UPDATE auth_mfa_challenges \
             SET attempts = $1, consumed_at = CASE WHEN $1 >= $2 THEN $3 ELSE consumed_at END \
             WHERE id = $4",
        )
        .bind(next_attempt)
        .bind(MFA_CHALLENGE_MAX_ATTEMPTS)
        .bind(now)
        .bind(challenge_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Err(ApiError::Unauthorized(
            "Invalid MFA credentials".to_string(),
        ));
    }

    sqlx::query("UPDATE auth_mfa_challenges SET consumed_at = $1 WHERE id = $2")
        .bind(now)
        .bind(challenge_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, last_login = $1 \
         WHERE id = $2::uuid",
    )
    .bind(now)
    .bind(&user.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    user.failed_login_attempts = 0;
    user.locked_until = None;
    user.last_login = Some(now as u64);
    state.auth.write().await.upsert_persisted_user(user.clone());
    let tokens = state
        .auth
        .read()
        .await
        .issue_tokens_for_user(&user)
        .map_err(|error| ApiError::Internal(format!("Token issue failed: {error:?}")))?;
    let expires_in = tokens.expires_at.saturating_sub(now_epoch());

    Ok(Json(json!({
        "token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "expires_in": expires_in,
        "mfa_verified": true,
        "username": user.username,
    })))
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

/// GET /api/auth/me — current authenticated user profile (no secrets).
#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub status: String,
    pub balance: i64,
    pub email: String,
}

pub async fn me(
    crate::middleware::auth::RequireAuth(auth_user): crate::middleware::auth::RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<MeResponse>, ApiError> {
    let row: Option<(String, String, String, String, i64, String)> = sqlx::query_as(
        "SELECT id::text, username, role, status, balance, email FROM users WHERE id = $1::uuid",
    )
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db)
    .await?;

    let (user_id, username, role, status, balance, email) = row.ok_or_else(|| {
        ApiError::NotFound("User not found".to_string())
    })?;

    Ok(Json(MeResponse {
        user_id,
        username,
        role,
        status,
        balance,
        email,
    }))
}
