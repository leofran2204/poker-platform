//! PIX ledger endpoints.
//!
//! PIX remains a deferred product capability. These handlers therefore build a
//! durable, auditable financial intent but never treat a gateway response as a
//! substitute for a database ledger transition. A future outbox worker may
//! execute the queued payout only after provider reconciliation and an
//! encrypted payment-instrument reference are implemented.

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::payment_gateway::get_payment_gateway;
use crate::state::AppState;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

type ExistingDepositRow = (i64, String, Option<String>, Option<String>, Option<String>);

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    /// Integer cents.
    pub amount: u64,
    /// Forwarded to DePix only; never persisted or logged by this service.
    #[serde(default)]
    pub payer_tax_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DepositResponse {
    pub tx_id: String,
    /// Integer cents.
    pub amount: u64,
    pub pix_copy_paste: String,
    pub qr_code_base64: String,
    pub expires_at: String,
    pub payment_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookPixPayload {
    /// Internal, opaque idempotency key returned when the charge was created.
    pub tx_id: String,
    /// Provider transaction identifier; it must match the pending ledger row.
    pub external_tx_id: Option<String>,
    /// Integer cents. It is checked against the persisted amount and is never
    /// used as the amount to credit.
    pub amount: u64,
    pub status: String,
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
}
#[derive(Debug, Deserialize)]
struct AsaasWebhookPayment {
    id: String,
    #[serde(rename = "externalReference")]
    external_reference: Option<String>,
    value: serde_json::Value,
    status: String,
}

#[derive(Debug, Deserialize)]
struct AsaasWebhookPayload {
    event: String,
    payment: AsaasWebhookPayment,
}

#[derive(Debug, Deserialize)]
struct DepixWebhookPayload {
    event: String,
    data: DepixWebhookData,
}

#[derive(Debug, Deserialize)]
struct DepixWebhookData {
    event_id: String,
    id: String,
    status: String,
    amount: u64,
    metadata: Option<DepixWebhookMetadata>,
}

#[derive(Debug, Deserialize)]
struct DepixWebhookMetadata {
    order_id: Option<String>,
}

fn brl_value_to_cents(value: &serde_json::Value) -> Result<u64, ApiError> {
    let raw = value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.as_number().map(ToString::to_string))
        .ok_or_else(|| ApiError::BadRequest("Asaas webhook amount is invalid".to_string()))?;
    let (whole, fractional) = raw
        .split_once('.')
        .map_or((raw.as_str(), ""), |(whole, fractional)| {
            (whole, fractional)
        });
    if whole.is_empty()
        || !whole.chars().all(|character| character.is_ascii_digit())
        || fractional.len() > 2
        || !fractional
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(ApiError::BadRequest(
            "Asaas webhook amount is not an exact BRL decimal".to_string(),
        ));
    }
    let whole_cents = whole
        .parse::<u64>()
        .map_err(|_| ApiError::BadRequest("Asaas webhook amount is too large".to_string()))?
        .checked_mul(100)
        .ok_or_else(|| ApiError::BadRequest("Asaas webhook amount is too large".to_string()))?;
    let fractional_cents = match fractional.len() {
        0 => 0,
        1 => {
            fractional
                .parse::<u64>()
                .map_err(|_| ApiError::BadRequest("Asaas webhook amount is invalid".to_string()))?
                * 10
        }
        2 => fractional
            .parse::<u64>()
            .map_err(|_| ApiError::BadRequest("Asaas webhook amount is invalid".to_string()))?,
        _ => unreachable!("fraction length was validated"),
    };
    whole_cents
        .checked_add(fractional_cents)
        .ok_or_else(|| ApiError::BadRequest("Asaas webhook amount is too large".to_string()))
}

