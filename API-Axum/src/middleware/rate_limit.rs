// rate_limit.rs — Middleware de Rate Limiting para Endpoints Sensíveis (Auth/Pagamentos)
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use axum::extract::{connect_info::ConnectInfo, FromRef, FromRequestParts};
use axum::http::request::Parts;

use crate::error::ApiError;
use crate::state::AppState;

/// Limitador de requisições em memória por IP (Token Bucket / Sliding Window)
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    pub max_requests: usize,
    pub window: Duration,
    trust_proxy_headers: bool,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_seconds),
            trust_proxy_headers: false,
        }
    }

    /// Avalia se o IP requisitante excedeu o limite
    pub async fn check_rate_limit(&self, client_ip: &str) -> bool {
        let mut map = self.requests.lock().await;
        let now = Instant::now();
        let timestamps = map.entry(client_ip.to_string()).or_insert_with(Vec::new);

        // Remove registros mais antigos que a janela de tempo
        timestamps.retain(|&t| now.duration_since(t) <= self.window);

        if timestamps.len() >= self.max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    fn with_trusted_proxy_headers(mut self, trust_proxy_headers: bool) -> Self {
        self.trust_proxy_headers = trust_proxy_headers;
        self
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Padrão: 30 requisições por minuto por IP para endpoints protegidos por rate limit
        let trust_proxy_headers = std::env::var("TRUST_PROXY_HEADERS")
            .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"));
        Self::new(30, 60).with_trusted_proxy_headers(trust_proxy_headers)
    }
}

/// Extractor para impor rate limiting em rotas sensíveis
#[derive(Debug, Clone)]
pub struct EnforceRateLimit;

#[axum::async_trait]
impl<S> FromRequestParts<S> for EnforceRateLimit
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let client_ip = extract_client_ip(parts, app_state.rate_limiter.trust_proxy_headers);

        if !app_state.rate_limiter.check_rate_limit(&client_ip).await {
            return Err(ApiError::TooManyRequests(
                "Taxa de requisições excedida. Tente novamente mais tarde.".to_string(),
            ));
        }

        Ok(EnforceRateLimit)
    }
}

/// Uses a forwarded address only after deployment has explicitly declared a
/// trusted proxy boundary. Otherwise caller-controlled forwarding headers are
/// ignored and the transport peer address is used.
fn extract_client_ip(parts: &Parts, trust_proxy_headers: bool) -> String {
    if trust_proxy_headers {
        for header_name in ["x-forwarded-for", "x-real-ip"] {
            if let Some(address) = parts
                .headers
                .get(header_name)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(',').next())
                .map(str::trim)
                .and_then(|value| value.parse::<IpAddr>().ok())
            {
                return address.to_string();
            }
        }
    }

    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip().to_string())
        .unwrap_or_else(|| "unknown-peer".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn ignores_forwarded_headers_without_a_trusted_proxy_boundary() {
        let (mut parts, _) = Request::builder()
            .header("x-forwarded-for", "198.51.100.8")
            .body(())
            .expect("request should be valid")
            .into_parts();
        parts
            .extensions
            .insert(ConnectInfo(SocketAddr::from(([10, 0, 0, 8], 443))));

        assert_eq!(extract_client_ip(&parts, false), "10.0.0.8");
    }

    #[test]
    fn accepts_the_first_forwarded_address_only_with_trusted_proxy_configuration() {
        let (parts, _) = Request::builder()
            .header("x-forwarded-for", "198.51.100.8, 10.0.0.8")
            .body(())
            .expect("request should be valid")
            .into_parts();

        assert_eq!(extract_client_ip(&parts, true), "198.51.100.8");
    }
}
