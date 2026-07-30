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
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    /// Integer cents.
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct DepositResponse {
    pub tx_id: String,
    /// Integer cents.
    pub amount: u64,
    pub pix_copy_paste: String,
    pub qr_code_base64: String,
    pub expires_at: String,
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

type WalletRow = (uuid::Uuid, uuid::Uuid, i64, String, String, Option<String>);

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
        .to_ascii_lowercase()
}
fn ensure_pix_depositor_is_allowlisted(user_id: &str) -> Result<(), ApiError> {
    let provider = pix_provider();
    let mode = std::env::var("PIX_MODE")
        .unwrap_or_else(|_| "mock".to_string())
        .to_ascii_lowercase();
    if provider != "asaas" || mode != "sandbox" {
        return Ok(());
    }

    let allowed = std::env::var("PIX_ALLOWED_DEPOSITOR_IDS").map_err(|_| {
        ApiError::Forbidden("Asaas Sandbox deposits require PIX_ALLOWED_DEPOSITOR_IDS".to_string())
    })?;
    if allowed
        .split(',')
        .map(str::trim)
        .any(|configured_user_id| configured_user_id == user_id)
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "This account is not authorized for Asaas Sandbox deposits".to_string(),
        ))
    }
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
    Json(payload): Json<DepositRequest>,
) -> Result<Json<DepositResponse>, ApiError> {
    let amount = cents_to_i64(payload.amount, "Deposit amount")?;
    ensure_pix_depositor_is_allowlisted(&auth_user.user_id)?;
    let tx_id = format!("pix_dep_{}", uuid::Uuid::new_v4());
    let provider = pix_provider();
    let _audit = crate::audit_span!(&auth_user.user_id, "PIX_DEPOSIT_CREATED");

    let mut transaction = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO wallet_transactions \
         (user_id, amount, transaction_type, status, idempotency_key, provider) \
         VALUES ($1::uuid, $2, 'DEPOSIT', 'PENDING', $3, $4)",
    )
    .bind(&auth_user.user_id)
    .bind(amount)
    .bind(&tx_id)
    .bind(&provider)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'PIX_DEPOSIT_CREATED', $2)",
    )
    .bind(&auth_user.user_id)
    .bind(serde_json::json!({"tx_id": &tx_id, "amount_cents": amount, "provider": &provider}))
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    let gateway = get_payment_gateway();
    let charge_tx_id = tx_id.clone();
    let charge_user_id = auth_user.user_id.clone();
    let charge_amount = payload.amount;
    let charge = match tokio::task::spawn_blocking(move || {
        gateway.create_deposit_charge(&charge_tx_id, &charge_user_id, charge_amount)
    })
    .await
    {
        Ok(Ok(charge)) => charge,
        Ok(Err(error)) => {
            sqlx::query(
                "UPDATE wallet_transactions SET status = 'CANCELLED', provider_status = 'CREATE_FAILED' \
                 WHERE idempotency_key = $1 AND status = 'PENDING'",
            )
            .bind(&tx_id)
            .execute(&state.db)
            .await?;
            return Err(ApiError::Internal(format!(
                "PIX charge creation failed: {error}"
            )));
        }
        Err(error) => {
            sqlx::query(
                "UPDATE wallet_transactions SET status = 'CANCELLED', provider_status = 'CREATE_FAILED' \
                 WHERE idempotency_key = $1 AND status = 'PENDING'",
            )
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
         SET external_tx_id = $1, pix_copy_paste = $2, provider_status = 'AWAITING_PAYMENT' \
         WHERE idempotency_key = $3 AND status = 'PENDING'",
    )
    .bind(&charge.external_tx_id)
    .bind(&charge.pix_copy_paste)
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
    let webhook_token = headers
        .get("asaas-access-token")
        .or_else(|| headers.get("X-Signature"))
        .or_else(|| headers.get("X-Webhook-Secret"))
        .and_then(|header| header.to_str().ok());
    let gateway = get_payment_gateway();
    if !gateway.verify_webhook_hmac(&body, webhook_token) {
        return Err(ApiError::Unauthorized(
            "Invalid or missing PIX webhook authentication".to_string(),
        ));
    }

    let payload = parse_verified_webhook(&provider, &body)?;
    if !matches!(
        payload.status.as_str(),
        "APPROVED" | "COMPLETED" | "RECEIVED"
    ) {
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

    let mut transaction = state.db.begin().await?;
    let row: Option<WalletRow> = sqlx::query_as(
        "SELECT id, user_id, amount, transaction_type, status, external_tx_id \
         FROM wallet_transactions WHERE idempotency_key = $1 FOR UPDATE",
    )
    .bind(&payload.tx_id)
    .fetch_optional(&mut *transaction)
    .await?;
    let (wallet_id, user_id, amount, transaction_type, status, stored_external_id) =
        row.ok_or_else(|| ApiError::NotFound("Unknown PIX transaction".to_string()))?;

    if transaction_type != "DEPOSIT" {
        return Err(ApiError::BadRequest(
            "PIX webhook does not reference a deposit".to_string(),
        ));
    }
    if status == "COMPLETED" {
        transaction.rollback().await?;
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
    if amount != callback_amount || stored_external_id.as_deref() != Some(callback_external_id) {
        return Err(ApiError::BadRequest(
            "PIX webhook does not match the pending ledger entry".to_string(),
        ));
    }

    let credited = sqlx::query("UPDATE users SET balance = balance + $1 WHERE id = $2")
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
    .bind(&payload.status)
    .bind(wallet_id)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'PIX_DEPOSIT_SETTLED', $2)",
    )
    .bind(user_id.to_string())
    .bind(serde_json::json!({"tx_id": payload.tx_id, "external_tx_id": callback_external_id, "amount_cents": amount}))
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
        "UPDATE users SET balance = balance - $1 \
         WHERE id = $2::uuid AND balance >= $1 RETURNING balance",
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
    use super::{brl_value_to_cents, parse_verified_webhook};

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
}
