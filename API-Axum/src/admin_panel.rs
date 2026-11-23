//! Platform ops admin panel endpoints (stats, users, tables list, tournaments, presence, audit).

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

pub(crate) fn require_admin(auth_user: &crate::middleware::auth::AuthUser) -> Result<(), ApiError> {
    if auth_user.role == "admin" {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "Administrator access is required".to_string(),
        ))
    }
}

async fn write_audit(
    state: &AppState,
    user_id: &str,
    action: &str,
    metadata: serde_json::Value,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(action)
        .bind(metadata)
        .execute(&state.db)
        .await?;
    Ok(())
}

// ─── Stats ───

#[derive(Debug, Serialize)]
pub struct AdminStatsResponse {
    pub users_total: i64,
    pub users_by_status: HashMap<String, i64>,
    pub users_verified: i64,
    pub tables_open: i64,
    pub tables_paused: i64,
    pub tables_closed: i64,
    pub tournaments_open: i64,
    pub tournament_registrations: i64,
    pub online_count: u64,
    pub wallet_balance_sum: i64,
}

// ─── Passivo com jogadores (Jogo Real) ───

/// Quanto do caixa pertence aos jogadores, em centavos. Comparar
/// `total_owed_to_players_cents` com o saldo disponível da conta recebedora:
/// saldo abaixo do total = alerta de conciliação.
#[derive(Debug, Serialize)]
pub struct AdminLiabilitiesResponse {
    /// Saldos `balance_real` (contas humanas).
    pub wallet_balances_cents: i64,
    /// Fichas em assentos ativos de cash Real (gravadas a cada mão).
    pub cash_table_chips_cents: i64,
    /// Torneios Real abertos: maior entre prêmio acumulado e garantido.
    pub open_tournament_prizes_cents: i64,
    /// Saques já reservados (debitados da carteira) e ainda não pagos.
    pub pending_withdrawals_cents: i64,
    pub total_owed_to_players_cents: i64,
    /// Crédito provisório concedido antes da liquidação do PIX (risco, não passivo).
    pub provisional_deposits_cents: i64,
    pub generated_at: String,
}

pub async fn admin_liabilities(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<AdminLiabilitiesResponse>, ApiError> {
    require_admin(&auth_user)?;

    // Uma única consulta = um único snapshot coerente do banco.
    let row: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT \
           (SELECT COALESCE(SUM(balance_real), 0)::BIGINT FROM users WHERE NOT is_bot), \
           (SELECT COALESCE(SUM(chips), 0)::BIGINT FROM cash_game_seats \
              WHERE status = 'ACTIVE' AND wallet_kind = 'real'), \
           (SELECT COALESCE(SUM(GREATEST(prize_pool, guaranteed_prize)), 0)::BIGINT FROM tournaments \
              WHERE money_mode = 'real' AND status IN ('registering', 'running', 'paused')), \
           (SELECT COALESCE(SUM(amount), 0)::BIGINT FROM wallet_transactions \
              WHERE transaction_type = 'WITHDRAW' AND status = 'PENDING'), \
           (SELECT COALESCE(SUM(credited_amount_cents), 0)::BIGINT FROM wallet_transactions \
              WHERE transaction_type = 'DEPOSIT' AND status = 'PENDING' \
                AND credited_amount_cents IS NOT NULL)",
    )
    .fetch_one(&state.db)
    .await?;
    let (wallets, chips, prizes, withdrawals, provisional) = row;

    Ok(Json(AdminLiabilitiesResponse {
        wallet_balances_cents: wallets,
        cash_table_chips_cents: chips,
        open_tournament_prizes_cents: prizes,
        pending_withdrawals_cents: withdrawals,
        total_owed_to_players_cents: owed_total(wallets, chips, prizes, withdrawals)?,
        provisional_deposits_cents: provisional,
        generated_at: chrono::Utc::now().to_rfc3339(),
    }))
}

fn owed_total(wallets: i64, chips: i64, prizes: i64, withdrawals: i64) -> Result<i64, ApiError> {
    [chips, prizes, withdrawals]
        .into_iter()
        .try_fold(wallets, i64::checked_add)
        .ok_or_else(|| ApiError::Internal("Liability total overflow".into()))
}

pub async fn admin_stats(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<AdminStatsResponse>, ApiError> {
    require_admin(&auth_user)?;

    let users_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    let status_rows: Vec<(String, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) FROM users GROUP BY status")
            .fetch_all(&state.db)
            .await?;
    let users_by_status = status_rows.into_iter().collect();

    let users_verified: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE email_verified_at IS NOT NULL")
            .fetch_one(&state.db)
            .await?;

    let tables_open: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tables WHERE status = 'OPEN'")
        .fetch_one(&state.db)
        .await?;
    let tables_paused: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM tables WHERE status = 'PAUSED'")
            .fetch_one(&state.db)
            .await?;
    let tables_closed: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM tables WHERE status = 'CLOSED'")
            .fetch_one(&state.db)
            .await?;

    let tournaments_open: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tournaments WHERE status IN ('registering', 'running', 'paused')",
    )
    .fetch_one(&state.db)
    .await?;

    let tournament_registrations: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM tournament_players")
            .fetch_one(&state.db)
            .await?;

    // PostgreSQL promotes SUM(BIGINT) to NUMERIC. Cast after aggregation so
    // sqlx can decode the bounded platform total as i64.
    let wallet_balance_sum: (i64,) =
        sqlx::query_as("SELECT COALESCE(SUM(balance), 0)::BIGINT FROM users")
            .fetch_one(&state.db)
            .await?;

    let online_count = state.presence.online_count(&state.redis).await.unwrap_or(0);

    Ok(Json(AdminStatsResponse {
        users_total: users_total.0,
        users_by_status,
        users_verified: users_verified.0,
        tables_open: tables_open.0,
        tables_paused: tables_paused.0,
        tables_closed: tables_closed.0,
        tournaments_open: tournaments_open.0,
        tournament_registrations: tournament_registrations.0,
        online_count,
        wallet_balance_sum: wallet_balance_sum.0,
    }))
}

