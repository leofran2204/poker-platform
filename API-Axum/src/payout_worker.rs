// payout_worker.rs — worker reconciliado de saques DePix (off-ramp). Parte 1/3:
// infra (env, auditoria, reserva/rejeição/confirmação, claim do outbox).

use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::middleware::auth::{AuthUser, RequireAuth};
use crate::state::AppState;

const MAX_ATTEMPTS: i64 = 10;
const STALE_SENDING_MINUTES: i64 = 10;
const RECONCILE_AFTER_SECONDS: i64 = 60;

fn poll_interval_secs() -> u64 {
    std::env::var("DEPIX_PAYOUT_POLL_SECS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| (10..=300).contains(value))
        .unwrap_or(20)
}

fn dry_run() -> bool {
    std::env::var("DEPIX_PAYOUT_DRYRUN")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn require_admin(auth_user: &AuthUser) -> Result<(), ApiError> {
    if auth_user.role == "admin" {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "Administrator access is required".to_string(),
        ))
    }
}

async fn audit(
    executor: impl sqlx::PgExecutor<'_>,
    user_id: &str,
    action: &str,
    metadata: serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(action)
        .bind(metadata)
        .execute(executor)
        .await?;
    Ok(())
}

/// Recredita o saldo reservado e encerra como REJECTED/FAILED. Falha do
/// provedor ou do pedido — nunca estorno PIX, que não existe aqui.
async fn reject_and_recredit(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    wallet_id: uuid::Uuid,
    user_id: &str,
    tx_key: &str,
    amount: i64,
    reason: &str,
) -> Result<(), ApiError> {
    sqlx::query("UPDATE users SET balance_real = balance_real + $1 WHERE id = $2::uuid")
        .bind(amount)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "UPDATE wallet_transactions SET status = 'REJECTED', provider_status = 'FAILED', updated_at = NOW() \
         WHERE id = $1",
    )
    .bind(wallet_id)
    .execute(&mut **tx)
    .await?;
    audit(
        &mut **tx,
        user_id,
        "PIX_WITHDRAWAL_REJECTED",
        serde_json::json!({"tx_id": tx_key, "amount_cents": amount, "reason": reason}),
    )
    .await?;
    Ok(())
}

