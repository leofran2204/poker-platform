//! Presence online: usuários autenticados com heartbeat recente.
//!
//! - Redis (preferido em produção/demo): ZSET `poker:presence:online`
//!   score = unix epoch seconds, member = user_id
//! - Fallback em memória quando Redis não está configurado (lab/testes)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use redis::AsyncCommands;
use tokio::sync::Mutex;

/// Janela de “ainda online” sem novo heartbeat (segundos).
pub const PRESENCE_TTL_SECS: u64 = 90;

const REDIS_ZSET_KEY: &str = "poker:presence:online";

#[derive(Clone, Default)]
pub struct PresenceTracker {
    memory: Arc<Mutex<HashMap<String, u64>>>,
}

impl PresenceTracker {
    pub fn new() -> Self {
        Self {
            memory: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn now_epoch() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    }

    /// Marca o usuário como online e renova o TTL.
    pub async fn heartbeat(
        &self,
        redis: &Option<redis::aio::ConnectionManager>,
        user_id: &str,
    ) -> Result<(), String> {
        if user_id.trim().is_empty() {
            return Err("user_id vazio".into());
        }
        let now = Self::now_epoch();

        if let Some(redis) = redis {
            let mut conn = redis.clone();
            let _: () = conn
                .zadd(REDIS_ZSET_KEY, user_id, now as f64)
                .await
                .map_err(|e| format!("redis zadd presence: {e}"))?;
            // Remove entradas expiradas de forma oportunista.
            let cutoff = now.saturating_sub(PRESENCE_TTL_SECS) as f64;
            let _: u64 = conn
                .zrembyscore(REDIS_ZSET_KEY, "-inf", cutoff)
                .await
                .unwrap_or(0);
            return Ok(());
        }

        let mut map = self.memory.lock().await;
        map.insert(user_id.to_string(), now);
        let cutoff = now.saturating_sub(PRESENCE_TTL_SECS);
        map.retain(|_, ts| *ts >= cutoff);
        Ok(())
    }

    /// Conta usuários com heartbeat dentro do TTL.
    pub async fn online_count(
        &self,
        redis: &Option<redis::aio::ConnectionManager>,
    ) -> Result<u64, String> {
        let now = Self::now_epoch();
        let cutoff = now.saturating_sub(PRESENCE_TTL_SECS);

        if let Some(redis) = redis {
            let mut conn = redis.clone();
            let _: u64 = conn
                .zrembyscore(REDIS_ZSET_KEY, "-inf", cutoff as f64)
                .await
                .unwrap_or(0);
            let count: u64 = conn
                .zcard(REDIS_ZSET_KEY)
                .await
                .map_err(|e| format!("redis zcard presence: {e}"))?;
            return Ok(count);
        }

        let mut map = self.memory.lock().await;
        map.retain(|_, ts| *ts >= cutoff);
        Ok(map.len() as u64)
    }

    /// Lista (user_id, last_seen_epoch) ainda dentro do TTL.
    pub async fn online_roster(
        &self,
        redis: &Option<redis::aio::ConnectionManager>,
    ) -> Result<Vec<(String, u64)>, String> {
        let now = Self::now_epoch();
        let cutoff = now.saturating_sub(PRESENCE_TTL_SECS);

        if let Some(redis) = redis {
            let mut conn = redis.clone();
            let _: u64 = conn
                .zrembyscore(REDIS_ZSET_KEY, "-inf", cutoff as f64)
                .await
                .unwrap_or(0);
            let rows: Vec<(String, f64)> = conn
                .zrangebyscore_withscores(REDIS_ZSET_KEY, cutoff as f64, "+inf")
                .await
                .map_err(|e| format!("redis zrangebyscore presence: {e}"))?;
            return Ok(rows
                .into_iter()
                .map(|(id, score)| (id, score as u64))
                .collect());
        }

        let mut map = self.memory.lock().await;
        map.retain(|_, ts| *ts >= cutoff);
        let mut out: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        out.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_presence_counts_and_expires() {
        let tracker = PresenceTracker::new();
        tracker.heartbeat(&None, "user-a").await.unwrap();
        tracker.heartbeat(&None, "user-b").await.unwrap();
        assert_eq!(tracker.online_count(&None).await.unwrap(), 2);

        // Force-expire by rewriting timestamps in the past.
        {
            let mut map = tracker.memory.lock().await;
            for ts in map.values_mut() {
                *ts = 1;
            }
        }
        assert_eq!(tracker.online_count(&None).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn heartbeat_is_idempotent_per_user() {
        let tracker = PresenceTracker::new();
        tracker.heartbeat(&None, "same").await.unwrap();
        tracker.heartbeat(&None, "same").await.unwrap();
        assert_eq!(tracker.online_count(&None).await.unwrap(), 1);
    }
}
