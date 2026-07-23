// payment_gateway.rs — Abstração e Provedores de Gateway de Pagamento PIX (Asaas / Mercado Pago / Mock)
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
        amount: f64,
    ) -> Result<PixChargeResult, String>;

    fn execute_withdraw_payout(
        &self,
        tx_id: &str,
        user_id: &str,
        amount: f64,
        pix_key_type: &str,
        pix_key: &str,
    ) -> Result<PixPayoutResult, String>;

    fn verify_webhook_signature(&self, header_secret: Option<&str>) -> bool;
    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool;
}

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
        amount: f64,
    ) -> Result<PixChargeResult, String> {
        let external_id = format!("asaas_pay_{}", tx_id);
        let pix_copy = format!(
            "00020126580014BR.GOV.BCB.PIX0136poker-platform-{}5204000053039865405{:.2}5802BR5914POKER_PLATFORM6009SAO_PAULO62070503***6304ABCD",
            tx_id, amount
        );
        let qr_code = format!("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...mock_qr_{}", tx_id);

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
        amount: f64,
        pix_key_type: &str,
        pix_key: &str,
    ) -> Result<PixPayoutResult, String> {
        if amount <= 0.0 {
            return Err("Valor de saque inválido".to_string());
        }
        if pix_key.trim().is_empty() {
            return Err("Chave PIX obrigatória".to_string());
        }

        let external_id = format!("asaas_out_{}", tx_id);
        Ok(PixPayoutResult {
            external_tx_id: external_id,
            status: "PROCESSING".to_string(),
            message: format!("Saque de R$ {:.2} enviado via PIX ({}: {})", amount, pix_key_type, pix_key),
        })
    }

    fn verify_webhook_signature(&self, header_secret: Option<&str>) -> bool {
        match header_secret {
            Some(secret) => secret == self.secret,
            None => false,
        }
    }

    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool {
        let sig_str = match signature_header {
            Some(s) => s,
            None => return false,
        };

        // Suporte legado para testes/transição: aceita se corresponder diretamente à chave secreta
        if sig_str == self.secret {
            return true;
        }

        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = match HmacSha256::new_from_slice(self.secret.as_bytes()) {
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
}

pub fn get_payment_gateway() -> Box<dyn PixGateway> {
    let secret = env::var("PIX_WEBHOOK_SECRET").unwrap_or_else(|_| "poker-pix-webhook-secret-key-32chars".to_string());
    Box::new(MockPixGateway::new(&secret))
}