// ─── Users ───

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub q: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AdminUserRow {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub balance: i64,
    pub email_verified_at: Option<i64>,
    pub created_at: i64,
    pub last_login: Option<i64>,
    pub mfa_enabled: bool,
    pub agent_status: String,
    pub agent_commission_balance_cents: i64,
    pub agent_play_target_cents: i64,
    pub agent_real_target_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub balance: i64,
    pub email_verified: bool,
    pub created_at: i64,
    pub last_login: Option<i64>,
    pub mfa_enabled: bool,
    pub agent_status: String,
    pub agent_commission_balance_cents: i64,
    pub agent_play_target_cents: i64,
    pub agent_real_target_cents: i64,
}

impl From<AdminUserRow> for AdminUserResponse {
    fn from(r: AdminUserRow) -> Self {
        Self {
            id: r.id.to_string(),
            username: r.username,
            email: r.email,
            role: r.role,
            status: r.status,
            balance: r.balance,
            email_verified: r.email_verified_at.is_some(),
            created_at: r.created_at,
            last_login: r.last_login,
            mfa_enabled: r.mfa_enabled,
            agent_status: r.agent_status,
            agent_commission_balance_cents: r.agent_commission_balance_cents,
            agent_play_target_cents: r.agent_play_target_cents,
            agent_real_target_cents: r.agent_real_target_cents,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminUsersListResponse {
    pub users: Vec<AdminUserResponse>,
    pub total: i64,
}

pub async fn list_users(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<AdminUsersListResponse>, ApiError> {
    require_admin(&auth_user)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);
    let q = query.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let like = q.map(|s| format!("%{s}%"));

    let total: (i64,) = match (like.as_ref(), status) {
        (Some(like), Some(st)) => {
            sqlx::query_as(
                "SELECT COUNT(*) FROM users WHERE status = $1 AND (username ILIKE $2 OR email ILIKE $2)",
            )
            .bind(st)
            .bind(like)
            .fetch_one(&state.db)
            .await?
        }
        (Some(like), None) => {
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE username ILIKE $1 OR email ILIKE $1")
                .bind(like)
                .fetch_one(&state.db)
                .await?
        }
        (None, Some(st)) => {
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE status = $1")
                .bind(st)
                .fetch_one(&state.db)
                .await?
        }
        (None, None) => {
            sqlx::query_as("SELECT COUNT(*) FROM users")
                .fetch_one(&state.db)
                .await?
        }
    };

    let rows: Vec<AdminUserRow> = match (like.as_ref(), status) {
        (Some(like), Some(st)) => sqlx::query_as(
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled, \
                    agent_status, agent_commission_balance_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'play'), 0)::BIGINT AS agent_play_target_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'real'), 0)::BIGINT AS agent_real_target_cents \
             FROM users WHERE status = $1 AND (username ILIKE $2 OR email ILIKE $2) \
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(st)
        .bind(like)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
        (Some(like), None) => sqlx::query_as(
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled, \
                    agent_status, agent_commission_balance_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'play'), 0)::BIGINT AS agent_play_target_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'real'), 0)::BIGINT AS agent_real_target_cents \
             FROM users WHERE username ILIKE $1 OR email ILIKE $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(like)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
        (None, Some(st)) => sqlx::query_as(
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled, \
                    agent_status, agent_commission_balance_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'play'), 0)::BIGINT AS agent_play_target_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'real'), 0)::BIGINT AS agent_real_target_cents \
             FROM users WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(st)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
        (None, None) => sqlx::query_as(
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled, \
                    agent_status, agent_commission_balance_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'play'), 0)::BIGINT AS agent_play_target_cents, \
                    COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'real'), 0)::BIGINT AS agent_real_target_cents \
             FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
    };

    Ok(Json(AdminUsersListResponse {
        users: rows.into_iter().map(AdminUserResponse::from).collect(),
        total: total.0,
    }))
}

#[derive(Debug, Deserialize)]
pub struct PatchUserBody {
    pub status: Option<String>,
    pub role: Option<String>,
}

