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
    /// Taxa 15% cobrada por cima do buy-in (0 em freeroll).
    pub fee_cents: i64,
    /// Total debitado da carteira (buy-in + fee).
    pub total_debited_cents: i64,
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
    /// Taxa 15% por cima do buy-in (0 em freeroll).
    pub fee_cents: u64,
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
    /// `holdem` | `short_deck` | `short_deck_omaha` | `ultimate_pineapple`
    pub poker_variant: String,
    /// Variant activated when the final-table player threshold is reached.
    pub final_table_variant: Option<String>,
    pub final_table_max_players: Option<u8>,
    pub scheduled_start_at: Option<i64>,
    pub auto_start_min_players: Option<i32>,
    pub live_table_id: Option<String>,
    /// As 3 mesas físicas do torneio (ordem dos índices do motor).
    pub live_table_ids: Vec<String>,
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
        fee_cents: poker_engine::tournament_engine::entry_fee_cents(cfg.buy_in),
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
        gameplay_ready: store.live_table_id.is_some(),
        money_mode: store.money_mode.clone(),
        poker_variant: store.poker_variant.clone(),
        final_table_variant: store.final_table_variant.clone(),
        final_table_max_players: store.final_table_max_players,
        scheduled_start_at: store.scheduled_start_at,
        auto_start_min_players: store.auto_start_min_players,
        live_table_id: store.live_table_id.clone(),
        live_table_ids: store.live_table_ids.clone(),
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

