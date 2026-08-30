//! Tournament handlers — list / get / register (MVP; gameplay MTT = fase 2).

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub tournament_id: String,
    /// Optional; ignored when it does not match the authenticated user.
    pub player_id: Option<String>,
    pub player_name: Option<String>,
    /// `play` (default) debits Play Money MTT; `real` debits Jogo Real.
    #[serde(default)]
    pub wallet_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub tournament_id: String,
    pub player_id: String,
    pub stack: u64,
    pub registered: bool,
    pub gameplay_ready: bool,
}

#[derive(Debug, Serialize)]
pub struct BlindLevelDto {
    pub level: u32,
    pub small_blind: u64,
    pub big_blind: u64,
    pub ante: u64,
    pub duration_minutes: u32,
}

#[derive(Debug, Serialize)]
pub struct TournamentInfoResponse {
    pub id: String,
    pub name: String,
    pub buy_in: u64,
    pub starting_stack: u64,
    pub max_players: u32,
    pub table_max_players: u8,
    pub registered_players: u32,
    pub status: String,
    pub players_remaining: u32,
    pub prize_pool: u64,
    pub guaranteed_prize: u64,
    pub is_freeroll: bool,
    pub allow_rebuy: bool,
    pub rebuy_cost: u64,
    pub rebuy_chips: u64,
    pub rebuy_max_count: u32,
    pub rebuy_stack_threshold: u64,
    pub rebuy_max_level: u32,
    pub blind_levels: Vec<BlindLevelDto>,
    pub gameplay_ready: bool,
    /// `play` | `real`
    pub money_mode: String,
    /// `holdem` | `short_deck` | `short_deck_omaha`
    pub poker_variant: String,
}

fn status_string(status: &poker_engine::tournament_engine::TournamentStatus) -> String {
    match status {
        poker_engine::tournament_engine::TournamentStatus::Registering => "registering".into(),
        poker_engine::tournament_engine::TournamentStatus::Running => "running".into(),
        poker_engine::tournament_engine::TournamentStatus::Paused => "paused".into(),
        poker_engine::tournament_engine::TournamentStatus::Finished => "finished".into(),
        poker_engine::tournament_engine::TournamentStatus::Cancelled => "cancelled".into(),
    }
}

