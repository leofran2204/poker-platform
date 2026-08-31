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
    pub payment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixChargeStatus {
    pub external_tx_id: String,
    pub amount: u64,
    pub status: String,
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
        payer_tax_number: Option<&str>,
    ) -> Result<PixChargeResult, String>;

    fn fetch_deposit_status(&self, _external_tx_id: &str) -> Result<PixChargeStatus, String> {
        Err("PIX deposit polling is unavailable for this provider".to_string())
    }

    fn simulate_deposit_payment(&self, _external_tx_id: &str) -> Result<(), String> {
        Err("PIX payment simulation is unavailable for this provider".to_string())
    }

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
        _payer_tax_number: Option<&str>,
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
            payment_url: None,
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
        _payer_tax_number: Option<&str>,
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
            payment_url: None,
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

// ─── Provedor 3: DePix Checkout ───
//
// Sandbox and live use the same checkout contract. Values are always integer
// cents and the internal ledger id is also the provider idempotency key. Live
// mode is enabled only by the strict runtime gate in `depix_config` below.

#[derive(Debug, Deserialize)]
struct DepixCheckoutPix {
    qr_code: String,
}

#[derive(Debug, Deserialize)]
struct DepixCheckout {
    id: String,
    status: String,
    amount: u64,
    expires_at: String,
    payment_url: Option<String>,
    pix: Option<DepixCheckoutPix>,
    pix_payload: Option<String>,
    #[serde(default)]
    is_live: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DepixCheckoutEnvelope {
    checkout: DepixCheckout,
}

#[derive(Debug, Deserialize)]
struct DepixApiErrorEnvelope {
    error: Option<DepixApiError>,
}

#[derive(Debug, Deserialize)]
struct DepixApiError {
    code: Option<String>,
    request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DepixPixGateway {
    api_key: String,
    webhook_secret: String,
    api_url: String,
    callback_url: Option<String>,
    redirect_url: Option<String>,
    is_live: bool,
}

impl DepixPixGateway {
    fn new(
        api_key: &str,
        webhook_secret: &str,
        api_url: &str,
        callback_url: Option<String>,
        redirect_url: Option<String>,
        is_live: bool,
    ) -> Self {
        Self {
            api_key: api_key.to_string(),
            webhook_secret: webhook_secret.to_string(),
            api_url: api_url.trim_end_matches('/').to_string(),
            callback_url,
            redirect_url,
            is_live,
        }
    }

    pub fn sandbox(
        api_key: &str,
        webhook_secret: &str,
        api_url: &str,
        callback_url: Option<String>,
        redirect_url: Option<String>,
    ) -> Self {
        Self::new(
            api_key,
            webhook_secret,
            api_url,
            callback_url,
            redirect_url,
            false,
        )
    }

    fn checkout_is_live(checkout: &DepixCheckout) -> Option<bool> {
        match checkout.is_live.as_ref()? {
            serde_json::Value::Bool(value) => Some(*value),
            serde_json::Value::Number(value) => value.as_u64().map(|value| value == 1),
            _ => None,
        }
    }

    fn validate_checkout_mode(&self, checkout: &DepixCheckout) -> Result<(), String> {
        if Self::checkout_is_live(checkout) == Some(self.is_live) {
            Ok(())
        } else {
            Err("DePix checkout environment does not match the configured API key mode".into())
        }
    }

    fn client() -> Result<reqwest::blocking::Client, String> {
        reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| format!("Could not initialize DePix HTTPS client: {error}"))
    }

    fn parse_response<T: serde::de::DeserializeOwned>(
        response: reqwest::blocking::Response,
    ) -> Result<T, String> {
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Could not read DePix response: {error}"))?;
        if status.is_success() {
            serde_json::from_str(&body)
                .map_err(|error| format!("DePix response was invalid: {error}"))
        } else {
            let parsed = serde_json::from_str::<DepixApiErrorEnvelope>(&body).ok();
            let code = parsed
                .as_ref()
                .and_then(|envelope| envelope.error.as_ref())
                .and_then(|error| error.code.as_deref())
                .unwrap_or("unknown_error");
            let request_id = parsed
                .as_ref()
                .and_then(|envelope| envelope.error.as_ref())
                .and_then(|error| error.request_id.as_deref())
                .unwrap_or("unavailable");
            Err(format!(
                "DePix request failed with status {status}, code {code}, request_id {request_id}"
            ))
        }
    }