async fn confirm_payout(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    wallet_id: uuid::Uuid,
    user_id: &str,
    tx_key: &str,
    amount: i64,
    provider_tx_id: &str,
) -> Result<(), ApiError> {
    sqlx::query(
        "UPDATE wallet_transactions SET status = 'COMPLETED', provider_status = 'CONFIRMED', \
         provider_tx_id = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(provider_tx_id)
    .bind(wallet_id)
    .execute(&mut **tx)
    .await?;
    audit(
        &mut **tx,
        user_id,
        "PIX_WITHDRAWAL_CONFIRMED",
        serde_json::json!({"tx_id": tx_key, "amount_cents": amount, "provider_tx_id": provider_tx_id}),
    )
    .await?;
    Ok(())
}

/// Reivindica um evento QUEUED (SKIP LOCKED: N réplicas, um dono).
async fn claim_next(db: &PgPool) -> Result<Option<(uuid::Uuid, String)>, sqlx::Error> {
    let mut tx = db.begin().await?;
    let row: Option<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT id, aggregate_id FROM outbox_events \
         WHERE event_type = 'PIX_WITHDRAWAL_REQUESTED' AND status = 'PENDING' \
         ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED",
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some((outbox_id, tx_key)) = row else {
        tx.rollback().await?;
        return Ok(None);
    };
    let claimed: Option<uuid::Uuid> = sqlx::query_scalar(
        "UPDATE wallet_transactions SET provider_status = 'SENDING', updated_at = NOW() \
         WHERE idempotency_key = $1 AND transaction_type = 'WITHDRAW' AND status = 'PENDING' \
           AND provider_status = 'QUEUED' RETURNING id",
    )
    .bind(&tx_key)
    .fetch_optional(&mut *tx)
    .await?;
    if claimed.is_none() {
        sqlx::query(
            "UPDATE outbox_events SET status = 'PROCESSED', processed_at = NOW() WHERE id = $1",
        )
        .bind(outbox_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(None);
    }
    tx.commit().await?;
    Ok(Some((outbox_id, tx_key)))
}

async fn finish_outbox(
    db: &PgPool,
    outbox_id: uuid::Uuid,
    ok: bool,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    if ok {
        sqlx::query(
            "UPDATE outbox_events SET status = 'PROCESSED', processed_at = NOW() WHERE id = $1",
        )
        .bind(outbox_id)
        .execute(db)
        .await?;
    } else {
        sqlx::query(
            "UPDATE outbox_events SET retry_count = retry_count + 1, error_message = $1 WHERE id = $2",
        )
        .bind(error)
        .bind(outbox_id)
        .execute(db)
        .await?;
    }
    Ok(())
}
struct PayoutJob {
    outbox_id: uuid::Uuid,
    wallet_id: uuid::Uuid,
    tx_key: String,
    user_id: String,
    amount_cents: i64,
    pix_blob: String,
}

async fn load_job(
    db: &PgPool,
    outbox_id: uuid::Uuid,
    tx_key: &str,
) -> Result<Option<PayoutJob>, sqlx::Error> {
    let row: Option<(uuid::Uuid, String, i64, Option<String>)> = sqlx::query_as(
        "SELECT id, user_id::text, amount, pix_key_ciphertext FROM wallet_transactions \
         WHERE idempotency_key = $1 AND transaction_type = 'WITHDRAW' AND provider_status = 'SENDING'",
    )
    .bind(tx_key)
    .fetch_optional(db)
    .await?;
    match row {
        Some((wallet_id, user_id, amount_cents, Some(pix_blob))) => Ok(Some(PayoutJob {
            outbox_id,
            wallet_id,
            tx_key: tx_key.to_string(),
            user_id,
            amount_cents,
            pix_blob,
        })),
        _ => Ok(None),
    }
}

fn parse_pix_blob(blob: &str) -> Result<(String, String), String> {
    let value: serde_json::Value =
        serde_json::from_str(blob).map_err(|_| "pix-key crypto: invalid blob".to_string())?;
    let pix_key = value
        .get("pix_key")
        .and_then(serde_json::Value::as_str)
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| "pix-key crypto: invalid blob".to_string())?;
    let tax_number = value
        .get("tax_number")
        .and_then(serde_json::Value::as_str)
        .filter(|tax| !tax.trim().is_empty())
        .ok_or_else(|| "pix-key crypto: invalid blob".to_string())?;
    Ok((pix_key.to_string(), tax_number.to_string()))
}

async fn fail_job(db: &PgPool, job: &PayoutJob, reason: &str) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    let attempts: i64 = sqlx::query_scalar("SELECT retry_count FROM outbox_events WHERE id = $1")
        .bind(job.outbox_id)
        .fetch_one(&mut *tx)
        .await?;
    if attempts + 1 >= MAX_ATTEMPTS {
        reject_and_recredit(
            &mut tx,
            job.wallet_id,
            &job.user_id,
            &job.tx_key,
            job.amount_cents,
            reason,
        )
        .await?;
        finish_outbox(db, job.outbox_id, true, None).await?;
        tracing::warn!(
            tx_id = %job.tx_key,
            reason = reason,
            "payout worker: attempts exhausted, balance recredited"
        );
    } else {
        sqlx::query(
            "UPDATE wallet_transactions SET provider_status = 'QUEUED', updated_at = NOW() WHERE id = $1",
        )
        .bind(job.wallet_id)
        .execute(&mut *tx)
        .await?;
        finish_outbox(db, job.outbox_id, false, Some(reason)).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn send_job(db: &PgPool, job: &PayoutJob) -> Result<(), ApiError> {
    if dry_run() {
        let mut tx = db.begin().await?;
        sqlx::query(
            "UPDATE wallet_transactions SET provider_status = 'SENT', \
             provider_tx_id = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(format!("dryrun_{}", job.tx_key))
        .bind(job.wallet_id)
        .execute(&mut *tx)
        .await?;
        audit(
            &mut *tx,
            &job.user_id,
            "PIX_WITHDRAWAL_SENT",
            serde_json::json!({"tx_id": &job.tx_key, "amount_cents": job.amount_cents, "dry_run": true}),
        )
        .await?;
        tx.commit().await?;
        finish_outbox(db, job.outbox_id, true, None).await?;
        return Ok(());
    }
    let gateway = crate::payment_gateway::depix_gateway_from_env()
        .map_err(|reason| ApiError::Internal(format!("DePix payout unavailable: {reason}")))?;
    let key = crate::pix_key_crypto::load_key()
        .map_err(|_| ApiError::Internal("Automatic PIX payouts are unavailable".to_string()))?;
    let blob = crate::pix_key_crypto::decrypt_blob(&key, &job.pix_blob)
        .map_err(|_| ApiError::Internal("Automatic PIX payouts are unavailable".to_string()))?;
    let (pix_key, tax_number) = parse_pix_blob(&blob)
        .map_err(|_| ApiError::Internal("Automatic PIX payouts are unavailable".to_string()))?;
    let amount_u64: u64 = job
        .amount_cents
        .try_into()
        .map_err(|_| ApiError::Internal("Automatic PIX payouts are unavailable".to_string()))?;
    let tx_key = job.tx_key.clone();
    let result = tokio::task::spawn_blocking(move || {
        gateway.create_withdraw_payout(&tx_key, amount_u64, &pix_key, &tax_number)
    })
    .await
    .map_err(|_| ApiError::Internal("Automatic PIX payouts are unavailable".to_string()))?;
    match result {
        Ok(payout) => {
            let mut tx = db.begin().await?;
            sqlx::query(
                "UPDATE wallet_transactions SET provider_status = 'SENT', \
                 provider_tx_id = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&payout.external_tx_id)
            .bind(job.wallet_id)
            .execute(&mut *tx)
            .await?;
            audit(
                &mut *tx,
                &job.user_id,
                "PIX_WITHDRAWAL_SENT",
                serde_json::json!({"tx_id": &job.tx_key, "amount_cents": job.amount_cents, "provider_tx_id": &payout.external_tx_id}),
            )
            .await?;
            tx.commit().await?;
            finish_outbox(db, job.outbox_id, true, None).await?;
            Ok(())
        }
        Err(reason) => {
            tracing::warn!(tx_id = %job.tx_key, "payout worker: provider send failed, will retry");
            fail_job(db, job, &reason).await
        }
    }
}
/// Reconcilia saques SENT contra o provedor (polling). Linhas `dryrun_*`
/// ficam como estão: evidência de lab, sem HTTP externo.
async fn reconcile_sent(db: &PgPool) -> Result<(), ApiError> {
    if dry_run() {
        return Ok(());
    }
    let gateway = match crate::payment_gateway::depix_gateway_from_env() {
        Ok(gateway) => gateway,
        Err(_) => return Ok(()),
    };
    let rows: Vec<(uuid::Uuid, String, String, i64, String)> = sqlx::query_as(
        "SELECT id, idempotency_key, user_id::text, amount, provider_tx_id \
         FROM wallet_transactions \
         WHERE transaction_type = 'WITHDRAW' AND status = 'PENDING' AND provider_status = 'SENT' \
           AND provider_tx_id IS NOT NULL AND provider_tx_id NOT LIKE 'dryrun\\_%' \
           AND updated_at < NOW() - ($1 * INTERVAL '1 second')",
    )
    .bind(RECONCILE_AFTER_SECONDS)
    .fetch_all(db)
    .await?;
    for (wallet_id, tx_key, user_id, amount, provider_tx_id) in rows {
        let status = match gateway.fetch_withdrawal_status(&provider_tx_id) {
            Ok(status) => status.status.to_ascii_lowercase(),
            Err(_) => {
                tracing::warn!(tx_id = %tx_key, "payout worker: reconcile poll failed, retrying later");
                continue;
            }
        };
        let mut tx = db.begin().await?;
        match status.as_str() {
            "sent" => {
                confirm_payout(
                    &mut tx,
                    wallet_id,
                    &user_id,
                    &tx_key,
                    amount,
                    &provider_tx_id,
                )
                .await?;
            }
            "error" | "canceled" | "refunded" | "replaced" => {
                reject_and_recredit(&mut tx, wallet_id, &user_id, &tx_key, amount, &status).await?;
            }
            _ => {
                tx.rollback().await?;
                continue;
            }
        }
        tx.commit().await?;
    }
    Ok(())
}

/// Recupera SENDING órfãos (crash entre claim e envio) para QUEUED.
async fn recover_stale(db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE wallet_transactions SET provider_status = 'QUEUED', updated_at = NOW() \
         WHERE transaction_type = 'WITHDRAW' AND status = 'PENDING' AND provider_status = 'SENDING' \
           AND updated_at < NOW() - ($1 * INTERVAL '1 minute')",
    )
    .bind(STALE_SENDING_MINUTES)
    .execute(db)
    .await?;
    Ok(())
}

async fn pump_once(db: &PgPool) -> Result<(), ApiError> {
    if let Err(reason) = recover_stale(db).await {
        tracing::warn!("payout worker: stale recovery failed: {reason}");
    }
    match claim_next(db).await {
        Ok(Some((outbox_id, tx_key))) => match load_job(db, outbox_id, &tx_key).await {
            Ok(Some(job)) => {
                if let Err(reason) = send_job(db, &job).await {
                    tracing::warn!(tx_id = %tx_key, "payout worker: send failed: {reason:?}");
                    let _ = fail_job(db, &job, "send failed").await;
                }
            }
            Ok(None) => {
                let _ = finish_outbox(db, outbox_id, true, None).await;
            }
            Err(reason) => {
                tracing::warn!("payout worker: load failed: {reason}");
            }
        },
        Ok(None) => {}
        Err(reason) => {
            tracing::warn!("payout worker: claim failed: {reason}");
        }
    }
    if let Err(reason) = reconcile_sent(db).await {
        tracing::warn!("payout worker: reconcile failed: {reason:?}");
    }
    Ok(())
}

/// Loop do worker. Sem segredos configurados, `send_job` falha fechado por
/// linha (volta para QUEUED) e o loop segue dormindo — nunca trava o boot.
pub async fn run_payout_worker(db: PgPool) {
    tracing::info!("payout worker started");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(poll_interval_secs()));
    loop {
        interval.tick().await;
        if let Err(reason) = pump_once(&db).await {
            tracing::warn!("payout worker: pump failed: {reason:?}");
        }
    }
}

/// GET /api/admin/payouts/held — fila de revisão manual (sem dados sensíveis).
pub async fn list_held_payouts(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_admin(&auth_user)?;
    let rows: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT idempotency_key, user_id::text, amount, created_at::text \
         FROM wallet_transactions \
         WHERE transaction_type = 'WITHDRAW' AND status = 'PENDING' AND provider_status = 'HELD' \
         ORDER BY created_at LIMIT 100",
    )
    .fetch_all(&state.db)
    .await?;
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(tx_id, user_id, amount_cents, created_at)| {
            serde_json::json!({"tx_id": tx_id, "user_id": user_id, "amount_cents": amount_cents, "created_at": created_at})
        })
        .collect();
    Ok(Json(serde_json::json!({"held": items})))
}

