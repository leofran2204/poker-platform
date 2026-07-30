// payment_gateway.rs — Abstração e Provedores de Gateway de Pagamento PIX via HTTPS Estrito (TLS 1.2/1.3)
// Arquitetura Financeira: Todos os valores `amount` utilizam `u64` centavos inteiros.

use serde::{Deserialize, Serialize};
use std::{env, time::Duration};
use subtle::ConstantTimeEq;

fn format_brl_cents(amount_centavos: u64) -> String {
    format!("{}.{:02}", amount_centavos / 100, amount_centavos % 100)
}

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
        let amount_brl = format_brl_cents(amount_centavos);
        let pix_copy = format!(
            "00020126580014BR.GOV.BCB.PIX0136poker-platform-{}5204000053039865405{}5802BR5914POKER_PLATFORM6009SAO_PAULO62070503***6304ABCD",
            tx_id, amount_brl
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

        let amount_brl = format_brl_cents(amount_centavos);
        let external_id = format!("asaas_out_{}", tx_id);
        Ok(PixPayoutResult {
            external_tx_id: external_id,
            status: "PROCESSING".to_string(),
            message: format!(
                "Saque de R$ {} registrado para processamento PIX ({})",
                amount_brl, pix_key_type
            ),
        })
    }

    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool {
        verify_hmac_helper(body, signature_header, &self.secret)
    }
}

// ─── Provedor 2: Asaas PIX Sandbox ───
//
// This adapter makes real HTTPS calls only to the Asaas Sandbox. Production is
// deliberately rejected by the factory until the operator has written approval
// from a compatible PSP and the regulated real-money controls are implemented.

#[derive(Debug, Deserialize)]
struct AsaasPaymentResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct AsaasPixQrCodeResponse {
    payload: String,
    #[serde(rename = "encodedImage")]
    encoded_image: String,
    #[serde(rename = "expirationDate")]
    expiration_date: String,
}

#[derive(Debug, Clone)]
pub struct AsaasPixGateway {
    pub api_key: String,
    pub webhook_token: String,
    pub customer_id: String,
    pub api_url: String,
}

impl AsaasPixGateway {
    pub fn sandbox(api_key: &str, webhook_token: &str, customer_id: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            webhook_token: webhook_token.to_string(),
            customer_id: customer_id.to_string(),
            api_url: "https://api-sandbox.asaas.com/v3".to_string(),
        }
    }

    fn client() -> Result<reqwest::blocking::Client, String> {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|error| format!("Could not initialize Asaas HTTPS client: {error}"))
    }

    fn value_number(amount_centavos: u64) -> Result<serde_json::Number, String> {
        format_brl_cents(amount_centavos)
            .parse::<serde_json::Number>()
            .map_err(|_| "Could not represent PIX amount exactly".to_string())
    }

    fn check_success(
        response: reqwest::blocking::Response,
    ) -> Result<reqwest::blocking::Response, String> {
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(format!(
                "Asaas HTTPS request failed with status {}",
                response.status()
            ))
        }
    }
}

impl PixGateway for AsaasPixGateway {
    fn create_deposit_charge(
        &self,
        tx_id: &str,
        _user_id: &str,
        amount_centavos: u64,
    ) -> Result<PixChargeResult, String> {
        let client = Self::client()?;
        let payment_request = serde_json::json!({
            "customer": self.customer_id,
            "billingType": "PIX",
            "value": Self::value_number(amount_centavos)?,
            "dueDate": chrono::Utc::now().date_naive().to_string(),
            "description": "Poker_Project sandbox deposit",
            "externalReference": tx_id,
        });
        let payment = client
            .post(format!("{}/payments", self.api_url))
            .header("access_token", &self.api_key)
            .json(&payment_request)
            .send()
            .map_err(|error| format!("Asaas payment request failed: {error}"))
            .and_then(Self::check_success)?
            .json::<AsaasPaymentResponse>()
            .map_err(|error| format!("Asaas payment response was invalid: {error}"))?;
        let qr_code = client
            .get(format!(
                "{}/payments/{}/pixQrCode",
                self.api_url, payment.id
            ))
            .header("access_token", &self.api_key)
            .send()
            .map_err(|error| format!("Asaas PIX QR request failed: {error}"))
            .and_then(Self::check_success)?
            .json::<AsaasPixQrCodeResponse>()
            .map_err(|error| format!("Asaas PIX QR response was invalid: {error}"))?;

        Ok(PixChargeResult {
            external_tx_id: payment.id,
            pix_copy_paste: qr_code.payload,
            qr_code_base64: format!("data:image/png;base64,{}", qr_code.encoded_image),
            expires_at: qr_code.expiration_date,
        })
    }

    fn execute_withdraw_payout(
        &self,
        _tx_id: &str,
        _user_id: &str,
        _amount_centavos: u64,
        _pix_key_type: &str,
        _pix_key: &str,
    ) -> Result<PixPayoutResult, String> {
        Err("PIX payouts are unavailable: no reconciled outbox worker is enabled".to_string())
    }

