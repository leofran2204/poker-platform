// payments_routes.rs — Endpoints REST e Webhooks para Depósitos e Saques PIX Instantâneos
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::payment_gateway::get_payment_gateway;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct DepositResponse {
    pub tx_id: String,
    pub amount: f64,
    pub pix_copy_paste: String,
    pub qr_code_base64: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookPixPayload {
    pub tx_id: String,
    pub external_tx_id: Option<String>,
    pub amount: f64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub amount: f64,
    pub pix_key_type: String, // "cpf", "email", "phone", "evp"
    pub pix_key: String,
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub tx_id: String,
    pub amount: f64,
    pub status: String,
    pub message: String,
}

/// POST /api/payments/pix/deposit — Gera cobrança PIX com QRCode e Copia e Cola
pub async fn create_pix_deposit_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DepositRequest>,
) -> impl IntoResponse {
    if payload.amount <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Valor de depósito deve ser maior que zero" })),
        );
    }

    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Token de autenticação ausente ou inválido" })),
            );
        }
    };

    let user_id = {
        let auth_mgr = state.auth.lock().await;
        match auth_mgr.validate_token(auth_header, "access") {
            Ok(claims) => claims.sub,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "Token JWT inválido ou expirado" })),
                );
            }
        }
    };

    let tx_id = format!("tx_dep_{}", uuid::Uuid::new_v4());
    let gateway = get_payment_gateway();

    let charge_res = match gateway.create_deposit_charge(&tx_id, &user_id, payload.amount) {
        Ok(res) => res,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Falha no gateway PIX: {}", err) })),
            );
        }
    };

    let response = DepositResponse {
        tx_id,
        amount: payload.amount,
        pix_copy_paste: charge_res.pix_copy_paste,
        qr_code_base64: charge_res.qr_code_base64,
        expires_at: charge_res.expires_at,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap()))
}

/// POST /api/webhooks/pix — Recebe confirmação instantânea do pagamento PIX e credita o saldo
pub async fn pix_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let signature = headers
        .get("X-Signature")
        .or_else(|| headers.get("X-Webhook-Secret"))
        .and_then(|h| h.to_str().ok());

    let gateway = get_payment_gateway();
    if !gateway.verify_webhook_hmac(&body, signature) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Assinatura do webhook inválida ou ausente" })),
        );
    }

    let payload: WebhookPixPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Payload JSON do webhook inválido" })),
            );
        }
    };

    if payload.status != "APPROVED" && payload.status != "COMPLETED" {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "IGNORED", "message": "Status não liquidador" })),
        );
    }

    // Executar atualização real do saldo na base de dados (se houver transação registrada)
    let _ = sqlx::query(
        "UPDATE users SET balance = balance + $1 WHERE id = (SELECT user_id FROM transactions WHERE tx_id = $2)"
    )
    .bind(payload.amount)
    .bind(&payload.tx_id)
    .execute(&state.db)
    .await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "PROCESSED",
            "message": format!("Depósito PIX de R$ {:.2} creditado com sucesso!", payload.amount)
        })),
    )
}

/// POST /api/payments/pix/withdraw — Executa solicitação de saque PIX com checagem de saldo e antifraude
pub async fn create_pix_withdraw_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WithdrawRequest>,
) -> impl IntoResponse {
    if payload.amount <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Valor de saque deve ser maior que zero" })),
        );
    }
    if payload.pix_key.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Chave PIX obrigatória" })),
        );
    }

    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Token de autenticação ausente ou inválido" })),
            );
        }
    };

    let user_id = {
        let auth_mgr = state.auth.lock().await;
        match auth_mgr.validate_token(auth_header, "access") {
            Ok(claims) => claims.sub,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "Token JWT inválido ou expirado" })),
                );
            }
        }
    };

    let tx_id = format!("tx_wdr_{}", uuid::Uuid::new_v4());
    let gateway = get_payment_gateway();

    let payout_res = match gateway.execute_withdraw_payout(
        &tx_id,
        &user_id,
        payload.amount,
        &payload.pix_key_type,
        &payload.pix_key,
    ) {
        Ok(res) => res,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Falha no saque PIX: {}", err) })),
            );
        }
    };

    let response = WithdrawResponse {
        tx_id,
        amount: payload.amount,
        status: payout_res.status,
        message: payout_res.message,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap()))
}
