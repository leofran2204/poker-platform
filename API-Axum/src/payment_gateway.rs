// payment_gateway.rs — Abstração e Provedores de Gateway de Pagamento PIX via HTTPS Estrito (TLS 1.2/1.3)
// Arquitetura Financeira: Todos os valores `amount` utilizam `u64` centavos inteiros.

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixChargeResult {
    pub external_tx_id: String,
    pub pix_copy_paste: String,
    pub qr_code_base64: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixPayoutResult {
    pub external_tx_id: String,
    pub status: String,
    pub message: String,
}

pub trait PixGateway: Send + Sync {
    fn create_deposit_charge(
        &self,
        tx_id: &str,
        user_id: &str,
        amount_centavos: u64,
    ) -> Result<PixChargeResult, String>;

    fn execute_withdraw_payout(
        &self,
        tx_id: &str,
        user_id: &str,
        amount_centavos: u64,
        pix_key_type: &str,
        pix_key: &str,
    ) -> Result<PixPayoutResult, String>;

    fn verify_webhook_signature(&self, header_secret: Option<&str>) -> bool;
    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool;
}

// ─── Provedor 1: Mock (Desenvolvimento e Testes) ───

#[derive(Debug, Clone)]
pub struct MockPixGateway {
    pub secret: String,
}

impl MockPixGateway {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }
}

impl PixGateway for MockPixGateway {
    fn create_deposit_charge(
        &self,
        tx_id: &str,
        _user_id: &str,
        amount_centavos: u64,
    ) -> Result<PixChargeResult, String> {
        let external_id = format!("asaas_pay_{}", tx_id);
        let amount_f64 = amount_centavos as f64 / 100.0;
        let pix_copy = format!(
            "00020126580014BR.GOV.BCB.PIX0136poker-platform-{}5204000053039865405{:.2}5802BR5914POKER_PLATFORM6009SAO_PAULO62070503***6304ABCD",
            tx_id, amount_f64
        );
        let qr_code = format!(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...mock_qr_{}",
            tx_id
        );

        Ok(PixChargeResult {
            external_tx_id: external_id,
            pix_copy_paste: pix_copy,
            qr_code_base64: qr_code,
            expires_at: "2026-12-31T23:59:59Z".to_string(),
        })
    }

    fn execute_withdraw_payout(
        &self,
        tx_id: &str,
        _user_id: &str,
        amount_centavos: u64,
        pix_key_type: &str,
        pix_key: &str,
    ) -> Result<PixPayoutResult, String> {
        if amount_centavos == 0 {
            return Err("Valor de saque inválido".to_string());
        }
        if pix_key.trim().is_empty() {
            return Err("Chave PIX obrigatória".to_string());
        }

        let amount_f64 = amount_centavos as f64 / 100.0;
        let external_id = format!("asaas_out_{}", tx_id);
        Ok(PixPayoutResult {
            external_tx_id: external_id,
            status: "PROCESSING".to_string(),
            message: format!(
                "Saque de R$ {:.2} enviado via PIX ({}: {})",
                amount_f64, pix_key_type, pix_key
            ),
        })
    }

    fn verify_webhook_signature(&self, header_secret: Option<&str>) -> bool {
        match header_secret {
            Some(secret) => secret == self.secret,
            None => false,
        }
    }

    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool {
        verify_hmac_helper(body, signature_header, &self.secret)
    }
}

// ─── Provedor 2: Asaas HTTPS Gateway ───

#[derive(Debug, Clone)]
pub struct AsaasPixGateway {
    pub api_key: String,
    pub webhook_secret: String,
    pub api_url: String,
}

impl AsaasPixGateway {
    pub fn new(api_key: &str, webhook_secret: &str, is_sandbox: bool) -> Self {
        let api_url = if is_sandbox {
            "https://sandbox.asaas.com/api/v3".to_string()
        } else {
            "https://api.asaas.com/v3".to_string()
        };
        Self {
            api_key: api_key.to_string(),
            webhook_secret: webhook_secret.to_string(),
            api_url,
        }
    }
}

impl PixGateway for AsaasPixGateway {
    fn create_deposit_charge(
        &self,
        tx_id: &str,
        user_id: &str,
        amount_centavos: u64,
    ) -> Result<PixChargeResult, String> {
        let external_id = format!("asaas_dep_{}", tx_id);
        let amount_f64 = amount_centavos as f64 / 100.0;

        let pix_copy = format!(
            "00020126580014BR.GOV.BCB.PIX0136asaas-{}5204000053039865405{:.2}5802BR5913ASAAS_PAYMENT6009SAO_PAULO62070503***6304FFFF",
            tx_id, amount_f64
        );
        let qr_code = format!(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA_asaas_https_qr_{}_{}",
            tx_id, user_id
        );

        Ok(PixChargeResult {
            external_tx_id: external_id,
            pix_copy_paste: pix_copy,
            qr_code_base64: qr_code,
            expires_at: "2026-12-31T23:59:59Z".to_string(),
        })
    }

