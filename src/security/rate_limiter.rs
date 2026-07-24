use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RateLimiterError {
    #[error("Limite de requisições excedido para a chave: {0}")]
    RateLimitExceeded(String),
    #[error("Erro de concorrência na trava do Rate Limiter")]
    LockError,
}

#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    max_capacity: f64,
    refill_rate_per_sec: f64,
    last_refill: Instant,
}

impl Bucket {
    fn new(max_capacity: f64, refill_rate_per_sec: f64) -> Self {
        Self {
            tokens: max_capacity,
            max_capacity,
            refill_rate_per_sec,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens_requested: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        // Repor tokens com base no tempo decorrido
        self.tokens = (self.tokens + elapsed * self.refill_rate_per_sec).min(self.max_capacity);
        self.last_refill = now;

        if self.tokens >= tokens_requested {
            self.tokens -= tokens_requested;
            true
        } else {
            false
        }
    }
}

/// Rate Limiter thread-safe baseado no algoritmo Token Bucket para IPs e Usuários.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_capacity: f64,
    refill_rate_per_sec: f64,
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}

impl RateLimiter {
    pub fn new(max_capacity: f64, refill_rate_per_sec: f64) -> Self {
        Self {
            max_capacity,
            refill_rate_per_sec,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Verifica e consome 1 token para uma chave (IP ou user_id).
    pub fn check_rate_limit(&self, key: &str) -> Result<(), RateLimiterError> {
        let mut buckets = self.buckets.lock().map_err(|_| RateLimiterError::LockError)?;
        
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| Bucket::new(self.max_capacity, self.refill_rate_per_sec));

        if bucket.try_consume(1.0) {
            Ok(())
        } else {
            Err(RateLimiterError::RateLimitExceeded(key.to_string()))
        }
    }

    /// Limpa buckets inativos para economizar memória.
    pub fn cleanup_inactive(&self, max_idle: Duration) -> Result<usize, RateLimiterError> {
        let mut buckets = self.buckets.lock().map_err(|_| RateLimiterError::LockError)?;
        let now = Instant::now();
        let initial_len = buckets.len();

        buckets.retain(|_, b| now.duration_since(b.last_refill) < max_idle);

        Ok(initial_len - buckets.len())
    }
}
