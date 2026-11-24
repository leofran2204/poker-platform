//! Multi-wallet: Play Money cash / Play Money MTT / Real.

use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, PgPool};

use crate::error::ApiError;

pub const PM_CASH_DAILY_CENTS: i64 = 15_000; // R$ 150 — grant de cadastro/reset (cash games, play money)
pub const PM_MTT_DAILY_CENTS: i64 = 15_000; // R$ 150 — grant de cadastro/reset (torneios, play money)

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

pub fn today_sao_paulo() -> time::Date {
    // UTC−3 (no DST in BR since 2019)
    crate::clock::now()
        .to_offset(time::macros::offset!(-3))
        .date()
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
            ledger_reason = 'pm_daily_reset',
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

/// Motivo de cada linha do livro único do dinheiro (`wallet_ledger`, migration 066).
/// Quem muda um saldo informa o motivo na mesma instrução (`users.ledger_reason`);
/// o gatilho do banco grava a linha. Sem motivo, a linha sai `unspecified`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerReason {
    CashBuyIn,
    CashOut,
    BootCashOut,
    TournamentBuyIn,
    TournamentFee,
    TournamentUnregisterRefund,
    TournamentRebuy,
    TournamentAddon,
    TournamentPrize,
    PixDeposit,
    PixDepositReversal,
    PixWithdraw,
    PixWithdrawReversal,
    ManualDeposit,
    AdminAdjustment,
    PmDailyReset,
    BotTopUp,
}

impl LedgerReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CashBuyIn => "cash_buy_in",
            Self::CashOut => "cash_out",
            Self::BootCashOut => "boot_cash_out",
            Self::TournamentBuyIn => "tournament_buy_in",
            Self::TournamentFee => "tournament_fee",
            Self::TournamentUnregisterRefund => "tournament_unregister_refund",
            Self::TournamentRebuy => "tournament_rebuy",
            Self::TournamentAddon => "tournament_addon",
            Self::TournamentPrize => "tournament_prize",
            Self::PixDeposit => "pix_deposit",
            Self::PixDepositReversal => "pix_deposit_reversal",
            Self::PixWithdraw => "pix_withdraw",
            Self::PixWithdrawReversal => "pix_withdraw_reversal",
            Self::ManualDeposit => "manual_deposit",
            Self::AdminAdjustment => "admin_adjustment",
            Self::PmDailyReset => "pm_daily_reset",
            Self::BotTopUp => "bot_top_up",
        }
    }
}

pub async fn debit_wallet<'e, E>(
    executor: E,
    user_id: &str,
    amount: i64,
    kind: WalletKind,
    reason: LedgerReason,
    reference: Option<&str>,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    if amount <= 0 {
        return Err(ApiError::BadRequest("Amount must be positive".into()));
    }
    let sql = match kind {
        WalletKind::PmCash => {
            "UPDATE users SET balance_pm_cash = balance_pm_cash - $1, balance = balance_pm_cash - $1, \
             ledger_reason = $3, ledger_ref = $4 \
             WHERE id = $2::uuid AND balance_pm_cash >= $1 RETURNING balance_pm_cash"
        }
        WalletKind::PmMtt => {
            "UPDATE users SET balance_pm_mtt = balance_pm_mtt - $1, ledger_reason = $3, ledger_ref = $4 \
             WHERE id = $2::uuid AND balance_pm_mtt >= $1 RETURNING balance_pm_mtt"
        }
        WalletKind::Real => {
            // `NOT is_bot`: segunda barreira — bot nunca é debitado no Real.
            "UPDATE users SET balance_real = balance_real - $1, ledger_reason = $3, ledger_ref = $4 \
             WHERE id = $2::uuid AND balance_real >= $1 AND NOT is_bot RETURNING balance_real"
        }
    };
    // Note: for PmCash the SET balance = balance_pm_cash - $1 uses NEW value in PostgreSQL?
    // In PostgreSQL, UPDATE uses the original row values on the RHS for all columns in the same SET.
    // So `balance = balance_pm_cash - $1` is correct (old pm_cash - amount).
    let updated: Option<(i64,)> = sqlx::query_as(sql)
        .bind(amount)
        .bind(user_id)
        .bind(reason.as_str())
        .bind(reference)
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
    reason: LedgerReason,
    reference: Option<&str>,
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
            "UPDATE users SET balance_pm_cash = balance_pm_cash + $1, balance = balance_pm_cash + $1, \
             ledger_reason = $3, ledger_ref = $4 WHERE id = $2::uuid"
        }
        WalletKind::PmMtt => {
            "UPDATE users SET balance_pm_mtt = balance_pm_mtt + $1, ledger_reason = $3, \
             ledger_ref = $4 WHERE id = $2::uuid"
        }
        WalletKind::Real => {
            "UPDATE users SET balance_real = balance_real + $1, ledger_reason = $3, \
             ledger_ref = $4 WHERE id = $2::uuid"
        }
    };
    let credited = sqlx::query(sql)
        .bind(amount)
        .bind(user_id)
        .bind(reason.as_str())
        .bind(reference)
        .execute(executor)
        .await?
        .rows_affected();
    // Crédito que não alcança conta nenhuma é dinheiro perdido em silêncio:
    // o erro faz a transação de quem chamou ser desfeita.
    if credited != 1 {
        return Err(ApiError::Internal(format!(
            "Wallet credit reached {credited} accounts instead of 1"
        )));
    }
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
    let row: (
        i64,
        i64,
        i64,
        String,
        Option<time::Date>,
        Option<time::Date>,
        Option<time::Date>,
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
        // Sem rebuy grátis (play money): zerou, espera a renovação diária à 00:00 (Brasília).
        pm_cash_rebuy_available: false,
        pm_mtt_rebuy_available: false,
    })
}