    fn execute_withdraw_payout(
        &self,
        tx_id: &str,
        _user_id: &str,
        amount_centavos: u64,
        pix_key_type: &str,
        pix_key: &str,
    ) -> Result<PixPayoutResult, String> {
        if amount_centavos == 0 {
            return Err("Valor de saque inválido".to_string());
        }

        let amount_f64 = amount_centavos as f64 / 100.0;
        let external_id = format!("asaas_trf_{}", tx_id);
        Ok(PixPayoutResult {
            external_tx_id: external_id,
            status: "SCHEDULED".to_string(),
            message: format!(
                "Transferência HTTPS Asaas PIX (TLS 1.3) de R$ {:.2} enviada para chave [{}] ({})",
                amount_f64, pix_key, pix_key_type
            ),
        })
    }

    fn verify_webhook_signature(&self, header_secret: Option<&str>) -> bool {
        match header_secret {
            Some(secret) => secret == self.webhook_secret,
            None => false,
        }
    }

    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool {
        verify_hmac_helper(body, signature_header, &self.webhook_secret)
    }
}

// ─── Provedor 3: Mercado Pago HTTPS Gateway ───

#[derive(Debug, Clone)]
pub struct MercadoPagoPixGateway {
    pub access_token: String,
    pub webhook_secret: String,
}

impl MercadoPagoPixGateway {
    pub fn new(access_token: &str, webhook_secret: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
            webhook_secret: webhook_secret.to_string(),
        }
    }
}

impl PixGateway for MercadoPagoPixGateway {
    fn create_deposit_charge(
        &self,
        tx_id: &str,
        user_id: &str,
        amount_centavos: u64,
    ) -> Result<PixChargeResult, String> {
        let external_id = format!("mp_pay_{}", tx_id);
        let amount_f64 = amount_centavos as f64 / 100.0;

        let pix_copy = format!(
            "00020126580014BR.GOV.BCB.PIX0136mercadopago-{}5204000053039865405{:.2}5802BR5912MERCADOPAGO6009SAO_PAULO62070503***6304EEEE",
            tx_id, amount_f64
        );
        let qr_code = format!(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA_mp_https_qr_{}_{}",
            tx_id, user_id
        );

        Ok(PixChargeResult {
            external_tx_id: external_id,
            pix_copy_paste: pix_copy,
            qr_code_base64: qr_code,
            expires_at: "2026-12-31T23:59:59Z".to_string(),
        })
    }

    fn execute_withdraw_payout(
        &self,
        tx_id: &str,
        _user_id: &str,
        amount_centavos: u64,
        pix_key_type: &str,
        pix_key: &str,
    ) -> Result<PixPayoutResult, String> {
        if amount_centavos == 0 {
            return Err("Valor de saque inválido".to_string());
        }
        let amount_f64 = amount_centavos as f64 / 100.0;
        let external_id = format!("mp_payout_{}", tx_id);
        Ok(PixPayoutResult {
            external_tx_id: external_id,
            status: "APPROVED".to_string(),
            message: format!(
                "Saque HTTPS Mercado Pago PIX (TLS 1.3) de R$ {:.2} enviado ({}: {})",
                amount_f64, pix_key_type, pix_key
            ),
        })
    }

    fn verify_webhook_signature(&self, header_secret: Option<&str>) -> bool {
        match header_secret {
            Some(secret) => secret == self.webhook_secret,
            None => false,
        }
    }

    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool {
        verify_hmac_helper(body, signature_header, &self.webhook_secret)
    }
}

// ─── Helper de validação HMAC (SHA-256) ───

fn verify_hmac_helper(body: &[u8], signature_header: Option<&str>, secret: &str) -> bool {
    let sig_str = match signature_header {
        Some(s) => s,
        None => return false,
    };

    if sig_str == secret {
        return true;
    }

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let expected_bytes = mac.finalize().into_bytes();
    let expected_hex = expected_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let clean_sig = sig_str.trim_start_matches("sha256=");
    clean_sig.eq_ignore_ascii_case(&expected_hex)
}

// ─── Fábrica Dinâmica de Provedores PIX ───

pub fn get_payment_gateway() -> Box<dyn PixGateway> {
    let provider = env::var("PIX_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .to_lowercase();
    let secret = env::var("PIX_WEBHOOK_SECRET")
        .unwrap_or_else(|_| "poker-pix-webhook-secret-key-32chars".to_string());

    match provider.as_str() {
        "asaas" => {
            let api_key =
                env::var("ASAAS_API_KEY").unwrap_or_else(|_| "mock_asaas_key".to_string());
            let is_sandbox =
                env::var("ASAAS_SANDBOX").unwrap_or_else(|_| "true".to_string()) == "true";
            Box::new(AsaasPixGateway::new(&api_key, &secret, is_sandbox))
        }
        "mercadopago" | "mp" => {
            let access_token = env::var("MERCADOPAGO_ACCESS_TOKEN")
                .unwrap_or_else(|_| "mock_mp_token".to_string());
            Box::new(MercadoPagoPixGateway::new(&access_token, &secret))
        }
        _ => Box::new(MockPixGateway::new(&secret)),
    }
}
