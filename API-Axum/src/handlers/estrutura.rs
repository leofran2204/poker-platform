use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AgentDirectPlayer {
    pub username: String,
    pub play_gross_revenue_cents: i64,
    pub play_cash_rake_cents: i64,
    pub play_tournament_fees_cents: i64,
    pub real_gross_revenue_cents: i64,
    pub real_cash_rake_cents: i64,
    pub real_tournament_fees_cents: i64,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct AgentModeSummary {
    pub money_mode: String,
    pub active_players_count: i64,
    pub gross_revenue_cents: i64,
    pub deductions_cents: i64,
    pub ngr_cents: i64,
    pub target_ngr_cents: i64,
    pub target_reached: bool,
    pub commission_percent: i64,
    pub projected_commission_cents: i64,
    pub adjustments: Vec<AgentAdjustmentDetail>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AgentAdjustmentDetail {
    pub id: uuid::Uuid,
    pub category: String,
    pub amount_cents: i64,
    pub note: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AgentCycleQuery {
    pub cycle_start: NaiveDate,
    pub money_mode: String,
}

/// Prévia administrativa: a confirmação exige os mesmos totais no fechamento.
pub async fn preview_agent_cycle(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<AgentCycleQuery>,
) -> Result<Json<AgentModeSummary>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let uid = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;
    let cycle = validate_cycle_start(query.cycle_start)?;
    let mode = validate_money_mode(&query.money_mode)?;
    Ok(Json(load_mode_summary(&state.db, uid, cycle, mode).await?))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AgentCycleSummary {
    pub cycle_start: NaiveDate,
    pub money_mode: String,
    pub target_ngr_cents: i64,
    pub status: String,
    pub gross_revenue_cents: i64,
    pub deductions_cents: i64,
    pub ngr_cents: i64,
    pub commission_percent: Option<i16>,
    pub commission_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct EstruturaResponse {
    pub referral_code: Option<String>,
    pub agent_status: String,
    pub cycle_start: NaiveDate,
    pub base_percent: i64,
    pub bonus_percent: i64,
    pub estrutura_points: i64,
    pub agent_commission_balance_cents: i64,
    pub direct_players_count: i64,
    pub active_players_count: i64,
    pub play: AgentModeSummary,
    pub real: AgentModeSummary,
    pub direct_players: Vec<AgentDirectPlayer>,
    pub recent_cycles: Vec<AgentCycleSummary>,
}

#[derive(Debug, Serialize)]
pub struct EstruturaBackfillPreview {
    pub root_user_id: String,
    pub root_username: String,
    pub unlinked_accounts: i64,
    pub root_has_sponsor: bool,
}

#[derive(Debug, Deserialize)]
pub struct EstruturaBackfillRequest {
    pub root_user_id: String,
    pub confirm: bool,
}

#[derive(Debug, Serialize)]
pub struct EstruturaBackfillResult {
    pub root_user_id: String,
    pub root_username: String,
    pub updated_accounts: u64,
}

/// GET /api/estrutura — painel do Agente ZT (um nível direto).
#[derive(Debug, Deserialize)]
pub struct EstruturaQuery {
    pub cycle_start: Option<NaiveDate>,
}

pub async fn get_estrutura(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Query(query): Query<EstruturaQuery>,
) -> Result<Json<EstruturaResponse>, ApiError> {
    let uid = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| ApiError::Internal("Authenticated user id is invalid".to_string()))?;

    let current_cycle: NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', timezone('America/Sao_Paulo', now()))::date",
    )
    .fetch_one(&state.db)
    .await?;
    let cycle_start = validate_cycle_start(query.cycle_start.unwrap_or(current_cycle))?;
    if cycle_start > current_cycle {
        return Err(ApiError::BadRequest(
            "Escolha o mês atual ou um mês anterior.".into(),
        ));
    }

    let (referral_code, agent_status, estrutura_points, agent_commission_balance_cents): (
        Option<String>,
        String,
        i64,
        i64,
    ) = sqlx::query_as(
        "SELECT referral_code, agent_status, estrutura_points, agent_commission_balance_cents \
         FROM users WHERE id = $1",
    )
    .bind(uid)
    .fetch_one(&state.db)
    .await?;

    let play = load_mode_summary(&state.db, uid, cycle_start, "play").await?;
    let real = load_mode_summary(&state.db, uid, cycle_start, "real").await?;

    let direct_rows: Vec<(String, i64, i64, i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT u.username, \
                COALESCE(SUM(el.source_rake_cents) FILTER (WHERE el.money_mode = 'play'), 0)::BIGINT, \
                COALESCE(SUM(el.source_rake_cents) FILTER (WHERE el.money_mode = 'play' AND el.source_type = 'hand'), 0)::BIGINT, \
                COALESCE(SUM(el.source_rake_cents) FILTER (WHERE el.money_mode = 'play' AND el.source_type = 'fee'), 0)::BIGINT, \
                COALESCE(SUM(el.source_rake_cents) FILTER (WHERE el.money_mode = 'real'), 0)::BIGINT, \
                COALESCE(SUM(el.source_rake_cents) FILTER (WHERE el.money_mode = 'real' AND el.source_type = 'hand'), 0)::BIGINT, \
                COALESCE(SUM(el.source_rake_cents) FILTER (WHERE el.money_mode = 'real' AND el.source_type = 'fee'), 0)::BIGINT \
         FROM users u \
         LEFT JOIN estrutura_ledger el ON el.source_user_id = u.id \
              AND el.beneficiary_user_id = $1 AND el.program_version = 2 \
              AND el.cycle_start = $2 \
         WHERE u.sponsored_by = $1 AND NOT u.is_bot \
         GROUP BY u.id, u.username \
         ORDER BY COALESCE(SUM(el.source_rake_cents), 0) DESC, u.username",
    )
    .bind(uid)
    .bind(cycle_start)
    .fetch_all(&state.db)
    .await?;

    let direct_players_count = direct_rows.len() as i64;
    let direct_players: Vec<AgentDirectPlayer> = direct_rows
        .into_iter()
        .map(
            |(
                username,
                play_gross_revenue_cents,
                play_cash_rake_cents,
                play_tournament_fees_cents,
                real_gross_revenue_cents,
                real_cash_rake_cents,
                real_tournament_fees_cents,
            )| {
                AgentDirectPlayer {
                    username,
                    play_gross_revenue_cents,
                    play_cash_rake_cents,
                    play_tournament_fees_cents,
                    real_gross_revenue_cents,
                    real_cash_rake_cents,
                    real_tournament_fees_cents,
                    active: play_gross_revenue_cents > 0 || real_gross_revenue_cents > 0,
                }
            },
        )
        .collect();
    let active_players_count = direct_players.iter().filter(|p| p.active).count() as i64;

    let recent_cycles: Vec<AgentCycleSummary> = sqlx::query_as(
        "SELECT cycle_start, money_mode, target_ngr_cents, status, gross_revenue_cents, \
                deductions_cents, ngr_cents, commission_percent, commission_cents \
         FROM agent_monthly_cycles WHERE agent_user_id = $1 AND status = 'closed' \
         ORDER BY cycle_start DESC, money_mode LIMIT 12",
    )
    .bind(uid)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(EstruturaResponse {
        referral_code,
        agent_status,
        cycle_start,
        base_percent: crate::estrutura::BASE_PERCENT,
        bonus_percent: crate::estrutura::BONUS_PERCENT,
        estrutura_points,
        agent_commission_balance_cents,
        direct_players_count,
        active_players_count,
        play,
        real,
        direct_players,
        recent_cycles,
    }))
}

async fn load_mode_summary(
    db: &sqlx::PgPool,
    agent_id: uuid::Uuid,
    cycle_start: NaiveDate,
    money_mode: &str,
) -> Result<AgentModeSummary, ApiError> {
    let (gross_revenue_cents, active_players_count): (i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(source_rake_cents), 0)::BIGINT, \
                COUNT(DISTINCT source_user_id)::BIGINT FROM estrutura_ledger \
         WHERE beneficiary_user_id = $1 AND program_version = 2 \
           AND cycle_start = $2 AND money_mode = $3",
    )
    .bind(agent_id)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_one(db)
    .await?;
    let deductions_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0)::BIGINT FROM agent_ngr_adjustments \
         WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3",
    )
    .bind(agent_id)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_one(db)
    .await?;
    let target_ngr_cents: i64 = sqlx::query_scalar(
        "SELECT target_ngr_cents FROM agent_monthly_cycles \
         WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3",
    )
    .bind(agent_id)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_optional(db)
    .await?
    .unwrap_or(0);
    let calculation =
        crate::estrutura::calculate_cycle(gross_revenue_cents, deductions_cents, target_ngr_cents);
    let ngr_cents = calculation.ngr;
    let target_reached = target_ngr_cents > 0 && ngr_cents >= target_ngr_cents;
    let commission_percent = calculation.percent;
    let projected_commission_cents = calculation.commission;

    let adjustments = sqlx::query_as::<_, AgentAdjustmentDetail>(
        "SELECT id, category, amount_cents, note, created_at FROM agent_ngr_adjustments \
         WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3 \
         ORDER BY created_at DESC, id DESC LIMIT 100",
    )
    .bind(agent_id)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_all(db)
    .await?;
    Ok(AgentModeSummary {
        money_mode: money_mode.to_string(),
        active_players_count,
        gross_revenue_cents,
        deductions_cents,
        ngr_cents,
        target_ngr_cents,
        target_reached,
        commission_percent,
        projected_commission_cents,
        adjustments,
    })
}