    fn verify_webhook_hmac(&self, _body: &[u8], token_header: Option<&str>) -> bool {
        token_header
            .map(|token| {
                self.webhook_token
                    .as_bytes()
                    .ct_eq(token.trim().as_bytes())
                    .unwrap_u8()
                    == 1
            })
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct DisabledPixGateway {
    reason: String,
}

impl DisabledPixGateway {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl PixGateway for DisabledPixGateway {
    fn create_deposit_charge(
        &self,
        _tx_id: &str,
        _user_id: &str,
        _amount_centavos: u64,
    ) -> Result<PixChargeResult, String> {
        Err(self.reason.clone())
    }

    fn execute_withdraw_payout(
        &self,
        _tx_id: &str,
        _user_id: &str,
        _amount_centavos: u64,
        _pix_key_type: &str,
        _pix_key: &str,
    ) -> Result<PixPayoutResult, String> {
        Err(self.reason.clone())
    }

    fn verify_webhook_hmac(&self, _body: &[u8], _signature_header: Option<&str>) -> bool {
        false
    }
}

// ─── Webhook authentication for the local mock ───

fn verify_hmac_helper(body: &[u8], signature_header: Option<&str>, secret: &str) -> bool {
    let signature = match signature_header {
        Some(signature) => signature,
        None => return false,
    };

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };
    mac.update(body);
    let expected = mac.finalize().into_bytes();
    let expected_hex = expected
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    expected_hex
        .as_bytes()
        .ct_eq(signature.trim_start_matches("sha256=").as_bytes())
        .unwrap_u8()
        == 1
}

// ─── Factory ───
//
// Mock is the safe default for unit tests. The only non-mock adapter is the
// documented Asaas Sandbox integration. Production is hard-disabled until the
// operator is approved by a PSP compatible with the intended regulated activity.

pub fn get_payment_gateway() -> Box<dyn PixGateway> {
    let provider = env::var("PIX_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .trim()
        .to_ascii_lowercase();
    let mode = env::var("PIX_MODE")
        .unwrap_or_else(|_| "mock".to_string())
        .trim()
        .to_ascii_lowercase();

    match (provider.as_str(), mode.as_str()) {
        ("mock", "mock") => {
            let secret = env::var("PIX_WEBHOOK_SECRET")
                .unwrap_or_else(|_| "poker-pix-webhook-secret-key-32chars".to_string());
            Box::new(MockPixGateway::new(&secret))
        }
        ("asaas", "sandbox") => {
            let api_key = env::var("ASAAS_API_KEY").ok();
            let webhook_token = env::var("ASAAS_WEBHOOK_TOKEN").ok();
            let customer_id = env::var("ASAAS_TEST_CUSTOMER_ID").ok();
            match (api_key, webhook_token, customer_id) {
                (Some(api_key), Some(webhook_token), Some(customer_id))
                    if !api_key.trim().is_empty()
                        && !customer_id.trim().is_empty()
                        && webhook_token.len() >= 32
                        && !webhook_token.contains(char::is_whitespace) =>
                {
                    Box::new(AsaasPixGateway::sandbox(
                        &api_key,
                        &webhook_token,
                        &customer_id,
                    ))
                }
                _ => Box::new(DisabledPixGateway::new(
                    "Asaas Sandbox requires ASAAS_API_KEY, ASAAS_WEBHOOK_TOKEN (at least 32 characters), and ASAAS_TEST_CUSTOMER_ID",
                )),
            }
        }
        ("asaas", "production") => Box::new(DisabledPixGateway::new(
            "Asaas production PIX is intentionally disabled pending PSP approval and real-money compliance controls",
        )),
        ("mercadopago" | "mp", _) => Box::new(DisabledPixGateway::new(
            "Mercado Pago PIX has no verified adapter in this project",
        )),
        _ => Box::new(DisabledPixGateway::new(
            "PIX is disabled; use PIX_PROVIDER=mock with PIX_MODE=mock or configure the verified Asaas Sandbox adapter",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{AsaasPixGateway, MockPixGateway, PixGateway};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    fn signature(body: &[u8], secret: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("valid HMAC key");
        mac.update(body);
        let digest = mac.finalize().into_bytes();
        format!(
            "sha256={}",
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        )
    }

    #[test]
    fn mock_webhook_requires_the_exact_hmac() {
        let body = br#"{\"tx_id\":\"pix_dep_test\"}"#;
        let gateway = MockPixGateway::new("test-webhook-secret");
        let valid = signature(body, "test-webhook-secret");

        assert!(gateway.verify_webhook_hmac(body, Some(&valid)));
        assert!(!gateway.verify_webhook_hmac(body, Some("sha256=00")));
        assert!(!gateway.verify_webhook_hmac(b"different", Some(&valid)));
    }

    #[test]
    fn asaas_sandbox_webhook_requires_the_configured_token() {
        let token = "0123456789abcdef0123456789abcdef";
        let gateway = AsaasPixGateway::sandbox("sandbox-key", token, "cus_test");

        assert!(gateway.verify_webhook_hmac(b"ignored", Some(token)));
        assert!(!gateway.verify_webhook_hmac(b"ignored", Some("different-token")));
    }
}
