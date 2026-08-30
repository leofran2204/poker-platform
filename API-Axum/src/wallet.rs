//! Multi-wallet: Play Money cash / Play Money MTT / Real.

use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, PgPool};

use crate::error::ApiError;

pub const PM_CASH_DAILY_CENTS: i64 = 100_000; // R$ 1.000
pub const PM_MTT_DAILY_CENTS: i64 = 1_500_000; // R$ 15.000

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WalletMode {
    Play,
    Real,
}

impl WalletMode {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
            Some("real") => Self::Real,
            _ => Self::Play,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Play => "play",
            Self::Real => "real",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletKind {
    PmCash,
    PmMtt,
    Real,
}

impl WalletKind {
    pub fn column(self) -> &'static str {
        match self {
            Self::PmCash => "balance_pm_cash",
            Self::PmMtt => "balance_pm_mtt",
            Self::Real => "balance_real",
        }
    }

    pub fn seat_label(self) -> &'static str {
        match self {
            Self::PmCash => "pm_cash",
            Self::Real => "real",
            Self::PmMtt => "pm_mtt",
        }
    }

    pub fn from_seat(raw: &str) -> Self {
        match raw {
            "real" => Self::Real,
            _ => Self::PmCash,
        }
    }
}

pub fn today_sao_paulo() -> chrono::NaiveDate {
    // UTC−3 (no DST in BR since 2019)
    let offset = FixedOffset::west_opt(3 * 3600).expect("offset");
    Utc::now().with_timezone(&offset).date_naive()
}

/// Reset PM wallets to full daily grant if the SP calendar day rolled.
pub async fn ensure_pm_daily_reset<'e, E>(executor: E, user_id: &str) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let today = today_sao_paulo();
    let result = sqlx::query(
        r#"
        UPDATE users SET
            balance_pm_cash = GREATEST(
                0,
                $2 - COALESCE((
                    SELECT SUM(seats.chips)
                    FROM cash_game_seats AS seats
                    WHERE seats.user_id = users.id
                      AND seats.status = 'ACTIVE'
                      AND seats.wallet_kind = 'pm_cash'
                ), 0)
            ),
            balance_pm_mtt = $3,
            balance = GREATEST(
                0,
                $2 - COALESCE((
                    SELECT SUM(seats.chips)
                    FROM cash_game_seats AS seats
                    WHERE seats.user_id = users.id
                      AND seats.status = 'ACTIVE'
                      AND seats.wallet_kind = 'pm_cash'
                ), 0)
            ),
            last_pm_reset_date = $4,
            pm_cash_rebuy_used_on = CASE
                WHEN pm_cash_rebuy_used_on IS DISTINCT FROM $4 THEN NULL
                ELSE pm_cash_rebuy_used_on
            END,
            pm_mtt_rebuy_used_on = CASE
                WHEN pm_mtt_rebuy_used_on IS DISTINCT FROM $4 THEN NULL
                ELSE pm_mtt_rebuy_used_on
            END
        WHERE id = $1::uuid
          AND (last_pm_reset_date IS NULL OR last_pm_reset_date < $4)
        "#,
    )
    .bind(user_id)
    .bind(PM_CASH_DAILY_CENTS)
    .bind(PM_MTT_DAILY_CENTS)
    .bind(today)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn ensure_pm_daily_reset_pool(pool: &PgPool, user_id: &str) -> Result<bool, ApiError> {
    ensure_pm_daily_reset(pool, user_id).await
}

