//! Redis contract tests for the shared rate-limit boundary.
//!
//! These tests are intentionally ignored by the default suite because they
//! require a real Redis instance. CI and the authorized full-validation API
//! phase provide one explicitly.

use poker_api::middleware::rate_limit::RateLimiter;

#[tokio::test]
#[ignore = "Requires Redis — set REDIS_URL to run"]
async fn redis_rate_limit_is_shared_between_independent_limiter_instances() {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL is required");
    let client = redis::Client::open(redis_url).expect("REDIS_URL must be valid");
    let manager = redis::aio::ConnectionManager::new(client)
        .await
        .expect("Redis must be reachable");
    let client_ip = format!("contract-{}", uuid::Uuid::new_v4());
    let key = format!("poker:rate-limit:{client_ip}");
    let first_replica = RateLimiter::new(1, 60);
    let second_replica = RateLimiter::new(1, 60);

    assert!(first_replica
        .check_rate_limit(&client_ip, Some(&manager))
        .await
        .expect("shared limiter must be available"));
    assert!(!second_replica
        .check_rate_limit(&client_ip, Some(&manager))
        .await
        .expect("both replicas must observe the same counter"));

    let mut cleanup_connection = manager.clone();
    let _: i64 = redis::cmd("DEL")
        .arg(key)
        .query_async(&mut cleanup_connection)
        .await
        .expect("test key cleanup must succeed");
}