fn parse_verified_webhook(provider: &str, body: &[u8]) -> Result<WebhookPixPayload, ApiError> {
    if provider == "asaas" {
        let payload: AsaasWebhookPayload = serde_json::from_slice(body)?;
        let tx_id = payload.payment.external_reference.ok_or_else(|| {
            ApiError::BadRequest("Asaas webhook is missing externalReference".to_string())
        })?;
        return Ok(WebhookPixPayload {
            tx_id,
            external_tx_id: Some(payload.payment.id),
            amount: brl_value_to_cents(&payload.payment.value)?,
            status: if payload.event == "PAYMENT_RECEIVED"
                && payload.payment.status.eq_ignore_ascii_case("RECEIVED")
            {
                "RECEIVED".to_string()
            } else {
                payload.payment.status
            },
            event_id: None,
            event_type: Some(payload.event),
        });
    }
    if provider == "depix" {
        let payload: DepixWebhookPayload = serde_json::from_slice(body)?;
        let tx_id = payload
            .data
            .metadata
            .and_then(|metadata| metadata.order_id)
            .ok_or_else(|| {
                ApiError::BadRequest("DePix webhook is missing metadata.order_id".into())
            })?;
        return Ok(WebhookPixPayload {
            tx_id,
            external_tx_id: Some(payload.data.id),
            amount: payload.data.amount,
            status: payload.data.status,
            event_id: Some(payload.data.event_id),
            event_type: Some(payload.event),
        });
    }

    Ok(serde_json::from_slice(body)?)
}
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    /// Integer cents.
    pub amount: u64,
    pub pix_key_type: String,
    pub pix_key: String,
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub tx_id: String,
    /// Integer cents.
    pub amount: u64,
    pub status: String,
    pub message: String,
}

type WalletRow = (
    uuid::Uuid,
    uuid::Uuid,
    i64,
    String,
    String,
    Option<String>,
    String,
);

fn cents_to_i64(amount: u64, field: &str) -> Result<i64, ApiError> {
    let cents = i64::try_from(amount)
        .map_err(|_| ApiError::BadRequest(format!("{field} exceeds the supported range")))?;
    if cents <= 0 {
        return Err(ApiError::BadRequest(format!(
            "{field} must be greater than zero"
        )));
    }
    Ok(cents)
}

fn pix_provider() -> String {
    std::env::var("PIX_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .trim()
        .to_ascii_lowercase()
}

fn pix_mode() -> String {
    std::env::var("PIX_MODE")
        .unwrap_or_else(|_| "mock".to_string())
        .trim()
        .to_ascii_lowercase()
}

fn normalized_tax_number(value: Option<&str>) -> Result<Option<String>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let normalized = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect::<String>();
    if !matches!(normalized.len(), 11 | 14) {
        return Err(ApiError::BadRequest(
            "payer_tax_number must be a valid CPF or CNPJ".to_string(),
        ));
    }
    Ok(Some(normalized))
}
fn configured_user_list_contains(variable: &str, user_id: &str) -> bool {
    std::env::var(variable)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .any(|configured_user_id| configured_user_id == user_id)
}

pub(crate) fn pix_depositor_is_allowed(user_id: &str) -> bool {
    let provider = pix_provider();
    let mode = pix_mode();
    let environment = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .trim()
        .to_ascii_lowercase();
    match (provider.as_str(), mode.as_str()) {
        ("asaas" | "depix", "sandbox") => {
            environment != "production"
                && configured_user_list_contains("PIX_ALLOWED_DEPOSITOR_IDS", user_id)
        }
        ("depix", "production") => {
            environment == "production"
                && std::env::var("PIX_LIVE_ENABLED")
                    .map(|value| value.trim().eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                && configured_user_list_contains("PIX_LIVE_ALLOWED_DEPOSITOR_IDS", user_id)
        }
        _ => true,
    }
}

fn ensure_pix_depositor_is_allowed(user_id: &str) -> Result<(), ApiError> {
    if pix_depositor_is_allowed(user_id) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "This account is not authorized for the configured PIX rollout".to_string(),
        ))
    }
}

pub(crate) fn depix_deposit_max_cents() -> u64 {
    if pix_mode() != "production" {
        return 600_000;
    }
    std::env::var("DEPIX_LIVE_MAX_DEPOSIT_CENTS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| (500..=600_000).contains(value))
        .unwrap_or(100_000)
}

fn deposit_tx_id(headers: &HeaderMap, user_id: &str, provider: &str) -> Result<String, ApiError> {
    if provider != "depix" {
        return Ok(format!("pix_dep_{}", uuid::Uuid::new_v4()));
    }
    let raw = headers
        .get("Idempotency-Key")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            ApiError::BadRequest("Idempotency-Key is required for DePix deposits".into())
        })?;
    let key = uuid::Uuid::parse_str(raw.trim())
        .map_err(|_| ApiError::BadRequest("Idempotency-Key must be a UUID".into()))?;
    let digest = format!(
        "{:x}",
        Sha256::digest(format!("{user_id}:{key}").as_bytes())
    );
    Ok(format!("pix_dep_{}", &digest[..32]))
}
fn pix_key_fingerprint(pix_key: &str) -> String {
    format!("{:x}", Sha256::digest(pix_key.trim().as_bytes()))
}