    fn checkout_to_status(checkout: DepixCheckout) -> PixChargeStatus {
        PixChargeStatus {
            external_tx_id: checkout.id,
            amount: checkout.amount,
            status: checkout.status,
        }
    }
}

impl PixGateway for DepixPixGateway {
    fn create_deposit_charge(
        &self,
        tx_id: &str,
        _user_id: &str,
        amount_centavos: u64,
        payer_tax_number: Option<&str>,
    ) -> Result<PixChargeResult, String> {
        let payer_tax_number = payer_tax_number
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "DePix checkout requires payer_tax_number".to_string())?;
        let mut payload = serde_json::json!({
            "amount": amount_centavos,
            "payer_tax_number": payer_tax_number,
            "payment_method": "pix",
            "description": "Crédito de carteira Zero Tilt Poker",
            "expires_in": 1200,
            "metadata": { "order_id": tx_id }
        });
        if let Some(callback_url) = &self.callback_url {
            payload["callback_url"] = serde_json::Value::String(callback_url.clone());
        }
        if let Some(redirect_url) = &self.redirect_url {
            payload["redirect_url"] = serde_json::Value::String(redirect_url.clone());
        }

        let response = Self::client()?
            .post(format!("{}/api/checkouts", self.api_url))
            .bearer_auth(&self.api_key)
            .header("Idempotency-Key", tx_id)
            .json(&payload)
            .send()
            .map_err(|error| format!("DePix checkout request failed: {error}"))?;
        let checkout: DepixCheckout = Self::parse_response(response)?;
        self.validate_checkout_mode(&checkout)?;
        if !checkout.id.starts_with("chk_")
            || checkout.status != "pending"
            || checkout.amount != amount_centavos
        {
            return Err("DePix checkout response violated id, status, or amount invariants".into());
        }
        let payment_url = checkout
            .payment_url
            .filter(|url| url.starts_with("https://pay.depixapp.com/"))
            .ok_or_else(|| {
                "DePix checkout response did not include a trusted payment URL".to_string()
            })?;
        let pix_copy_paste = checkout
            .pix
            .map(|pix| pix.qr_code)
            .or(checkout.pix_payload)
            .filter(|payload| !payload.trim().is_empty())
            .ok_or_else(|| "DePix checkout response did not include a PIX payload".to_string())?;

        Ok(PixChargeResult {
            external_tx_id: checkout.id,
            pix_copy_paste,
            qr_code_base64: String::new(),
            expires_at: checkout.expires_at,
            payment_url: Some(payment_url),
        })
    }

    fn fetch_deposit_status(&self, external_tx_id: &str) -> Result<PixChargeStatus, String> {
        let response = Self::client()?
            .get(format!("{}/api/checkouts/{external_tx_id}", self.api_url))
            .bearer_auth(&self.api_key)
            .send()
            .map_err(|error| format!("DePix checkout status request failed: {error}"))?;
        let envelope: DepixCheckoutEnvelope = Self::parse_response(response)?;
        self.validate_checkout_mode(&envelope.checkout)?;
        Ok(Self::checkout_to_status(envelope.checkout))
    }

    fn simulate_deposit_payment(&self, external_tx_id: &str) -> Result<(), String> {
        if self.is_live {
            return Err("DePix payment simulation is disabled for live credentials".into());
        }
        let response = Self::client()?
            .post(format!(
                "{}/api/checkouts/{external_tx_id}/simulate-payment",
                self.api_url
            ))
            .bearer_auth(&self.api_key)
            .send()
            .map_err(|error| format!("DePix payment simulation failed: {error}"))?;
        let _: serde_json::Value = Self::parse_response(response)?;
        Ok(())
    }

    fn execute_withdraw_payout(
        &self,
        _tx_id: &str,
        _user_id: &str,
        _amount_centavos: u64,
        _pix_key_type: &str,
        _pix_key: &str,
    ) -> Result<PixPayoutResult, String> {
        Err("DePix payouts are unavailable until the reconciled payout worker is enabled".into())
    }

