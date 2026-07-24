// rate_limit.rs — Middleware de Rate Limiting para Endpoints Sensíveis (Auth/Pagamentos)
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::HeaderMap;

use crate::error::ApiError;
use crate::state::AppState;

/// Limitador de requisições em memória por IP (Token Bucket / Sliding Window)
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    pub max_requests: usize,
    pub window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_seconds),
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
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Padrão: 30 requisições por minuto por IP para endpoints protegidos por rate limit
        Self::new(30, 60)
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
        let client_ip = extract_client_ip(&parts.headers);

        if !app_state.rate_limiter.check_rate_limit(&client_ip).await {
            return Err(ApiError::BadRequest(
                "Taxa de requisições excedida. Tente novamente mais tarde.".to_string(),
            ));
        }

        Ok(EnforceRateLimit)
    }
}

/// Extrai IP do cliente através dos headers X-Forwarded-For ou X-Real-IP (ou fallback "127.0.0.1")
fn extract_client_ip(headers: &HeaderMap) -> String {
    if let Some(forwarded) = headers.get("X-Forwarded-For").and_then(|h| h.to_str().ok()) {
        if let Some(first_ip) = forwarded.split(',').next() {
            return first_ip.trim().to_string();
        }
    }
    if let Some(real_ip) = headers.get("X-Real-IP").and_then(|h| h.to_str().ok()) {
        return real_ip.trim().to_string();
    }
    "127.0.0.1".to_string()
}