/// POST /api/admin/payouts/:id/approve — HELD volta para a fila automática.
pub async fn approve_payout(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_admin(&auth_user)?;
    let mut tx = state.db.begin().await?;
    let row: Option<(uuid::Uuid, String)> = sqlx::query_as(
        "UPDATE wallet_transactions SET provider_status = 'QUEUED', updated_at = NOW() \
         WHERE idempotency_key = $1 AND transaction_type = 'WITHDRAW' AND status = 'PENDING' \
           AND provider_status = 'HELD' RETURNING id, user_id::text",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((_, user_id)) = row else {
        return Err(ApiError::NotFound("Held payout not found".to_string()));
    };
    audit(
        &mut *tx,
        &user_id,
        "PIX_WITHDRAWAL_APPROVED",
        serde_json::json!({"tx_id": &id, "admin_id": &auth_user.user_id}),
    )
    .await?;
    // Re arma o outbox: o worker pode já ter dispensado o evento original
    // enquanto a linha estava HELD (claim só consome QUEUED).
    sqlx::query(
        "INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, payload) \
         VALUES ('wallet_transaction', $1, 'PIX_WITHDRAWAL_REQUESTED', $2)",
    )
    .bind(&id)
    .bind(serde_json::json!({"tx_id": &id, "approved_by": &auth_user.user_id}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(serde_json::json!({"tx_id": id, "status": "QUEUED"})))
}

/// POST /api/admin/payouts/:id/reject — HELD recredita o saldo e encerra.
pub async fn reject_payout(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_admin(&auth_user)?;
    let mut tx = state.db.begin().await?;
    let row: Option<(uuid::Uuid, String, i64)> = sqlx::query_as(
        "SELECT id, user_id::text, amount FROM wallet_transactions \
         WHERE idempotency_key = $1 AND transaction_type = 'WITHDRAW' AND status = 'PENDING' \
           AND provider_status = 'HELD' FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((wallet_id, user_id, amount)) = row else {
        tx.rollback().await?;
        return Err(ApiError::NotFound("Held payout not found".to_string()));
    };
    reject_and_recredit(&mut tx, wallet_id, &user_id, &id, amount, "manual_reject").await?;
    audit(
        &mut *tx,
        &user_id,
        "PIX_WITHDRAWAL_MANUAL_REJECT",
        serde_json::json!({"tx_id": &id, "admin_id": &auth_user.user_id}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(serde_json::json!({"tx_id": id, "status": "REJECTED"})))
}

fn extract_withdrawal_id(body: &serde_json::Value) -> Option<String> {
    for scope in [body.get("data"), body.get("response"), Some(body)] {
        let Some(scope) = scope else { continue };
        for key in ["id", "withdrawalId", "withdrawal_id"] {
            if let Some(id) = scope.get(key).and_then(serde_json::Value::as_str) {
                if !id.trim().is_empty() {
                    return Some(id.trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_withdraw_status(body: &serde_json::Value) -> String {
    for scope in [body.get("data"), body.get("response"), Some(body)] {
        let Some(scope) = scope else { continue };
        if let Some(status) = scope.get("status").and_then(serde_json::Value::as_str) {
            return status.to_ascii_lowercase();
        }
    }
    String::new()
}

fn classify_withdraw_status(status: &str) -> &'static str {
    match status {
        "sent" => "confirmed",
        "error" | "canceled" | "refunded" | "replaced" => "rejected",
        _ => "ack",
    }
}

/// Aplica um evento `withdraw.*` autenticado (chamado pela rota de webhook
/// após HMAC + conferência de cabeçalhos + dedup por `event_id`).
/// Desconhecido ou intermediário: confirma recebimento sem ação na carteira.
pub async fn apply_withdraw_webhook(
    db: &PgPool,
    body: &serde_json::Value,
    event_id: &str,
    event_type: &str,
) -> Result<
    (
        axum::http::StatusCode,
        Json<crate::payments_routes::WebhookResponse>,
    ),
    ApiError,
> {
    use crate::payments_routes::WebhookResponse;
    use axum::http::StatusCode;

    let ack = |status: &str, message: &str| {
        Ok((
            StatusCode::OK,
            Json(WebhookResponse {
                status: status.to_string(),
                message: message.to_string(),
            }),
        ))
    };
    let withdrawal_id = extract_withdrawal_id(body);
    let status = extract_withdraw_status(body);
    let mut tx = db.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO payment_webhook_events \
         (provider, event_id, event_type, payload_sha256) VALUES ('depix', $1, $2, 'withdraw') \
         ON CONFLICT (provider, event_id) DO NOTHING",
    )
    .bind(event_id)
    .bind(event_type)
    .execute(&mut *tx)
    .await?;
    if inserted.rows_affected() == 0 {
        tx.rollback().await?;
        return ack("IGNORED", "DePix event was already processed");
    }
    let Some(withdrawal_id) = withdrawal_id else {
        tx.commit().await?;
        return ack(
            "IGNORED",
            "Authenticated DePix event requires no automatic wallet action",
        );
    };
    let row: Option<(uuid::Uuid, String, i64, String)> = sqlx::query_as(
        "SELECT id, user_id::text, amount, idempotency_key FROM wallet_transactions \
         WHERE provider_tx_id = $1 AND transaction_type = 'WITHDRAW' AND status = 'PENDING' FOR UPDATE",
    )
    .bind(&withdrawal_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((wallet_id, user_id, amount, tx_key)) = row else {
        tx.commit().await?;
        return ack(
            "IGNORED",
            "Authenticated DePix event requires no automatic wallet action",
        );
    };
    match classify_withdraw_status(&status) {
        "confirmed" => {
            confirm_payout(
                &mut tx,
                wallet_id,
                &user_id,
                &tx_key,
                amount,
                &withdrawal_id,
            )
            .await?;
        }
        "rejected" => {
            reject_and_recredit(&mut tx, wallet_id, &user_id, &tx_key, amount, &status).await?;
        }
        _ => {
            tx.commit().await?;
            return ack(
                "IGNORED",
                "Authenticated DePix event requires no automatic wallet action",
            );
        }
    }
    tx.commit().await?;
    ack("SETTLED", "DePix withdrawal event applied")
}

#[cfg(test)]
mod tests {
    use super::{
        classify_withdraw_status, extract_withdraw_status, extract_withdrawal_id, parse_pix_blob,
    };

    #[test]
    fn pix_blob_roundtrip_fields() {
        let (key, tax) = parse_pix_blob(r#"{"pix_key":"a@b.c","tax_number":"123"}"#).unwrap();
        assert_eq!(key, "a@b.c");
        assert_eq!(tax, "123");
        assert!(parse_pix_blob(r#"{"pix_key":"","tax_number":"123"}"#).is_err());
        assert!(parse_pix_blob("nao-json").is_err());
    }

    #[test]
    fn withdrawal_id_accepts_provider_shapes() {
        let flat = serde_json::json!({"id": "w1", "status": "sent"});
        let wrapped = serde_json::json!({"response": {"id": "w2", "status": "sent"}});
        let camel = serde_json::json!({"data": {"withdrawalId": "w3"}});
        let snake = serde_json::json!({"data": {"withdrawal_id": "w4"}});
        assert_eq!(extract_withdrawal_id(&flat).as_deref(), Some("w1"));
        assert_eq!(extract_withdrawal_id(&wrapped).as_deref(), Some("w2"));
        assert_eq!(extract_withdrawal_id(&camel).as_deref(), Some("w3"));
        assert_eq!(extract_withdrawal_id(&snake).as_deref(), Some("w4"));
        assert_eq!(extract_withdraw_status(&flat), "sent");
        assert!(extract_withdrawal_id(&serde_json::json!({})).is_none());
    }

    #[test]
    fn withdrawal_status_classes_match_provider_lifecycle() {
        assert_eq!(classify_withdraw_status("sent"), "confirmed");
        assert_eq!(classify_withdraw_status("error"), "rejected");
        assert_eq!(classify_withdraw_status("canceled"), "rejected");
        assert_eq!(classify_withdraw_status("refunded"), "rejected");
        assert_eq!(classify_withdraw_status("replaced"), "rejected");
        assert_eq!(classify_withdraw_status("unsent"), "ack");
        assert_eq!(classify_withdraw_status("sending"), "ack");
        assert_eq!(classify_withdraw_status(""), "ack");
    }
}
// PAYOUT-WORKER-PART3