/// POST /api/payments/pix/deposit
///
/// Creates a pending ledger row before exposing a charge to the client. The
/// provider callback can only credit that exact row once.
pub async fn create_pix_deposit_handler(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    headers: HeaderMap,
    Json(payload): Json<DepositRequest>,
) -> Result<Json<DepositResponse>, ApiError> {
    let amount = cents_to_i64(payload.amount, "Deposit amount")?;
    let provider = pix_provider();
    let payer_tax_number = normalized_tax_number(payload.payer_tax_number.as_deref())?;
    if provider == "depix" {
        let max_cents = depix_deposit_max_cents();
        if !(500..=max_cents).contains(&payload.amount) {
            return Err(ApiError::BadRequest(format!(
                "DePix deposit amount must be between 500 and {max_cents} cents"
            )));
        }
        if payer_tax_number.is_none() {
            return Err(ApiError::BadRequest(
                "payer_tax_number is required for DePix PIX checkout".to_string(),
            ));
        }
    }
    ensure_pix_depositor_is_allowed(&auth_user.user_id)?;
    let tx_id = deposit_tx_id(&headers, &auth_user.user_id, &provider)?;
    let _audit = crate::audit_span!(&auth_user.user_id, "PIX_DEPOSIT_CREATED");

    let existing: Option<ExistingDepositRow> = sqlx::query_as(
        "SELECT amount, status, pix_copy_paste, provider_payment_url, provider_expires_at \
             FROM wallet_transactions \
             WHERE idempotency_key = $1 AND user_id = $2::uuid AND provider = $3",
    )
    .bind(&tx_id)
    .bind(&auth_user.user_id)
    .bind(&provider)
    .fetch_optional(&state.db)
    .await?;
    if let Some((stored_amount, status, Some(pix_copy_paste), payment_url, expires_at)) = existing {
        if stored_amount != amount {
            return Err(ApiError::Conflict(
                "Idempotency-Key was already used with a different amount".into(),
            ));
        }
        if status == "CANCELLED" {
            return Err(ApiError::Conflict(
                "PIX deposit is already cancelled".into(),
            ));
        }
        return Ok(Json(DepositResponse {
            tx_id,
            amount: payload.amount,
            pix_copy_paste,
            qr_code_base64: String::new(),
            expires_at: expires_at.unwrap_or_default(),
            payment_url,
        }));
    }

    let mut transaction = state.db.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO wallet_transactions \
         (user_id, amount, transaction_type, status, idempotency_key, provider) \
         VALUES ($1::uuid, $2, 'DEPOSIT', 'PENDING', $3, $4) \
         ON CONFLICT (idempotency_key) WHERE idempotency_key IS NOT NULL DO NOTHING",
    )
    .bind(&auth_user.user_id)
    .bind(amount)
    .bind(&tx_id)
    .bind(&provider)
    .execute(&mut *transaction)
    .await?;
    if inserted.rows_affected() == 1 {
        sqlx::query(
            "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'PIX_DEPOSIT_CREATED', $2)",
        )
        .bind(&auth_user.user_id)
        .bind(serde_json::json!({"tx_id": &tx_id, "amount_cents": amount, "provider": &provider}))
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;

    let gateway = get_payment_gateway();
    let charge_tx_id = tx_id.clone();
    let charge_user_id = auth_user.user_id.clone();
    let charge_amount = payload.amount;
    let charge_tax_number = payer_tax_number.clone();
    let charge = match tokio::task::spawn_blocking(move || {
        gateway.create_deposit_charge(
            &charge_tx_id,
            &charge_user_id,
            charge_amount,
            charge_tax_number.as_deref(),
        )
    })
    .await
    {
        Ok(Ok(charge)) => charge,
        Ok(Err(error)) => {
            let ledger_status = if provider == "depix" {
                "PENDING"
            } else {
                "CANCELLED"
            };
            let provider_status = if provider == "depix" {
                "CREATE_UNKNOWN"
            } else {
                "CREATE_FAILED"
            };
            sqlx::query(
                "UPDATE wallet_transactions SET status = $1, provider_status = $2 \
                 WHERE idempotency_key = $3 AND status = 'PENDING'",
            )
            .bind(ledger_status)
            .bind(provider_status)
            .bind(&tx_id)
            .execute(&state.db)
            .await?;
            return Err(ApiError::Internal(format!(
                "PIX charge creation failed: {error}"
            )));
        }
        Err(error) => {
            let ledger_status = if provider == "depix" {
                "PENDING"
            } else {
                "CANCELLED"
            };
            let provider_status = if provider == "depix" {
                "CREATE_UNKNOWN"
            } else {
                "CREATE_FAILED"
            };
            sqlx::query(
                "UPDATE wallet_transactions SET status = $1, provider_status = $2 \
                 WHERE idempotency_key = $3 AND status = 'PENDING'",
            )
            .bind(ledger_status)
            .bind(provider_status)
            .bind(&tx_id)
            .execute(&state.db)
            .await?;
            return Err(ApiError::Internal(format!(
                "PIX charge task failed: {error}"
            )));
        }
    };

    let updated = sqlx::query(
        "UPDATE wallet_transactions \
         SET external_tx_id = $1, pix_copy_paste = $2, provider_payment_url = $3, \
             provider_expires_at = $4, provider_status = 'AWAITING_PAYMENT' \
         WHERE idempotency_key = $5 AND status = 'PENDING'",
    )
    .bind(&charge.external_tx_id)
    .bind(&charge.pix_copy_paste)
    .bind(&charge.payment_url)
    .bind(&charge.expires_at)
    .bind(&tx_id)
    .execute(&state.db)
    .await?;
    if updated.rows_affected() != 1 {
        return Err(ApiError::Internal(
            "Pending PIX ledger entry was not available after charge creation".to_string(),
        ));
    }

    Ok(Json(DepositResponse {
        tx_id,
        amount: payload.amount,
        pix_copy_paste: charge.pix_copy_paste,
        qr_code_base64: charge.qr_code_base64,
        expires_at: charge.expires_at,
        payment_url: charge.payment_url,
    }))
}