pub async fn patch_user(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(body): Json<PatchUserBody>,
) -> Result<Json<AdminUserResponse>, ApiError> {
    require_admin(&auth_user)?;
    let uid = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;

    if body.status.is_none() && body.role.is_none() {
        return Err(ApiError::BadRequest(
            "Provide status and/or role to update".into(),
        ));
    }

    if let Some(ref status) = body.status {
        if !matches!(
            status.as_str(),
            "active" | "suspended" | "banned" | "pending_email_verification"
        ) {
            return Err(ApiError::BadRequest("Invalid status".into()));
        }
        if user_id == auth_user.user_id && matches!(status.as_str(), "suspended" | "banned") {
            return Err(ApiError::BadRequest(
                "Cannot suspend or ban your own admin account".into(),
            ));
        }
    }
    if let Some(ref role) = body.role {
        if !matches!(role.as_str(), "player" | "admin" | "moderator") {
            return Err(ApiError::BadRequest("Invalid role".into()));
        }
        if user_id == auth_user.user_id && role != "admin" {
            let admins: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM users WHERE role = 'admin' AND status = 'active'",
            )
            .fetch_one(&state.db)
            .await?;
            if admins.0 <= 1 {
                return Err(ApiError::BadRequest(
                    "Cannot demote the last active admin".into(),
                ));
            }
        }
    }

    let mut tx = state.db.begin().await?;
    if let Some(ref status) = body.status {
        sqlx::query(
            "UPDATE users SET status = $1, token_version = token_version + 1 WHERE id = $2",
        )
        .bind(status)
        .bind(uid)
        .execute(&mut *tx)
        .await?;
    }
    if let Some(ref role) = body.role {
        sqlx::query("UPDATE users SET role = $1, token_version = token_version + 1 WHERE id = $2")
            .bind(role)
            .bind(uid)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    write_audit(
        &state,
        &auth_user.user_id,
        "USER_PATCH",
        serde_json::json!({ "target_user_id": user_id, "status": body.status, "role": body.role }),
    )
    .await?;

    let row: AdminUserRow = sqlx::query_as(
        "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled, \
                agent_status, agent_commission_balance_cents, \
                COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'play'), 0)::BIGINT AS agent_play_target_cents, \
                COALESCE((SELECT c.target_ngr_cents FROM agent_monthly_cycles c WHERE c.agent_user_id = users.id AND c.cycle_start = date_trunc('month', timezone('America/Sao_Paulo', now()))::date AND c.money_mode = 'real'), 0)::BIGINT AS agent_real_target_cents \
         FROM users WHERE id = $1",
    )
    .bind(uid)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row.into()))
}

#[derive(Debug, Deserialize)]
pub struct AdjustBalanceBody {
    pub delta_cents: i64,
    pub reason: String,
    /// `pm_cash` | `pm_mtt` | `real` — obrigatório; sem default para Real.
    pub wallet: String,
}

#[derive(Debug, Serialize)]
pub struct AdjustBalanceResponse {
    pub user_id: String,
    pub balance: i64,
    pub wallet: String,
}

pub async fn adjust_balance(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(body): Json<AdjustBalanceBody>,
) -> Result<Json<AdjustBalanceResponse>, ApiError> {
    require_admin(&auth_user)?;
    let reason = body.reason.trim();
    if reason.is_empty() || reason.len() > 200 {
        return Err(ApiError::BadRequest(
            "reason must be 1–200 characters".into(),
        ));
    }
    if body.delta_cents == 0 {
        return Err(ApiError::BadRequest("delta_cents must be non-zero".into()));
    }
    let _uid = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user id".into()))?;

    let kind = match body.wallet.trim().to_ascii_lowercase().as_str() {
        "pm_cash" | "cash" | "play_cash" => crate::wallet::WalletKind::PmCash,
        "pm_mtt" | "mtt" | "tournament" => crate::wallet::WalletKind::PmMtt,
        "real" => crate::wallet::WalletKind::Real,
        _ => {
            return Err(ApiError::BadRequest(
                "wallet is required: pm_cash | pm_mtt | real".into(),
            ))
        }
    };

    // Ajuste, saldo resultante e trilha de auditoria numa única transação:
    // dinheiro nunca muda sem o registro BALANCE_ADJUST correspondente.
    let mut tx = state.db.begin().await?;

    // Reset diário PM antes do ajuste: aplicado depois, ele apagaria o crédito.
    crate::wallet::ensure_pm_daily_reset(&mut *tx, &user_id).await?;

    // Trava a linha e garante que o usuário existe (credit_wallet sozinho
    // não falha para id inexistente).
    let balance_sql = format!(
        "SELECT {} FROM users WHERE id = $1::uuid FOR UPDATE",
        kind.column()
    );
    let exists: Option<i64> = sqlx::query_scalar(&balance_sql)
        .bind(&user_id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound("User not found".into()));
    }

    if body.delta_cents > 0 {
        crate::wallet::credit_wallet(&mut *tx, &user_id, body.delta_cents, kind).await?;
    } else {
        crate::wallet::debit_wallet(&mut *tx, &user_id, -body.delta_cents, kind).await?;
    }

    let balance: i64 = sqlx::query_scalar(&balance_sql)
        .bind(&user_id)
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(&auth_user.user_id)
        .bind("BALANCE_ADJUST")
        .bind(serde_json::json!({
            "target_user_id": user_id,
            "delta_cents": body.delta_cents,
            "wallet": kind.seat_label(),
            "reason": reason,
            "new_balance": balance
        }))
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AdjustBalanceResponse {
        user_id,
        balance,
        wallet: kind.seat_label().to_string(),
    }))
}

// ─── Tables list ───

#[derive(Debug, Serialize, sqlx::FromRow)]
struct AdminTableListRow {
    id: uuid::Uuid,
    name: String,
    status: String,
    visibility: String,
    small_blind: i64,
    big_blind: i64,
    min_buy_in: i64,
    max_buy_in: i64,
    max_players: i16,
    current_players: i16,
}

