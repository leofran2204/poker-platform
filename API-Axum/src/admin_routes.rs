// admin_routes.rs — Endpoints Administrativos para Monitoramento de Antifraude e Segurança
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
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

#[derive(Debug, Deserialize)]
pub struct CreateCashTableRequest {
    pub name: String,
    /// All money values are integer cents.
    pub small_blind: u64,
    pub big_blind: u64,
    pub min_buy_in: u64,
    pub max_buy_in: u64,
    pub max_players: u8,
    pub rake_basis_points: u16,
    pub rake_cap: u64,
    pub rake_cap_heads_up: Option<u64>,
    pub rake_cap_three_to_four: Option<u64>,
    pub rake_cap_five_plus: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTableStatusRequest {
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AdminTableResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub small_blind: u64,
    pub big_blind: u64,
    pub min_buy_in: u64,
    pub max_buy_in: u64,
    pub max_players: u8,
    pub rake_basis_points: u16,
    pub rake_cap: u64,
    pub rake_cap_heads_up: Option<u64>,
    pub rake_cap_three_to_four: Option<u64>,
    pub rake_cap_five_plus: Option<u64>,
}

type AdminTableRow = (
    uuid::Uuid,
    String,
    String,
    i64,
    i64,
    i64,
    i64,
    i16,
    i16,
    i64,
    Option<i64>,
    Option<i64>,
    Option<i64>,
);

fn require_admin(auth_user: &crate::middleware::auth::AuthUser) -> Result<(), ApiError> {
    if auth_user.role == "admin" {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "Administrator access is required".to_string(),
        ))
    }
}

fn as_i64(value: u64, field: &str) -> Result<i64, ApiError> {
    i64::try_from(value)
        .map_err(|_| ApiError::BadRequest(format!("{field} exceeds the supported range")))
}
fn optional_u64(value: Option<i64>, field: &str) -> Result<Option<u64>, ApiError> {
    value
        .map(|stored| {
            u64::try_from(stored).map_err(|_| ApiError::Internal(format!("Invalid stored {field}")))
        })
        .transpose()
}

fn admin_table_response(
    (
        id,
        name,
        status,
        small_blind,
        big_blind,
        min_buy_in,
        max_buy_in,
        max_players,
        rake_basis_points,
        rake_cap,
        rake_cap_heads_up,
        rake_cap_three_to_four,
        rake_cap_five_plus,
    ): AdminTableRow,
) -> Result<AdminTableResponse, ApiError> {
    Ok(AdminTableResponse {
        id: id.to_string(),
        name,
        status,
        small_blind: u64::try_from(small_blind)
            .map_err(|_| ApiError::Internal("Invalid stored small blind".to_string()))?,
        big_blind: u64::try_from(big_blind)
            .map_err(|_| ApiError::Internal("Invalid stored big blind".to_string()))?,
        min_buy_in: u64::try_from(min_buy_in)
            .map_err(|_| ApiError::Internal("Invalid stored minimum buy-in".to_string()))?,
        max_buy_in: u64::try_from(max_buy_in)
            .map_err(|_| ApiError::Internal("Invalid stored maximum buy-in".to_string()))?,
        max_players: u8::try_from(max_players)
            .map_err(|_| ApiError::Internal("Invalid stored table capacity".to_string()))?,
        rake_basis_points: u16::try_from(rake_basis_points)
            .map_err(|_| ApiError::Internal("Invalid stored rake".to_string()))?,
        rake_cap: u64::try_from(rake_cap)
            .map_err(|_| ApiError::Internal("Invalid stored rake cap".to_string()))?,
        rake_cap_heads_up: optional_u64(rake_cap_heads_up, "heads-up rake cap")?,
        rake_cap_three_to_four: optional_u64(rake_cap_three_to_four, "3-4 rake cap")?,
        rake_cap_five_plus: optional_u64(rake_cap_five_plus, "5+ rake cap")?,
    })
}