fn to_info(store: &crate::tournament_store::TournamentStore) -> TournamentInfoResponse {
    let cfg = &store.state.config;
    TournamentInfoResponse {
        id: store.id.clone(),
        name: cfg.name.clone(),
        buy_in: cfg.buy_in,
        starting_stack: cfg.starting_stack,
        max_players: cfg.max_players,
        table_max_players: store.table_max_players,
        registered_players: store.state.players.len() as u32,
        status: status_string(&store.state.status),
        players_remaining: store.state.players_remaining,
        prize_pool: store.state.prize_pool.max(cfg.guaranteed_prize),
        guaranteed_prize: cfg.guaranteed_prize,
        is_freeroll: cfg.is_freeroll,
        allow_rebuy: cfg.allow_rebuy,
        rebuy_cost: if cfg.rebuy_cost > 0 {
            cfg.rebuy_cost
        } else {
            cfg.buy_in
        },
        rebuy_chips: if cfg.rebuy_chips > 0 {
            cfg.rebuy_chips
        } else {
            cfg.starting_stack
        },
        rebuy_max_count: cfg.rebuy_max_count,
        rebuy_stack_threshold: cfg.rebuy_stack_threshold,
        rebuy_max_level: cfg.rebuy_max_level,
        blind_levels: cfg
            .blind_levels
            .iter()
            .map(|b| BlindLevelDto {
                level: b.level,
                small_blind: b.small_blind,
                big_blind: b.big_blind,
                ante: b.ante,
                duration_minutes: b.duration_minutes,
            })
            .collect(),
        // MTT hands not wired to TableActor yet.
        gameplay_ready: false,
        money_mode: store.money_mode.clone(),
        poker_variant: store.poker_variant.clone(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTournamentsQuery {
    pub mode: Option<String>,
}

/// GET /api/lobby/tournaments?mode=play|real
pub async fn list_tournaments(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListTournamentsQuery>,
) -> Result<Json<Vec<TournamentInfoResponse>>, ApiError> {
    let mode = crate::wallet::WalletMode::parse(query.mode.as_deref());
    let want = mode.as_str();
    let tournaments = state.tournaments.read().await;
    let mut list: Vec<_> = tournaments
        .values()
        .filter(|s| s.money_mode.eq_ignore_ascii_case(want))
        .map(to_info)
        .collect();
    list.sort_by(|a, b| {
        b.is_freeroll
            .cmp(&a.is_freeroll)
            .then(a.buy_in.cmp(&b.buy_in))
            .then(a.name.cmp(&b.name))
    });
    Ok(Json(list))
}

/// GET /api/tournament/{id}
pub async fn get_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> Result<Json<TournamentInfoResponse>, ApiError> {
    let tournaments = state.tournaments.read().await;
    let store = tournaments
        .get(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;
    Ok(Json(to_info(store)))
}

/// POST /api/tournament/register
pub async fn register_player(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<RegisterBody>,
) -> Result<Json<RegisterResponse>, ApiError> {
    if let Some(ref pid) = body.player_id {
        if pid != &auth_user.user_id {
            return Err(ApiError::Forbidden(
                "Tournament registration must use the authenticated player identity".to_string(),
            ));
        }
    }

    let tournament_id = body.tournament_id.clone();

    let mut tournaments = state.tournaments.write().await;
    let store = tournaments
        .get_mut(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;

    let buy_in = store.state.config.buy_in;
    let starting_stack = store.state.config.starting_stack;
    let mode = crate::wallet::WalletMode::parse(body.wallet_mode.as_deref());
    let tourney_is_real = store.money_mode.eq_ignore_ascii_case("real");
    let mode_is_real = matches!(mode, crate::wallet::WalletMode::Real);
    if tourney_is_real != mode_is_real {
        return Err(ApiError::BadRequest(
            if tourney_is_real {
                "Este torneio é de Jogo Real. Fichas Play Money não podem ser usadas. Mude o modo no header para Jogo Real."
            } else {
                "Este torneio é de Play Money. Saldo de Jogo Real não entra aqui. Mude o modo no header para Play Money."
            }
            .into(),
        ));
    }

    poker_engine::tournament_engine::register_player(
        &mut store.state,
        &auth_user.user_id,
        &auth_user.username,
    )
    .map_err(ApiError::BadRequest)?;

    let mut tx = state.db.begin().await.inspect_err(|_| {
        store.state.players.remove(&auth_user.user_id);
    })?;

    if buy_in > 0 {
        let buy_in_i = i64::try_from(buy_in).map_err(|_| {
            store.state.players.remove(&auth_user.user_id);
            ApiError::BadRequest("Invalid buy-in".into())
        })?;
        let kind = crate::wallet::mtt_kind_for_mode(mode);
        if let Err(e) = crate::wallet::ensure_pm_daily_reset(&mut *tx, &auth_user.user_id).await {
            store.state.players.remove(&auth_user.user_id);
            return Err(e);
        }
        if let Err(e) =
            crate::wallet::debit_wallet(&mut *tx, &auth_user.user_id, buy_in_i, kind).await
        {
            store.state.players.remove(&auth_user.user_id);
            return Err(e);
        }
    }

    sqlx::query(
        r#"
        INSERT INTO tournament_players
            (tournament_id, player_id, player_name, stack, registered_at)
        VALUES ($1::uuid, $2, $3, $4, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ON CONFLICT (tournament_id, player_id) DO NOTHING
        "#,
    )
    .bind(&tournament_id)
    .bind(&auth_user.user_id)
    .bind(&auth_user.username)
    .bind(starting_stack as i64)
    .execute(&mut *tx)
    .await
    .inspect_err(|_| {
        store.state.players.remove(&auth_user.user_id);
    })?;

    sqlx::query(
        r#"
        UPDATE tournaments
        SET prize_pool = $2,
            players_remaining = $3,
            total_buyins = $4
        WHERE id = $1::uuid
        "#,
    )
    .bind(&tournament_id)
    .bind(store.state.prize_pool as i64)
    .bind(store.state.players_remaining as i32)
    .bind(store.state.players.len() as i32)
    .execute(&mut *tx)
    .await
    .inspect_err(|_| {
        store.state.players.remove(&auth_user.user_id);
    })?;

    tx.commit().await.inspect_err(|_| {
        store.state.players.remove(&auth_user.user_id);
    })?;

    Ok(Json(RegisterResponse {
        tournament_id,
        player_id: auth_user.user_id,
        stack: starting_stack,
        registered: true,
        gameplay_ready: false,
    }))
}
