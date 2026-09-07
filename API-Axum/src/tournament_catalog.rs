//! Load official tournament catalog from PostgreSQL into in-memory stores.

use std::collections::HashMap;

use poker_engine::tournament_engine::{
    BlindLevel, TournamentConfig, TournamentSpeed, TournamentStatus,
};
use sqlx::PgPool;

use crate::tournament_store::TournamentStore;

#[derive(Debug, sqlx::FromRow)]
struct TournamentRow {
    id: uuid::Uuid,
    name: String,
    buy_in: i64,
    starting_stack: i64,
    max_players: i32,
    table_max_players: i16,
    late_registration: bool,
    late_reg_max_level: i32,
    speed: String,
    status: String,
    prize_pool: i64,
    current_level: i32,
    players_remaining: i32,
    total_buyins: i32,
    guaranteed_prize: i64,
    is_freeroll: bool,
    rebuy_cost: i64,
    rebuy_chips: i64,
    rebuy_max_count: i32,
    rebuy_stack_threshold: i64,
    rebuy_max_level: i32,
    allow_rebuy: bool,
    blind_levels: serde_json::Value,
    game_type: String,
    money_mode: String,
    poker_variant: String,
    final_table_variant: Option<String>,
    final_table_max_players: Option<i16>,
    scheduled_start_at: Option<i64>,
    auto_start_min_players: Option<i32>,
    live_table_id: Option<uuid::Uuid>,
    live_table_ids: Option<Vec<uuid::Uuid>>,
}

fn parse_speed(raw: &str) -> TournamentSpeed {
    match raw.to_ascii_lowercase().as_str() {
        "turbo" => TournamentSpeed::Turbo,
        "slow" => TournamentSpeed::Slow,
        _ => TournamentSpeed::Normal,
    }
}

fn parse_status(raw: &str) -> TournamentStatus {
    match raw.to_ascii_lowercase().as_str() {
        "running" => TournamentStatus::Running,
        "paused" => TournamentStatus::Paused,
        "finished" => TournamentStatus::Finished,
        "cancelled" => TournamentStatus::Cancelled,
        _ => TournamentStatus::Registering,
    }
}

fn parse_blinds(value: &serde_json::Value) -> Vec<BlindLevel> {
    serde_json::from_value::<Vec<BlindLevel>>(value.clone()).unwrap_or_default()
}

fn row_to_store(row: TournamentRow) -> TournamentStore {
    let id = row.id.to_string();
    let config = TournamentConfig {
        name: row.name,
        game_type: row.game_type,
        buy_in: row.buy_in.max(0) as u64,
        starting_stack: row.starting_stack.max(0) as u64,
        max_players: row.max_players.max(0) as u32,
        speed: parse_speed(&row.speed),
        blind_levels: parse_blinds(&row.blind_levels),
        prize_pool_pct: 1.0,
        prize_distribution: vec![0.50, 0.30, 0.20],
        late_registration: row.late_registration,
        late_registration_max_level: row.late_reg_max_level.max(0) as u32,
        allow_rebuy: row.allow_rebuy,
        allow_addon: false,
        rebuy_max_level: row.rebuy_max_level.max(0) as u32,
        guaranteed_prize: row.guaranteed_prize.max(0) as u64,
        is_freeroll: row.is_freeroll,
        rebuy_cost: row.rebuy_cost.max(0) as u64,
        rebuy_chips: row.rebuy_chips.max(0) as u64,
        rebuy_max_count: row.rebuy_max_count.max(0) as u32,
        rebuy_stack_threshold: row.rebuy_stack_threshold.max(0) as u64,
    };

    let money_mode = if row.money_mode.eq_ignore_ascii_case("real") {
        "real".into()
    } else {
        "play".into()
    };
    let mut store =
        TournamentStore::with_mode_and_variant(id, config, money_mode, row.poker_variant);
    store.table_max_players = row.table_max_players.clamp(2, 9) as u8;
    store.final_table_variant = row
        .final_table_variant
        .map(|variant| variant.to_ascii_lowercase())
        .filter(|variant| variant == "short_deck");
    store.final_table_max_players = row
        .final_table_max_players
        .map(|players| players.clamp(2, 8) as u8);
    store.scheduled_start_at = row.scheduled_start_at;
    store.auto_start_min_players = row.auto_start_min_players;
    store.live_table_id = row.live_table_id.map(|id| id.to_string());
    let mut live_ids: Vec<String> = row
        .live_table_ids
        .unwrap_or_default()
        .into_iter()
        .map(|id| id.to_string())
        .collect();
    if live_ids.is_empty() {
        if let Some(ref legacy) = store.live_table_id {
            live_ids.push(legacy.clone());
        }
    } else if store.live_table_id.is_none() {
        store.live_table_id = live_ids.first().cloned();
    }
    store.live_table_ids = live_ids;
    store.state.status = parse_status(&row.status);
    store.state.current_level = row.current_level.max(0) as u32;
    store.state.players_remaining = row.players_remaining.max(0) as u32;
    // DB `total_buyins` is entry count; engine tracks money collected.
    let entries = row.total_buyins.max(0) as u64;
    store.state.total_buyins = entries.saturating_mul(store.state.config.buy_in);
    store.state.prize_pool =
        (row.prize_pool.max(0) as u64).max(store.state.config.guaranteed_prize);
    store
}

