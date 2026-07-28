// API Axum — main entry point
//
// Wires all routes, initializes DB pool, runs migrations, sets up CORS,
// tracing, and starts the HTTP server.
//
// The router construction logic lives in `lib.rs` (`poker_api::build_router`)
// so that integration tests can reuse it without binding a TCP port.

use std::net::SocketAddr;

use axum::http::{header, HeaderValue, Method};
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

    let is_production =
        std::env::var("ENVIRONMENT").is_ok_and(|value| value.eq_ignore_ascii_case("production"));

    // Never run production with a documented sample credential. Length alone
    // does not prove entropy, but it prevents common accidental deployments.
    let is_known_sample_secret = matches!(
        jwt_secret.as_str(),
        "supersecretkey12345678901234567890"
            | "test-secret-key-for-poker-platform-2026"
            | "test-secret-key-for-tests"
    );
    if is_production
        && (jwt_secret.len() < 32 || is_known_sample_secret || jwt_secret.contains("change_me"))
    {
        return Err("Refusing to boot production with an insecure JWT_SECRET".into());
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

    // Initialize Redis connection if REDIS_URL is provided
    let redis_conn = if let Ok(redis_url) = std::env::var("REDIS_URL") {
        match redis::Client::open(redis_url) {
            Ok(client) => match client.get_connection_manager().await {
                Ok(cm) => {
                    tracing::info!("Connected to Redis Cache");
                    Some(cm)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to get Redis connection manager: {}. Continuing without Redis.",
                        e
                    );
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Invalid REDIS_URL: {}. Continuing without Redis.", e);
                None
            }
        }
    } else {
        tracing::info!("REDIS_URL not set. Running in-memory state mode.");
        None
    };
    if is_production && redis_conn.is_none() {
        return Err(
            "REDIS_URL with a reachable Redis instance is required in production for WebSocket tickets and table snapshots"
                .into(),
        );
    }

    // Build app state with high-concurrency RwLock
    let state = AppState {
        db: pool,
        auth: std::sync::Arc::new(tokio::sync::RwLock::new(
            poker_engine::auth::AuthManager::new(&jwt_secret),
        )),
        tournaments: std::sync::Arc::new(
            tokio::sync::RwLock::new(std::collections::HashMap::new()),
        ),
        active_tables: std::sync::Arc::new(tokio::sync::RwLock::new(
            std::collections::HashMap::new(),
        )),
        jwt_secret,
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: redis_conn,
        ws_tickets: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
    };

    // CORS
    let cors = if cors_origins.is_empty() {
        if is_production {
            return Err("CORS_ORIGINS must be explicitly set in production".into());
        }
        tracing::warn!("CORS_ORIGINS is not set; allowing all origins outside production only");
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins: Vec<HeaderValue> = cors_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        if origins.is_empty() {
            return Err("CORS_ORIGINS must contain at least one valid origin".into());
        }
        CorsLayer::new().allow_origin(origins)
    }
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // Build router
    let app = build_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