#[derive(Debug, Deserialize)]
pub struct AgentConfigRequest {
    pub status: Option<String>,
    pub cycle_start: Option<NaiveDate>,
    pub money_mode: Option<String>,
    pub target_ngr_cents: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AgentConfigResponse {
    pub user_id: String,
    pub agent_status: String,
    pub cycle_start: NaiveDate,
    pub money_mode: String,
    pub target_ngr_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct AgentAdjustmentRequest {
    pub request_id: uuid::Uuid,
    pub cycle_start: NaiveDate,
    pub money_mode: String,
    pub category: String,
    pub amount_cents: i64,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct AgentAdjustmentResponse {
    pub id: String,
    pub amount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct AgentCloseRequest {
    pub cycle_start: NaiveDate,
    pub money_mode: String,
    pub reconciled: bool,
    pub expected_gross_revenue_cents: i64,
    pub expected_deductions_cents: i64,
    pub expected_target_ngr_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct AgentCloseResponse {
    pub user_id: String,
    pub cycle_start: NaiveDate,
    pub money_mode: String,
    pub target_ngr_cents: i64,
    pub gross_revenue_cents: i64,
    pub deductions_cents: i64,
    pub ngr_cents: i64,
    pub commission_percent: i16,
    pub commission_cents: i64,
    pub already_closed: bool,
}

fn validate_money_mode(mode: &str) -> Result<&str, ApiError> {
    match mode {
        "play" | "real" => Ok(mode),
        _ => Err(ApiError::BadRequest(
            "money_mode must be play or real".into(),
        )),
    }
}

fn validate_cycle_start(cycle_start: NaiveDate) -> Result<NaiveDate, ApiError> {
    use chrono::Datelike;
    if cycle_start.day() != 1 {
        return Err(ApiError::BadRequest(
            "cycle_start must be the first day of a month".into(),
        ));
    }
    Ok(cycle_start)
}

/// PATCH /api/admin/agentes/:id — aprova/suspende agente e fixa meta do ciclo.
pub async fn configure_agent(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(body): Json<AgentConfigRequest>,
) -> Result<Json<AgentConfigResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    if body.status.is_none() && body.target_ngr_cents.is_none() {
        return Err(ApiError::BadRequest(
            "Provide status and/or target_ngr_cents".into(),
        ));
    }
    let uid = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;
    let current_cycle: NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', timezone('America/Sao_Paulo', now()))::date",
    )
    .fetch_one(&state.db)
    .await?;
    let cycle_start = validate_cycle_start(body.cycle_start.unwrap_or(current_cycle))?;
    let money_mode = validate_money_mode(body.money_mode.as_deref().unwrap_or("play"))?;

    if let Some(ref status) = body.status {
        if !matches!(status.as_str(), "inactive" | "active" | "suspended") {
            return Err(ApiError::BadRequest("Invalid agent status".into()));
        }
    }
    if let Some(target) = body.target_ngr_cents {
        if target < 0 {
            return Err(ApiError::BadRequest(
                "target_ngr_cents cannot be negative".into(),
            ));
        }
        if cycle_start <= current_cycle {
            return Err(ApiError::BadRequest(
                "Defina a meta antes do início do mês; escolha um ciclo futuro.".into(),
            ));
        }
    }

    let mut tx = state.db.begin().await?;
    if let Some(ref status) = body.status {
        let updated =
            sqlx::query("UPDATE users SET agent_status = $1 WHERE id = $2 AND NOT is_bot")
                .bind(status)
                .bind(uid)
                .execute(&mut *tx)
                .await?;
        if updated.rows_affected() != 1 {
            return Err(ApiError::NotFound("User not found".into()));
        }
    }
    if let Some(target) = body.target_ngr_cents {
        let updated = sqlx::query(
            "INSERT INTO agent_monthly_cycles \
                (agent_user_id, cycle_start, money_mode, target_ngr_cents) \
             SELECT $1, $2, $3, $4 \
             WHERE $2 > date_trunc('month', timezone('America/Sao_Paulo', clock_timestamp()))::date \
             ON CONFLICT (agent_user_id, cycle_start, money_mode) DO UPDATE \
             SET target_ngr_cents = EXCLUDED.target_ngr_cents, updated_at = NOW() \
             WHERE agent_monthly_cycles.status = 'open' \
               AND agent_monthly_cycles.cycle_start > date_trunc('month', timezone('America/Sao_Paulo', clock_timestamp()))::date",
        )
        .bind(uid)
        .bind(cycle_start)
        .bind(money_mode)
        .bind(target)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() != 1 {
            return Err(ApiError::Conflict(
                "O ciclo já começou ou foi fechado; escolha um mês futuro.".into(),
            ));
        }
    }
    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(&auth_user.user_id)
        .bind("AGENT_CONFIG")
        .bind(serde_json::json!({
            "target_user_id": user_id,
            "status": body.status,
            "cycle_start": cycle_start,
            "money_mode": money_mode,
            "target_ngr_cents": body.target_ngr_cents,
        }))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let agent_status: String = sqlx::query_scalar("SELECT agent_status FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&state.db)
        .await?;
    let target_ngr_cents: i64 = sqlx::query_scalar(
        "SELECT target_ngr_cents FROM agent_monthly_cycles \
         WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3",
    )
    .bind(uid)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(0);

    Ok(Json(AgentConfigResponse {
        user_id,
        agent_status,
        cycle_start,
        money_mode: money_mode.to_string(),
        target_ngr_cents,
    }))
}

/// POST /api/admin/agentes/:id/adjustments — dedução identificada do NGR.
pub async fn add_agent_adjustment(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(body): Json<AgentAdjustmentRequest>,
) -> Result<Json<AgentAdjustmentResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let uid = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;
    let cycle_start = validate_cycle_start(body.cycle_start)?;
    let money_mode = validate_money_mode(body.money_mode.trim())?;
    if body.amount_cents <= 0 {
        return Err(ApiError::BadRequest("amount_cents must be positive".into()));
    }
    let note = body.note.trim();
    if note.is_empty() || note.chars().count() > 240 {
        return Err(ApiError::BadRequest("note must be 1-240 characters".into()));
    }
    if !matches!(
        body.category.as_str(),
        "reward" | "refund" | "chargeback" | "tax" | "payment_cost" | "other"
    ) {
        return Err(ApiError::BadRequest("Invalid adjustment category".into()));
    }

    let mut tx = state.db.begin().await?;
    // A chave vem do formulário e deve ser reutilizada após falha de rede.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 42))")
        .bind(body.request_id.to_string())
        .execute(&mut *tx)
        .await?;
    let existing: Option<bool> = sqlx::query_scalar(
        "SELECT agent_user_id = $2 AND cycle_start = $3 AND money_mode = $4 \
          AND category = $5 AND amount_cents = $6 AND note = $7 \
          AND created_by IS NOT DISTINCT FROM $8 FROM agent_ngr_adjustments WHERE id = $1",
    )
    .bind(body.request_id)
    .bind(uid)
    .bind(cycle_start)
    .bind(money_mode)
    .bind(&body.category)
    .bind(body.amount_cents)
    .bind(note)
    .bind(uuid::Uuid::parse_str(&auth_user.user_id).ok())
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(same) = existing {
        if !same {
            return Err(ApiError::Conflict(
                "Chave de dedução já usada com outros dados.".into(),
            ));
        }
        return Ok(Json(AgentAdjustmentResponse {
            id: body.request_id.to_string(),
            amount_cents: body.amount_cents,
        }));
    }
    let reserved: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM estrutura_ledger WHERE id = $1) OR EXISTS(SELECT 1 FROM agent_monthly_cycles WHERE id = $1)")
        .bind(body.request_id).fetch_one(&mut *tx).await?;
    if reserved {
        return Err(ApiError::Conflict(
            "Chave reservada para ajuste automático.".into(),
        ));
    }
    let cycle_status = crate::estrutura::lock_cycle(&mut tx, uid, cycle_start, money_mode).await?;
    if cycle_status == "closed" {
        return Err(ApiError::Conflict("The cycle is already closed".into()));
    }
    let newer_closed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM agent_monthly_cycles WHERE agent_user_id = $1 \
          AND money_mode = $2 AND cycle_start > $3 AND status = 'closed')",
    )
    .bind(uid)
    .bind(money_mode)
    .bind(cycle_start)
    .fetch_one(&mut *tx)
    .await?;
    if newer_closed {
        return Err(ApiError::Conflict(
            "Um ciclo posterior já foi fechado; registre a dedução no ciclo atual.".into(),
        ));
    }
    let adjustment_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO agent_ngr_adjustments \
            (agent_user_id, cycle_start, money_mode, category, amount_cents, note, created_by, id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(uid)
    .bind(cycle_start)
    .bind(money_mode)
    .bind(&body.category)
    .bind(body.amount_cents)
    .bind(note)
    .bind(uuid::Uuid::parse_str(&auth_user.user_id).ok())
    .bind(body.request_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(&auth_user.user_id)
        .bind("AGENT_NGR_ADJUSTMENT")
        .bind(serde_json::json!({
            "target_user_id": user_id,
            "cycle_start": cycle_start,
            "money_mode": money_mode,
            "category": body.category,
            "amount_cents": body.amount_cents,
            "note": note,
        }))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(AgentAdjustmentResponse {
        id: adjustment_id.to_string(),
        amount_cents: body.amount_cents,
    }))
}

/// POST /api/admin/agentes/:id/close — fecha mês anterior e credita uma vez.
pub async fn close_agent_cycle(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(body): Json<AgentCloseRequest>,
) -> Result<Json<AgentCloseResponse>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let uid = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;
    let cycle_start = validate_cycle_start(body.cycle_start)?;
    let money_mode = validate_money_mode(body.money_mode.trim())?;
    let current_cycle: NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', timezone('America/Sao_Paulo', now()))::date",
    )
    .fetch_one(&state.db)
    .await?;
    if cycle_start >= current_cycle {
        return Err(ApiError::BadRequest(
            "Only a completed month can be closed".into(),
        ));
    }
    let closing_day = cycle_start
        .checked_add_months(chrono::Months::new(1))
        .and_then(|d| d.checked_add_signed(chrono::Duration::days(24)))
        .ok_or_else(|| ApiError::BadRequest("Invalid cycle".into()))?;
    let today: NaiveDate =
        sqlx::query_scalar("SELECT timezone('America/Sao_Paulo', clock_timestamp())::date")
            .fetch_one(&state.db)
            .await?;
    if today < closing_day || !body.reconciled {
        return Err(ApiError::BadRequest(
            "Fechamento exige conciliação e só é permitido a partir do dia 25 do mês seguinte."
                .into(),
        ));
    }

    let mut tx = state.db.begin().await?;
    crate::estrutura::lock_cycle(&mut tx, uid, cycle_start, money_mode).await?;

    let cycle: (i64, String, i64, i64, i64, Option<i16>, i64) = sqlx::query_as(
        "SELECT target_ngr_cents, status, gross_revenue_cents, deductions_cents, \
                ngr_cents, commission_percent, commission_cents \
         FROM agent_monthly_cycles \
         WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3 FOR UPDATE",
    )
    .bind(uid)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_one(&mut *tx)
    .await?;
    if cycle.1 == "closed" {
        tx.commit().await?;
        return Ok(Json(AgentCloseResponse {
            user_id,
            cycle_start,
            money_mode: money_mode.to_string(),
            target_ngr_cents: cycle.0,
            gross_revenue_cents: cycle.2,
            deductions_cents: cycle.3,
            ngr_cents: cycle.4,
            commission_percent: cycle.5.unwrap_or(crate::estrutura::BASE_PERCENT as i16),
            commission_cents: cycle.6,
            already_closed: true,
        }));
    }

    let gross_revenue_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(source_rake_cents), 0)::BIGINT FROM estrutura_ledger \
         WHERE beneficiary_user_id = $1 AND cycle_start = $2 \
           AND money_mode = $3 AND program_version = 2",
    )
    .bind(uid)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_one(&mut *tx)
    .await?;
    let deductions_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0)::BIGINT FROM agent_ngr_adjustments \
         WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3",
    )
    .bind(uid)
    .bind(cycle_start)
    .bind(money_mode)
    .fetch_one(&mut *tx)
    .await?;
    if (gross_revenue_cents, deductions_cents, cycle.0)
        != (
            body.expected_gross_revenue_cents,
            body.expected_deductions_cents,
            body.expected_target_ngr_cents,
        )
    {
        return Err(ApiError::Conflict(
            "A apuração mudou. Consulte os valores e confirme novamente.".into(),
        ));
    }
    let older_open: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM agent_monthly_cycles WHERE agent_user_id = $1 AND money_mode = $2 AND cycle_start < $3 AND status = 'open')")
        .bind(uid).bind(money_mode).bind(cycle_start).fetch_one(&mut *tx).await?;
    if older_open {
        return Err(ApiError::Conflict(
            "Feche primeiro os ciclos anteriores deste agente e modo.".into(),
        ));
    }
    let calculation =
        crate::estrutura::calculate_cycle(gross_revenue_cents, deductions_cents, cycle.0);
    let ngr_cents = calculation.ngr;
    let commission_percent = calculation.percent as i16;
    let commission_cents = calculation.commission;
    let deficit = calculation.carry_forward;
    if deficit > 0 {
        let next = cycle_start
            .checked_add_months(chrono::Months::new(1))
            .ok_or_else(|| ApiError::BadRequest("Invalid cycle".into()))?;
        if crate::estrutura::lock_cycle(&mut tx, uid, next, money_mode).await? != "open" {
            return Err(ApiError::Conflict(
                "O ciclo seguinte já foi fechado; concilie o déficit antes de continuar.".into(),
            ));
        }
        sqlx::query("INSERT INTO agent_ngr_adjustments (id, agent_user_id, cycle_start, money_mode, category, amount_cents, note, created_by) SELECT id, agent_user_id, $4, money_mode, 'other', $5, $6, $7 FROM agent_monthly_cycles WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3 ON CONFLICT (id) DO NOTHING")
            .bind(uid).bind(cycle_start).bind(money_mode).bind(next).bind(deficit)
            .bind(format!("Déficit de NGR transportado de {cycle_start}"))
            .bind(uuid::Uuid::parse_str(&auth_user.user_id).ok()).execute(&mut *tx).await?;
    }

    sqlx::query(
        "UPDATE agent_monthly_cycles SET status = 'closed', gross_revenue_cents = $4, \
            deductions_cents = $5, ngr_cents = $6, commission_percent = $7, \
            commission_cents = $8, closed_at = NOW(), closed_by = $9, updated_at = NOW() \
         WHERE agent_user_id = $1 AND cycle_start = $2 AND money_mode = $3",
    )
    .bind(uid)
    .bind(cycle_start)
    .bind(money_mode)
    .bind(gross_revenue_cents)
    .bind(deductions_cents)
    .bind(ngr_cents)
    .bind(commission_percent)
    .bind(commission_cents)
    .bind(uuid::Uuid::parse_str(&auth_user.user_id).ok())
    .execute(&mut *tx)
    .await?;
    if money_mode == "play" {
        sqlx::query("UPDATE users SET estrutura_points = estrutura_points + $2 WHERE id = $1")
            .bind(uid)
            .bind(commission_cents)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query(
            "UPDATE users SET agent_commission_balance_cents = \
                agent_commission_balance_cents + $2 WHERE id = $1",
        )
        .bind(uid)
        .bind(commission_cents)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(&auth_user.user_id)
        .bind("AGENT_CYCLE_CLOSE")
        .bind(serde_json::json!({
            "target_user_id": user_id,
            "cycle_start": cycle_start,
            "money_mode": money_mode,
            "target_ngr_cents": cycle.0,
            "gross_revenue_cents": gross_revenue_cents,
            "deductions_cents": deductions_cents,
            "ngr_cents": ngr_cents,
            "commission_percent": commission_percent,
            "commission_cents": commission_cents,
        }))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(AgentCloseResponse {
        user_id,
        cycle_start,
        money_mode: money_mode.to_string(),
        target_ngr_cents: cycle.0,
        gross_revenue_cents,
        deductions_cents,
        ngr_cents,
        commission_percent,
        commission_cents,
        already_closed: false,
    }))
}

