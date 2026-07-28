// db_pool_stress_tests.rs — Testes de Estresse Massivo Concorrente no PostgreSQL Pool
// Submete a camada de persistência (sqlx + PostgreSQL) a centenas de operações simultâneas via tokio tasks.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::AuthManager;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

/// Cria um AppState conectado a uma instância PostgreSQL real
async fn make_real_db_state(pool: sqlx::PgPool) -> AppState {
    AppState {
        db: pool,
        auth: Arc::new(RwLock::new(AuthManager::new(
            "stress-test-jwt-secret-key-32chars",
        ))),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "stress-test-jwt-secret-key-32chars".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: None,
        ws_tickets: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    }
}

fn get_db_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

#[tokio::test]
#[ignore]
async fn test_high_concurrency_db_user_registration() {
    let db_url = match get_db_url() {
        Some(url) => url,
        None => return,
    };

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&db_url)
        .await
        .expect("Falha ao conectar ao PostgreSQL");

    let state = make_real_db_state(pool).await;

    const NUM_CONCURRENT_USERS: usize = 30;
    let mut handles = Vec::with_capacity(NUM_CONCURRENT_USERS);

    // Mantém cada lote identificável e removível, sem deixar usuários de
    // stress acumulados no banco local entre execuções autorizadas.
    let run_id = Uuid::new_v4().simple().to_string();
    let username_prefix = format!("stress_{}", &run_id[..12]);

    for i in 0..NUM_CONCURRENT_USERS {
        let app = build_router(state.clone());
        let username = format!("{username_prefix}_{i}");
        let email = format!("{username_prefix}_{i}@example.com");

        handles.push(tokio::spawn(async move {
            let req_body = json!({
                "username": username,
                "email": email,
                "password": "Password123"
            });

            let req = Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap();

            let response = app.oneshot(req).await.unwrap();
            let status = response.status();
            let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

            (status, body_bytes)
        }));
    }

    let mut success_count = 0;
    for handle in handles {
        let (status, body) = handle.await.expect("Task panocou");
        if status == StatusCode::OK {
            let json_val: Value = serde_json::from_slice(&body).unwrap();
            assert!(json_val.get("token").is_some());
            assert!(json_val.get("refresh_token").is_some());
            success_count += 1;
        }
    }

    sqlx::query("DELETE FROM users WHERE username LIKE $1")
        .bind(format!("{username_prefix}_%"))
        .execute(&state.db)
        .await
        .expect("Falha ao remover usuários criados pelo stress test");

    assert_eq!(
        success_count, NUM_CONCURRENT_USERS,
        "Todas as {} inscrições concorrentes no DB devem ser concluídas com sucesso",
        NUM_CONCURRENT_USERS
    );
}

#[tokio::test]
#[ignore]
async fn test_concurrency_hand_history_persistence() {
    let db_url = match get_db_url() {
        Some(url) => url,
        None => return,
    };

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Falha ao conectar ao PostgreSQL");

    const NUM_HANDS: usize = 20;
    let mut handles = Vec::with_capacity(NUM_HANDS);
    let start_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    for i in 0..NUM_HANDS {
        let pool_clone = pool.clone();
        handles.push(tokio::spawn(async move {
            let hand_id = uuid::Uuid::new_v4();
            let pot = (1000 + i * 50) as i64;
            let rake = (50 + i) as i64;

            let result = sqlx::query(
                r#"
                INSERT INTO hand_history
                    (id, hand_number, game_type, small_blind, big_blind, pot_total, rake_collected, end_reason, created_at)
                VALUES ($1, $2, 'cash', 10, 20, $3, $4, 'showdown', $5)
                "#,
            )
            .bind(hand_id)
            .bind((i + 1) as i32)
            .bind(pot)
            .bind(rake)
            .bind((start_ns / 1_000_000_000) as i64)
            .execute(&pool_clone)
            .await;

            (hand_id, result)
        }));
    }

    let mut inserted_count = 0;
    let mut inserted_ids = Vec::with_capacity(NUM_HANDS);
    for handle in handles {
        let (hand_id, res) = handle.await.unwrap();
        assert!(
            res.is_ok(),
            "Falha ao gravar historico de mão: {:?}",
            res.err()
        );
        inserted_count += 1;
        inserted_ids.push(hand_id);
    }

    sqlx::query("DELETE FROM hand_history WHERE id = ANY($1)")
        .bind(&inserted_ids)
        .execute(&pool)
        .await
        .expect("Falha ao remover históricos criados pelo stress test");

    assert_eq!(inserted_count, NUM_HANDS);
}