#[derive(Debug, Serialize)]
pub struct AdminTableSeat {
    pub seat: i16,
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub chips: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminTableListItem {
    pub id: String,
    pub name: String,
    pub status: String,
    pub visibility: String,
    pub small_blind: i64,
    pub big_blind: i64,
    pub min_buy_in: i64,
    pub max_buy_in: i64,
    pub max_players: i16,
    pub current_players: i16,
    pub seats: Vec<AdminTableSeat>,
}

#[derive(Debug, sqlx::FromRow)]
struct SeatOccRow {
    table_id: uuid::Uuid,
    seat: i16,
    chips: i64,
    user_id: uuid::Uuid,
    username: String,
    email: String,
}

pub async fn list_admin_tables(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminTableListItem>>, ApiError> {
    require_admin(&auth_user)?;
    let rows: Vec<AdminTableListRow> = sqlx::query_as(
        "SELECT id, name, status, visibility, small_blind, big_blind, min_buy_in, max_buy_in, max_players, current_players \
         FROM tables WHERE game_type = 'cash' ORDER BY status, big_blind, name",
    )
    .fetch_all(&state.db)
    .await?;

    let occupied: Vec<SeatOccRow> = sqlx::query_as(
        "SELECT s.table_id, s.seat, s.chips, u.id AS user_id, u.username, u.email \
         FROM cash_game_seats s \
         JOIN users u ON u.id = s.user_id \
         WHERE s.status = 'ACTIVE' \
         ORDER BY s.table_id, s.seat",
    )
    .fetch_all(&state.db)
    .await?;

    let mut by_table: HashMap<uuid::Uuid, Vec<AdminTableSeat>> = HashMap::new();
    for row in occupied {
        by_table
            .entry(row.table_id)
            .or_default()
            .push(AdminTableSeat {
                seat: row.seat,
                user_id: row.user_id.to_string(),
                username: row.username,
                email: row.email,
                chips: row.chips,
            });
    }

    Ok(Json(
        rows.into_iter()
            .map(|r| AdminTableListItem {
                id: r.id.to_string(),
                name: r.name,
                status: r.status,
                visibility: r.visibility,
                small_blind: r.small_blind,
                big_blind: r.big_blind,
                min_buy_in: r.min_buy_in,
                max_buy_in: r.max_buy_in,
                max_players: r.max_players,
                current_players: r.current_players,
                seats: by_table.remove(&r.id).unwrap_or_default(),
            })
            .collect(),
    ))
}

// ─── Tournaments ───

#[derive(Debug, Serialize)]
pub struct AdminTournamentItem {
    pub id: String,
    pub name: String,
    pub buy_in: u64,
    pub guaranteed_prize: u64,
    pub prize_pool: u64,
    pub status: String,
    pub is_freeroll: bool,
    pub registered_players: u32,
    pub max_players: u32,
    pub table_max_players: u8,
    pub scheduled_start_at: Option<i64>,
    pub current_level: u32,
    pub allow_rebuy: bool,
    pub rebuy_cost: u64,
    pub rebuy_chips: u64,
    /// 0 = sem limite de quantidade.
    pub rebuy_max_count: u32,
    pub rebuy_max_level: u32,
    pub rebuy_stack_threshold: u64,
    pub allow_addon: bool,
    pub addon_cost: u64,
    pub addon_chips: u64,
    pub addon_max_level: u32,
    pub total_rebuys: u64,
    pub total_addons: u64,
}

fn admin_item(store: &crate::tournament_store::TournamentStore) -> AdminTournamentItem {
    let cfg = &store.state.config;
    AdminTournamentItem {
        id: store.id.clone(),
        name: cfg.name.clone(),
        buy_in: cfg.buy_in,
        guaranteed_prize: cfg.guaranteed_prize,
        prize_pool: store.state.prize_pool.max(cfg.guaranteed_prize),
        status: format!("{:?}", store.state.status).to_lowercase(),
        is_freeroll: cfg.is_freeroll,
        registered_players: store.state.players.len() as u32,
        max_players: cfg.max_players,
        table_max_players: store.table_max_players,
        scheduled_start_at: store.scheduled_start_at,
        current_level: store.state.current_level,
        allow_rebuy: cfg.allow_rebuy,
        rebuy_cost: store.rebuy_cost(),
        rebuy_chips: store.rebuy_chips(),
        rebuy_max_count: cfg.rebuy_max_count,
        rebuy_max_level: cfg.rebuy_max_level,
        rebuy_stack_threshold: cfg.rebuy_stack_threshold,
        allow_addon: cfg.allow_addon,
        addon_cost: store.addon_cost(),
        addon_chips: store.addon_chips(),
        addon_max_level: store.addon_max_level(),
        total_rebuys: store.state.total_rebuys,
        total_addons: store.state.total_addons,
    }
}

/// Rebuy e add-on de um evento, já validados (centavos e fichas inteiros).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RebuyAddonTerms {
    pub allow_rebuy: bool,
    pub rebuy_cost: i64,
    pub rebuy_chips: i64,
    pub rebuy_max_count: i32,
    pub rebuy_max_level: i32,
    pub rebuy_stack_threshold: i64,
    pub allow_addon: bool,
    pub addon_cost: i64,
    pub addon_chips: i64,
    pub addon_max_level: i32,
}

/// Maior nível aceito para rebuy e add-on (a estrutura padrão tem 26).
const REBUY_ADDON_LEVEL_CAP: i32 = 26;

/// Campos opcionais de rebuy e add-on vindos do admin (criação ou edição).
#[derive(Debug, Default, Deserialize)]
pub struct RebuyAddonBody {
    pub allow_rebuy: Option<bool>,
    pub rebuy_cost_cents: Option<i64>,
    pub rebuy_chips: Option<i64>,
    pub rebuy_max_count: Option<i32>,
    pub rebuy_max_level: Option<i32>,
    pub rebuy_stack_threshold: Option<i64>,
    pub allow_addon: Option<bool>,
    pub addon_cost_cents: Option<i64>,
    pub addon_chips: Option<i64>,
    pub addon_max_level: Option<i32>,
}

impl RebuyAddonBody {
    fn touches_anything(&self) -> bool {
        self.allow_rebuy.is_some()
            || self.rebuy_cost_cents.is_some()
            || self.rebuy_chips.is_some()
            || self.rebuy_max_count.is_some()
            || self.rebuy_max_level.is_some()
            || self.rebuy_stack_threshold.is_some()
            || self.allow_addon.is_some()
            || self.addon_cost_cents.is_some()
            || self.addon_chips.is_some()
            || self.addon_max_level.is_some()
    }

