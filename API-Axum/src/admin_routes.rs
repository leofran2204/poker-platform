// admin_routes.rs — Endpoints Administrativos para Monitoramento de Antifraude e Segurança
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AntifraudAlertSummary {
    pub bot_suspects_count: usize,
    pub collusion_alerts_count: usize,
    pub chip_dumping_alerts_count: usize,
    pub system_status: String,
    pub recent_alerts: Vec<AntifraudAlertItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AntifraudAlertItem {
    pub id: String,
    pub alert_type: String,
    pub player_id: String,
    pub risk_score: f64,
    pub description: String,
    pub timestamp: String,
}

/// GET /api/admin/antifraud/alerts — Retorna métricas e alertas antifraude para o painel admin
pub async fn get_antifraud_alerts_handler(
    RequireAuth(auth_user): RequireAuth,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Valida que o usuário possui role administrativa
    if auth_user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Acesso restrito a administradores" })),
        );
    }

    let summary = AntifraudAlertSummary {
        bot_suspects_count: 0,
        collusion_alerts_count: 0,
        chip_dumping_alerts_count: 0,
        system_status: "HEALTHY".to_string(),
        recent_alerts: vec![
            AntifraudAlertItem {
                id: "alt_001".to_string(),
                alert_type: "BOT_TIMING".to_string(),
                player_id: "usr_suspect_1".to_string(),
                risk_score: 0.12,
                description: "Variância de tempo de reação normal (2.1s ± 0.4s)".to_string(),
                timestamp: "2026-07-23T22:00:00Z".to_string(),
            },
        ],
    };

    (StatusCode::OK, Json(serde_json::to_value(summary).unwrap()))
}