/// POST /api/webhooks/pix
///
/// Authenticated provider callback. The credit and the `COMPLETED` state are
/// committed in one PostgreSQL transaction; duplicate callbacks are harmless.
pub async fn pix_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<WebhookResponse>), ApiError> {
    let provider = pix_provider();
    let webhook_token = if provider == "depix" {
        headers.get("X-DePix-Signature")
    } else {
        headers
            .get("asaas-access-token")
            .or_else(|| headers.get("X-Signature"))
            .or_else(|| headers.get("X-Webhook-Secret"))
    }
    .and_then(|header| header.to_str().ok());
    let gateway = get_payment_gateway();
    if !gateway.verify_webhook_hmac(&body, webhook_token) {
        return Err(ApiError::Unauthorized(
            "Invalid or missing PIX webhook authentication".to_string(),
        ));
    }

    if provider == "depix" {
        let header_event_id = headers
            .get("X-DePix-Event-Id")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| ApiError::BadRequest("Missing X-DePix-Event-Id".into()))?;
        let header_event = headers
            .get("X-DePix-Event")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| ApiError::BadRequest("Missing X-DePix-Event".into()))?;
        let generic: serde_json::Value = serde_json::from_slice(&body)?;
        let body_event = generic.get("event").and_then(serde_json::Value::as_str);
        let body_event_id = generic
            .get("data")
            .and_then(|data| data.get("event_id"))
            .and_then(serde_json::Value::as_str);
        if body_event != Some(header_event) || body_event_id != Some(header_event_id) {
            return Err(ApiError::BadRequest(
                "DePix webhook headers do not match the signed body".into(),
            ));
        }
        let supported = matches!(
            header_event,
            "checkout.processing"
                | "checkout.approved"
                | "checkout.completed"
                | "checkout.cancelled"
                | "checkout.expired"
        );
        if !supported {
            let payload_sha256 = format!("{:x}", Sha256::digest(body.as_ref()));
            sqlx::query(
                "INSERT INTO payment_webhook_events \
                 (provider, event_id, event_type, payload_sha256) VALUES ('depix', $1, $2, $3) \
                 ON CONFLICT (provider, event_id) DO NOTHING",
            )
            .bind(header_event_id)
            .bind(header_event)
            .bind(payload_sha256)
            .execute(&state.db)
            .await?;
            return Ok((
                StatusCode::OK,
                Json(WebhookResponse {
                    status: "IGNORED".to_string(),
                    message: "Authenticated DePix event requires no automatic wallet action"
                        .to_string(),
                }),
            ));
        }
    }
    let payload = parse_verified_webhook(&provider, &body)?;

    let settling = if provider == "depix" {
        payload.event_type.as_deref() == Some("checkout.completed")
            && payload.status.eq_ignore_ascii_case("completed")
    } else {
        matches!(
            payload.status.to_ascii_uppercase().as_str(),
            "APPROVED" | "COMPLETED" | "RECEIVED"
        )
    };

    let mut transaction = state.db.begin().await?;
    if provider == "depix" {
        let event_id = payload
            .event_id
            .as_deref()
            .expect("validated DePix event id");
        let event_type = payload
            .event_type
            .as_deref()
            .expect("validated DePix event type");
        let payload_sha256 = format!("{:x}", Sha256::digest(body.as_ref()));
        let inserted = sqlx::query(
            "INSERT INTO payment_webhook_events \
             (provider, event_id, event_type, payload_sha256) VALUES ('depix', $1, $2, $3) \
             ON CONFLICT (provider, event_id) DO NOTHING",
        )
        .bind(event_id)
        .bind(event_type)
        .bind(payload_sha256)
        .execute(&mut *transaction)
        .await?;
        if inserted.rows_affected() == 0 {
            transaction.rollback().await?;
            return Ok((
                StatusCode::OK,
                Json(WebhookResponse {
                    status: "IGNORED".to_string(),
                    message: "DePix event was already processed".to_string(),
                }),
            ));
        }
    }

    if !settling {
        if provider == "depix" {
            let terminal_failure = matches!(
                payload.event_type.as_deref(),
                Some("checkout.cancelled" | "checkout.expired")
            );
            sqlx::query(
                "UPDATE wallet_transactions \
                 SET provider_status = $1, status = CASE WHEN $2 THEN 'CANCELLED' ELSE status END, updated_at = NOW() \
                 WHERE idempotency_key = $3 AND external_tx_id = $4 AND status = 'PENDING'",
            )
            .bind(payload.status.to_ascii_uppercase())
            .bind(terminal_failure)
            .bind(&payload.tx_id)
            .bind(payload.external_tx_id.as_deref())
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        return Ok((
            StatusCode::OK,
            Json(WebhookResponse {
                status: "IGNORED".to_string(),
                message: "Non-settling provider status".to_string(),
            }),
        ));
    }

    let callback_amount = cents_to_i64(payload.amount, "Webhook amount")?;
    let callback_external_id = payload.external_tx_id.as_deref().ok_or_else(|| {
        ApiError::BadRequest("Settling PIX webhook requires external_tx_id".to_string())
    })?;
    let row: Option<WalletRow> = sqlx::query_as(
        "SELECT id, user_id, amount, transaction_type, status, external_tx_id, provider \
         FROM wallet_transactions WHERE idempotency_key = $1 FOR UPDATE",
    )
    .bind(&payload.tx_id)
    .fetch_optional(&mut *transaction)
    .await?;
    let (wallet_id, user_id, amount, transaction_type, status, stored_external_id, stored_provider) =
        row.ok_or_else(|| ApiError::NotFound("Unknown PIX transaction".to_string()))?;

    if transaction_type != "DEPOSIT" || stored_provider != provider {
        return Err(ApiError::BadRequest(
            "PIX webhook does not reference a deposit".to_string(),
        ));
    }
    if amount != callback_amount {
        return Err(ApiError::BadRequest(
            "PIX webhook amount does not match the pending ledger entry".to_string(),
        ));
    }
    match stored_external_id.as_deref() {
        Some(stored_external_id) if stored_external_id != callback_external_id => {
            return Err(ApiError::BadRequest(
                "PIX webhook provider id does not match the ledger entry".to_string(),
            ));
        }
        None if provider == "depix" && status == "PENDING" => {
            sqlx::query(
                "UPDATE wallet_transactions SET external_tx_id = $1, provider_status = 'WEBHOOK_RECOVERED' \
                 WHERE id = $2 AND external_tx_id IS NULL",
            )
            .bind(callback_external_id)
            .bind(wallet_id)
            .execute(&mut *transaction)
            .await?;
        }
        None => {
            return Err(ApiError::BadRequest(
                "PIX webhook cannot recover this provider transaction".to_string(),
            ));
        }
        Some(_) => {}
    }
    if status == "COMPLETED" {
        transaction.commit().await?;
        return Ok((
            StatusCode::OK,
            Json(WebhookResponse {
                status: "IGNORED".to_string(),
                message: "PIX transaction was already settled".to_string(),
            }),
        ));
    }
    if status != "PENDING" {
        return Err(ApiError::Conflict(
            "PIX transaction is not pending settlement".to_string(),
        ));
    }

    let credited = sqlx::query("UPDATE users SET balance_real = balance_real + $1 WHERE id = $2")
        .bind(amount)
        .bind(user_id)
        .execute(&mut *transaction)
        .await?;
    if credited.rows_affected() != 1 {
        return Err(ApiError::Internal(
            "PIX deposit user account is missing".to_string(),
        ));
    }
    sqlx::query(
        "UPDATE wallet_transactions SET status = 'COMPLETED', provider_status = $1, updated_at = NOW() \
         WHERE id = $2",
    )
    .bind(payload.status.to_ascii_uppercase())
    .bind(wallet_id)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'PIX_DEPOSIT_SETTLED', $2)",
    )
    .bind(user_id.to_string())
    .bind(serde_json::json!({
        "tx_id": payload.tx_id,
        "external_tx_id": callback_external_id,
        "amount_cents": amount,
        "provider": provider,
        "event_id": payload.event_id,
    }))
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::OK,
        Json(WebhookResponse {
            status: "COMPLETED".to_string(),
            message: "PIX deposit settled".to_string(),
        }),
    ))
}