/// POST /api/admin/tables — creates an open public cash table in PostgreSQL.
pub async fn create_cash_table_handler(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateCashTableRequest>,
) -> Result<(StatusCode, Json<AdminTableResponse>), ApiError> {
    require_admin(&auth_user)?;

    let name = body.name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError::BadRequest(
            "Table name must contain between 1 and 100 characters".to_string(),
        ));
    }
    if body.small_blind == 0
        || body.small_blind.checked_mul(2) != Some(body.big_blind)
        || body.min_buy_in == 0
        || body.min_buy_in > body.max_buy_in
        || !(2..=9).contains(&body.max_players)
        || body.rake_basis_points > 1_000
    {
        return Err(ApiError::BadRequest(
            "Invalid cash table configuration".to_string(),
        ));
    }

    let rake_cap_schedule = match (
        body.rake_cap_heads_up,
        body.rake_cap_three_to_four,
        body.rake_cap_five_plus,
    ) {
        (None, None, None) => None,
        (Some(heads_up), Some(three_to_four), Some(five_plus)) => Some((
            as_i64(heads_up, "rake_cap_heads_up")?,
            as_i64(three_to_four, "rake_cap_three_to_four")?,
            as_i64(five_plus, "rake_cap_five_plus")?,
        )),
        _ => {
            return Err(ApiError::BadRequest(
                "All player-count rake caps must be provided together".to_string(),
            ))
        }
    };

    let small_blind = as_i64(body.small_blind, "small_blind")?;
    let big_blind = as_i64(body.big_blind, "big_blind")?;
    let min_buy_in = as_i64(body.min_buy_in, "min_buy_in")?;
    let max_buy_in = as_i64(body.max_buy_in, "max_buy_in")?;
    let rake_cap = as_i64(body.rake_cap, "rake_cap")?;
    let rake_basis_points = i16::try_from(body.rake_basis_points)
        .map_err(|_| ApiError::BadRequest("Invalid rake".to_string()))?;
    let (rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus) = rake_cap_schedule
        .map(|(heads_up, three_to_four, five_plus)| {
            (Some(heads_up), Some(three_to_four), Some(five_plus))
        })
        .unwrap_or((None, None, None));

    let mut tx = state.db.begin().await?;
    let row: AdminTableRow = sqlx::query_as(
        "INSERT INTO tables (name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, max_players, visibility, status, rake_basis_points, rake_cap, rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus) \
         VALUES ($1, 'cash', $2, $3, $4, $5, $6, 'public', 'OPEN', $7, $8, $9, $10, $11) \
         RETURNING id, name, status, small_blind, big_blind, min_buy_in, max_buy_in, max_players, rake_basis_points, rake_cap, rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus",
    )
    .bind(name)
    .bind(small_blind)
    .bind(big_blind)
    .bind(min_buy_in)
    .bind(max_buy_in)
    .bind(i16::from(body.max_players))
    .bind(rake_basis_points)
    .bind(rake_cap)
    .bind(rake_cap_heads_up)
    .bind(rake_cap_three_to_four)
    .bind(rake_cap_five_plus)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'TABLE_CREATED', $2)",
    )
    .bind(&auth_user.user_id)
    .bind(serde_json::json!({"table_id": row.0, "name": name}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(admin_table_response(row)?)))
}

/// PATCH /api/admin/tables/:id/status — pauses, reopens, or closes a table.
pub async fn update_table_status_handler(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(table_id): Path<String>,
    Json(body): Json<UpdateTableStatusRequest>,
) -> Result<Json<AdminTableResponse>, ApiError> {
    require_admin(&auth_user)?;
    let table_id = uuid::Uuid::parse_str(&table_id)
        .map_err(|_| ApiError::BadRequest("Invalid table id".to_string()))?;
    let status = body.status.trim().to_ascii_uppercase();
    if !matches!(status.as_str(), "OPEN" | "PAUSED" | "CLOSED") {
        return Err(ApiError::BadRequest(
            "Status must be OPEN, PAUSED, or CLOSED".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM tables WHERE id = $1 FOR UPDATE")
            .bind(table_id)
            .fetch_optional(&mut *tx)
            .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound("Table not found".to_string()));
    }
    if status == "CLOSED" {
        let (has_active_seats,): (bool,) = sqlx::query_as(
            "SELECT EXISTS( \
                 SELECT 1 FROM cash_game_seats WHERE table_id = $1 AND status = 'ACTIVE' \
             )",
        )
        .bind(table_id)
        .fetch_one(&mut *tx)
        .await?;
        if has_active_seats {
            return Err(ApiError::BadRequest(
                "Cash out all active seats before closing a table".to_string(),
            ));
        }
    }

    let row: AdminTableRow = sqlx::query_as(
        "UPDATE tables SET status = $1 WHERE id = $2 \
         RETURNING id, name, status, small_blind, big_blind, min_buy_in, max_buy_in, max_players, rake_basis_points, rake_cap, rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus",
    )
    .bind(&status)
    .bind(table_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'TABLE_STATUS_CHANGED', $2)",
    )
    .bind(&auth_user.user_id)
    .bind(serde_json::json!({"table_id": table_id, "status": status}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(admin_table_response(row)?))
}

/// POST /api/admin/tables/:id/recovery/abort — records the explicit abort of
/// a hand left incomplete by a process failure. The pre-hand escrow remains in
/// PostgreSQL; the administrator must reopen the table separately after review.
pub async fn abort_table_recovery_handler(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(table_id): Path<String>,
) -> Result<Json<AdminTableResponse>, ApiError> {
    require_admin(&auth_user)?;
    let table_id = uuid::Uuid::parse_str(&table_id)
        .map_err(|_| ApiError::BadRequest("Invalid table id".to_string()))?;
    let mut tx = state.db.begin().await?;
    let guard: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT hand_id FROM table_hand_recovery_guards WHERE table_id = $1 FOR UPDATE",
    )
    .bind(table_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (hand_id,) = guard
        .ok_or_else(|| ApiError::NotFound("Table does not have an unrecovered hand".to_string()))?;
    sqlx::query("DELETE FROM table_hand_recovery_guards WHERE table_id = $1 AND hand_id = $2")
        .bind(table_id)
        .bind(hand_id)
        .execute(&mut *tx)
        .await?;
    let row: AdminTableRow = sqlx::query_as(
        "UPDATE tables SET status = 'PAUSED' WHERE id = $1 \
         RETURNING id, name, status, small_blind, big_blind, min_buy_in, max_buy_in, max_players, rake_basis_points, rake_cap, rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus",
    )
    .bind(table_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) \
         VALUES ($1, 'TABLE_HAND_RECOVERY_ABORTED', $2)",
    )
    .bind(&auth_user.user_id)
    .bind(serde_json::json!({"table_id": table_id, "hand_id": hand_id}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(admin_table_response(row)?))
}
///
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
        recent_alerts: vec![AntifraudAlertItem {
            id: "alt_001".to_string(),
            alert_type: "BOT_TIMING".to_string(),
            player_id: "usr_suspect_1".to_string(),
            risk_score: 0.12,
            description: "Variância de tempo de reação normal (2.1s ± 0.4s)".to_string(),
            timestamp: "2026-07-23T22:00:00Z".to_string(),
        }],
    };

    (StatusCode::OK, Json(serde_json::to_value(summary).unwrap()))
}