/// GET /api/admin/estrutura/backfill — prévia do vínculo das contas antigas.
pub async fn preview_backfill(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<EstruturaBackfillPreview>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;

    let root: Option<(uuid::Uuid, String, Option<uuid::Uuid>)> = sqlx::query_as(
        "SELECT id, username, sponsored_by FROM users \
         WHERE NOT is_bot ORDER BY created_at ASC, id ASC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;
    let (root_id, root_username, root_sponsor) =
        root.ok_or_else(|| ApiError::NotFound("No users available for backfill".into()))?;

    let unlinked_accounts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users \
         WHERE sponsored_by IS NULL AND id <> $1 AND NOT is_bot",
    )
    .bind(root_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(EstruturaBackfillPreview {
        root_user_id: root_id.to_string(),
        root_username,
        unlinked_accounts,
        root_has_sponsor: root_sponsor.is_some(),
    }))
}

/// POST /api/admin/estrutura/backfill — vincula contas legadas sem patrocinador
/// ao primeiro usuário. O root esperado vem da prévia para evitar executar
/// contra uma base diferente da que o administrador revisou.
pub async fn execute_backfill(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Json(body): Json<EstruturaBackfillRequest>,
) -> Result<Json<EstruturaBackfillResult>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    if !body.confirm {
        return Err(ApiError::BadRequest(
            "Explicit backfill confirmation is required".into(),
        ));
    }
    let expected_root = uuid::Uuid::parse_str(&body.root_user_id)
        .map_err(|_| ApiError::BadRequest("Invalid root user id".into()))?;

    let mut tx = state.db.begin().await?;
    let root: Option<(uuid::Uuid, String, Option<uuid::Uuid>)> = sqlx::query_as(
        "SELECT id, username, sponsored_by FROM users \
         WHERE NOT is_bot ORDER BY created_at ASC, id ASC LIMIT 1 FOR UPDATE",
    )
    .fetch_optional(&mut *tx)
    .await?;
    let (root_id, root_username, root_sponsor) =
        root.ok_or_else(|| ApiError::NotFound("No users available for backfill".into()))?;

    if root_id != expected_root {
        return Err(ApiError::Conflict(
            "Network root changed after preview; reload before continuing".into(),
        ));
    }
    if root_sponsor.is_some() {
        return Err(ApiError::Conflict(
            "The first user already has a sponsor; review the network before backfill".into(),
        ));
    }

    let updated = sqlx::query(
        "UPDATE users SET sponsored_by = $1 \
         WHERE sponsored_by IS NULL AND id <> $1 AND NOT is_bot",
    )
    .bind(root_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(&auth_user.user_id)
        .bind("ESTRUTURA_BACKFILL")
        .bind(serde_json::json!({
            "root_user_id": root_id,
            "root_username": root_username,
            "updated_accounts": updated,
        }))
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(EstruturaBackfillResult {
        root_user_id: root_id.to_string(),
        root_username,
        updated_accounts: updated,
    }))
}