/// GET /api/tournament/:id/registration — diz se o autenticado está inscrito.
pub async fn my_registration(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Path(tournament_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tournaments = state.tournaments.read().await;
    let store = tournaments
        .get(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;
    Ok(Json(
        serde_json::json!({"registered": store.state.players.contains_key(&auth_user.user_id)}),
    ))
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

    // Taxa 15% por cima do buy-in: debita junto e reparte 18/12/70 na rede.
    // Freeroll (buy-in zero) não tem fee.
    let fee_cents =
        i64::try_from(poker_engine::tournament_engine::entry_fee_cents(buy_in)).unwrap_or(0);
    if fee_cents > 0 {
        let kind = crate::wallet::mtt_kind_for_mode(mode);
        if let Err(e) =
            crate::wallet::debit_wallet(&mut *tx, &auth_user.user_id, fee_cents, kind).await
        {
            store.state.players.remove(&auth_user.user_id);
            return Err(e);
        }
        let payer = uuid::Uuid::parse_str(&auth_user.user_id).map_err(|_| {
            store.state.players.remove(&auth_user.user_id);
            ApiError::BadRequest("Invalid player id".into())
        })?;
        let week_start: i64 = sqlx::query_scalar(
            "SELECT EXTRACT(EPOCH FROM date_trunc('week', timezone('America/Sao_Paulo', now())))::BIGINT",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| {
            store.state.players.remove(&auth_user.user_id);
            ApiError::Internal("week clock unavailable".into())
        })?;
        if let Err(e) =
            crate::estrutura::distribute_fee(&mut tx, payer, fee_cents, week_start).await
        {
            store.state.players.remove(&auth_user.user_id);
            return Err(ApiError::Internal(format!("fee split failed: {e}")));
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
        gameplay_ready: store.live_table_id.is_some(),
        fee_cents,
        total_debited_cents: buy_in as i64 + fee_cents,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UnregisterBody {
    pub tournament_id: String,
}

#[derive(Debug, Serialize)]
pub struct UnregisterResponse {
    pub tournament_id: String,
    pub player_id: String,
    pub refunded_buy_in_cents: i64,
    pub refunded_fee_cents: i64,
}

/// POST /api/tournament/unregister — cancela a inscrição PRÉ-START com
/// reembolso total (buy-in + fee 15%) e anulação das linhas de fee do pagador
/// (estorna pontos onde houve crédito). Pós-start: 403.
pub async fn unregister_player(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<UnregisterBody>,
) -> Result<Json<UnregisterResponse>, ApiError> {
    let tournament_id = body.tournament_id.clone();

    let mut tournaments = state.tournaments.write().await;
    let store = tournaments
        .get_mut(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;

    let buy_in = store.state.config.buy_in;
    let fee_cents =
        u64::try_from(poker_engine::tournament_engine::entry_fee_cents(buy_in)).unwrap_or(0);
    let mode = if store.money_mode.eq_ignore_ascii_case("real") {
        crate::wallet::WalletMode::Real
    } else {
        crate::wallet::WalletMode::Play
    };
    let kind = crate::wallet::mtt_kind_for_mode(mode);

    // Snapshot (entrada + contadores) p/ desfazer no motor se o banco falhar.
    let engine_entry = store.state.players.get(&auth_user.user_id).cloned();
    let engine_snapshot = (
        store.state.total_buyins,
        store.state.total_fees,
        store.state.prize_pool,
        store.state.players_remaining,
    );
    let buy_in_i =
        i64::try_from(buy_in).map_err(|_| ApiError::BadRequest("Invalid buy-in".into()))?;
    let fee_i = i64::try_from(fee_cents).map_err(|_| ApiError::BadRequest("Invalid fee".into()))?;

    if let Err(e) =
        poker_engine::tournament_engine::unregister_player(&mut store.state, &auth_user.user_id)
    {
        return Err(ApiError::BadRequest(e));
    }

    let db_result: Result<(), ApiError> = async {
        let mut tx = state.db.begin().await?;
        if buy_in_i > 0 {
            crate::wallet::credit_wallet(&mut *tx, &auth_user.user_id, buy_in_i, kind).await?;
        }
        if fee_i > 0 {
            crate::wallet::credit_wallet(&mut *tx, &auth_user.user_id, fee_i, kind).await?;
            // Anula o fee: estorna pontos creditados e apaga as linhas do pagador.
            let payer = uuid::Uuid::parse_str(&auth_user.user_id)
                .map_err(|_| ApiError::BadRequest("Invalid player id".into()))?;
            sqlx::query(
                "UPDATE users SET estrutura_points = GREATEST(estrutura_points - el.commission_cents, 0) \
                 FROM estrutura_ledger el \
                 WHERE el.source_type = 'fee' AND el.source_user_id = $1 AND el.eligible \
                   AND users.id = el.beneficiary_user_id",
            )
            .bind(payer)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "DELETE FROM estrutura_ledger WHERE source_type = 'fee' AND source_user_id = $1",
            )
            .bind(payer)
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query("DELETE FROM tournament_players WHERE tournament_id = $1::uuid AND player_id = $2")
            .bind(&tournament_id)
            .bind(&auth_user.user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "UPDATE tournaments SET prize_pool = $2, players_remaining = $3, total_buyins = $4, total_fees = $5 WHERE id = $1::uuid",
        )
        .bind(&tournament_id)
        .bind(store.state.prize_pool as i64)
        .bind(store.state.players_remaining as i32)
        .bind(store.state.players.len() as i32)
        .bind(store.state.total_fees as i64)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, 'MTT_UNREGISTER', $2)",
        )
        .bind(&auth_user.user_id)
        .bind(serde_json::json!({"tournament_id": tournament_id, "buy_in": buy_in_i, "fee": fee_i}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
    .await;
    if let Err(e) = db_result {
        // Desfaz no motor (o banco deu rollback sozinho).
        if let Some(entry) = engine_entry {
            store.state.players.insert(auth_user.user_id.clone(), entry);
        }
        store.state.total_buyins = engine_snapshot.0;
        store.state.total_fees = engine_snapshot.1;
        store.state.prize_pool = engine_snapshot.2;
        store.state.players_remaining = engine_snapshot.3;
        return Err(e);
    }

    Ok(Json(UnregisterResponse {
        tournament_id,
        player_id: auth_user.user_id,
        refunded_buy_in_cents: buy_in_i,
        refunded_fee_cents: fee_i,
    }))
}