pub async fn debit_wallet<'e, E>(
    executor: E,
    user_id: &str,
    amount: i64,
    kind: WalletKind,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    if amount <= 0 {
        return Err(ApiError::BadRequest("Amount must be positive".into()));
    }
    let sql = match kind {
        WalletKind::PmCash => {
            "UPDATE users SET balance_pm_cash = balance_pm_cash - $1, balance = balance_pm_cash - $1 \
             WHERE id = $2::uuid AND balance_pm_cash >= $1 RETURNING balance_pm_cash"
        }
        WalletKind::PmMtt => {
            "UPDATE users SET balance_pm_mtt = balance_pm_mtt - $1 \
             WHERE id = $2::uuid AND balance_pm_mtt >= $1 RETURNING balance_pm_mtt"
        }
        WalletKind::Real => {
            "UPDATE users SET balance_real = balance_real - $1 \
             WHERE id = $2::uuid AND balance_real >= $1 RETURNING balance_real"
        }
    };
    // Note: for PmCash the SET balance = balance_pm_cash - $1 uses NEW value in PostgreSQL?
    // In PostgreSQL, UPDATE uses the original row values on the RHS for all columns in the same SET.
    // So `balance = balance_pm_cash - $1` is correct (old pm_cash - amount).
    let updated: Option<(i64,)> = sqlx::query_as(sql)
        .bind(amount)
        .bind(user_id)
        .fetch_optional(executor)
        .await?;
    if updated.is_none() {
        return Err(ApiError::BadRequest(
            "Insufficient wallet balance".to_string(),
        ));
    }
    Ok(())
}

pub async fn credit_wallet<'e, E>(
    executor: E,
    user_id: &str,
    amount: i64,
    kind: WalletKind,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    if amount < 0 {
        return Err(ApiError::BadRequest("Amount invalid".into()));
    }
    if amount == 0 {
        return Ok(());
    }
    let sql = match kind {
        WalletKind::PmCash => {
            "UPDATE users SET balance_pm_cash = balance_pm_cash + $1, balance = balance_pm_cash + $1 \
             WHERE id = $2::uuid"
        }
        WalletKind::PmMtt => {
            "UPDATE users SET balance_pm_mtt = balance_pm_mtt + $1 WHERE id = $2::uuid"
        }
        WalletKind::Real => {
            "UPDATE users SET balance_real = balance_real + $1 WHERE id = $2::uuid"
        }
    };
    sqlx::query(sql)
        .bind(amount)
        .bind(user_id)
        .execute(executor)
        .await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct WalletSnapshot {
    pub balance_pm_cash: i64,
    pub balance_pm_mtt: i64,
    pub balance_real: i64,
    pub preferred_wallet_mode: String,
    pub last_pm_reset_date: Option<String>,
    pub pm_cash_rebuy_available: bool,
    pub pm_mtt_rebuy_available: bool,
}