    fn verify_webhook_hmac(&self, body: &[u8], signature_header: Option<&str>) -> bool {
        verify_depix_hmac(body, signature_header, &self.webhook_secret)
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
        _payer_tax_number: Option<&str>,
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

fn verify_depix_hmac(body: &[u8], signature_header: Option<&str>, secret: &str) -> bool {
    let mut timestamp = None;
    let mut received = None;
    for part in signature_header.unwrap_or_default().split(',') {
        if let Some((key, value)) = part.trim().split_once('=') {
            match key {
                "t" => timestamp = Some(value),
                "v1" => received = Some(value),
                _ => {}
            }
        }
    }
    let (timestamp, received) = match (timestamp, received) {
        (Some(timestamp), Some(received)) if received.len() == 64 => (timestamp, received),
        _ => return false,
    };
    let sent_at = match timestamp.parse::<i64>() {
        Ok(sent_at) => sent_at,
        Err(_) => return false,
    };
    if (chrono::Utc::now().timestamp() - sent_at).unsigned_abs() > 300 {
        return false;
    }

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(body);
    let expected = mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    expected.as_bytes().ct_eq(received.as_bytes()).unwrap_u8() == 1
}
// ─── Factory ───
//
// Mock is the safe default for unit tests. DePix live is fail-closed and needs
// all explicit gates: production environment, a live key, a public HTTPS
// callback, the official API origin, and the operator kill switch.

#[derive(Debug)]
struct DepixRuntimeConfig {
    api_key: String,
    webhook_secret: String,
    api_url: String,
    callback_url: Option<String>,
    redirect_url: Option<String>,
    is_live: bool,
}

fn public_https_url(value: &str, required_path: Option<&str>) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.port(), None | Some(443))
    {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    if host.eq_ignore_ascii_case("localhost")
        || host.parse::<std::net::IpAddr>().is_ok_and(|ip| match ip {
            std::net::IpAddr::V4(ip) => {
                ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
            }
            std::net::IpAddr::V6(ip) => {
                ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local()
            }
        })
    {
        return false;
    }
    required_path.is_none_or(|required| url.path() == required)
}

fn depix_config(mode: &str) -> Result<DepixRuntimeConfig, String> {
    let environment = env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .trim()
        .to_ascii_lowercase();
    let api_key = env::var("DEPIX_API_KEY").unwrap_or_default();
    let webhook_secret = env::var("DEPIX_WEBHOOK_SECRET").unwrap_or_default();
    let api_url = env::var("DEPIX_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.depixapp.com".to_string())
        .trim_end_matches('/')
        .to_string();
    let callback_url = env::var("DEPIX_CALLBACK_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let redirect_url = env::var("DEPIX_REDIRECT_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if webhook_secret.len() < 24 || webhook_secret.contains(char::is_whitespace) {
        return Err(
            "DePix requires a non-whitespace webhook secret of at least 24 characters".into(),
        );
    }
    if api_url != "https://api.depixapp.com" {
        return Err("DePix requires the official https://api.depixapp.com origin".into());
    }
    if callback_url
        .as_deref()
        .is_some_and(|url| !public_https_url(url, Some("/api/webhooks/pix")))
        || redirect_url
            .as_deref()
            .is_some_and(|url| !public_https_url(url, None))
    {
        return Err("DePix callback or redirect URL is not a trusted public HTTPS URL".into());
    }

    let is_live = match mode {
        "sandbox" => {
            if environment == "production"
                || !api_key.starts_with("sk_test_")
                || api_key.contains(char::is_whitespace)
            {
                return Err(
                    "DePix Sandbox requires development/staging and an sk_test_ key".into(),
                );
            }
            false
        }
        "production" => {
            let enabled = env::var("PIX_LIVE_ENABLED")
                .map(|value| value.trim().eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let allowed_depositors = env::var("PIX_LIVE_ALLOWED_DEPOSITOR_IDS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .all(|value| uuid::Uuid::parse_str(value).is_ok());
            let has_allowed_depositor = env::var("PIX_LIVE_ALLOWED_DEPOSITOR_IDS")
                .unwrap_or_default()
                .split(',')
                .any(|value| !value.trim().is_empty());
            if environment != "production"
                || !enabled
                || !api_key.starts_with("sk_live_")
                || api_key.contains(char::is_whitespace)
                || callback_url.is_none()
                || !has_allowed_depositor
                || !allowed_depositors
            {
                return Err("DePix live requires production, the kill switch, an sk_live_ key, a public callback, and a valid depositor allow-list".into());
            }
            true
        }
        _ => return Err("Unsupported DePix mode".into()),
    };

    Ok(DepixRuntimeConfig {
        api_key,
        webhook_secret,
        api_url,
        callback_url,
        redirect_url,
        is_live,
    })
}

pub fn depix_runtime_ready(mode: &str) -> bool {
    depix_config(mode).is_ok()
}

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
        ("depix", "sandbox" | "production") => match depix_config(&mode) {
            Ok(config) => Box::new(DepixPixGateway::new(
                &config.api_key,
                &config.webhook_secret,
                &config.api_url,
                config.callback_url,
                config.redirect_url,
                config.is_live,
            )),
            Err(reason) => Box::new(DisabledPixGateway::new(reason)),
        },
        ("mercadopago" | "mp", _) => Box::new(DisabledPixGateway::new(
            "Mercado Pago PIX has no verified adapter in this project",
        )),
        _ => Box::new(DisabledPixGateway::new(
            "PIX is disabled; configure mock/mock, an approved sandbox, or the gated DePix live adapter",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        public_https_url, AsaasPixGateway, DepixCheckout, DepixPixGateway, MockPixGateway,
        PixGateway,
    };
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

    #[test]
    fn depix_live_callback_must_be_public_https_and_use_the_webhook_path() {
        assert!(public_https_url(
            "https://zerotiltpoker.net/api/webhooks/pix",
            Some("/api/webhooks/pix")
        ));
        assert!(!public_https_url(
            "http://zerotiltpoker.net/api/webhooks/pix",
            Some("/api/webhooks/pix")
        ));
        assert!(!public_https_url(
            "https://127.0.0.1/api/webhooks/pix",
            Some("/api/webhooks/pix")
        ));
        assert!(!public_https_url(
            "https://zerotiltpoker.net/other",
            Some("/api/webhooks/pix")
        ));
    }

    #[test]
    fn depix_checkout_cannot_cross_test_and_live_modes() {
        let live: DepixCheckout = serde_json::from_value(serde_json::json!({
            "id": "chk_live",
            "status": "pending",
            "amount": 500,
            "expires_at": "2026-08-31T12:00:00.000Z",
            "payment_url": "https://pay.depixapp.com/chk_live",
            "pix": { "qr_code": "pix" },
            "is_live": true
        }))
        .unwrap();
        let sandbox: DepixCheckout = serde_json::from_value(serde_json::json!({
            "id": "chk_test",
            "status": "pending",
            "amount": 500,
            "expires_at": "2026-08-31T12:00:00.000Z",
            "payment_url": "https://pay.depixapp.com/chk_test",
            "pix": { "qr_code": "pix" },
            "is_live": 0
        }))
        .unwrap();
        let live_gateway = DepixPixGateway::new(
            "sk_live_test",
            "webhook-secret-at-least-24-bytes",
            "https://api.depixapp.com",
            Some("https://zerotiltpoker.net/api/webhooks/pix".into()),
            None,
            true,
        );
        let sandbox_gateway = DepixPixGateway::sandbox(
            "sk_test_test",
            "webhook-secret-at-least-24-bytes",
            "https://api.depixapp.com",
            None,
            None,
        );

        assert!(live_gateway.validate_checkout_mode(&live).is_ok());
        assert!(live_gateway.validate_checkout_mode(&sandbox).is_err());
        assert!(sandbox_gateway.validate_checkout_mode(&sandbox).is_ok());
        assert!(sandbox_gateway.validate_checkout_mode(&live).is_err());
    }

    #[test]
    fn depix_webhook_requires_timestamped_raw_body_hmac() {
        let secret = "depix-test-webhook-secret-32-bytes";
        let body = br#"{"event":"checkout.completed","data":{"id":"chk_1"}}"#;
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(body);
        let signature = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let gateway = DepixPixGateway::sandbox(
            "sk_test_example",
            secret,
            "https://api.depixapp.com",
            None,
            None,
        );

        assert!(gateway.verify_webhook_hmac(body, Some(&format!("t={timestamp},v1={signature}"))));
        assert!(!gateway.verify_webhook_hmac(
            br#"{"event":"checkout.cancelled"}"#,
            Some(&format!("t={timestamp},v1={signature}"))
        ));
        assert!(!gateway.verify_webhook_hmac(body, Some("v1=00")));
    }
    #[test]
    fn depix_webhook_rejects_a_stale_signed_delivery() {
        let secret = "depix-test-webhook-secret-32-bytes";
        let body = br#"{"event":"checkout.completed","data":{"id":"chk_1"}}"#;
        let timestamp = (chrono::Utc::now().timestamp() - 301).to_string();
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(body);
        let signature = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let gateway = DepixPixGateway::sandbox(
            "sk_test_example",
            secret,
            "https://api.depixapp.com",
            None,
            None,
        );

        assert!(!gateway.verify_webhook_hmac(body, Some(&format!("t={timestamp},v1={signature}"))));
    }
}
