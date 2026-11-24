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
    pub allow_addon: bool,
    pub addon_cost: u64,
    pub addon_chips: u64,
    /// Último nível (inclusive) para comprar o add-on.
    pub addon_max_level: u32,
    /// Nível de blinds em jogo (0 antes do início).
    pub current_level: u32,
    pub blind_levels: Vec<BlindLevelDto>,
    pub gameplay_ready: bool,
    /// `play` | `real`
    pub money_mode: String,
    /// `holdem` | `short_deck` | `omaha` | `pineapple_draw`
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

fn gameplay_ready(store: &crate::tournament_store::TournamentStore) -> bool {
    matches!(
        store.state.status,
        poker_engine::tournament_engine::TournamentStatus::Running
            | poker_engine::tournament_engine::TournamentStatus::Paused
    ) && store.live_table_id.is_some()
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
        rebuy_cost: store.rebuy_cost(),
        rebuy_chips: store.rebuy_chips(),
        rebuy_max_count: cfg.rebuy_max_count,
        rebuy_stack_threshold: cfg.rebuy_stack_threshold,
        rebuy_max_level: cfg.rebuy_max_level,
        allow_addon: cfg.allow_addon,
        addon_cost: store.addon_cost(),
        addon_chips: store.addon_chips(),
        addon_max_level: store.addon_max_level(),
        current_level: store.state.current_level,
        blind_levels: cfg
            .blind_levels
            .iter()
            .map(|b| BlindLevelDto {
                level: b.level,
                small_blind: b.small_blind,
                big_blind: b.big_blind,
                // Brazilian Pineapple não tem ante: só small e big blind.
                ante: if poker_engine::types::PokerVariant::parse(store.active_poker_variant())
                    == poker_engine::types::PokerVariant::PineappleDraw
                {
                    0
                } else {
                    b.ante
                },
                duration_minutes: b.duration_minutes,
            })
            .collect(),
        gameplay_ready: gameplay_ready(store),
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

#[cfg(test)]
mod response_tests {
    use super::*;
    use crate::tournament_store::TournamentStore;
    use poker_engine::tournament_engine::{TournamentConfig, TournamentStatus};

    #[test]
    fn gameplay_ready_requires_active_status_and_live_table() {
        let mut store = TournamentStore::new("tournament".into(), TournamentConfig::default());
        store.live_table_id = Some("table".into());
        assert!(!gameplay_ready(&store));

        store.state.status = TournamentStatus::Running;
        assert!(gameplay_ready(&store));

        store.live_table_id = None;
        assert!(!gameplay_ready(&store));
    }

    #[test]
    fn pineapple_has_no_ante_in_any_level_or_money_mode() {
        for mode in ["play", "real"] {
            for variant in ["holdem", "omaha", "short_deck", "pineapple_draw"] {
                let config = TournamentConfig {
                    blind_levels: [100, 200, 1000]
                        .into_iter()
                        .enumerate()
                        .map(|(i, bb)| poker_engine::tournament_engine::BlindLevel {
                            level: i as u32 + 1,
                            small_blind: bb / 2,
                            big_blind: bb,
                            ante: if i == 0 { 0 } else { 50 },
                            duration_minutes: 10,
                        })
                        .collect(),
                    ..TournamentConfig::default()
                };
                let store = TournamentStore::with_mode_and_variant(
                    "ante-levels".into(),
                    config,
                    mode.into(),
                    variant.into(),
                );
                let response = to_info(&store);
                for (actual, configured) in response
                    .blind_levels
                    .iter()
                    .zip(&store.state.config.blind_levels)
                {
                    assert_eq!(
                        actual.ante,
                        if variant == "pineapple_draw" {
                            0
                        } else {
                            configured.ante
                        }
                    );
                }
            }
        }
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
        // Cancelados/finalizados nunca aparecem como opção (lobby e dropdown de bots).
        .filter(|s| {
            matches!(
                s.state.status,
                poker_engine::tournament_engine::TournamentStatus::Registering
                    | poker_engine::tournament_engine::TournamentStatus::Paused
                    | poker_engine::tournament_engine::TournamentStatus::Running
            )
        })
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

#[derive(Debug, Serialize)]
pub struct MyRegistrationResponse {
    pub registered: bool,
    pub eliminated: bool,
    pub rebuys: u32,
    pub addon_done: bool,
    /// O motor aceitaria um rebuy agora (período, limite e elegibilidade).
    pub can_rebuy: bool,
    /// O motor aceitaria o add-on agora.
    pub can_addon: bool,
}

/// GET /api/tournament/:id/registration — inscrição do autenticado e o que ele
/// pode comprar agora (rebuy e add-on), sem alterar nada.
pub async fn my_registration(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Path(tournament_id): Path<String>,
) -> Result<Json<MyRegistrationResponse>, ApiError> {
    let tournaments = state.tournaments.read().await;
    let store = tournaments
        .get(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;
    let Some(entry) = store.state.players.get(&auth_user.user_id) else {
        return Ok(Json(MyRegistrationResponse {
            registered: false,
            eliminated: false,
            rebuys: 0,
            addon_done: false,
            can_rebuy: false,
            can_addon: false,
        }));
    };
    let can_rebuy = store.state.config.allow_rebuy && {
        let mut probe = store.state.clone();
        poker_engine::tournament_engine::process_rebuy(&mut probe, &auth_user.user_id).is_ok()
    };
    let can_addon = store.state.config.allow_addon && {
        let mut probe = store.state.clone();
        poker_engine::tournament_engine::process_addon_until(
            &mut probe,
            &auth_user.user_id,
            store.addon_chips(),
            store.addon_cost(),
            store.addon_max_level(),
        )
        .is_ok()
    };
    Ok(Json(MyRegistrationResponse {
        registered: true,
        eliminated: entry.eliminated_at.is_some(),
        rebuys: entry.rebuys,
        addon_done: entry.addon_done,
        can_rebuy,
        can_addon,
    }))
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
    if mode_is_real {
        crate::responsible_gaming::ensure_real_money_allowed(&state.db, &auth_user.user_id).await?;
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
        if let Err(e) = crate::wallet::debit_wallet(
            &mut *tx,
            &auth_user.user_id,
            buy_in_i,
            kind,
            crate::wallet::LedgerReason::TournamentBuyIn,
            Some(&tournament_id),
        )
        .await
        {
            store.state.players.remove(&auth_user.user_id);
            return Err(e);
        }
    }

    // Taxa 15% por cima do buy-in: debita junto e registra o NGR do agente direto.
    // Freeroll (buy-in zero) não tem fee.
    let fee_cents =
        i64::try_from(poker_engine::tournament_engine::entry_fee_cents(buy_in)).unwrap_or(0);
    if fee_cents > 0 {
        let kind = crate::wallet::mtt_kind_for_mode(mode);
        if let Err(e) = crate::wallet::debit_wallet(
            &mut *tx,
            &auth_user.user_id,
            fee_cents,
            kind,
            crate::wallet::LedgerReason::TournamentFee,
            Some(&tournament_id),
        )
        .await
        {
            store.state.players.remove(&auth_user.user_id);
            return Err(e);
        }
        let payer = uuid::Uuid::parse_str(&auth_user.user_id).map_err(|_| {
            store.state.players.remove(&auth_user.user_id);
            ApiError::BadRequest("Invalid player id".into())
        })?;
        let source_reference_id = uuid::Uuid::parse_str(&tournament_id).map_err(|_| {
            store.state.players.remove(&auth_user.user_id);
            ApiError::BadRequest("Invalid tournament id".into())
        })?;
        if let Err(e) = crate::estrutura::distribute_fee(
            &mut tx,
            payer,
            fee_cents,
            source_reference_id,
            mode.as_str(),
        )
        .await
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
            total_buyins = $4,
            total_fees = $5
        WHERE id = $1::uuid
        "#,
    )
    .bind(&tournament_id)
    .bind(store.state.prize_pool as i64)
    .bind(store.state.players_remaining as i32)
    .bind(store.state.players.len() as i32)
    .bind(i64::try_from(store.state.total_fees).unwrap_or(i64::MAX))
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
        gameplay_ready: gameplay_ready(store),
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
/// reembolso total (buy-in + fee 15%) e dedução identificada do fee no NGR.
/// Receitas e fechamentos anteriores permanecem no histórico. Pós-start: 403.
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
    let fee_cents = poker_engine::tournament_engine::entry_fee_cents(buy_in);
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
            crate::wallet::credit_wallet(
                &mut *tx,
                &auth_user.user_id,
                buy_in_i,
                kind,
                crate::wallet::LedgerReason::TournamentUnregisterRefund,
                Some(&tournament_id),
            )
            .await?;
        }
        if fee_i > 0 {
            crate::wallet::credit_wallet(
                &mut *tx,
                &auth_user.user_id,
                fee_i,
                kind,
                crate::wallet::LedgerReason::TournamentUnregisterRefund,
                Some(&tournament_id),
            )
            .await?;
            // Dedução idempotente somente das receitas desta inscrição.
            let payer = uuid::Uuid::parse_str(&auth_user.user_id)
                .map_err(|_| ApiError::BadRequest("Invalid player id".into()))?;
            let tournament_ref = uuid::Uuid::parse_str(&tournament_id)
                .map_err(|_| ApiError::BadRequest("Invalid tournament id".into()))?;
            crate::estrutura::refund_fee(&mut tx, payer, tournament_ref).await?;
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

#[derive(Debug, Deserialize)]
pub struct ChipPurchaseBody {
    pub tournament_id: String,
}

#[derive(Debug, Serialize)]
pub struct ChipPurchaseResponse {
    pub tournament_id: String,
    pub player_id: String,
    /// `rebuy` | `addon`
    pub kind: &'static str,
    pub chips: u64,
    /// Valor debitado da carteira do torneio (sem taxa em rebuy e add-on).
    pub cost_cents: i64,
    /// `seated` = voltou a uma mesa agora; `next_hand` = fichas entram no começo
    /// da próxima mão em que o jogador estiver.
    pub delivery: &'static str,
    pub table_id: Option<String>,
    pub rebuys: u32,
    pub addon_done: bool,
    pub prize_pool: u64,
}

/// Assento novo (índice da mesa viva, cadeira) de quem voltou e a mesa do jogador.
type Delivered = (Option<(usize, i16)>, Option<String>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChipPurchase {
    Rebuy,
    Addon,
}

impl ChipPurchase {
    fn as_str(self) -> &'static str {
        match self {
            ChipPurchase::Rebuy => "rebuy",
            ChipPurchase::Addon => "addon",
        }
    }
}

/// Onde o jogador eliminado volta a sentar: a mesa viva com menos jogadores e
/// o menor assento livre. Mesa final cheia recusa o rebuy.
async fn choose_return_seat(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tournament_id: &str,
    live_table_ids: &[String],
    seat_cap: i16,
) -> Result<(usize, i16), ApiError> {
    let occupied: Vec<(String, i16)> = sqlx::query_as(
        "SELECT table_id::text, seat FROM tournament_seats \
         WHERE tournament_id = $1::uuid AND status = 'ACTIVE' FOR UPDATE",
    )
    .bind(tournament_id)
    .fetch_all(&mut **tx)
    .await?;
    let mut best: Option<(usize, usize, i16)> = None;
    for (idx, table_id) in live_table_ids.iter().enumerate() {
        let taken: Vec<i16> = occupied
            .iter()
            .filter(|(t, _)| t == table_id)
            .map(|(_, seat)| *seat)
            .collect();
        let Some(free) = (0..seat_cap).find(|s| !taken.contains(s)) else {
            continue;
        };
        if best.is_none_or(|(_, count, _)| taken.len() < count) {
            best = Some((idx, taken.len(), free));
        }
    }
    best.map(|(idx, _, seat)| (idx, seat)).ok_or_else(|| {
        ApiError::BadRequest("Sem assento livre nas mesas do torneio para o rebuy".into())
    })
}

/// Rebuy ou add-on: valida no motor, debita a carteira do torneio, entrega as
/// fichas (assento novo para quem foi eliminado; pendentes para a próxima mão
/// de quem está na mesa) e grava totais e auditoria numa transação só.
async fn purchase_chips(
    state: &AppState,
    user_id: &str,
    tournament_id: String,
    kind: ChipPurchase,
) -> Result<ChipPurchaseResponse, ApiError> {
    use poker_engine::tournament_engine as engine;

    let mut tournaments = state.tournaments.write().await;
    let store = tournaments
        .get_mut(&tournament_id)
        .ok_or_else(|| ApiError::NotFound(format!("Tournament {tournament_id} not found")))?;
    let mode = if store.money_mode.eq_ignore_ascii_case("real") {
        crate::wallet::WalletMode::Real
    } else {
        crate::wallet::WalletMode::Play
    };
    let (player_name, was_eliminated) = store
        .state
        .players
        .get(user_id)
        .map(|e| (e.player_name.clone(), e.eliminated_at.is_some()))
        .ok_or_else(|| ApiError::BadRequest("Você não está inscrito neste torneio".into()))?;
    if matches!(mode, crate::wallet::WalletMode::Real) {
        crate::responsible_gaming::ensure_real_money_allowed(&state.db, user_id).await?;
    }
    let (chips, cost, addon_max_level) = match kind {
        ChipPurchase::Rebuy => (store.rebuy_chips(), store.rebuy_cost(), 0),
        ChipPurchase::Addon => (
            store.addon_chips(),
            store.addon_cost(),
            store.addon_max_level(),
        ),
    };
    let chips_i =
        i64::try_from(chips).map_err(|_| ApiError::BadRequest("Fichas inválidas".into()))?;
    let cost_i = i64::try_from(cost).map_err(|_| ApiError::BadRequest("Preço inválido".into()))?;
    let seat_cap = i16::from(if store.live_table_ids.len() > 1 {
        store.table_max_players
    } else {
        store
            .final_table_max_players
            .unwrap_or(store.table_max_players)
    });
    let live_table_ids = store.live_table_ids.clone();
    let tournament_name = store.state.config.name.clone();

    let before = store.state.clone();
    let engine_result = match kind {
        ChipPurchase::Rebuy => engine::process_rebuy(&mut store.state, user_id),
        ChipPurchase::Addon => {
            engine::process_addon_until(&mut store.state, user_id, chips, cost, addon_max_level)
        }
    };
    engine_result.map_err(ApiError::BadRequest)?;

    let persisted: Result<Delivered, ApiError> = async {
        let mut tx = state.db.begin().await?;
        if cost_i > 0 {
            if matches!(mode, crate::wallet::WalletMode::Play) {
                crate::wallet::ensure_pm_daily_reset(&mut *tx, user_id).await?;
            }
            crate::wallet::debit_wallet(
                &mut *tx,
                user_id,
                cost_i,
                crate::wallet::mtt_kind_for_mode(mode),
                match kind {
                    ChipPurchase::Rebuy => crate::wallet::LedgerReason::TournamentRebuy,
                    ChipPurchase::Addon => crate::wallet::LedgerReason::TournamentAddon,
                },
                Some(&tournament_id),
            )
            .await?;
        }
        let (seat, table_id) = if was_eliminated {
            let (idx, seat) =
                choose_return_seat(&mut tx, &tournament_id, &live_table_ids, seat_cap).await?;
            sqlx::query(
                "INSERT INTO tournament_seats (tournament_id, table_id, seat, player_id, player_name, stack) \
                 VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6)",
            )
            .bind(&tournament_id)
            .bind(&live_table_ids[idx])
            .bind(seat)
            .bind(user_id)
            .bind(&player_name)
            .bind(chips_i)
            .execute(&mut *tx)
            .await?;
            (Some((idx, seat)), Some(live_table_ids[idx].clone()))
        } else {
            // Ainda na mesa: as fichas esperam o começo da próxima mão.
            let table: Option<(String,)> = sqlx::query_as(
                "UPDATE tournament_seats SET pending_chips = pending_chips + $3 \
                 WHERE tournament_id = $1::uuid AND player_id = $2 AND status = 'ACTIVE' \
                 RETURNING table_id::text",
            )
            .bind(&tournament_id)
            .bind(user_id)
            .bind(chips_i)
            .fetch_optional(&mut *tx)
            .await?;
            let Some((table_id,)) = table else {
                return Err(ApiError::BadRequest(
                    "Você acabou de ser eliminado: aguarde um instante e use o rebuy".into(),
                ));
            };
            (None, Some(table_id))
        };
        let player_rows = match kind {
            ChipPurchase::Rebuy => sqlx::query(
                "UPDATE tournament_players SET rebuys = rebuys + 1, rebuy_paid = rebuy_paid + $3 \
                 WHERE tournament_id = $1::uuid AND player_id = $2",
            ),
            ChipPurchase::Addon => sqlx::query(
                "UPDATE tournament_players SET addon_done = TRUE, addon_paid = addon_paid + $3 \
                 WHERE tournament_id = $1::uuid AND player_id = $2 AND addon_done = FALSE",
            ),
        }
        .bind(&tournament_id)
        .bind(user_id)
        .bind(cost_i)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if player_rows != 1 {
            return Err(ApiError::BadRequest(
                "Inscrição não encontrada para esta compra".into(),
            ));
        }
        let tournament_rows = sqlx::query(
            "UPDATE tournaments SET prize_pool = $2, players_remaining = $3, \
                    total_rebuys = $4, total_addons = $5 \
             WHERE id = $1::uuid AND status = 'running'",
        )
        .bind(&tournament_id)
        .bind(i64::try_from(store.state.prize_pool).unwrap_or(i64::MAX))
        .bind(store.state.players_remaining as i32)
        .bind(i64::try_from(store.state.total_rebuys).unwrap_or(i64::MAX))
        .bind(i64::try_from(store.state.total_addons).unwrap_or(i64::MAX))
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if tournament_rows != 1 {
            return Err(ApiError::BadRequest("Torneio não está em andamento".into()));
        }
        sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(match kind {
                ChipPurchase::Rebuy => "MTT_REBUY",
                ChipPurchase::Addon => "MTT_ADDON",
            })
            .bind(serde_json::json!({
                "tournament_id": tournament_id,
                "cost": cost_i,
                "chips": chips_i,
                "level": store.state.current_level,
                "was_eliminated": was_eliminated,
                "money_mode": mode.as_str(),
            }))
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok((seat, table_id))
    }
    .await;

    let (seat, table_id) = match persisted {
        Ok(done) => done,
        Err(error) => {
            // O banco desfez a transação; o motor volta ao estado anterior.
            store.state = before;
            return Err(error);
        }
    };
    if let (Some((idx, seat)), Some(entry)) = (seat, store.state.players.get_mut(user_id)) {
        entry.table_id = Some(idx as u32);
        entry.seat = Some(seat.max(0) as u32);
    }
    let (rebuys, addon_done) = store
        .state
        .players
        .get(user_id)
        .map(|e| (e.rebuys, e.addon_done))
        .unwrap_or((0, false));
    let prize_pool = store
        .state
        .prize_pool
        .max(store.state.config.guaranteed_prize);
    drop(tournaments);

    // Quem voltou do zero senta já; a mão em andamento segue sem ele.
    if let (Some((idx, seat)), Some(table_id)) = (seat, table_id.as_ref()) {
        let handle = state.active_tables.read().await.get(table_id).cloned();
        match handle {
            Some(handle) => {
                let (tx_resp, rx_resp) = tokio::sync::oneshot::channel();
                let _ = handle
                    .tx_cmd
                    .send(crate::game_actor::PlayerCommand::Sit {
                        player_id: user_id.to_string(),
                        username: player_name.clone(),
                        seat: Some(seat.max(0) as usize),
                        chips,
                        respond_to: tx_resp,
                    })
                    .await;
                let _ = tokio::time::timeout(std::time::Duration::from_secs(5), rx_resp).await;
            }
            None => {
                crate::tournament_actor::ensure_tournament_actor(
                    state,
                    &tournament_id,
                    table_id,
                    idx as u32,
                    format!("{tournament_name} mesa {}", idx + 1),
                )
                .await;
            }
        }
    }

    Ok(ChipPurchaseResponse {
        tournament_id,
        player_id: user_id.to_string(),
        kind: kind.as_str(),
        chips,
        cost_cents: cost_i,
        delivery: if seat.is_some() {
            "seated"
        } else {
            "next_hand"
        },
        table_id,
        rebuys,
        addon_done,
        prize_pool,
    })
}

/// POST /api/tournament/rebuy — eliminado volta com stack novo; stack baixo
/// (limiar do evento) soma as fichas na próxima mão.
pub async fn rebuy_player(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<ChipPurchaseBody>,
) -> Result<Json<ChipPurchaseResponse>, ApiError> {
    purchase_chips(
        &state,
        &auth_user.user_id,
        body.tournament_id,
        ChipPurchase::Rebuy,
    )
    .await
    .map(Json)
}

/// POST /api/tournament/addon — uma compra por jogador ainda no torneio, até o
/// nível definido no evento; as fichas entram na próxima mão.
pub async fn addon_player(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<ChipPurchaseBody>,
) -> Result<Json<ChipPurchaseResponse>, ApiError> {
    purchase_chips(
        &state,
        &auth_user.user_id,
        body.tournament_id,
        ChipPurchase::Addon,
    )
    .await
    .map(Json)
}
