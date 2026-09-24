use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTicketBody {
    pub category: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminTicketQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReplyTicketBody {
    pub status: String,
    pub response: String,
}

#[derive(Debug, Serialize)]
pub struct TicketResponse {
    pub id: String,
    pub category: String,
    pub subject: String,
    pub message: String,
    pub status: String,
    pub admin_response: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

type TicketRow = (
    uuid::Uuid,
    String,
    String,
    String,
    String,
    Option<String>,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    Option<String>,
);

fn response(row: TicketRow) -> TicketResponse {
    TicketResponse {
        id: row.0.to_string(),
        category: row.1,
        subject: row.2,
        message: row.3,
        status: row.4,
        admin_response: row.5,
        created_at: row.6.to_rfc3339(),
        updated_at: row.7.to_rfc3339(),
        username: row.8,
    }
}

fn valid_category(value: &str) -> bool {
    matches!(
        value,
        "account" | "payments" | "responsible_gaming" | "technical" | "other"
    )
}

pub async fn create_ticket(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateTicketBody>,
) -> Result<Json<TicketResponse>, ApiError> {
    if !valid_category(&body.category) {
        return Err(ApiError::BadRequest("Categoria de suporte inválida".into()));
    }
    let subject = body.subject.trim();
    let message = body.message.trim();
    if !(5..=120).contains(&subject.chars().count()) {
        return Err(ApiError::BadRequest(
            "Assunto deve ter entre 5 e 120 caracteres".into(),
        ));
    }
    if !(10..=5000).contains(&message.chars().count()) {
        return Err(ApiError::BadRequest(
            "Mensagem deve ter entre 10 e 5000 caracteres".into(),
        ));
    }
    let open: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM support_tickets WHERE user_id=$1::uuid AND status IN ('open','in_progress')",
    )
    .bind(&auth.user_id)
    .fetch_one(&state.db)
    .await?;
    if open >= 5 {
        return Err(ApiError::BadRequest(
            "Você já possui cinco chamados em atendimento.".into(),
        ));
    }
    let row: TicketRow = sqlx::query_as(
        "INSERT INTO support_tickets(user_id,category,subject,message) VALUES($1::uuid,$2,$3,$4) \
         RETURNING id,category,subject,message,status,admin_response,created_at,updated_at,NULL::text",
    )
    .bind(&auth.user_id)
    .bind(&body.category)
    .bind(subject)
    .bind(message)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(response(row)))
}

pub async fn my_tickets(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<Vec<TicketResponse>>, ApiError> {
    let rows: Vec<TicketRow> = sqlx::query_as(
        "SELECT id,category,subject,message,status,admin_response,created_at,updated_at,NULL::text \
         FROM support_tickets WHERE user_id=$1::uuid ORDER BY created_at DESC LIMIT 100",
    )
    .bind(&auth.user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows.into_iter().map(response).collect()))
}

pub async fn admin_tickets(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<AdminTicketQuery>,
) -> Result<Json<Vec<TicketResponse>>, ApiError> {
    if auth.role != "admin" {
        return Err(ApiError::Forbidden("Admin role required".into()));
    }
    let status = query.status.unwrap_or_else(|| "open".into());
    if !matches!(
        status.as_str(),
        "open" | "in_progress" | "resolved" | "closed"
    ) {
        return Err(ApiError::BadRequest("Status de suporte inválido".into()));
    }
    let rows: Vec<TicketRow> = sqlx::query_as(
        "SELECT t.id,t.category,t.subject,t.message,t.status,t.admin_response,t.created_at,t.updated_at,u.username \
         FROM support_tickets t JOIN users u ON u.id=t.user_id WHERE t.status=$1 ORDER BY t.created_at LIMIT 200",
    )
    .bind(status)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows.into_iter().map(response).collect()))
}

pub async fn reply_ticket(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ReplyTicketBody>,
) -> Result<Json<TicketResponse>, ApiError> {
    if auth.role != "admin" {
        return Err(ApiError::Forbidden("Admin role required".into()));
    }
    if !matches!(body.status.as_str(), "in_progress" | "resolved" | "closed") {
        return Err(ApiError::BadRequest("Status de suporte inválido".into()));
    }
    let message = body.response.trim();
    if !(2..=5000).contains(&message.chars().count()) {
        return Err(ApiError::BadRequest(
            "Resposta deve ter entre 2 e 5000 caracteres".into(),
        ));
    }
    let row: Option<TicketRow> = sqlx::query_as(
        "UPDATE support_tickets SET status=$2,admin_response=$3,responded_by=$4::uuid,updated_at=NOW() WHERE id=$1::uuid \
         RETURNING id,category,subject,message,status,admin_response,created_at,updated_at,NULL::text",
    )
    .bind(id)
    .bind(&body.status)
    .bind(message)
    .bind(&auth.user_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(Json(response(row.ok_or_else(|| {
        ApiError::NotFound("Chamado não encontrado".into())
    })?)))
}

#[cfg(test)]
mod tests {
    use super::valid_category;

    #[test]
    fn only_known_support_categories_are_accepted() {
        assert!(valid_category("payments"));
        assert!(valid_category("responsible_gaming"));
        assert!(!valid_category("spam"));
    }
}
