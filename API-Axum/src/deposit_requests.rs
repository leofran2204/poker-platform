//! Play-money chip requests: manual PIX outside the site + admin approval.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

fn require_admin(auth_user: &crate::middleware::auth::AuthUser) -> Result<(), ApiError> {
    if auth_user.role == "admin" {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "Administrator access is required".to_string(),
        ))
    }
}

fn env_pix_key() -> String {
    std::env::var("PLAY_MONEY_PIX_KEY")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn env_receiver_name() -> String {
    std::env::var("PLAY_MONEY_PIX_RECEIVER_NAME")
        .unwrap_or_else(|_| "Zero Tilt".to_string())
        .trim()
        .to_string()
}

fn env_max_cents() -> i64 {
    std::env::var("DEPOSIT_MAX_CENTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500_000)
        .max(100)
}

fn env_max_pending() -> i64 {
    std::env::var("DEPOSIT_MAX_PENDING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
        .clamp(1, 10)
}

async fn write_audit(
    state: &AppState,
    user_id: &str,
    action: &str,
    metadata: serde_json::Value,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(action)
        .bind(metadata)
        .execute(&state.db)
        .await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct DepositInfoResponse {
    pub available: bool,
    pub pix_key: String,
    pub receiver_name: String,
    pub max_cents: i64,
    pub max_pending: i64,
    pub presets_cents: Vec<i64>,
    pub instructions: String,
    pub automated_available: bool,
    pub automated_provider: Option<String>,
    pub automated_mode: Option<String>,
}

pub async fn deposit_info(
    RequireAuth(auth): RequireAuth,
) -> Result<Json<DepositInfoResponse>, ApiError> {
    let pix_key = env_pix_key();
    let available = !pix_key.is_empty();
    let automated_provider = std::env::var("PIX_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .trim()
        .to_ascii_lowercase();
    let automated_mode = std::env::var("PIX_MODE")
        .unwrap_or_else(|_| "mock".to_string())
        .trim()
        .to_ascii_lowercase();
    let environment = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .trim()
        .to_ascii_lowercase();
    let allowlisted = std::env::var("PIX_ALLOWED_DEPOSITOR_IDS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .any(|user_id| user_id == auth.user_id);
    let automated_available = automated_provider == "depix"
        && automated_mode == "sandbox"
        && environment != "production"
        && allowlisted
        && std::env::var("DEPIX_API_KEY")
            .map(|key| key.starts_with("sk_test_") && !key.contains(char::is_whitespace))
            .unwrap_or(false)
        && std::env::var("DEPIX_WEBHOOK_SECRET")
            .map(|secret| secret.len() >= 24 && !secret.contains(char::is_whitespace))
            .unwrap_or(false);
    Ok(Json(DepositInfoResponse {
        available,
        pix_key: if available { pix_key } else { String::new() },
        receiver_name: env_receiver_name(),
        max_cents: env_max_cents(),
        max_pending: env_max_pending(),
        presets_cents: vec![10_000, 50_000, 100_000],
        instructions: if automated_available {
            "Sandbox DePix ativo: gere uma cobrança de teste. Nenhum PIX real é criado e somente a confirmação completed credita o saldo.".into()
        } else if available {
            "1) Copie a chave PIX e pague no app do seu banco. 2) Cole o comprovante (protocolo/E2E ou texto) e envie o pedido. 3) Após verificação manual, as fichas são creditadas.".into()
        } else {
            "Depósito temporariamente indisponível: integração PIX ainda não configurada para esta conta.".into()
        },
        automated_available,
        automated_provider: automated_available.then_some(automated_provider),
        automated_mode: automated_available.then_some(automated_mode),
    }))
}

#[derive(Debug, Deserialize)]
pub struct CreateDepositBody {
    pub amount_cents: i64,
    #[serde(default)]
    pub player_note: Option<String>,
    pub proof_text: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DepositRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    amount_cents: i64,
    status: String,
    player_note: Option<String>,
    proof_text: String,
    admin_note: Option<String>,
    reviewed_by: Option<uuid::Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DepositRequestResponse {
    pub id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub amount_cents: i64,
    pub status: String,
    pub player_note: Option<String>,
    pub proof_text: String,
    pub admin_note: Option<String>,
    pub reviewed_by: Option<String>,
    pub created_at: String,
    pub reviewed_at: Option<String>,
}

impl From<DepositRow> for DepositRequestResponse {
    fn from(r: DepositRow) -> Self {
        Self {
            id: r.id.to_string(),
            user_id: r.user_id.to_string(),
            username: r.username,
            amount_cents: r.amount_cents,
            status: r.status,
            player_note: r.player_note,
            proof_text: r.proof_text,
            admin_note: r.admin_note,
            reviewed_by: r.reviewed_by.map(|u| u.to_string()),
            created_at: r.created_at.to_rfc3339(),
            reviewed_at: r.reviewed_at.map(|t| t.to_rfc3339()),
        }
    }
}

pub async fn create_deposit_request(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateDepositBody>,
) -> Result<Json<DepositRequestResponse>, ApiError> {
    if env_pix_key().is_empty() {
        return Err(ApiError::BadRequest(
            "Depósito indisponível: chave PIX não configurada".into(),
        ));
    }

    let max_cents = env_max_cents();
    if body.amount_cents < 100 || body.amount_cents > max_cents {
        return Err(ApiError::BadRequest(format!(
            "amount_cents must be between 100 and {max_cents}"
        )));
    }

    let proof = body.proof_text.trim();
    if proof.len() < 8 || proof.len() > 4000 {
        return Err(ApiError::BadRequest(
            "proof_text must be 8–4000 characters (cole o comprovante/protocolo PIX)".into(),
        ));
    }

    let note = body
        .player_note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(500).collect::<String>());

    let uid = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;

    let pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM deposit_requests WHERE user_id = $1 AND status = 'pending'",
    )
    .bind(uid)
    .fetch_one(&state.db)
    .await?;
    if pending.0 >= env_max_pending() {
        return Err(ApiError::BadRequest(
            "Você já tem pedidos pendentes demais. Aguarde a análise.".into(),
        ));
    }

    let row: DepositRow = sqlx::query_as(
        r#"
        INSERT INTO deposit_requests (user_id, amount_cents, player_note, proof_text)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, amount_cents, status, player_note, proof_text, admin_note,
                  reviewed_by, created_at, reviewed_at, NULL::text AS username
        "#,
    )
    .bind(uid)
    .bind(body.amount_cents)
    .bind(note)
    .bind(proof)
    .fetch_one(&state.db)
    .await?;

    write_audit(
        &state,
        &auth_user.user_id,
        "DEPOSIT_REQUEST_CREATED",
        serde_json::json!({
            "request_id": row.id.to_string(),
            "amount_cents": body.amount_cents
        }),
    )
    .await?;

    Ok(Json(row.into()))
}

pub async fn list_my_deposit_requests(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<Vec<DepositRequestResponse>>, ApiError> {
    let uid = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;
    let rows: Vec<DepositRow> = sqlx::query_as(
        r#"
        SELECT id, user_id, amount_cents, status, player_note, proof_text, admin_note,
               reviewed_by, created_at, reviewed_at, NULL::text AS username
        FROM deposit_requests
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(uid)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter().map(DepositRequestResponse::from).collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct AdminDepositQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn admin_list_deposit_requests(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<AdminDepositQuery>,
) -> Result<Json<Vec<DepositRequestResponse>>, ApiError> {
    require_admin(&auth_user)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let rows: Vec<DepositRow> = if let Some(status) = status {
        sqlx::query_as(
            r#"
            SELECT d.id, d.user_id, d.amount_cents, d.status, d.player_note, d.proof_text,
                   d.admin_note, d.reviewed_by, d.created_at, d.reviewed_at, u.username
            FROM deposit_requests d
            JOIN users u ON u.id = d.user_id
            WHERE d.status = $1
            ORDER BY d.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(status)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT d.id, d.user_id, d.amount_cents, d.status, d.player_note, d.proof_text,
                   d.admin_note, d.reviewed_by, d.created_at, d.reviewed_at, u.username
            FROM deposit_requests d
            JOIN users u ON u.id = d.user_id
            ORDER BY CASE d.status WHEN 'pending' THEN 0 ELSE 1 END, d.created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(
        rows.into_iter().map(DepositRequestResponse::from).collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct RejectBody {
    #[serde(default)]
    pub admin_note: Option<String>,
}

pub async fn approve_deposit_request(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DepositRequestResponse>, ApiError> {
    require_admin(&auth_user)?;
    let rid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid request id".into()))?;
    let admin_id = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| ApiError::BadRequest("Invalid admin id".into()))?;

    let mut tx = state.db.begin().await?;

    let row: Option<(uuid::Uuid, i64, String)> = sqlx::query_as(
        "SELECT user_id, amount_cents, status FROM deposit_requests WHERE id = $1 FOR UPDATE",
    )
    .bind(rid)
    .fetch_optional(&mut *tx)
    .await?;

    let (user_id, amount, status) =
        row.ok_or_else(|| ApiError::NotFound("Deposit request not found".into()))?;
    if status != "pending" {
        return Err(ApiError::Conflict(format!("Request already {status}")));
    }

    crate::wallet::credit_wallet(
        &mut *tx,
        &user_id.to_string(),
        amount,
        crate::wallet::WalletKind::Real,
    )
    .await?;

    sqlx::query(
        r#"
        UPDATE deposit_requests
        SET status = 'approved',
            reviewed_by = $2,
            reviewed_at = NOW(),
            admin_note = COALESCE(admin_note, 'approved')
        WHERE id = $1
        "#,
    )
    .bind(rid)
    .bind(admin_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    write_audit(
        &state,
        &auth_user.user_id,
        "DEPOSIT_REQUEST_APPROVED",
        serde_json::json!({
            "request_id": id,
            "user_id": user_id.to_string(),
            "amount_cents": amount
        }),
    )
    .await?;

    let out: DepositRow = sqlx::query_as(
        r#"
        SELECT d.id, d.user_id, d.amount_cents, d.status, d.player_note, d.proof_text,
               d.admin_note, d.reviewed_by, d.created_at, d.reviewed_at, u.username
        FROM deposit_requests d
        JOIN users u ON u.id = d.user_id
        WHERE d.id = $1
        "#,
    )
    .bind(rid)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(out.into()))
}

pub async fn reject_deposit_request(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<RejectBody>,
) -> Result<Json<DepositRequestResponse>, ApiError> {
    require_admin(&auth_user)?;
    let rid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid request id".into()))?;
    let admin_id = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| ApiError::BadRequest("Invalid admin id".into()))?;
    let note = body
        .admin_note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("rejected")
        .chars()
        .take(500)
        .collect::<String>();

    let updated: Option<(String,)> = sqlx::query_as(
        r#"
        UPDATE deposit_requests
        SET status = 'rejected',
            reviewed_by = $2,
            reviewed_at = NOW(),
            admin_note = $3
        WHERE id = $1 AND status = 'pending'
        RETURNING status
        "#,
    )
    .bind(rid)
    .bind(admin_id)
    .bind(&note)
    .fetch_optional(&state.db)
    .await?;

    if updated.is_none() {
        return Err(ApiError::Conflict(
            "Request not found or not pending".into(),
        ));
    }

    write_audit(
        &state,
        &auth_user.user_id,
        "DEPOSIT_REQUEST_REJECTED",
        serde_json::json!({ "request_id": id, "admin_note": note }),
    )
    .await?;

    let out: DepositRow = sqlx::query_as(
        r#"
        SELECT d.id, d.user_id, d.amount_cents, d.status, d.player_note, d.proof_text,
               d.admin_note, d.reviewed_by, d.created_at, d.reviewed_at, u.username
        FROM deposit_requests d
        JOIN users u ON u.id = d.user_id
        WHERE d.id = $1
        "#,
    )
    .bind(rid)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(out.into()))
}