pub async fn load_snapshot(pool: &PgPool, user_id: &str) -> Result<WalletSnapshot, ApiError> {
    let _: bool = ensure_pm_daily_reset_pool(pool, user_id).await?;
    let today = today_sao_paulo();
    let row: (
        i64,
        i64,
        i64,
        String,
        Option<chrono::NaiveDate>,
        Option<chrono::NaiveDate>,
        Option<chrono::NaiveDate>,
    ) = sqlx::query_as(
        r#"
        SELECT balance_pm_cash, balance_pm_mtt, balance_real, preferred_wallet_mode,
               last_pm_reset_date, pm_cash_rebuy_used_on, pm_mtt_rebuy_used_on
        FROM users WHERE id = $1::uuid
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(WalletSnapshot {
        balance_pm_cash: row.0,
        balance_pm_mtt: row.1,
        balance_real: row.2,
        preferred_wallet_mode: row.3,
        last_pm_reset_date: row.4.map(|d| d.to_string()),
        pm_cash_rebuy_available: row.0 == 0 && row.5.map(|d| d < today).unwrap_or(true),
        pm_mtt_rebuy_available: row.1 == 0 && row.6.map(|d| d < today).unwrap_or(true),
    })
}

pub async fn pm_rebuy(
    pool: &PgPool,
    user_id: &str,
    kind: WalletKind,
) -> Result<WalletSnapshot, ApiError> {
    ensure_pm_daily_reset_pool(pool, user_id).await?;
    let today = today_sao_paulo();
    let mut tx = pool.begin().await?;

    let row: (i64, Option<chrono::NaiveDate>) = match kind {
        WalletKind::PmCash => {
            sqlx::query_as(
                "SELECT balance_pm_cash, pm_cash_rebuy_used_on FROM users WHERE id = $1::uuid FOR UPDATE",
            )
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?
        }
        WalletKind::PmMtt => {
            sqlx::query_as(
                "SELECT balance_pm_mtt, pm_mtt_rebuy_used_on FROM users WHERE id = $1::uuid FOR UPDATE",
            )
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?
        }
        WalletKind::Real => {
            return Err(ApiError::BadRequest(
                "Rebuy diário só existe em Play Money".into(),
            ));
        }
    };

    if row.0 != 0 {
        return Err(ApiError::BadRequest(
            "Rebuy só quando o saldo Play Money estiver zerado".into(),
        ));
    }
    if row.1 == Some(today) {
        return Err(ApiError::BadRequest(
            "Rebuy diário já utilizado hoje".into(),
        ));
    }

    match kind {
        WalletKind::PmCash => {
            sqlx::query(
                "UPDATE users SET balance_pm_cash = $2, balance = $2, pm_cash_rebuy_used_on = $3 \
                 WHERE id = $1::uuid",
            )
            .bind(user_id)
            .bind(PM_CASH_DAILY_CENTS)
            .bind(today)
            .execute(&mut *tx)
            .await?;
        }
        WalletKind::PmMtt => {
            sqlx::query(
                "UPDATE users SET balance_pm_mtt = $2, pm_mtt_rebuy_used_on = $3 WHERE id = $1::uuid",
            )
            .bind(user_id)
            .bind(PM_MTT_DAILY_CENTS)
            .bind(today)
            .execute(&mut *tx)
            .await?;
        }
        WalletKind::Real => unreachable!(),
    }

    tx.commit().await?;
    load_snapshot(pool, user_id).await
}

pub fn cash_kind_for_mode(mode: WalletMode) -> WalletKind {
    match mode {
        WalletMode::Play => WalletKind::PmCash,
        WalletMode::Real => WalletKind::Real,
    }
}

pub fn mtt_kind_for_mode(mode: WalletMode) -> WalletKind {
    match mode {
        WalletMode::Play => WalletKind::PmMtt,
        WalletMode::Real => WalletKind::Real,
    }
}

#[derive(Debug, Deserialize)]
pub struct SetModeBody {
    pub mode: String,
}

#[derive(Debug, Deserialize)]
pub struct PmRebuyBody {
    pub kind: String,
}

pub async fn set_wallet_mode(
    crate::middleware::auth::RequireAuth(auth_user): crate::middleware::auth::RequireAuth,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    axum::Json(body): axum::Json<SetModeBody>,
) -> Result<axum::Json<WalletSnapshot>, ApiError> {
    let mode = WalletMode::parse(Some(&body.mode));
    sqlx::query("UPDATE users SET preferred_wallet_mode = $2 WHERE id = $1::uuid")
        .bind(&auth_user.user_id)
        .bind(mode.as_str())
        .execute(&state.db)
        .await?;
    load_snapshot(&state.db, &auth_user.user_id)
        .await
        .map(axum::Json)
}

pub async fn pm_rebuy_handler(
    crate::middleware::auth::RequireAuth(auth_user): crate::middleware::auth::RequireAuth,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    axum::Json(body): axum::Json<PmRebuyBody>,
) -> Result<axum::Json<WalletSnapshot>, ApiError> {
    let kind = match body.kind.trim().to_ascii_lowercase().as_str() {
        "cash" | "pm_cash" => WalletKind::PmCash,
        "mtt" | "pm_mtt" | "tournament" => WalletKind::PmMtt,
        _ => return Err(ApiError::BadRequest("kind must be cash or mtt".into())),
    };
    pm_rebuy(&state.db, &auth_user.user_id, kind)
        .await
        .map(axum::Json)
}
