use axum::body::Body;
use axum::http::{Request, StatusCode};
use hmac::{Hmac, Mac};
use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::AuthManager;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

fn depix_signature(body: &[u8], secret: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(body);
    let digest = mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("t={timestamp},v1={digest}")
}

async fn state() -> AppState {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("test database must be reachable");
    AppState {
        db,
        auth: Arc::new(RwLock::new(AuthManager::new(
            "depix-tests-jwt-secret-at-least-32-bytes",
        ))),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "depix-tests-jwt-secret-at-least-32-bytes".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: None,
        ws_tickets: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        require_email_verification: false,
        presence: poker_api::presence::PresenceTracker::new(),
    }
}

async fn deliver(
    state: &AppState,
    body: String,
    event: &str,
    event_id: &str,
    secret: &str,
) -> (StatusCode, serde_json::Value) {
    let signature = depix_signature(body.as_bytes(), secret);
    let request = Request::builder()
        .method("POST")
        .uri("/api/webhooks/pix")
        .header("Content-Type", "application/json")
        .header("X-DePix-Signature", signature)
        .header("X-DePix-Event", event)
        .header("X-DePix-Event-Id", event_id)
        .body(Body::from(body))
        .unwrap();
    let response = build_router(state.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 10_000)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL; run as an isolated test binary"]
async fn depix_webhook_credits_completed_once_and_never_credits_processing() {
    const SECRET: &str = "depix-integration-webhook-secret-32-bytes";
    std::env::set_var("PIX_PROVIDER", "depix");
    std::env::set_var("PIX_MODE", "sandbox");
    std::env::set_var("ENVIRONMENT", "development");
    std::env::set_var("DEPIX_API_KEY", "sk_test_integration_placeholder");
    std::env::set_var("DEPIX_WEBHOOK_SECRET", SECRET);
    std::env::set_var("DEPIX_API_BASE_URL", "https://api.depixapp.com");

    let state = state().await;
    let user_id = uuid::Uuid::new_v4();
    let username = format!("depix_{}", &user_id.simple().to_string()[..12]);
    let email = format!("{username}@test.invalid");
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, status, balance, mfa_enabled, created_at) \
         VALUES ($1, $2, $3, 'test-hash', 'player', 'active', 0, false, 0)",
    )
    .bind(user_id)
    .bind(&username)
    .bind(&email)
    .execute(&state.db)
    .await
    .unwrap();

    let completed_tx = format!("pix_dep_{}", uuid::Uuid::new_v4().simple());
    let processing_tx = format!("pix_dep_{}", uuid::Uuid::new_v4().simple());
    for (tx_id, external_id) in [
        (&completed_tx, "chk_completed_test"),
        (&processing_tx, "chk_processing_test"),
    ] {
        sqlx::query(
            "INSERT INTO wallet_transactions \
             (user_id, amount, transaction_type, status, idempotency_key, provider, external_tx_id, provider_status) \
             VALUES ($1, 5000, 'DEPOSIT', 'PENDING', $2, 'depix', $3, 'AWAITING_PAYMENT')",
        )
        .bind(user_id)
        .bind(tx_id)
        .bind(external_id)
        .execute(&state.db)
        .await
        .unwrap();
    }

    let completed_event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let completed_body = serde_json::json!({
        "event": "checkout.completed",
        "data": {
            "event_id": completed_event_id,
            "id": "chk_completed_test",
            "status": "completed",
            "amount": 5000,
            "metadata": { "order_id": completed_tx }
        }
    })
    .to_string();
    let first = deliver(
        &state,
        completed_body.clone(),
        "checkout.completed",
        &completed_event_id,
        SECRET,
    )
    .await;
    let duplicate = deliver(
        &state,
        completed_body,
        "checkout.completed",
        &completed_event_id,
        SECRET,
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first.1["status"], "COMPLETED");
    assert_eq!(duplicate.0, StatusCode::OK);
    assert_eq!(duplicate.1["status"], "IGNORED");

    let processing_event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let processing_body = serde_json::json!({
        "event": "checkout.processing",
        "data": {
            "event_id": processing_event_id,
            "id": "chk_processing_test",
            "status": "processing",
            "amount": 5000,
            "metadata": { "order_id": processing_tx }
        }
    })
    .to_string();
    let processing = deliver(
        &state,
        processing_body,
        "checkout.processing",
        &processing_event_id,
        SECRET,
    )
    .await;
    assert_eq!(processing.0, StatusCode::OK);
    assert_eq!(processing.1["status"], "IGNORED");

    let balance_real: i64 = sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(balance_real, 5000);
    let processing_status: (String, Option<String>) = sqlx::query_as(
        "SELECT status, provider_status FROM wallet_transactions WHERE idempotency_key = $1",
    )
    .bind(&processing_tx)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(processing_status.0, "PENDING");
    assert_eq!(processing_status.1.as_deref(), Some("PROCESSING"));

    sqlx::query("DELETE FROM payment_webhook_events WHERE event_id = ANY($1)")
        .bind(vec![completed_event_id, processing_event_id])
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .unwrap();
}
