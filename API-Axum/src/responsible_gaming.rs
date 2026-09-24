use axum::extract::State;
use axum::Json;
use chrono::{Datelike, Months, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

const LIMIT_CHANGE_DELAY_HOURS: i64 = 24;
const MAX_MONEY_LIMIT_CENTS: i64 = 10_000_000_000;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Limits {
    pub deposit_limit_daily_cents: Option<i64>,
    pub deposit_limit_weekly_cents: Option<i64>,
    pub deposit_limit_monthly_cents: Option<i64>,
    pub loss_limit_daily_cents: Option<i64>,
    pub loss_limit_weekly_cents: Option<i64>,
    pub loss_limit_monthly_cents: Option<i64>,
    pub play_time_limit_daily_minutes: Option<i32>,
}

type SettingsRow = (
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i32>,
    Option<serde_json::Value>,
    Option<chrono::DateTime<Utc>>,
    Option<chrono::DateTime<Utc>>,
    bool,
    Option<chrono::DateTime<Utc>>,
);

#[derive(Debug, Serialize)]
pub struct ProtectionStatus {
    #[serde(flatten)]
    pub limits: Limits,
    pub pending_limits: Option<Limits>,
    pub pending_limits_effective_at: Option<String>,
    pub self_excluded_until: Option<String>,
    pub self_excluded_permanently: bool,
    pub self_exclusion_started_at: Option<String>,
    pub kyc_status: String,
    pub date_of_birth: Option<String>,
    pub over_18_declared: bool,
    pub deposited_today_cents: i64,
    pub deposited_week_cents: i64,
    pub deposited_month_cents: i64,
    pub loss_today_cents: i64,
    pub loss_week_cents: i64,
    pub loss_month_cents: i64,
    pub real_play_seconds_today: i32,
}

#[derive(Debug, Deserialize)]
pub struct SelfExclusionBody {
    pub duration: String,
    #[serde(default)]
    pub confirmation: String,
}

#[derive(Debug, Serialize)]
pub struct PlayHeartbeatResponse {
    pub real_play_seconds_today: i32,
    pub play_time_limit_daily_minutes: Option<i32>,
    pub limit_reached: bool,
}

fn limits_from_row(row: &SettingsRow) -> Limits {
    Limits {
        deposit_limit_daily_cents: row.0,
        deposit_limit_weekly_cents: row.1,
        deposit_limit_monthly_cents: row.2,
        loss_limit_daily_cents: row.3,
        loss_limit_weekly_cents: row.4,
        loss_limit_monthly_cents: row.5,
        play_time_limit_daily_minutes: row.6,
    }
}

fn validate_limits(limits: &Limits) -> Result<(), ApiError> {
    for (name, value) in [
        (
            "deposit_limit_daily_cents",
            limits.deposit_limit_daily_cents,
        ),
        (
            "deposit_limit_weekly_cents",
            limits.deposit_limit_weekly_cents,
        ),
        (
            "deposit_limit_monthly_cents",
            limits.deposit_limit_monthly_cents,
        ),
        ("loss_limit_daily_cents", limits.loss_limit_daily_cents),
        ("loss_limit_weekly_cents", limits.loss_limit_weekly_cents),
        ("loss_limit_monthly_cents", limits.loss_limit_monthly_cents),
    ] {
        if value.is_some_and(|amount| !(0..=MAX_MONEY_LIMIT_CENTS).contains(&amount)) {
            return Err(ApiError::BadRequest(format!(
                "{name} must be between 0 and {MAX_MONEY_LIMIT_CENTS} cents"
            )));
        }
    }
    if limits
        .play_time_limit_daily_minutes
        .is_some_and(|minutes| !(15..=1440).contains(&minutes))
    {
        return Err(ApiError::BadRequest(
            "play_time_limit_daily_minutes must be between 15 and 1440".into(),
        ));
    }
    Ok(())
}

fn is_relaxation<T: Ord + Copy>(current: Option<T>, proposed: Option<T>) -> bool {
    match (current, proposed) {
        (Some(_), None) => true,
        (Some(old), Some(new)) => new > old,
        _ => false,
    }
}

fn has_relaxation(current: &Limits, proposed: &Limits) -> bool {
    is_relaxation(
        current.deposit_limit_daily_cents,
        proposed.deposit_limit_daily_cents,
    ) || is_relaxation(
        current.deposit_limit_weekly_cents,
        proposed.deposit_limit_weekly_cents,
    ) || is_relaxation(
        current.deposit_limit_monthly_cents,
        proposed.deposit_limit_monthly_cents,
    ) || is_relaxation(
        current.loss_limit_daily_cents,
        proposed.loss_limit_daily_cents,
    ) || is_relaxation(
        current.loss_limit_weekly_cents,
        proposed.loss_limit_weekly_cents,
    ) || is_relaxation(
        current.loss_limit_monthly_cents,
        proposed.loss_limit_monthly_cents,
    ) || is_relaxation(
        current.play_time_limit_daily_minutes,
        proposed.play_time_limit_daily_minutes,
    )
}

fn stricter<T: Ord + Copy>(current: Option<T>, proposed: Option<T>) -> Option<T> {
    match (current, proposed) {
        (None, value) => value,
        (Some(old), Some(new)) if new <= old => Some(new),
        (Some(old), _) => Some(old),
    }
}

fn immediate_stricter_limits(current: &Limits, proposed: &Limits) -> Limits {
    Limits {
        deposit_limit_daily_cents: stricter(
            current.deposit_limit_daily_cents,
            proposed.deposit_limit_daily_cents,
        ),
        deposit_limit_weekly_cents: stricter(
            current.deposit_limit_weekly_cents,
            proposed.deposit_limit_weekly_cents,
        ),
        deposit_limit_monthly_cents: stricter(
            current.deposit_limit_monthly_cents,
            proposed.deposit_limit_monthly_cents,
        ),
        loss_limit_daily_cents: stricter(
            current.loss_limit_daily_cents,
            proposed.loss_limit_daily_cents,
        ),
        loss_limit_weekly_cents: stricter(
            current.loss_limit_weekly_cents,
            proposed.loss_limit_weekly_cents,
        ),
        loss_limit_monthly_cents: stricter(
            current.loss_limit_monthly_cents,
            proposed.loss_limit_monthly_cents,
        ),
        play_time_limit_daily_minutes: stricter(
            current.play_time_limit_daily_minutes,
            proposed.play_time_limit_daily_minutes,
        ),
    }
}

async fn ensure_settings(pool: &PgPool, user_id: &str) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO responsible_gaming_settings (user_id) VALUES ($1::uuid) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn load_settings(pool: &PgPool, user_id: &str) -> Result<SettingsRow, ApiError> {
    ensure_settings(pool, user_id).await?;
    let mut row: SettingsRow = sqlx::query_as(
        "SELECT deposit_limit_daily_cents, deposit_limit_weekly_cents, deposit_limit_monthly_cents, \
                loss_limit_daily_cents, loss_limit_weekly_cents, loss_limit_monthly_cents, \
                play_time_limit_daily_minutes, pending_limits, pending_limits_effective_at, \
                self_excluded_until, self_excluded_permanently, self_exclusion_started_at \
         FROM responsible_gaming_settings WHERE user_id = $1::uuid",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if row.8.is_some_and(|effective| effective <= Utc::now()) {
        if let Some(value) = row.7.clone() {
            let pending: Limits = serde_json::from_value(value)
                .map_err(|_| ApiError::Internal("Invalid pending limits".into()))?;
            sqlx::query(
                "UPDATE responsible_gaming_settings SET \
                    deposit_limit_daily_cents=$2, deposit_limit_weekly_cents=$3, deposit_limit_monthly_cents=$4, \
                    loss_limit_daily_cents=$5, loss_limit_weekly_cents=$6, loss_limit_monthly_cents=$7, \
                    play_time_limit_daily_minutes=$8, pending_limits=NULL, pending_limits_effective_at=NULL, updated_at=NOW() \
                 WHERE user_id=$1::uuid",
            )
            .bind(user_id)
            .bind(pending.deposit_limit_daily_cents)
            .bind(pending.deposit_limit_weekly_cents)
            .bind(pending.deposit_limit_monthly_cents)
            .bind(pending.loss_limit_daily_cents)
            .bind(pending.loss_limit_weekly_cents)
            .bind(pending.loss_limit_monthly_cents)
            .bind(pending.play_time_limit_daily_minutes)
            .execute(pool)
            .await?;
            row.0 = pending.deposit_limit_daily_cents;
            row.1 = pending.deposit_limit_weekly_cents;
            row.2 = pending.deposit_limit_monthly_cents;
            row.3 = pending.loss_limit_daily_cents;
            row.4 = pending.loss_limit_weekly_cents;
            row.5 = pending.loss_limit_monthly_cents;
            row.6 = pending.play_time_limit_daily_minutes;
            row.7 = None;
            row.8 = None;
        }
    }
    Ok(row)
}

async fn deposit_totals(pool: &PgPool, user_id: &str) -> Result<(i64, i64, i64), ApiError> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "WITH periods AS (SELECT \
             date_trunc('day', timezone('America/Sao_Paulo', now())) AT TIME ZONE 'America/Sao_Paulo' AS day_start, \
             date_trunc('week', timezone('America/Sao_Paulo', now())) AT TIME ZONE 'America/Sao_Paulo' AS week_start, \
             date_trunc('month', timezone('America/Sao_Paulo', now())) AT TIME ZONE 'America/Sao_Paulo' AS month_start), \
         deposits AS ( \
             SELECT amount, created_at FROM wallet_transactions \
              WHERE user_id=$1::uuid AND transaction_type='DEPOSIT' AND status IN ('PENDING','COMPLETED') \
             UNION ALL \
             SELECT amount_cents, created_at FROM deposit_requests \
              WHERE user_id=$1::uuid AND status IN ('pending','approved') \
         ) \
         SELECT COALESCE(SUM(amount) FILTER (WHERE created_at >= day_start),0)::bigint, \
                COALESCE(SUM(amount) FILTER (WHERE created_at >= week_start),0)::bigint, \
                COALESCE(SUM(amount) FILTER (WHERE created_at >= month_start),0)::bigint \
         FROM deposits CROSS JOIN periods",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

async fn loss_totals(pool: &PgPool, user_id: &str) -> Result<(i64, i64, i64), ApiError> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "WITH periods AS (SELECT \
             date_trunc('day', timezone('America/Sao_Paulo', now())) AT TIME ZONE 'America/Sao_Paulo' AS day_start, \
             date_trunc('week', timezone('America/Sao_Paulo', now())) AT TIME ZONE 'America/Sao_Paulo' AS week_start, \
             date_trunc('month', timezone('America/Sao_Paulo', now())) AT TIME ZONE 'America/Sao_Paulo' AS month_start), \
         results AS ( \
             SELECT GREATEST(s.buy_in-s.chips,0)::bigint AS loss, s.cashed_out_at AS happened_at \
               FROM cash_game_seats s JOIN tables t ON t.id=s.table_id \
              WHERE s.user_id=$1::uuid AND s.status='CASHED_OUT' AND t.money_mode='real' \
             UNION ALL \
             SELECT GREATEST(((t.buy_in + (t.buy_in*15/100)) * (1+tp.rebuys))-COALESCE(tp.prize,0),0)::bigint, \
                    to_timestamp(tp.registered_at) \
               FROM tournament_players tp JOIN tournaments t ON t.id=tp.tournament_id \
              WHERE tp.player_id=$1 AND t.money_mode='real' \
         ) \
         SELECT COALESCE(SUM(loss) FILTER (WHERE happened_at >= day_start),0)::bigint, \
                COALESCE(SUM(loss) FILTER (WHERE happened_at >= week_start),0)::bigint, \
                COALESCE(SUM(loss) FILTER (WHERE happened_at >= month_start),0)::bigint \
         FROM results CROSS JOIN periods",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

async fn play_seconds_today(pool: &PgPool, user_id: &str) -> Result<i32, ApiError> {
    Ok(sqlx::query_scalar(
        "SELECT COALESCE(real_play_seconds,0) FROM responsible_gaming_daily_activity \
         WHERE user_id=$1::uuid AND activity_date=(timezone('America/Sao_Paulo',now()))::date",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or(0))
}

pub async fn get_settings(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<ProtectionStatus>, ApiError> {
    let row = load_settings(&state.db, &auth.user_id).await?;
    let limits = limits_from_row(&row);
    let pending_limits = row
        .7
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| ApiError::Internal("Invalid pending limits".into()))?;
    let (today, week, month) = deposit_totals(&state.db, &auth.user_id).await?;
    let (loss_today, loss_week, loss_month) = loss_totals(&state.db, &auth.user_id).await?;
    let (kyc_status, date_of_birth, over_18): (String, Option<NaiveDate>, bool) = sqlx::query_as(
        "SELECT kyc_status, date_of_birth, over_18_declared_at IS NOT NULL FROM users WHERE id=$1::uuid",
    )
    .bind(&auth.user_id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(ProtectionStatus {
        limits,
        pending_limits,
        pending_limits_effective_at: row.8.map(|v| v.to_rfc3339()),
        self_excluded_until: row.9.map(|v| v.to_rfc3339()),
        self_excluded_permanently: row.10,
        self_exclusion_started_at: row.11.map(|v| v.to_rfc3339()),
        kyc_status,
        date_of_birth: date_of_birth.map(|v| v.to_string()),
        over_18_declared: over_18,
        deposited_today_cents: today,
        deposited_week_cents: week,
        deposited_month_cents: month,
        loss_today_cents: loss_today,
        loss_week_cents: loss_week,
        loss_month_cents: loss_month,
        real_play_seconds_today: play_seconds_today(&state.db, &auth.user_id).await?,
    }))
}

pub async fn update_limits(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Json(proposed): Json<Limits>,
) -> Result<Json<serde_json::Value>, ApiError> {
    validate_limits(&proposed)?;
    let row = load_settings(&state.db, &auth.user_id).await?;
    let current = limits_from_row(&row);
    let delayed = has_relaxation(&current, &proposed);
    if delayed {
        let immediate = immediate_stricter_limits(&current, &proposed);
        sqlx::query(
            "UPDATE responsible_gaming_settings SET deposit_limit_daily_cents=$2, \
             deposit_limit_weekly_cents=$3,deposit_limit_monthly_cents=$4,loss_limit_daily_cents=$5, \
             loss_limit_weekly_cents=$6,loss_limit_monthly_cents=$7,play_time_limit_daily_minutes=$8, \
             pending_limits=$9,pending_limits_effective_at=NOW()+INTERVAL '24 hours',updated_at=NOW() WHERE user_id=$1::uuid",
        )
        .bind(&auth.user_id)
        .bind(immediate.deposit_limit_daily_cents)
        .bind(immediate.deposit_limit_weekly_cents)
        .bind(immediate.deposit_limit_monthly_cents)
        .bind(immediate.loss_limit_daily_cents)
        .bind(immediate.loss_limit_weekly_cents)
        .bind(immediate.loss_limit_monthly_cents)
        .bind(immediate.play_time_limit_daily_minutes)
        .bind(json!(proposed))
        .execute(&state.db)
        .await?;
    } else {
        sqlx::query(
            "UPDATE responsible_gaming_settings SET deposit_limit_daily_cents=$2, \
             deposit_limit_weekly_cents=$3, deposit_limit_monthly_cents=$4, loss_limit_daily_cents=$5, \
             loss_limit_weekly_cents=$6, loss_limit_monthly_cents=$7, play_time_limit_daily_minutes=$8, \
             pending_limits=NULL, pending_limits_effective_at=NULL, updated_at=NOW() WHERE user_id=$1::uuid",
        )
        .bind(&auth.user_id)
        .bind(proposed.deposit_limit_daily_cents)
        .bind(proposed.deposit_limit_weekly_cents)
        .bind(proposed.deposit_limit_monthly_cents)
        .bind(proposed.loss_limit_daily_cents)
        .bind(proposed.loss_limit_weekly_cents)
        .bind(proposed.loss_limit_monthly_cents)
        .bind(proposed.play_time_limit_daily_minutes)
        .execute(&state.db)
        .await?;
    }
    sqlx::query("INSERT INTO audit_logs(user_id,action,metadata) VALUES($1,'RESPONSIBLE_LIMITS_UPDATED',$2)")
        .bind(&auth.user_id)
        .bind(json!({"delayed": delayed, "delay_hours": if delayed { LIMIT_CHANGE_DELAY_HOURS } else { 0 }}))
        .execute(&state.db)
        .await?;
    Ok(Json(json!({
        "ok": true,
        "delayed": delayed,
        "message": if delayed { "Aumento ou remoção de limite será aplicado em 24 horas." } else { "Limites reduzidos e aplicados imediatamente." }
    })))
}

pub async fn self_exclude(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Json(body): Json<SelfExclusionBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.confirmation.trim().to_uppercase() != "AUTOEXCLUIR" {
        return Err(ApiError::BadRequest(
            "Digite AUTOEXCLUIR para confirmar".into(),
        ));
    }
    ensure_settings(&state.db, &auth.user_id).await?;
    let (interval, permanent) = match body.duration.as_str() {
        "24h" => (Some("24 hours"), false),
        "7d" => (Some("7 days"), false),
        "30d" => (Some("30 days"), false),
        "180d" => (Some("180 days"), false),
        "permanent" => (None, true),
        _ => {
            return Err(ApiError::BadRequest(
                "Invalid self-exclusion duration".into(),
            ))
        }
    };
    let until = interval.map(|value| match value {
        "24 hours" => Utc::now() + chrono::Duration::hours(24),
        "7 days" => Utc::now() + chrono::Duration::days(7),
        "30 days" => Utc::now() + chrono::Duration::days(30),
        _ => Utc::now() + chrono::Duration::days(180),
    });
    sqlx::query(
        "UPDATE responsible_gaming_settings SET self_excluded_until=$2, self_excluded_permanently=$3, \
         self_exclusion_started_at=NOW(), updated_at=NOW() WHERE user_id=$1::uuid",
    )
    .bind(&auth.user_id)
    .bind(until)
    .bind(permanent)
    .execute(&state.db)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs(user_id,action,metadata) VALUES($1,'SELF_EXCLUSION_STARTED',$2)",
    )
    .bind(&auth.user_id)
    .bind(json!({"duration": body.duration, "until": until, "permanent": permanent}))
    .execute(&state.db)
    .await?;
    Ok(Json(
        json!({"ok": true, "self_excluded_until": until, "permanent": permanent}),
    ))
}

pub async fn play_heartbeat(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<PlayHeartbeatResponse>, ApiError> {
    ensure_real_money_action_allowed(&state.db, &auth.user_id).await?;
    let settings = load_settings(&state.db, &auth.user_id).await?;
    let seconds: i32 = sqlx::query_scalar(
        "INSERT INTO responsible_gaming_daily_activity(user_id,activity_date,real_play_seconds,last_heartbeat_at) \
         VALUES($1::uuid,(timezone('America/Sao_Paulo',now()))::date,0,NOW()) \
         ON CONFLICT(user_id,activity_date) DO UPDATE SET \
           real_play_seconds=responsible_gaming_daily_activity.real_play_seconds + \
             LEAST(60, GREATEST(0, EXTRACT(EPOCH FROM (NOW()-responsible_gaming_daily_activity.last_heartbeat_at))::int)), \
           last_heartbeat_at=NOW() RETURNING real_play_seconds",
    )
    .bind(&auth.user_id)
    .fetch_one(&state.db)
    .await?;
    let limit = settings.6;
    Ok(Json(PlayHeartbeatResponse {
        real_play_seconds_today: seconds,
        play_time_limit_daily_minutes: limit,
        limit_reached: limit.is_some_and(|minutes| seconds >= minutes * 60),
    }))
}

pub async fn ensure_real_money_action_allowed(
    pool: &PgPool,
    user_id: &str,
) -> Result<(), ApiError> {
    let row: (String, Option<chrono::DateTime<Utc>>, bool, Option<i32>) = sqlx::query_as(
        "SELECT u.kyc_status,s.self_excluded_until,COALESCE(s.self_excluded_permanently,FALSE),s.play_time_limit_daily_minutes \
         FROM users u LEFT JOIN responsible_gaming_settings s ON s.user_id=u.id WHERE u.id=$1::uuid",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if row.0 != "verified" {
        return Err(ApiError::Forbidden("Verificação KYC necessária".into()));
    }
    if row.2 || row.1.is_some_and(|until| until > Utc::now()) {
        return Err(ApiError::Forbidden("Autoexclusão ativa".into()));
    }
    if let Some(minutes) = row.3 {
        if play_seconds_today(pool, user_id).await? >= minutes * 60 {
            return Err(ApiError::Forbidden(
                "Limite diário de tempo atingido".into(),
            ));
        }
    }
    Ok(())
}

pub async fn ensure_deposit_allowed(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &str,
    amount_cents: i64,
) -> Result<(), ApiError> {
    let user: (String,) =
        sqlx::query_as("SELECT kyc_status FROM users WHERE id=$1::uuid FOR UPDATE")
            .bind(user_id)
            .fetch_one(&mut **tx)
            .await?;
    if user.0 != "verified" {
        return Err(ApiError::Forbidden(
            "Conclua a verificação KYC antes de depositar em Jogo Real.".into(),
        ));
    }
    let settings: Option<SettingsRow> = sqlx::query_as(
        "SELECT deposit_limit_daily_cents, deposit_limit_weekly_cents, deposit_limit_monthly_cents, \
         loss_limit_daily_cents, loss_limit_weekly_cents, loss_limit_monthly_cents, play_time_limit_daily_minutes, \
         pending_limits, pending_limits_effective_at, self_excluded_until, self_excluded_permanently, self_exclusion_started_at \
         FROM responsible_gaming_settings WHERE user_id=$1::uuid FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(settings) = settings else {
        return Ok(());
    };
    if settings.10 || settings.9.is_some_and(|until| until > Utc::now()) {
        return Err(ApiError::Forbidden(
            "Depósitos bloqueados durante a autoexclusão.".into(),
        ));
    }
    let effective_limits = if settings.8.is_some_and(|at| at <= Utc::now()) {
        settings
            .7
            .clone()
            .map(serde_json::from_value::<Limits>)
            .transpose()
            .map_err(|_| ApiError::Internal("Invalid pending limits".into()))?
            .unwrap_or_else(|| limits_from_row(&settings))
    } else {
        limits_from_row(&settings)
    };
    let pool = tx.as_mut();
    let totals: (i64, i64, i64) = sqlx::query_as(
        "WITH p AS (SELECT date_trunc('day',timezone('America/Sao_Paulo',now())) AT TIME ZONE 'America/Sao_Paulo' d, \
         date_trunc('week',timezone('America/Sao_Paulo',now())) AT TIME ZONE 'America/Sao_Paulo' w, \
         date_trunc('month',timezone('America/Sao_Paulo',now())) AT TIME ZONE 'America/Sao_Paulo' m), x AS ( \
         SELECT amount,created_at FROM wallet_transactions WHERE user_id=$1::uuid AND transaction_type='DEPOSIT' AND status IN ('PENDING','COMPLETED') \
         UNION ALL SELECT amount_cents,created_at FROM deposit_requests WHERE user_id=$1::uuid AND status IN ('pending','approved')) \
         SELECT COALESCE(SUM(amount) FILTER(WHERE created_at>=d),0)::bigint, COALESCE(SUM(amount) FILTER(WHERE created_at>=w),0)::bigint, \
         COALESCE(SUM(amount) FILTER(WHERE created_at>=m),0)::bigint FROM x CROSS JOIN p",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    for (limit, spent, label) in [
        (
            effective_limits.deposit_limit_daily_cents,
            totals.0,
            "diário",
        ),
        (
            effective_limits.deposit_limit_weekly_cents,
            totals.1,
            "semanal",
        ),
        (
            effective_limits.deposit_limit_monthly_cents,
            totals.2,
            "mensal",
        ),
    ] {
        if limit.is_some_and(|max| spent.saturating_add(amount_cents) > max) {
            return Err(ApiError::Forbidden(format!(
                "Este depósito ultrapassa seu limite {label}."
            )));
        }
    }
    Ok(())
}

pub async fn ensure_real_money_allowed(pool: &PgPool, user_id: &str) -> Result<(), ApiError> {
    let kyc: String = sqlx::query_scalar("SELECT kyc_status FROM users WHERE id=$1::uuid")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if kyc != "verified" {
        return Err(ApiError::Forbidden(
            "Conclua a verificação KYC antes de jogar com dinheiro real.".into(),
        ));
    }
    let settings = load_settings(pool, user_id).await?;
    if settings.10 || settings.9.is_some_and(|until| until > Utc::now()) {
        return Err(ApiError::Forbidden(
            "Jogo Real bloqueado durante a autoexclusão.".into(),
        ));
    }
    let (day, week, month) = loss_totals(pool, user_id).await?;
    for (limit, loss, label) in [
        (settings.3, day, "diário"),
        (settings.4, week, "semanal"),
        (settings.5, month, "mensal"),
    ] {
        if limit.is_some_and(|max| loss >= max) {
            return Err(ApiError::Forbidden(format!(
                "Seu limite de perda {label} foi atingido."
            )));
        }
    }
    if let Some(minutes) = settings.6 {
        if play_seconds_today(pool, user_id).await? >= minutes * 60 {
            return Err(ApiError::Forbidden(
                "Seu limite diário de tempo de jogo foi atingido.".into(),
            ));
        }
    }
    Ok(())
}

pub fn validate_adult(date: NaiveDate) -> Result<(), ApiError> {
    let today = Utc::now().date_naive();
    let cutoff = today
        .checked_sub_months(Months::new(18 * 12))
        .ok_or_else(|| ApiError::Internal("Age validation failed".into()))?;
    if date > cutoff || date.year() < 1900 {
        return Err(ApiError::Forbidden(
            "É necessário ter 18 anos ou mais.".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_only_limit_relaxations() {
        assert!(is_relaxation(Some(100_i64), Some(101)));
        assert!(is_relaxation(Some(100_i64), None));
        assert!(!is_relaxation(Some(100_i64), Some(99)));
        assert!(!is_relaxation(None, Some(100_i64)));
    }

    #[test]
    fn rejects_underage_birth_date() {
        let today = Utc::now().date_naive();
        assert!(validate_adult(today).is_err());
        assert!(validate_adult(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()).is_ok());
    }
}
