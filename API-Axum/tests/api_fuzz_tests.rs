// api_fuzz_tests.rs — Suíte de Fuzzing de Endpoints HTTP/REST (200.000 iterações/função)
// Garante que o servidor Axum processe 2 MILHÕES de requisições estocásticas com 0 panics ou crashes.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use poker_api::build_router;
use poker_api::state::AppState;
use poker_engine::auth::AuthManager;
use poker_engine::lobby::LobbyManager;
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

fn make_test_state() -> AppState {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://unused:unused@localhost:5432/unused".to_string()
    });
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&url)
        .unwrap();

    AppState {
        db,
        auth: Arc::new(Mutex::new(AuthManager::new("fuzz-test-jwt-secret-key-32chars"))),
        lobby: Arc::new(Mutex::new(LobbyManager::new())),
        tournaments: Arc::new(Mutex::new(HashMap::new())),
        active_tables: Arc::new(Mutex::new(HashMap::new())),
        jwt_secret: "fuzz-test-jwt-secret-key-32chars".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
    }
}

fn get_proptest_config() -> ProptestConfig {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(200_000);
    ProptestConfig {
        cases,
        max_shrink_iters: 100,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(get_proptest_config())]

    // ─── 1. Register Fuzz ───
    #[test]
    fn api_auth_register_fuzz(
        raw_input in ".*",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let req_builder = Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header("content-type", "application/json");

            if let Ok(req) = req_builder.body(Body::from(raw_input)) {
                let response = app.oneshot(req).await.unwrap();
                let status = response.status();
                prop_assert!(
                    status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::CONFLICT || status == StatusCode::OK || status == StatusCode::INTERNAL_SERVER_ERROR,
                    "Status inesperado no register fuzzing: {}", status
                );
            }
            Ok(())
        })?;
    }

    // ─── 2. Login Fuzz ───
    #[test]
    fn api_auth_login_fuzz(
        raw_input in ".*",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let req_builder = Request::builder()
                .method(Method::POST)
                .uri("/api/auth/login")
                .header("content-type", "application/json");

            if let Ok(req) = req_builder.body(Body::from(raw_input)) {
                let response = app.oneshot(req).await.unwrap();
                let status = response.status();
                prop_assert!(
                    status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::UNAUTHORIZED || status == StatusCode::OK || status == StatusCode::INTERNAL_SERVER_ERROR,
                    "Status inesperado no login fuzzing: {}", status
                );
            }
            Ok(())
        })?;
    }

    // ─── 3. MFA Verify Fuzz ───
    #[test]
    fn api_auth_mfa_verify_fuzz(
        raw_input in ".*",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let req_builder = Request::builder()
                .method(Method::POST)
                .uri("/api/auth/mfa/verify")
                .header("content-type", "application/json");

            if let Ok(req) = req_builder.body(Body::from(raw_input)) {
                let response = app.oneshot(req).await.unwrap();
                let status = response.status();
                prop_assert!(
                    status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::UNAUTHORIZED || status == StatusCode::INTERNAL_SERVER_ERROR,
                    "Status inesperado no MFA verify fuzzing: {}", status
                );
            }
            Ok(())
        })?;
    }

    // ─── 4. Refresh Token Fuzz ───
    #[test]
    fn api_auth_refresh_fuzz(
        raw_input in ".*",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let req_builder = Request::builder()
                .method(Method::POST)
                .uri("/api/auth/refresh")
                .header("content-type", "application/json");

            if let Ok(req) = req_builder.body(Body::from(raw_input)) {
                let response = app.oneshot(req).await.unwrap();
                let status = response.status();
                prop_assert!(
                    status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::UNAUTHORIZED || status == StatusCode::INTERNAL_SERVER_ERROR,
                    "Status inesperado no refresh fuzzing: {}", status
                );
            }
            Ok(())
        })?;
    }

    // ─── 5. Lobby List Tables Query Fuzz ───
    #[test]
    fn api_lobby_tables_query_fuzz(
        game_type in "[a-zA-Z0-9_-]{0,20}",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let uri = format!("/api/lobby/tables?game_type={}", game_type);

            let req = Request::builder()
                .method(Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(req).await.unwrap();
            let status = response.status();
            prop_assert_eq!(status, StatusCode::OK);
            Ok(())
        })?;
    }

    // ─── 6. Lobby Join Table Fuzz ───
    #[test]
    fn api_lobby_join_fuzz(
        raw_input in ".*",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let req_builder = Request::builder()
                .method(Method::POST)
                .uri("/api/lobby/join")
                .header("content-type", "application/json")
                .header("authorization", "Bearer valid_or_invalid_token");

            if let Ok(req) = req_builder.body(Body::from(raw_input)) {
                let response = app.oneshot(req).await.unwrap();
                let status = response.status();
                prop_assert!(
                    status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
                    "Status inesperado no lobby join fuzzing: {}", status
                );
            }
            Ok(())
        })?;
    }

    // ─── 7. Tournament Register Fuzz ───
    #[test]
    fn api_tournament_register_fuzz(
        raw_input in ".*",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let req_builder = Request::builder()
                .method(Method::POST)
                .uri("/api/tournament/register")
                .header("content-type", "application/json")
                .header("authorization", "Bearer valid_or_invalid_token");

            if let Ok(req) = req_builder.body(Body::from(raw_input)) {
                let response = app.oneshot(req).await.unwrap();
                let status = response.status();
                prop_assert!(
                    status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
                    "Status inesperado no tournament register fuzzing: {}", status
                );
            }
            Ok(())
        })?;
    }

    // ─── 8. Tournament Get Path Fuzz ───
    #[test]
    fn api_tournament_get_path_fuzz(
        tourn_id in "[a-zA-Z0-9_-]{1,64}",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let uri = format!("/api/tournament/{}", tourn_id);

            let req = Request::builder()
                .method(Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(req).await.unwrap();
            let status = response.status();
            prop_assert!(status == StatusCode::NOT_FOUND || status == StatusCode::OK);
            Ok(())
        })?;
    }

    // ─── 9. Hand History Path Fuzz ───
    #[test]
    fn api_hand_history_path_fuzz(
        random_path_param in "[a-zA-Z0-9_-]{1,64}",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let uri = format!("/api/hand-history/{}", random_path_param);

            let req = Request::builder()
                .method(Method::GET)
                .uri(&uri)
                .header("authorization", "Bearer fake_token")
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(req).await.unwrap();
            let status = response.status();
            prop_assert_eq!(status, StatusCode::UNAUTHORIZED);
            Ok(())
        })?;
    }

    // ─── 10. JWT Header Fuzz ───
    #[test]
    fn api_auth_header_jwt_fuzz(
        fake_token in ".*",
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let state = make_test_state();
            let app = build_router(state);

            let auth_header_val = format!("Bearer {}", fake_token);
            if let Ok(header_val) = axum::http::HeaderValue::from_str(&auth_header_val) {
                let req = Request::builder()
                    .method(Method::POST)
                    .uri("/api/lobby/join")
                    .header("content-type", "application/json")
                    .header("authorization", header_val)
                    .body(Body::from(r#"{"table_id":"00000000-0000-0000-0000-000000000000","buy_in":100.0}"#))
                    .unwrap();

                let response = app.oneshot(req).await.unwrap();
                let status = response.status();
                prop_assert!(
                    status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
                    "Status inesperado no auth header fuzzing: {}", status
                );
            }
            Ok(())
        })?;
    }
}
