// API Axum — main entry point
//
// Wires all routes, initializes DB pool, runs migrations, sets up CORS,
// tracing, and starts the HTTP server.
//
// The router construction logic lives in `lib.rs` (`poker_api::build_router`)
// so that integration tests can reuse it without binding a TCP port.

use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use poker_api::build_router;
use poker_api::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // Read & Validate config (Boot Guardian)
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");
    let jwt_secret =
        std::env::var("JWT_SECRET").expect("JWT_SECRET must be set (see .env.example)");

    // Security Hardening: Ensure JWT_SECRET is strong (>= 32 chars) and not a weak dev fallback
    if jwt_secret.len() < 32 || jwt_secret == "supersecretkey12345678901234567890" || jwt_secret.contains("change_me") {
        tracing::error!("FATAL: Insecure JWT_SECRET detected! Must be at least 32 random characters.");
        if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
            panic!("FATAL: Refusing to boot with weak JWT_SECRET in production.");
        }
    }

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()?;
    let cors_origins = std::env::var("CORS_ORIGINS").unwrap_or_default();

    // Initialize DB pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Migration failed: {}", e);
            e
        })?;

    tracing::info!("Migrations applied");

    // Build app state with high-concurrency RwLock
    let state = AppState {
        db: pool,
        auth: std::sync::Arc::new(tokio::sync::RwLock::new(
            poker_engine::auth::AuthManager::new(&jwt_secret),
        )),
        lobby: std::sync::Arc::new(tokio::sync::RwLock::new(
            poker_engine::lobby::LobbyManager::new(),
        )),
        tournaments: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        active_tables: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        jwt_secret,
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
    };

    // CORS
    let cors = if cors_origins.is_empty() {
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins: Vec<String> = cors_origins
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        CorsLayer::new().allow_origin(
            origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect::<Vec<_>>(),
        )
    };

    // Build router
    let app = build_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