pub async fn pm_rebuy(
    _pool: &PgPool,
    _user_id: &str,
    kind: WalletKind,
) -> Result<WalletSnapshot, ApiError> {
    // Sem rebuy grátis (play money, cash e torneio): zerou, espera a renovação
    // diária à 00:00 (Brasília). Reentradas em mesa/torneio continuam ilimitadas
    // enquanto houver saldo (cada join/register debita o buy-in).
    match kind {
        WalletKind::PmCash | WalletKind::PmMtt => Err(ApiError::BadRequest(
            "Sem rebuy: saldo Play Money zerado — aguarde a renovação diária à 00:00 (Brasília)"
                .into(),
        )),
        WalletKind::Real => Err(ApiError::BadRequest(
            "Rebuy diário só existe em Play Money".into(),
        )),
    }
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

// ─── Livro único do dinheiro (migration 066) ─────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct LedgerQuery {
    /// `pm_cash`, `pm_mtt` ou `real`; sem filtro, as três carteiras.
    pub wallet: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct LedgerEntry {
    pub id: i64,
    pub wallet: String,
    pub amount_cents: i64,
    pub balance_after_cents: i64,
    pub reason: String,
    pub reference: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}

/// GET /api/wallet/ledger — extrato completo da própria conta (até 200 linhas,
/// recentes primeiro): mesa, torneio, Loss Deflator na mesa, PIX, depósito
/// manual, ajuste do admin e renovação diária do Play Money.
pub async fn my_ledger(
    crate::middleware::auth::RequireAuth(auth_user): crate::middleware::auth::RequireAuth,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    axum::extract::Query(query): axum::extract::Query<LedgerQuery>,
) -> Result<axum::Json<Vec<LedgerEntry>>, ApiError> {
    let wallet = query.wallet.map(|w| w.trim().to_ascii_lowercase());
    if let Some(w) = wallet.as_deref() {
        if !matches!(w, "pm_cash" | "pm_mtt" | "real") {
            return Err(ApiError::BadRequest(
                "wallet must be pm_cash, pm_mtt or real".into(),
            ));
        }
    }
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let rows: Vec<LedgerEntry> = sqlx::query_as(
        "SELECT id, wallet, amount_cents, balance_after_cents, reason, reference, created_at \
         FROM wallet_ledger WHERE user_id = $1::uuid AND ($2::text IS NULL OR wallet = $2) \
         ORDER BY id DESC LIMIT $3",
    )
    .bind(&auth_user.user_id)
    .bind(wallet)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(axum::Json(rows))
}

#[derive(Debug, serde::Serialize)]
pub struct LedgerReconciliation {
    /// Contas e carteiras em que o saldo não bate com a soma do livro (até 100).
    pub mismatches: Vec<serde_json::Value>,
    pub mismatch_count: i64,
    /// Linhas gravadas sem motivo (mudança de saldo fora das portas conhecidas).
    pub unspecified_count: i64,
    pub entries: i64,
    pub ok: bool,
}

/// GET /api/admin/ledger/reconcile — saldo = soma do livro em todas as contas.
pub async fn admin_ledger_reconcile(
    crate::middleware::auth::RequireAuth(auth_user): crate::middleware::auth::RequireAuth,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> Result<axum::Json<LedgerReconciliation>, ApiError> {
    crate::admin_panel::require_admin(&auth_user)?;
    let rows: Vec<(uuid::Uuid, String, i64, i64)> = sqlx::query_as(
        "SELECT user_id, wallet, balance_cents, ledger_cents::bigint FROM wallet_ledger_mismatches \
         ORDER BY user_id LIMIT 100",
    )
    .fetch_all(&state.db)
    .await?;
    let mismatch_count: i64 = sqlx::query_scalar("SELECT count(*) FROM wallet_ledger_mismatches")
        .fetch_one(&state.db)
        .await?;
    let (entries, unspecified_count): (i64, i64) = sqlx::query_as(
        "SELECT count(*), count(*) FILTER (WHERE reason = 'unspecified') FROM wallet_ledger",
    )
    .fetch_one(&state.db)
    .await?;
    Ok(axum::Json(LedgerReconciliation {
        mismatches: rows
            .into_iter()
            .map(|(user_id, wallet, balance, ledger)| {
                serde_json::json!({
                    "user_id": user_id,
                    "wallet": wallet,
                    "balance_cents": balance,
                    "ledger_cents": ledger,
                })
            })
            .collect(),
        mismatch_count,
        unspecified_count,
        entries,
        ok: mismatch_count == 0 && unspecified_count == 0,
    }))
}