    /// Aplica os campos informados sobre `base` e valida o resultado.
    pub(crate) fn merge(&self, base: RebuyAddonTerms) -> Result<RebuyAddonTerms, ApiError> {
        let terms = RebuyAddonTerms {
            allow_rebuy: self.allow_rebuy.unwrap_or(base.allow_rebuy),
            rebuy_cost: self.rebuy_cost_cents.unwrap_or(base.rebuy_cost),
            rebuy_chips: self.rebuy_chips.unwrap_or(base.rebuy_chips),
            rebuy_max_count: self.rebuy_max_count.unwrap_or(base.rebuy_max_count),
            rebuy_max_level: self.rebuy_max_level.unwrap_or(base.rebuy_max_level),
            rebuy_stack_threshold: self
                .rebuy_stack_threshold
                .unwrap_or(base.rebuy_stack_threshold),
            allow_addon: self.allow_addon.unwrap_or(base.allow_addon),
            addon_cost: self.addon_cost_cents.unwrap_or(base.addon_cost),
            addon_chips: self.addon_chips.unwrap_or(base.addon_chips),
            addon_max_level: self.addon_max_level.unwrap_or(base.addon_max_level),
        };
        if terms.rebuy_cost < 0 || terms.addon_cost < 0 {
            return Err(ApiError::BadRequest(
                "preço de rebuy e add-on não pode ser negativo".into(),
            ));
        }
        if terms.rebuy_chips < 0 || terms.addon_chips < 0 || terms.rebuy_stack_threshold < 0 {
            return Err(ApiError::BadRequest(
                "fichas de rebuy e add-on não podem ser negativas".into(),
            ));
        }
        if !(0..=100).contains(&terms.rebuy_max_count) {
            return Err(ApiError::BadRequest(
                "quantidade de rebuys deve ser 0 (sem limite) a 100".into(),
            ));
        }
        if !(0..=REBUY_ADDON_LEVEL_CAP).contains(&terms.rebuy_max_level)
            || !(0..=REBUY_ADDON_LEVEL_CAP).contains(&terms.addon_max_level)
        {
            return Err(ApiError::BadRequest(format!(
                "nível de rebuy e add-on deve ser 0..{REBUY_ADDON_LEVEL_CAP}"
            )));
        }
        if terms.allow_rebuy && terms.rebuy_max_level < 1 {
            return Err(ApiError::BadRequest(
                "rebuy ligado exige o último nível (1 ou mais)".into(),
            ));
        }
        if terms.allow_addon && terms.addon_max_level < 1 && terms.rebuy_max_level < 1 {
            return Err(ApiError::BadRequest(
                "add-on ligado exige o último nível (1 ou mais)".into(),
            ));
        }
        Ok(terms)
    }
}

fn store_terms(store: &crate::tournament_store::TournamentStore) -> RebuyAddonTerms {
    let cfg = &store.state.config;
    RebuyAddonTerms {
        allow_rebuy: cfg.allow_rebuy,
        rebuy_cost: cfg.rebuy_cost as i64,
        rebuy_chips: cfg.rebuy_chips as i64,
        rebuy_max_count: cfg.rebuy_max_count as i32,
        rebuy_max_level: cfg.rebuy_max_level as i32,
        rebuy_stack_threshold: cfg.rebuy_stack_threshold as i64,
        allow_addon: cfg.allow_addon,
        addon_cost: store.addon.cost as i64,
        addon_chips: store.addon.chips as i64,
        addon_max_level: store.addon.max_level as i32,
    }
}

fn apply_terms(store: &mut crate::tournament_store::TournamentStore, terms: RebuyAddonTerms) {
    let cfg = &mut store.state.config;
    cfg.allow_rebuy = terms.allow_rebuy;
    cfg.rebuy_cost = terms.rebuy_cost.max(0) as u64;
    cfg.rebuy_chips = terms.rebuy_chips.max(0) as u64;
    cfg.rebuy_max_count = terms.rebuy_max_count.max(0) as u32;
    cfg.rebuy_max_level = terms.rebuy_max_level.max(0) as u32;
    cfg.rebuy_stack_threshold = terms.rebuy_stack_threshold.max(0) as u64;
    cfg.allow_addon = terms.allow_addon;
    store.addon = crate::tournament_store::AddonTerms {
        cost: terms.addon_cost.max(0) as u64,
        chips: terms.addon_chips.max(0) as u64,
        max_level: terms.addon_max_level.max(0) as u32,
    };
}

pub async fn list_admin_tournaments(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminTournamentItem>>, ApiError> {
    require_admin(&auth_user)?;
    let tournaments = state.tournaments.read().await;
    let mut list: Vec<_> = tournaments.values().map(admin_item).collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(list))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TourneyPlayerRow {
    player_id: String,
    player_name: String,
    stack: i64,
    rebuys: i32,
    registered_at: i64,
    email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminTournamentPlayer {
    pub player_id: String,
    pub player_name: String,
    pub stack: i64,
    pub rebuys: i32,
    pub registered_at: i64,
    pub email: Option<String>,
}

pub async fn list_tournament_players(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> Result<Json<Vec<AdminTournamentPlayer>>, ApiError> {
    require_admin(&auth_user)?;
    let tid = uuid::Uuid::parse_str(&tournament_id)
        .map_err(|_| ApiError::BadRequest("Invalid tournament id".into()))?;
    let rows: Vec<TourneyPlayerRow> = sqlx::query_as(
        "SELECT tp.player_id, tp.player_name, tp.stack, tp.rebuys, tp.registered_at, u.email \
         FROM tournament_players tp \
         LEFT JOIN users u ON u.id::text = tp.player_id \
         WHERE tp.tournament_id = $1 \
         ORDER BY tp.registered_at",
    )
    .bind(tid)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| AdminTournamentPlayer {
                player_id: r.player_id,
                player_name: r.player_name,
                stack: r.stack,
                rebuys: r.rebuys,
                registered_at: r.registered_at,
                email: r.email,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct PatchTournamentBody {
    pub status: Option<String>,
    pub scheduled_start_at: Option<i64>,
    /// Rebuy e add-on: só antes do início (inscrições abertas).
    #[serde(flatten)]
    pub rebuy_addon: RebuyAddonBody,
}

pub async fn patch_tournament(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Json(body): Json<PatchTournamentBody>,
) -> Result<Json<AdminTournamentItem>, ApiError> {
    require_admin(&auth_user)?;
    if body.status.is_none()
        && body.scheduled_start_at.is_none()
        && !body.rebuy_addon.touches_anything()
    {
        return Err(ApiError::BadRequest(
            "Provide status, scheduled_start_at and/or rebuy/add-on fields".into(),
        ));
    }
    if body.scheduled_start_at.is_some_and(|sched| sched <= 0) {
        return Err(ApiError::BadRequest(
            "scheduled_start_at deve ser uma data/hora válida definida pelo admin".into(),
        ));
    }

    let tid = uuid::Uuid::parse_str(&tournament_id)
        .map_err(|_| ApiError::BadRequest("Invalid tournament id".into()))?;

    if body.rebuy_addon.touches_anything() {
        let mut tournaments = state.tournaments.write().await;
        let store = tournaments
            .get_mut(&tournament_id)
            .ok_or_else(|| ApiError::NotFound("Tournament not found".into()))?;
        if store.state.status != poker_engine::tournament_engine::TournamentStatus::Registering {
            return Err(ApiError::BadRequest(
                "rebuy e add-on só mudam antes do início do torneio".into(),
            ));
        }
        let before = store_terms(store);
        let terms = body.rebuy_addon.merge(before)?;
        let updated = sqlx::query(
            "UPDATE tournaments SET allow_rebuy=$2, rebuy_cost=$3, rebuy_chips=$4, \
             rebuy_max_count=$5, rebuy_max_level=$6, rebuy_stack_threshold=$7, \
             allow_addon=$8, addon_cost=$9, addon_chips=$10, addon_max_level=$11 \
             WHERE id=$1 AND status='registering'",
        )
        .bind(tid)
        .bind(terms.allow_rebuy)
        .bind(terms.rebuy_cost)
        .bind(terms.rebuy_chips)
        .bind(terms.rebuy_max_count)
        .bind(terms.rebuy_max_level)
        .bind(terms.rebuy_stack_threshold)
        .bind(terms.allow_addon)
        .bind(terms.addon_cost)
        .bind(terms.addon_chips)
        .bind(terms.addon_max_level)
        .execute(&state.db)
        .await?
        .rows_affected();
        if updated != 1 {
            return Err(ApiError::BadRequest(
                "rebuy e add-on só mudam antes do início do torneio".into(),
            ));
        }
        apply_terms(store, terms);
        drop(tournaments);
        write_audit(
            &state,
            &auth_user.user_id,
            "TOURNAMENT_REBUY_ADDON",
            serde_json::json!({
                "tournament_id": tournament_id,
                "before": format!("{before:?}"),
                "after": format!("{terms:?}"),
            }),
        )
        .await?;
    }

    if let Some(ref status_raw) = body.status {
        let status = status_raw.trim().to_ascii_lowercase();
        if !matches!(status.as_str(), "registering" | "paused" | "cancelled") {
            return Err(ApiError::BadRequest(
                "status must be registering, paused, or cancelled".into(),
            ));
        }
        sqlx::query("UPDATE tournaments SET status = $1 WHERE id = $2")
            .bind(&status)
            .bind(tid)
            .execute(&state.db)
            .await?;
        {
            let mut tournaments = state.tournaments.write().await;
            if let Some(store) = tournaments.get_mut(&tournament_id) {
                store.state.status = match status.as_str() {
                    "paused" => poker_engine::tournament_engine::TournamentStatus::Paused,
                    "cancelled" => poker_engine::tournament_engine::TournamentStatus::Cancelled,
                    _ => poker_engine::tournament_engine::TournamentStatus::Registering,
                };
            }
        }
        write_audit(
            &state,
            &auth_user.user_id,
            "TOURNAMENT_STATUS",
            serde_json::json!({ "tournament_id": tournament_id, "status": status }),
        )
        .await?;
    }

    if let Some(sched) = body.scheduled_start_at {
        sqlx::query("UPDATE tournaments SET scheduled_start_at = $1 WHERE id = $2")
            .bind(sched)
            .bind(tid)
            .execute(&state.db)
            .await?;
        {
            let mut tournaments = state.tournaments.write().await;
            if let Some(store) = tournaments.get_mut(&tournament_id) {
                store.scheduled_start_at = Some(sched);
            }
        }
        write_audit(
            &state,
            &auth_user.user_id,
            "TOURNAMENT_SCHEDULE",
            serde_json::json!({ "tournament_id": tournament_id, "scheduled_start_at": sched }),
        )
        .await?;
    }

    let tournaments = state.tournaments.read().await;
    let store = tournaments
        .get(&tournament_id)
        .ok_or_else(|| ApiError::NotFound("Tournament not found".into()))?;
    Ok(Json(admin_item(store)))
}

#[derive(Debug, Deserialize)]
pub struct CreateTournamentBody {
    pub name: String,
    pub buy_in_cents: i64,
    pub starting_stack: i64,
    pub table_max_players: i16,
    pub max_players: Option<i32>,
    pub poker_variant: String,
    pub money_mode: String,
    pub guaranteed_prize_cents: i64,
    pub scheduled_start_at: Option<i64>,
    pub auto_start_min_players: Option<i32>,
    pub speed: Option<String>,
    pub late_registration: Option<bool>,
    pub late_reg_max_level: Option<i32>,
    #[serde(flatten)]
    pub rebuy_addon: RebuyAddonBody,
    pub final_table_variant: Option<String>,
    pub final_table_max_players: Option<i16>,
}

/// POST /api/admin/tournaments — cria torneio em `registering` com blinds
/// padrão de 26 níveis (BBA). `max_players` padrão/validado = 3× table_max.
pub async fn create_tournament(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateTournamentBody>,
) -> Result<Json<AdminTournamentItem>, ApiError> {
    require_admin(&auth_user)?;
    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError::BadRequest("nome inválido (1..100)".into()));
    }
    if !(2..=9).contains(&body.table_max_players) {
        return Err(ApiError::BadRequest(
            "table_max_players deve ser 2..9".into(),
        ));
    }
    let max_players = body
        .max_players
        .unwrap_or(i32::from(body.table_max_players) * 3);
    if max_players != i32::from(body.table_max_players) * 3 {
        return Err(ApiError::BadRequest(
            "max_players deve ser 3x table_max_players (3 mesas)".into(),
        ));
    }
    if body.buy_in_cents < 0 || body.starting_stack <= 0 || body.guaranteed_prize_cents < 0 {
        return Err(ApiError::BadRequest("valores monetários inválidos".into()));
    }
    let scheduled_start_at = body
        .scheduled_start_at
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            ApiError::BadRequest(
                "scheduled_start_at é obrigatório e deve ser definido pelo admin".into(),
            )
        })?;
    let variant = body.poker_variant.trim().to_ascii_lowercase();
    if !matches!(
        variant.as_str(),
        "holdem" | "short_deck" | "omaha" | "pineapple_draw"
    ) {
        return Err(ApiError::BadRequest("poker_variant inválida".into()));
    }
    let variant_cap = match variant.as_str() {
        "holdem" => 9,
        "short_deck" => 8,
        _ => 6,
    };
    if body.table_max_players > variant_cap {
        return Err(ApiError::BadRequest(format!(
            "table_max_players deve ser no máximo {variant_cap} para {variant}"
        )));
    }
    let final_table_variant = body
        .final_table_variant
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase());
    match (final_table_variant.as_deref(), body.final_table_max_players) {
        (None, None) => {}
        (Some(final_variant), Some(final_max)) => {
            if !matches!(
                final_variant,
                "holdem" | "short_deck" | "omaha" | "pineapple_draw"
            ) {
                return Err(ApiError::BadRequest("final_table_variant inválida".into()));
            }
            let final_cap = match final_variant {
                "holdem" => 9,
                "short_deck" => 8,
                _ => 6,
            };
            if !(2..=final_cap).contains(&final_max) {
                return Err(ApiError::BadRequest(format!(
                    "final_table_max_players deve ser 2..{final_cap} para {final_variant}"
                )));
            }
        }
        _ => {
            return Err(ApiError::BadRequest(
                "final_table_variant e final_table_max_players devem ser informados juntos".into(),
            ));
        }
    }
    let money_mode = if body.money_mode.eq_ignore_ascii_case("real") {
        "real"
    } else {
        "play"
    };
    let speed = body
        .speed
        .unwrap_or_else(|| "normal".into())
        .to_ascii_lowercase();
    if !matches!(speed.as_str(), "turbo" | "normal" | "slow") {
        return Err(ApiError::BadRequest("speed inválida".into()));
    }
    let is_freeroll = body.buy_in_cents == 0;
    // Padrões: rebuy desligado (1×, até o nível 6, preço do buy-in, stack inicial);
    // add-on desligado (preço do buy-in, stack inicial, mesmo nível do rebuy).
    let terms = body.rebuy_addon.merge(RebuyAddonTerms {
        allow_rebuy: false,
        rebuy_cost: body.buy_in_cents,
        rebuy_chips: body.starting_stack,
        rebuy_max_count: 1,
        rebuy_max_level: 6,
        rebuy_stack_threshold: 0,
        allow_addon: false,
        addon_cost: body.buy_in_cents,
        addon_chips: body.starting_stack,
        addon_max_level: 0,
    })?;
    let id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO tournaments
            (id, name, buy_in, starting_stack, max_players, table_max_players,
             late_registration, late_reg_max_level, speed, status,
             guaranteed_prize, is_freeroll,
             rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
             rebuy_max_level, allow_rebuy, game_type, money_mode,
             poker_variant, final_table_variant, final_table_max_players,
             scheduled_start_at, auto_start_min_players,
             prize_pool, current_level, players_remaining, total_buyins, total_fees,
             allow_addon, addon_cost, addon_chips, addon_max_level)
         VALUES
            ($1, $2, $3, $4, $5, $6,
             $7, $8, $9, 'registering',
             $10, $11,
             $12, $13, $14, $23,
             $15, $16, 'Holdem', $17,
             $18, $19, $20,
             $21, $22,
             0, 0, 0, 0, 0,
             $24, $25, $26, $27)"#,
    )
    .bind(id)
    .bind(&name)
    .bind(body.buy_in_cents)
    .bind(body.starting_stack)
    .bind(max_players)
    .bind(body.table_max_players)
    .bind(body.late_registration.unwrap_or(true))
    .bind(body.late_reg_max_level.unwrap_or(4))
    .bind(&speed)
    .bind(body.guaranteed_prize_cents)
    .bind(is_freeroll)
    .bind(terms.rebuy_cost)
    .bind(terms.rebuy_chips)
    .bind(terms.rebuy_max_count)
    .bind(terms.rebuy_max_level)
    .bind(terms.allow_rebuy)
    .bind(money_mode)
    .bind(&variant)
    .bind(final_table_variant)
    .bind(body.final_table_max_players)
    .bind(scheduled_start_at)
    .bind(body.auto_start_min_players.unwrap_or(5))
    .bind(terms.rebuy_stack_threshold)
    .bind(terms.allow_addon)
    .bind(terms.addon_cost)
    .bind(terms.addon_chips)
    .bind(terms.addon_max_level)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::BadRequest(format!("create tournament: {e}")))?;
    // Recarrega a linha (defaults do banco aplicados) e publica no mapa.
    let created: crate::tournament_catalog::TournamentRow = sqlx::query_as(
        "SELECT id, name, buy_in, starting_stack, max_players, table_max_players, \
                late_registration, late_reg_max_level, speed, status, \
                prize_pool, current_level, players_remaining, total_buyins, \
                guaranteed_prize, is_freeroll, \
                rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold, \
                rebuy_max_level, allow_rebuy, allow_addon, addon_cost, addon_chips,                 addon_max_level, total_rebuys, total_addons, blind_levels, game_type, money_mode, \
                poker_variant, final_table_variant, final_table_max_players, \
                scheduled_start_at, auto_start_min_players, live_table_id, live_table_ids \
         FROM tournaments WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    let store = crate::tournament_catalog::row_to_store(created);
    let item = admin_item(&store);
    state
        .tournaments
        .write()
        .await
        .insert(store.id.clone(), store);
    write_audit(
        &state,
        &auth_user.user_id,
        "TOURNAMENT_CREATED",
        serde_json::json!({
            "tournament_id": item.id,
            "name": item.name,
            "scheduled_start_at": item.scheduled_start_at
        }),
    )
    .await?;
    Ok(Json(item))
}

