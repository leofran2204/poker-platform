//! Platform ops admin panel endpoints (stats, users, tables list, tournaments, presence, audit).

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

fn require_admin(auth_user: &crate::middleware::auth::AuthUser) -> Result<(), ApiError> {
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
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled \
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
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled \
             FROM users WHERE username ILIKE $1 OR email ILIKE $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(like)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
        (None, Some(st)) => sqlx::query_as(
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled \
             FROM users WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(st)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
        (None, None) => sqlx::query_as(
            "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled \
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
        "SELECT id, username, email, role, status, balance, email_verified_at, created_at, last_login, mfa_enabled \
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
    /// `pm_cash` | `pm_mtt` | `real` (default `real`)
    #[serde(default)]
    pub wallet: Option<String>,
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

    let kind = match body
        .wallet
        .as_deref()
        .unwrap_or("real")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "pm_cash" | "cash" | "play_cash" => crate::wallet::WalletKind::PmCash,
        "pm_mtt" | "mtt" | "tournament" => crate::wallet::WalletKind::PmMtt,
        _ => crate::wallet::WalletKind::Real,
    };

    if body.delta_cents > 0 {
        crate::wallet::credit_wallet(&state.db, &user_id, body.delta_cents, kind).await?;
    } else {
        crate::wallet::debit_wallet(&state.db, &user_id, -body.delta_cents, kind).await?;
    }

    let snap = crate::wallet::load_snapshot(&state.db, &user_id).await?;
    let balance = match kind {
        crate::wallet::WalletKind::PmCash => snap.balance_pm_cash,
        crate::wallet::WalletKind::PmMtt => snap.balance_pm_mtt,
        crate::wallet::WalletKind::Real => snap.balance_real,
    };

    write_audit(
        &state,
        &auth_user.user_id,
        "BALANCE_ADJUST",
        serde_json::json!({
            "target_user_id": user_id,
            "delta_cents": body.delta_cents,
            "wallet": kind.seat_label(),
            "reason": reason,
            "new_balance": balance
        }),
    )
    .await?;

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
}

pub async fn list_admin_tournaments(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminTournamentItem>>, ApiError> {
    require_admin(&auth_user)?;
    let tournaments = state.tournaments.read().await;
    let mut list: Vec<_> = tournaments
        .values()
        .map(|store| AdminTournamentItem {
            id: store.id.clone(),
            name: store.state.config.name.clone(),
            buy_in: store.state.config.buy_in,
            guaranteed_prize: store.state.config.guaranteed_prize,
            prize_pool: store
                .state
                .prize_pool
                .max(store.state.config.guaranteed_prize),
            status: format!("{:?}", store.state.status).to_lowercase(),
            is_freeroll: store.state.config.is_freeroll,
            registered_players: store.state.players.len() as u32,
            max_players: store.state.config.max_players,
            table_max_players: store.table_max_players,
        })
        .collect();
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
}

#[derive(Debug, Serialize)]
pub struct AdminTournamentPlayer {
    pub player_id: String,
    pub player_name: String,
    pub stack: i64,
    pub rebuys: i32,
    pub registered_at: i64,
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
        "SELECT player_id, player_name, stack, rebuys, registered_at \
         FROM tournament_players WHERE tournament_id = $1 ORDER BY registered_at",
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
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct PatchTournamentBody {
    pub status: String,
}

pub async fn patch_tournament(
    RequireAuth(auth_user): RequireAuth,
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Json(body): Json<PatchTournamentBody>,
) -> Result<Json<AdminTournamentItem>, ApiError> {
    require_admin(&auth_user)?;
    let status = body.status.trim().to_ascii_lowercase();
    if !matches!(status.as_str(), "registering" | "paused" | "cancelled") {
        return Err(ApiError::BadRequest(
            "status must be registering, paused, or cancelled".into(),
        ));
    }

    let tid = uuid::Uuid::parse_str(&tournament_id)
        .map_err(|_| ApiError::BadRequest("Invalid tournament id".into()))?;

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

    let tournaments = state.tournaments.read().await;
    let store = tournaments
        .get(&tournament_id)
        .ok_or_else(|| ApiError::NotFound("Tournament not found".into()))?;
    Ok(Json(AdminTournamentItem {
        id: store.id.clone(),
        name: store.state.config.name.clone(),
        buy_in: store.state.config.buy_in,
        guaranteed_prize: store.state.config.guaranteed_prize,
        prize_pool: store
            .state
            .prize_pool
            .max(store.state.config.guaranteed_prize),
        status: format!("{:?}", store.state.status).to_lowercase(),
        is_freeroll: store.state.config.is_freeroll,
        registered_players: store.state.players.len() as u32,
        max_players: store.state.config.max_players,
        table_max_players: store.table_max_players,
    }))
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