#[derive(Debug, Serialize)]
pub struct PixDepositStatusResponse {
    pub tx_id: String,
    pub amount: u64,
    pub status: String,
    pub provider_status: String,
}

async fn reconcile_deposit_status(
    state: &AppState,
    user_id: &str,
    tx_id: &str,
    provider_status: crate::payment_gateway::PixChargeStatus,
) -> Result<PixDepositStatusResponse, ApiError> {
    let callback_amount = cents_to_i64(provider_status.amount, "Provider amount")?;
    let mut transaction = state.db.begin().await?;
    let row: Option<(uuid::Uuid, i64, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, amount, status, external_tx_id, provider \
         FROM wallet_transactions \
         WHERE idempotency_key = $1 AND user_id = $2::uuid AND transaction_type = 'DEPOSIT' \
         FOR UPDATE",
    )
    .bind(tx_id)
    .bind(user_id)
    .fetch_optional(&mut *transaction)
    .await?;
    let (wallet_id, amount, current_status, external_tx_id, stored_provider) =
        row.ok_or_else(|| ApiError::NotFound("PIX deposit was not found".into()))?;
    if stored_provider != pix_provider()
        || amount != callback_amount
        || external_tx_id.as_deref() != Some(provider_status.external_tx_id.as_str())
    {
        return Err(ApiError::BadRequest(
            "Provider status does not match the persisted deposit".into(),
        ));
    }

    let normalized_status = provider_status.status.to_ascii_uppercase();
    let mut ledger_status = current_status;
    if normalized_status == "COMPLETED" && ledger_status == "PENDING" {
        let credited =
            sqlx::query("UPDATE users SET balance_real = balance_real + $1 WHERE id = $2::uuid")
                .bind(amount)
                .bind(user_id)
                .execute(&mut *transaction)
                .await?;
        if credited.rows_affected() != 1 {
            return Err(ApiError::Internal(
                "PIX deposit user account is missing".into(),
            ));
        }
        ledger_status = "COMPLETED".to_string();
        sqlx::query(
            "INSERT INTO audit_logs (user_id, action, metadata) \
             VALUES ($1, 'PIX_DEPOSIT_RECONCILED', $2)",
        )
        .bind(user_id)
        .bind(serde_json::json!({
            "tx_id": tx_id,
            "external_tx_id": provider_status.external_tx_id,
            "amount_cents": amount,
            "provider": stored_provider,
        }))
        .execute(&mut *transaction)
        .await?;
    } else if matches!(normalized_status.as_str(), "CANCELLED" | "EXPIRED")
        && ledger_status == "PENDING"
    {
        ledger_status = "CANCELLED".to_string();
    }
    sqlx::query(
        "UPDATE wallet_transactions SET status = $1, provider_status = $2, updated_at = NOW() WHERE id = $3",
    )
    .bind(&ledger_status)
    .bind(&normalized_status)
    .bind(wallet_id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok(PixDepositStatusResponse {
        tx_id: tx_id.to_string(),
        amount: provider_status.amount,
        status: ledger_status,
        provider_status: normalized_status,
    })
}

async fn owned_external_deposit_id(
    state: &AppState,
    user_id: &str,
    tx_id: &str,
) -> Result<String, ApiError> {
    sqlx::query_scalar(
        "SELECT external_tx_id FROM wallet_transactions \
         WHERE idempotency_key = $1 AND user_id = $2::uuid AND transaction_type = 'DEPOSIT'",
    )
    .bind(tx_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .flatten()
    .ok_or_else(|| ApiError::NotFound("PIX deposit was not found".into()))
}

pub async fn get_pix_deposit_status_handler(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Path(tx_id): Path<String>,
) -> Result<Json<PixDepositStatusResponse>, ApiError> {
    let external_tx_id = owned_external_deposit_id(&state, &auth_user.user_id, &tx_id).await?;
    let gateway = get_payment_gateway();
    let provider_status =
        tokio::task::spawn_blocking(move || gateway.fetch_deposit_status(&external_tx_id))
            .await
            .map_err(|error| ApiError::Internal(format!("PIX status task failed: {error}")))?
            .map_err(|error| ApiError::Internal(format!("PIX status request failed: {error}")))?;
    Ok(Json(
        reconcile_deposit_status(&state, &auth_user.user_id, &tx_id, provider_status).await?,
    ))
}

pub async fn simulate_pix_deposit_handler(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Path(tx_id): Path<String>,
) -> Result<Json<PixDepositStatusResponse>, ApiError> {
    if pix_provider() != "depix"
        || pix_mode() != "sandbox"
        || !std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .trim()
            .eq_ignore_ascii_case("development")
    {
        return Err(ApiError::Forbidden(
            "PIX payment simulation is available only in DePix Sandbox".into(),
        ));
    }
    ensure_pix_depositor_is_allowed(&auth_user.user_id)?;
    let external_tx_id = owned_external_deposit_id(&state, &auth_user.user_id, &tx_id).await?;
    let fetch_external_id = external_tx_id.clone();
    let gateway = get_payment_gateway();
    let provider_status = tokio::task::spawn_blocking(move || {
        gateway.simulate_deposit_payment(&external_tx_id)?;
        gateway.fetch_deposit_status(&fetch_external_id)
    })
    .await
    .map_err(|error| ApiError::Internal(format!("PIX simulation task failed: {error}")))?
    .map_err(|error| ApiError::Internal(format!("PIX simulation failed: {error}")))?;
    Ok(Json(
        reconcile_deposit_status(&state, &auth_user.user_id, &tx_id, provider_status).await?,
    ))
}
/// POST /api/payments/pix/withdraw
///
/// Reserves funds atomically and records an outbox event. It deliberately does
/// not call an external payout provider in the request path: provider delivery
/// belongs to a reconciled outbox worker with an encrypted PIX-key reference,
/// not to a retryable HTTPS request. The raw PIX key is never persisted here.
pub async fn create_pix_withdraw_handler(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(payload): Json<WithdrawRequest>,
) -> Result<(StatusCode, Json<WithdrawResponse>), ApiError> {
    let amount = cents_to_i64(payload.amount, "Withdraw amount")?;
    if payload.pix_key.trim().is_empty() {
        return Err(ApiError::BadRequest("PIX key is required".to_string()));
    }
    if !matches!(
        payload.pix_key_type.as_str(),
        "cpf" | "email" | "phone" | "evp"
    ) {
        return Err(ApiError::BadRequest("Invalid PIX key type".to_string()));
    }

    let tx_id = format!("pix_wdr_{}", uuid::Uuid::new_v4());
    let provider = pix_provider();
    let pix_key_fingerprint = pix_key_fingerprint(&payload.pix_key);
    let _audit = crate::audit_span!(&auth_user.user_id, "PIX_WITHDRAWAL_RESERVED");
    let mut transaction = state.db.begin().await?;

    let debited = sqlx::query(
        "UPDATE users SET balance_real = balance_real - $1 \
         WHERE id = $2::uuid AND balance_real >= $1 RETURNING balance_real",
    )
    .bind(amount)
    .bind(&auth_user.user_id)
    .fetch_optional(&mut *transaction)
    .await?;
    if debited.is_none() {
        return Err(ApiError::BadRequest(
            "Insufficient wallet balance for PIX withdrawal".to_string(),
        ));
    }

    sqlx::query(
        "INSERT INTO wallet_transactions \
         (user_id, amount, transaction_type, status, idempotency_key, provider, pix_key_fingerprint, provider_status) \
         VALUES ($1::uuid, $2, 'WITHDRAW', 'PENDING', $3, $4, $5, 'QUEUED')",
    )
    .bind(&auth_user.user_id)
    .bind(amount)
    .bind(&tx_id)
    .bind(&provider)
    .bind(&pix_key_fingerprint)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, payload) \
         VALUES ('wallet_transaction', $1, 'PIX_WITHDRAWAL_REQUESTED', $2)",
    )
    .bind(&tx_id)
    .bind(serde_json::json!({
        "tx_id": &tx_id,
        "user_id": &auth_user.user_id,
        "amount_cents": amount,
        "pix_key_type": &payload.pix_key_type,
        "pix_key_fingerprint": &pix_key_fingerprint,
        "provider": &provider
    }))
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'PIX_WITHDRAWAL_RESERVED', $2)",
    )
    .bind(&auth_user.user_id)
    .bind(serde_json::json!({"tx_id": &tx_id, "amount_cents": amount, "provider": &provider}))
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(WithdrawResponse {
            tx_id,
            amount: payload.amount,
            status: "PENDING".to_string(),
            message: "PIX withdrawal reserved for reconciled processing".to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::{brl_value_to_cents, normalized_tax_number, parse_verified_webhook};

    #[test]
    fn asaas_webhook_keeps_exact_cent_values() {
        assert_eq!(
            brl_value_to_cents(&serde_json::json!("12.34")).unwrap(),
            1234
        );
        assert_eq!(brl_value_to_cents(&serde_json::json!(12.3)).unwrap(), 1230);
        assert_eq!(brl_value_to_cents(&serde_json::json!(7)).unwrap(), 700);
    }

    #[test]
    fn asaas_webhook_rejects_non_exact_brl_values() {
        assert!(brl_value_to_cents(&serde_json::json!("12.345")).is_err());
        assert!(brl_value_to_cents(&serde_json::json!("-1.00")).is_err());
        assert!(brl_value_to_cents(&serde_json::json!("1e2")).is_err());
    }

    #[test]
    fn asaas_webhook_uses_provider_ids_and_received_status() {
        let body = br#"{
            "event":"PAYMENT_RECEIVED",
            "payment":{
                "id":"pay_sandbox_1",
                "externalReference":"pix_dep_internal_1",
                "value":"9.99",
                "status":"RECEIVED"
            }
        }"#;
        let payload = parse_verified_webhook("asaas", body).unwrap();

        assert_eq!(payload.tx_id, "pix_dep_internal_1");
        assert_eq!(payload.external_tx_id.as_deref(), Some("pay_sandbox_1"));
        assert_eq!(payload.amount, 999);
        assert_eq!(payload.status, "RECEIVED");
    }
    #[test]
    fn depix_completed_webhook_uses_signed_provider_and_internal_ids() {
        let body = br#"{
            "event":"checkout.completed",
            "data":{
                "event_id":"evt_test_1",
                "id":"chk_test_1",
                "status":"completed",
                "amount":1234,
                "metadata":{"order_id":"pix_dep_internal_1"}
            }
        }"#;
        let payload = parse_verified_webhook("depix", body).unwrap();

        assert_eq!(payload.tx_id, "pix_dep_internal_1");
        assert_eq!(payload.external_tx_id.as_deref(), Some("chk_test_1"));
        assert_eq!(payload.event_id.as_deref(), Some("evt_test_1"));
        assert_eq!(payload.event_type.as_deref(), Some("checkout.completed"));
        assert_eq!(payload.amount, 1234);
        assert_eq!(payload.status, "completed");
    }

    #[test]
    fn payer_tax_number_is_normalized_without_being_persisted() {
        assert_eq!(
            normalized_tax_number(Some("529.982.247-25"))
                .unwrap()
                .as_deref(),
            Some("52998224725")
        );
        assert_eq!(
            normalized_tax_number(Some("12.ABC.345/0001-DE"))
                .unwrap()
                .as_deref(),
            Some("12ABC3450001DE")
        );
        assert!(normalized_tax_number(Some("123")).is_err());
    }
}