/// Reidrata `state.players` do banco (inscrições + assentos vivos).
/// Sem isso, um restart esvazia o motor: eliminações falham com
/// "Jogador não encontrado" e `players_remaining` congela.
async fn hydrate_players(pool: &PgPool, store: &mut TournamentStore) -> Result<(), sqlx::Error> {
    use poker_engine::tournament_engine::PlayerTournamentEntry;
    let entries: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT player_id, player_name, registered_at FROM tournament_players WHERE tournament_id = $1::uuid",
    )
    .bind(&store.id)
    .fetch_all(pool)
    .await?;
    if entries.is_empty() {
        return Ok(());
    }
    let seats: Vec<(String, String, i16, i64, String)> = sqlx::query_as(
        "SELECT player_id, table_id::text, seat, stack, status FROM tournament_seats WHERE tournament_id = $1::uuid",
    )
    .bind(&store.id)
    .fetch_all(pool)
    .await?;
    let mut seat_of = std::collections::HashMap::new();
    for (pid, table_id, seat, stack, status) in seats {
        seat_of.insert(pid, (table_id, seat, stack, status));
    }
    let mut remaining = 0u32;
    for (pid, pname, registered_at) in entries {
        let (table_idx, seat, stack, eliminated) = match seat_of.get(&pid) {
            Some((table_id, seat, stack, status)) => (
                store
                    .live_table_ids
                    .iter()
                    .position(|t| t == table_id)
                    .map(|i| i as u32),
                Some((*seat).max(0) as u32),
                (*stack).max(0) as u64,
                status == "ELIMINATED",
            ),
            None => (None, None, store.state.config.starting_stack, false),
        };
        if !eliminated {
            remaining += 1;
        }
        store.state.players.insert(
            pid.clone(),
            PlayerTournamentEntry {
                player_id: pid,
                player_name: pname,
                stack,
                table_id: table_idx,
                seat,
                rebuys: 0,
                addon_done: false,
                final_position: None,
                prize: None,
                registered_at: registered_at.max(0) as u64,
                eliminated_at: eliminated.then_some(0),
            },
        );
    }
    store.state.players_remaining = remaining;
    Ok(())
}

/// Load all non-finished catalog tournaments into memory.
pub async fn load_tournaments_from_db(
    pool: &PgPool,
) -> Result<HashMap<String, TournamentStore>, sqlx::Error> {
    let rows: Vec<TournamentRow> = sqlx::query_as(
        r#"
        SELECT id, name, buy_in, starting_stack, max_players, table_max_players,
               late_registration, late_reg_max_level, speed, status,
               prize_pool, current_level, players_remaining, total_buyins,
               guaranteed_prize, is_freeroll,
               rebuy_cost, rebuy_chips, rebuy_max_count, rebuy_stack_threshold,
               rebuy_max_level, allow_rebuy, blind_levels, game_type,
               COALESCE(money_mode, 'play') AS money_mode,
                COALESCE(poker_variant, 'holdem') AS poker_variant,
                 final_table_variant, final_table_max_players,
                 scheduled_start_at, auto_start_min_players, live_table_id,
                 live_table_ids
          FROM tournaments
        WHERE status IN ('registering', 'running', 'paused')
        ORDER BY money_mode, poker_variant, is_freeroll DESC, buy_in, name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut map = HashMap::new();
    for row in rows {
        let mut store = row_to_store(row);
        if let Err(error) = hydrate_players(pool, &mut store).await {
            tracing::warn!(tournament_id = %store.id, ?error, "players do torneio não reidratados");
        }
        map.insert(store.id.clone(), store);
    }
    Ok(map)
}