// ─── Presence ───

#[derive(Debug, Serialize)]
pub struct PresenceUser {
    pub user_id: String,
    pub username: String,
    pub last_seen: u64,
}

#[derive(Debug, Serialize)]
pub struct AdminPresenceResponse {
    pub online_count: u64,
    pub users: Vec<PresenceUser>,
}

pub async fn admin_presence(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<AdminPresenceResponse>, ApiError> {
    require_admin(&auth_user)?;
    let roster = state
        .presence
        .online_roster(&state.redis)
        .await
        .map_err(ApiError::Internal)?;

    let mut users = Vec::with_capacity(roster.len());
    for (user_id, last_seen) in roster {
        let username: Option<(String,)> =
            sqlx::query_as("SELECT username FROM users WHERE id = $1::uuid")
                .bind(&user_id)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);
        users.push(PresenceUser {
            username: username.map(|u| u.0).unwrap_or_else(|| user_id.clone()),
            user_id,
            last_seen,
        });
    }
    users.sort_by(|a, b| a.username.cmp(&b.username));
    let online_count = users.len() as u64;
    Ok(Json(AdminPresenceResponse {
        online_count,
        users,
    }))
}

// ─── Audit ───

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct AuditRow {
    id: uuid::Uuid,
    user_id: String,
    action: String,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogItem {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

pub async fn list_audit_logs(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogItem>>, ApiError> {
    require_admin(&auth_user)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let action = query
        .action
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let rows: Vec<AuditRow> = if let Some(action) = action {
        sqlx::query_as(
            "SELECT id, user_id, action, metadata, created_at FROM audit_logs \
             WHERE action = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(action)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, user_id, action, metadata, created_at FROM audit_logs \
             ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(
        rows.into_iter()
            .map(|r| AuditLogItem {
                id: r.id.to_string(),
                user_id: r.user_id,
                action: r.action,
                metadata: r.metadata,
                created_at: r.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::owed_total;

    #[test]
    fn liability_total_sums_every_player_bucket() {
        assert_eq!(owed_total(10_000, 2_500, 15_000, 500).unwrap(), 28_000);
        assert_eq!(owed_total(0, 0, 0, 0).unwrap(), 0);
        assert!(owed_total(i64::MAX, 1, 0, 0).is_err());
    }
}
