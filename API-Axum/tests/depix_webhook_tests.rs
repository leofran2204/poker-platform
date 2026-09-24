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
        require_invite: false,
        presence: poker_api::presence::PresenceTracker::new(),
        bots: poker_api::bots::BotFleet::for_test(),
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
async fn depix_live_webhook_credits_once_and_flags_post_settlement_cancellation() {
    const SECRET: &str = "depix-integration-webhook-secret-32-bytes";
    std::env::set_var("PIX_PROVIDER", "depix");
    std::env::set_var("PIX_MODE", "production");
    std::env::set_var("ENVIRONMENT", "production");
    std::env::set_var("PIX_LIVE_ENABLED", "true");
    std::env::set_var(
        "PIX_LIVE_ALLOWED_DEPOSITOR_IDS",
        "00000000-0000-0000-0000-000000000001",
    );
    std::env::set_var("DEPIX_API_KEY", "sk_live_integration_placeholder");
    std::env::set_var("DEPIX_WEBHOOK_SECRET", SECRET);
    std::env::set_var("DEPIX_API_BASE_URL", "https://api.depixapp.com");
    std::env::set_var(
        "DEPIX_CALLBACK_URL",
        "https://zerotiltpoker.net/api/webhooks/pix",
    );

    let state = state().await;
    // Isolamento: execução anterior abortada pode ter deixado as linhas fixas.
    sqlx::query("DELETE FROM wallet_transactions WHERE external_tx_id = ANY($1)")
        .bind(vec!["chk_completed_test", "chk_processing_test"])
        .execute(&state.db)
        .await
        .unwrap();
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

    let cancelled_event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let cancelled_body = serde_json::json!({
        "event": "checkout.cancelled",
        "data": {
            "event_id": cancelled_event_id,
            "id": "chk_completed_test",
            "status": "cancelled",
            "amount": 5000,
            "metadata": { "order_id": completed_tx }
        }
    })
    .to_string();
    let cancelled = deliver(
        &state,
        cancelled_body.clone(),
        "checkout.cancelled",
        &cancelled_event_id,
        SECRET,
    )
    .await;
    let cancelled_duplicate = deliver(
        &state,
        cancelled_body,
        "checkout.cancelled",
        &cancelled_event_id,
        SECRET,
    )
    .await;
    assert_eq!(cancelled.0, StatusCode::OK);
    assert_eq!(cancelled.1["status"], "REVIEW_REQUIRED");
    assert_eq!(cancelled_duplicate.0, StatusCode::OK);
    assert_eq!(cancelled_duplicate.1["status"], "IGNORED");

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
    assert_eq!(processing.1["status"], "PROVISIONAL");

    let balance_real: i64 = sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    // completed_tx already credited 5000; processing_tx (R$ 50) credits another 5000.
    assert_eq!(balance_real, 10_000);
    let processing_status: (String, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT status, provider_status, credited_amount_cents \
         FROM wallet_transactions WHERE idempotency_key = $1",
    )
    .bind(&processing_tx)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(processing_status.0, "PENDING");
    assert_eq!(processing_status.1.as_deref(), Some("PROCESSING"));
    assert_eq!(processing_status.2, Some(5000));

    let completed_status: (String, Option<String>) = sqlx::query_as(
        "SELECT status, provider_status FROM wallet_transactions WHERE idempotency_key = $1",
    )
    .bind(&completed_tx)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(completed_status.0, "COMPLETED");
    assert_eq!(
        completed_status.1.as_deref(),
        Some("CANCELLED_REVIEW_REQUIRED")
    );

    let review_outbox_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM outbox_events \
         WHERE aggregate_id = $1 AND event_type = 'PIX_DEPOSIT_REVIEW_REQUIRED'",
    )
    .bind(&completed_tx)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(review_outbox_count, 1);
    let review_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs \
         WHERE user_id = $1 AND action = 'PIX_DEPOSIT_REVIEW_REQUIRED'",
    )
    .bind(user_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(review_audit_count, 1);

    sqlx::query("DELETE FROM payment_webhook_events WHERE event_id = ANY($1)")
        .bind(vec![
            completed_event_id,
            processing_event_id,
            cancelled_event_id,
        ])
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM outbox_events WHERE aggregate_id = $1")
        .bind(&completed_tx)
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM audit_logs WHERE user_id = $1")
        .bind(user_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "Requires PostgreSQL; run as an isolated test binary"]
async fn depix_webhook_credits_net_received_and_records_fee() {
    const SECRET: &str = "depix-integration-webhook-secret-32-bytes";
    std::env::set_var("PIX_PROVIDER", "depix");
    std::env::set_var("PIX_MODE", "production");
    std::env::set_var("ENVIRONMENT", "production");
    std::env::set_var("PIX_LIVE_ENABLED", "true");
    std::env::set_var(
        "PIX_LIVE_ALLOWED_DEPOSITOR_IDS",
        "00000000-0000-0000-0000-000000000001",
    );
    std::env::set_var("DEPIX_API_KEY", "sk_live_integration_placeholder");
    std::env::set_var("DEPIX_WEBHOOK_SECRET", SECRET);
    std::env::set_var("DEPIX_API_BASE_URL", "https://api.depixapp.com");
    std::env::set_var(
        "DEPIX_CALLBACK_URL",
        "https://zerotiltpoker.net/api/webhooks/pix",
    );

    let state = state().await;
    sqlx::query("DELETE FROM wallet_transactions WHERE external_tx_id = $1")
        .bind("chk_net_test")
        .execute(&state.db)
        .await
        .unwrap();
    let user_id = uuid::Uuid::new_v4();
    let username = format!("depixnet_{}", &user_id.simple().to_string()[..12]);
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

    let tx_id = format!("pix_dep_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO wallet_transactions \
         (user_id, amount, transaction_type, status, idempotency_key, provider, external_tx_id, provider_status) \
         VALUES ($1, 5000, 'DEPOSIT', 'PENDING', $2, 'depix', 'chk_net_test', 'AWAITING_PAYMENT')",
    )
    .bind(user_id)
    .bind(&tx_id)
    .execute(&state.db)
    .await
    .unwrap();

    let event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let body = serde_json::json!({
        "event": "checkout.completed",
        "data": {
            "event_id": event_id,
            "id": "chk_net_test",
            "status": "completed",
            "amount": 5000,
            "amount_received": 4400,
            "metadata": { "order_id": tx_id }
        }
    })
    .to_string();
    let first = deliver(
        &state,
        body.clone(),
        "checkout.completed",
        &event_id,
        SECRET,
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first.1["status"], "COMPLETED");

    let balance_real: i64 = sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(balance_real, 4400);
    let row: (String, Option<i64>) = sqlx::query_as(
        "SELECT status, credited_amount_cents FROM wallet_transactions WHERE idempotency_key = $1",
    )
    .bind(&tx_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(row.0, "COMPLETED");
    assert_eq!(row.1, Some(4400));
    let fee: i64 = sqlx::query_scalar(
        "SELECT (metadata->>'provider_fee_cents')::bigint FROM audit_logs \
         WHERE user_id = $1 AND action = 'PIX_DEPOSIT_SETTLED'",
    )
    .bind(user_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(fee, 600);

    sqlx::query("DELETE FROM payment_webhook_events WHERE event_id = $1")
        .bind(event_id)
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM audit_logs WHERE user_id = $1")
        .bind(user_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .unwrap();
}

fn arm_depix_live_env(secret: &str) {
    std::env::set_var("PIX_PROVIDER", "depix");
    std::env::set_var("PIX_MODE", "production");
    std::env::set_var("ENVIRONMENT", "production");
    std::env::set_var("PIX_LIVE_ENABLED", "true");
    std::env::set_var(
        "PIX_LIVE_ALLOWED_DEPOSITOR_IDS",
        "00000000-0000-0000-0000-000000000001",
    );
    std::env::set_var("DEPIX_API_KEY", "sk_live_integration_placeholder");
    std::env::set_var("DEPIX_WEBHOOK_SECRET", secret);
    std::env::set_var("DEPIX_API_BASE_URL", "https://api.depixapp.com");
    std::env::set_var(
        "DEPIX_CALLBACK_URL",
        "https://zerotiltpoker.net/api/webhooks/pix",
    );
}

async fn insert_pending_deposit(
    state: &AppState,
    user_id: uuid::Uuid,
    amount: i64,
    tx_id: &str,
    external_id: &str,
) {
    sqlx::query(
        "INSERT INTO wallet_transactions \
         (user_id, amount, transaction_type, status, idempotency_key, provider, external_tx_id, provider_status) \
         VALUES ($1, $2, 'DEPOSIT', 'PENDING', $3, 'depix', $4, 'AWAITING_PAYMENT')",
    )
    .bind(user_id)
    .bind(amount)
    .bind(tx_id)
    .bind(external_id)
    .execute(&state.db)
    .await
    .unwrap();
}

async fn insert_depix_user(state: &AppState) -> uuid::Uuid {
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
    user_id
}

#[tokio::test]
#[ignore = "Requires PostgreSQL; run as an isolated test binary"]
async fn depix_processing_above_cap_does_not_credit() {
    const SECRET: &str = "depix-integration-webhook-secret-32-bytes";
    arm_depix_live_env(SECRET);
    let state = state().await;
    sqlx::query("DELETE FROM wallet_transactions WHERE external_tx_id = $1")
        .bind("chk_over_cap")
        .execute(&state.db)
        .await
        .unwrap();
    let user_id = insert_depix_user(&state).await;
    let tx_id = format!("pix_dep_{}", uuid::Uuid::new_v4().simple());
    insert_pending_deposit(&state, user_id, 10_000, &tx_id, "chk_over_cap").await;

    let event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let body = serde_json::json!({
        "event": "checkout.processing",
        "data": {
            "event_id": event_id,
            "id": "chk_over_cap",
            "status": "processing",
            "amount": 10000,
            "metadata": { "order_id": tx_id }
        }
    })
    .to_string();
    let response = deliver(&state, body, "checkout.processing", &event_id, SECRET).await;
    assert_eq!(response.0, StatusCode::OK);
    assert_eq!(response.1["status"], "IGNORED");
    let balance: i64 = sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(balance, 0);
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "Requires PostgreSQL; run as an isolated test binary"]
async fn depix_provisional_credit_settles_once_and_reverses() {
    const SECRET: &str = "depix-integration-webhook-secret-32-bytes";
    arm_depix_live_env(SECRET);
    let state = state().await;
    sqlx::query("DELETE FROM wallet_transactions WHERE external_tx_id = ANY($1)")
        .bind(vec![
            "chk_prov_settle",
            "chk_prov_reverse",
            "chk_prov_short",
        ])
        .execute(&state.db)
        .await
        .unwrap();
    let user_id = insert_depix_user(&state).await;
    let settle_tx = format!("pix_dep_{}", uuid::Uuid::new_v4().simple());
    let reverse_tx = format!("pix_dep_{}", uuid::Uuid::new_v4().simple());
    let shortfall_tx = format!("pix_dep_{}", uuid::Uuid::new_v4().simple());
    insert_pending_deposit(&state, user_id, 5000, &settle_tx, "chk_prov_settle").await;
    insert_pending_deposit(&state, user_id, 5000, &reverse_tx, "chk_prov_reverse").await;
    insert_pending_deposit(&state, user_id, 5000, &shortfall_tx, "chk_prov_short").await;

    for (chk, tx) in [
        ("chk_prov_settle", settle_tx.as_str()),
        ("chk_prov_reverse", reverse_tx.as_str()),
        ("chk_prov_short", shortfall_tx.as_str()),
    ] {
        let event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
        let body = serde_json::json!({
            "event": "checkout.processing",
            "data": {
                "event_id": event_id,
                "id": chk,
                "status": "processing",
                "amount": 5000,
                "metadata": { "order_id": tx }
            }
        })
        .to_string();
        let response = deliver(&state, body, "checkout.processing", &event_id, SECRET).await;
        assert_eq!(response.1["status"], "PROVISIONAL");
        let again_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
        let again_body = serde_json::json!({
            "event": "checkout.processing",
            "data": {
                "event_id": again_id,
                "id": chk,
                "status": "processing",
                "amount": 5000,
                "metadata": { "order_id": tx }
            }
        })
        .to_string();
        let again = deliver(&state, again_body, "checkout.processing", &again_id, SECRET).await;
        assert_eq!(again.1["status"], "IGNORED");
    }

    let balance: i64 = sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(balance, 15_000);

    let completed_event = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let completed_body = serde_json::json!({
        "event": "checkout.completed",
        "data": {
            "event_id": completed_event,
            "id": "chk_prov_settle",
            "status": "completed",
            "amount": 5000,
            "amount_received": 4400,
            "metadata": { "order_id": settle_tx }
        }
    })
    .to_string();
    let completed = deliver(
        &state,
        completed_body,
        "checkout.completed",
        &completed_event,
        SECRET,
    )
    .await;
    assert_eq!(completed.1["status"], "COMPLETED");
    let settle_row: (String, Option<i64>) = sqlx::query_as(
        "SELECT status, credited_amount_cents FROM wallet_transactions WHERE idempotency_key = $1",
    )
    .bind(&settle_tx)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(settle_row.0, "COMPLETED");
    assert_eq!(settle_row.1, Some(4400));

    let cancel_event = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let cancel_body = serde_json::json!({
        "event": "checkout.cancelled",
        "data": {
            "event_id": cancel_event,
            "id": "chk_prov_reverse",
            "status": "cancelled",
            "amount": 5000,
            "metadata": { "order_id": reverse_tx }
        }
    })
    .to_string();
    let reversed = deliver(
        &state,
        cancel_body,
        "checkout.cancelled",
        &cancel_event,
        SECRET,
    )
    .await;
    assert_eq!(reversed.1["status"], "REVERSED");

    sqlx::query("UPDATE users SET balance_real = 1000 WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .unwrap();
    let short_event = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let short_body = serde_json::json!({
        "event": "checkout.cancelled",
        "data": {
            "event_id": short_event,
            "id": "chk_prov_short",
            "status": "cancelled",
            "amount": 5000,
            "metadata": { "order_id": shortfall_tx }
        }
    })
    .to_string();
    let shortfall = deliver(
        &state,
        short_body,
        "checkout.cancelled",
        &short_event,
        SECRET,
    )
    .await;
    assert_eq!(shortfall.1["status"], "REVERSED_SHORTFALL");
    let balance: i64 = sqlx::query_scalar("SELECT balance_real FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(balance, 0);

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .unwrap();
}
