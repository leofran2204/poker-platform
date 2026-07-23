//! Módulo de Telemetria e Tracing Estruturado para o API-Axum.
//!
//! Fornece utilitários para inicialização de logs estruturados (JSON/EnvFilter)
//! e rastreamento de desempenho (spans) de transações financeiras e mesas de poker.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Inicializa o subsistema de tracing global.
pub fn init_telemetry() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,poker_api=debug,poker_engine=debug"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .init();

    tracing::info!("Telemetria e Tracing inicializados com sucesso.");
}

/// Macro auxiliar para registrar spans de auditoria financeira com contexto de usuário.
#[macro_export]
macro_rules! audit_span {
    ($user_id:expr, $action:expr) => {
        tracing::info_span!(
            "audit_event",
            user_id = %$user_id,
            action = %$action,
            timestamp = %chrono::Utc::now().to_rfc3339()
        )
    };
}