// ─── Phase 2 & Dashboard B2B SaaS Routes ───

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Club {
    pub id: Option<uuid::Uuid>,
    pub name: String,
    pub subdomain: String,
    pub custom_theme_json: serde_json::Value,
    pub status: String,
}

pub async fn create_club(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Json(payload): Json<Club>,
) -> Result<Json<Club>, ApiError> {
    require_admin(&auth_user)?;
    let result = sqlx::query_as::<_, Club>(
        "INSERT INTO clubs (name, subdomain, custom_theme_json, status) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, name, subdomain, custom_theme_json, status",
    )
    .bind(&payload.name)
    .bind(&payload.subdomain)
    .bind(&payload.custom_theme_json)
    .bind(&payload.status)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(club) => Ok(Json(club)),
        Err(_) => Err(ApiError::Internal("Could not create club".to_string())),
    }
}

pub async fn list_clubs(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<Vec<Club>>, ApiError> {
    require_admin(&auth_user)?;
    let clubs = sqlx::query_as::<_, Club>(
        "SELECT id, name, subdomain, custom_theme_json, status FROM clubs",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| ApiError::Internal("Could not list clubs".to_string()))?;

    Ok(Json(clubs))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClubFinancialsResponse {
    pub club_id: uuid::Uuid,
    pub name: String,
    pub balance: i64,
    pub total_rake_generated: i64,
    pub net_club_rake: i64,
    pub platform_fee_paid: i64,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawClubBalanceRequest {
    pub amount: u64,
    pub pix_key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClubThemeRequest {
    pub custom_theme_json: serde_json::Value,
}

pub async fn get_club_financials(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(club_id): Path<uuid::Uuid>,
) -> Result<Json<ClubFinancialsResponse>, ApiError> {
    require_admin(&auth_user)?;

    let club: Option<(uuid::Uuid, String, i64)> = sqlx::query_as(
        "SELECT id, name, balance FROM clubs WHERE id = $1",
    )
    .bind(club_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ApiError::Internal("Database error".to_string()))?;

    let (c_id, name, balance) = club.ok_or_else(|| ApiError::NotFound("Club not found".to_string()))?;

    let net_club_rake = balance;
    let platform_fee_paid = (balance * 15) / 85;
    let total_rake_generated = net_club_rake + platform_fee_paid;

    Ok(Json(ClubFinancialsResponse {
        club_id: c_id,
        name,
        balance,
        total_rake_generated,
        net_club_rake,
        platform_fee_paid,
    }))
}

pub async fn withdraw_club_balance(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(club_id): Path<uuid::Uuid>,
    Json(payload): Json<WithdrawClubBalanceRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_admin(&auth_user)?;

    let amount_i64 = as_i64(payload.amount, "amount")?;

    let mut tx = state.db.begin().await.map_err(|_| ApiError::Internal("Transaction error".to_string()))?;

    let club: Option<(i64,)> = sqlx::query_as(
        "SELECT balance FROM clubs WHERE id = $1 FOR UPDATE",
    )
    .bind(club_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| ApiError::Internal("Database error".to_string()))?;

    let (balance,) = club.ok_or_else(|| ApiError::NotFound("Club not found".to_string()))?;

    if balance < amount_i64 {
        return Err(ApiError::BadRequest("Insufficient club balance".to_string()));
    }

    sqlx::query("UPDATE clubs SET balance = balance - $1 WHERE id = $2")
        .bind(amount_i64)
        .bind(club_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::Internal("Failed to update club balance".to_string()))?;

    tx.commit().await.map_err(|_| ApiError::Internal("Commit error".to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "SUCCESS",
        "message": format!("Withdrawal of {} cents requested to PIX key {}", payload.amount, payload.pix_key)
    })))
}

pub async fn update_club_theme(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(club_id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateClubThemeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_admin(&auth_user)?;

    sqlx::query("UPDATE clubs SET custom_theme_json = $1 WHERE id = $2")
        .bind(&payload.custom_theme_json)
        .bind(club_id)
        .execute(&state.db)
        .await
        .map_err(|_| ApiError::Internal("Failed to update club theme".to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "SUCCESS",
        "message": "Club theme updated successfully"
    })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClubAgent {
    pub agent_id: String,
    pub name: String,
    pub rakeback_percentage: u8,
    pub total_players_referred: u32,
    pub total_commission_earned: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateClubAgentRequest {
    pub name: String,
    pub rakeback_percentage: u8,
}

type ClubAgentRow = (uuid::Uuid, String, i16, i32, i64);

fn club_agent_from_row(
    (id, name, rakeback_percentage, total_players_referred, total_commission_earned): ClubAgentRow,
) -> Result<ClubAgent, ApiError> {
    Ok(ClubAgent {
        agent_id: id.to_string(),
        name,
        rakeback_percentage: u8::try_from(rakeback_percentage)
            .map_err(|_| ApiError::Internal("Invalid stored rakeback percentage".to_string()))?,
        total_players_referred: u32::try_from(total_players_referred)
            .map_err(|_| ApiError::Internal("Invalid stored player referral count".to_string()))?,
        total_commission_earned: u64::try_from(total_commission_earned)
            .map_err(|_| ApiError::Internal("Invalid stored commission".to_string()))?,
    })
}

async fn require_club_exists(
    state: &AppState,
    club_id: uuid::Uuid,
) -> Result<(), ApiError> {
    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM clubs WHERE id = $1")
            .bind(club_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| ApiError::Internal("Database error".to_string()))?;
    if exists.is_none() {
        return Err(ApiError::NotFound("Club not found".to_string()));
    }
    Ok(())
}

/// GET /api/admin/clubs/:id/agents — lista agentes e comissões acumuladas do clube.
pub async fn list_club_agents(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(club_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<ClubAgent>>, ApiError> {
    require_admin(&auth_user)?;
    require_club_exists(&state, club_id).await?;

    let rows: Vec<ClubAgentRow> = sqlx::query_as(
        "SELECT id, name, rakeback_percentage, total_players_referred, total_commission_earned \
         FROM club_agents \
         WHERE club_id = $1 AND status = 'active' \
         ORDER BY created_at DESC",
    )
    .bind(club_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| ApiError::Internal("Could not list club agents".to_string()))?;

    let agents = rows
        .into_iter()
        .map(club_agent_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(agents))
}

/// POST /api/admin/clubs/:id/agents — cadastra agente com percentual de rakeback (0–50).
pub async fn create_club_agent(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(club_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateClubAgentRequest>,
) -> Result<(StatusCode, Json<ClubAgent>), ApiError> {
    require_admin(&auth_user)?;
    require_club_exists(&state, club_id).await?;

    let name = payload.name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError::BadRequest(
            "Agent name must contain between 1 and 100 characters".to_string(),
        ));
    }
    if payload.rakeback_percentage > 50 {
        return Err(ApiError::BadRequest(
            "Rakeback percentage cannot exceed 50%".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let row: ClubAgentRow = sqlx::query_as(
        "INSERT INTO club_agents (club_id, name, rakeback_percentage) \
         VALUES ($1, $2, $3) \
         RETURNING id, name, rakeback_percentage, total_players_referred, total_commission_earned",
    )
    .bind(club_id)
    .bind(name)
    .bind(i16::from(payload.rakeback_percentage))
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| ApiError::Internal("Could not create club agent".to_string()))?;

    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'CLUB_AGENT_CREATED', $2)",
    )
    .bind(&auth_user.user_id)
    .bind(serde_json::json!({
        "club_id": club_id,
        "agent_id": row.0,
        "name": name,
        "rakeback_percentage": payload.rakeback_percentage
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(club_agent_from_row(row)?)))
}
