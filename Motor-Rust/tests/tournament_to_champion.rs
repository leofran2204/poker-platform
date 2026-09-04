//! Joga cada MTT Play Money até restar 1 campeão, mesa a mesa.
//!
//! Campo = `table_max * 3` (várias mesas lotadas) + 2 reservas **por mesa**.
//! Rebuy: 1× até o nível 6. Depois do rebuy, reservas entram no lugar de quem quebra.
//! Addon: tentado e esperado recusar (catálogo `allow_addon=false`).
//! Relógio: 26 níveis × 5 min; o teste avança de nível a cada 20 órbitas
//! (uma mão em cada mesa da órbita ≈ 15 s/mão ao vivo).
//!
//!   cargo test --test tournament_to_champion -- --nocapture

use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::tournament_engine::{
    advance_blinds, create_tournament, eliminate_player, finish_tournament, process_addon,
    process_rebuy, register_player, start_tournament, BlindLevel, TournamentConfig,
    TournamentSpeed,
};
use poker_engine::types::{PokerVariant, TableConfig};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const HANDS_PER_LEVEL: u32 = 20;
const WAITERS_PER_TABLE: usize = 2;
const TABLES_IN_FIELD: usize = 3;

fn bba_levels() -> Vec<BlindLevel> {
    let bbs = [
        50u64, 100, 150, 200, 300, 400, 600, 800, 1000, 1200, 1600, 2000, 2400, 3000, 4000, 5000,
        6000, 8000, 10000, 12000, 16000, 20000, 24000, 30000, 40000, 50000,
    ];
    bbs.iter()
        .enumerate()
        .map(|(i, &bb)| BlindLevel {
            level: (i + 1) as u32,
            small_blind: if i == 0 { 50 } else { bb / 2 },
            big_blind: bb,
            ante: bb,
            duration_minutes: 5,
        })
        .collect()
}

struct MttSpec {
    name: &'static str,
    variant: PokerVariant,
    table_max: usize,
    buy_in: u64,
    starting_stack: u64,
    gtd: u64,
    rebuy_cost: u64,
    rebuy_chips: u64,
    is_freeroll: bool,
    final_table_variant: Option<PokerVariant>,
    final_table_max: usize,
}

const SPECS: &[MttSpec] = &[
    MttSpec {
        name: "Texas Hold’em — Torneio",
        variant: PokerVariant::Holdem,
        table_max: 9,
        buy_in: 1500,
        starting_stack: 10_000,
        gtd: 15_000,
        rebuy_cost: 1500,
        rebuy_chips: 15_000,
        is_freeroll: false,
        final_table_variant: None,
        final_table_max: 9,
    },
    MttSpec {
        name: "Texas Hold’em — Torneio Freeroll",
        variant: PokerVariant::Holdem,
        table_max: 9,
        buy_in: 0,
        starting_stack: 5_000,
        gtd: 7_500,
        rebuy_cost: 1000,
        rebuy_chips: 10_000,
        is_freeroll: true,
        final_table_variant: Some(PokerVariant::ShortDeck),
        final_table_max: 8,
    },
    MttSpec {
        name: "Omaha 4 Cartas — Torneio",
        variant: PokerVariant::ShortDeckOmaha,
        table_max: 5,
        buy_in: 1000,
        starting_stack: 10_000,
        gtd: 10_000,
        rebuy_cost: 2000,
        rebuy_chips: 20_000,
        is_freeroll: false,
        final_table_variant: None,
        final_table_max: 5,
    },
    MttSpec {
        name: "Ultimate Pineapple — Torneio",
        variant: PokerVariant::UltimatePineapple,
        table_max: 6,
        buy_in: 1000,
        starting_stack: 10_000,
        gtd: 10_000,
        rebuy_cost: 2000,
        rebuy_chips: 20_000,
        is_freeroll: false,
        final_table_variant: None,
        final_table_max: 6,
    },
];

fn auto_play(gl: &mut GameLoop, big_blind: u64, starting_stack: u64) {
    let mut steps = 0u32;
    while !gl.state.is_finished && steps < 3_000 {
        steps += 1;
        let Some(active) = gl.state.active_player().map(|p| p.id.clone()) else {
            break;
        };
        let to_call = {
            let p = gl.state.players.iter().find(|p| p.id == active).expect("p");
            gl.state.current_bet_to_match.saturating_sub(p.current_bet)
        };
        let roll = steps.wrapping_mul(41).wrapping_add(active.len() as u32) % 100;
        let late = big_blind >= starting_stack / 4;
        let mv = if to_call == 0 {
            if roll > 90 {
                PlayerMove::Raise(big_blind.max(1) * 2)
            } else {
                PlayerMove::Check
            }
        } else if late && roll < 32 {
            PlayerMove::AllIn
        } else if roll < 16 {
            PlayerMove::Fold
        } else if roll > 96 {
            PlayerMove::AllIn
        } else {
            PlayerMove::Call
        };
        if gl.player_action(&active, mv).is_err() {
            let fb = if to_call == 0 {
                PlayerMove::Check
            } else {
                PlayerMove::Call
            };
            if gl.player_action(&active, fb).is_err() {
                let _ = gl.player_action(&active, PlayerMove::Fold);
            }
        }
    }
}

fn seat_tables(alive: &[(String, u64)], table_max: usize) -> Vec<Vec<(String, u64)>> {
    let n = alive.len();
    if n < 2 {
        return Vec::new();
    }
    let mut n_tables = n.div_ceil(table_max).max(1);
    while n_tables > 1 && n / n_tables < 2 {
        n_tables -= 1;
    }
    let mut tables = vec![Vec::new(); n_tables];
    for (i, player) in alive.iter().cloned().enumerate() {
        tables[i % n_tables].push(player);
    }
    tables.retain(|table| table.len() >= 2);
    for table in &mut tables {
        if table.len() > table_max {
            table.truncate(table_max);
        }
    }
    tables
}

fn play_one_hand(
    spec: &MttSpec,
    variant: PokerVariant,
    seated: &[(String, u64)],
    blinds: &BlindLevel,
    hand_id: u32,
) -> Option<HashMap<String, u64>> {
    if seated.len() < 2 {
        return None;
    }
    let mut gl = GameLoop::new(
        TableConfig::new(blinds.big_blind, 0, 0)
            .with_small_blind(blinds.small_blind)
            .with_poker_variant(variant),
        format!("{}-{hand_id}", spec.name),
        spec.name.to_string(),
        GameType::Tournament,
    )
    .with_ante(blinds.ante)
    .with_skip_loss_deflator(true);
    for (id, stack) in seated {
        gl.add_player(id.clone(), *stack);
    }
    gl.set_dealer((hand_id as usize) % seated.len());
    if gl.start_hand().is_err() {
        return None;
    }
    auto_play(&mut gl, blinds.big_blind, spec.starting_stack);
    if !gl.state.is_finished {
        return None;
    }
    let res = gl.resolve_hand().ok()?;
    let mut next = HashMap::new();
    for p in &gl.state.players {
        let pay = res.payouts.get(&p.id).copied().unwrap_or(0);
        next.insert(p.id.clone(), p.stack + pay);
    }
    Some(next)
}

fn run_spec(spec: &MttSpec) {
    let field = spec.table_max * TABLES_IN_FIELD;
    let waiter_count = WAITERS_PER_TABLE * TABLES_IN_FIELD;
    let cfg = TournamentConfig {
        name: spec.name.to_string(),
        game_type: spec.variant.as_str().to_string(),
        buy_in: spec.buy_in,
        starting_stack: spec.starting_stack,
        max_players: (field + waiter_count) as u32,
        speed: TournamentSpeed::Normal,
        blind_levels: bba_levels(),
        prize_pool_pct: 1.0,
        prize_distribution: vec![0.50, 0.30, 0.20],
        late_registration: true,
        // Catálogo vivo fecha late-reg no nível 4; neste teste as reservas entram
        // depois do rebuy (nível 6), como pedido da simulação.
        late_registration_max_level: 26,
        allow_rebuy: true,
        allow_addon: false,
        rebuy_max_level: 6,
        guaranteed_prize: spec.gtd,
        is_freeroll: spec.is_freeroll,
        rebuy_cost: spec.rebuy_cost,
        rebuy_chips: spec.rebuy_chips,
        rebuy_max_count: 1,
        rebuy_stack_threshold: 0,
    };

    let mut state = create_tournament(cfg);
    for i in 0..field {
        register_player(&mut state, &format!("p{i}"), &format!("Jogador {i}"))
            .expect("register field");
    }
    let waiter_ids: Vec<String> = (0..waiter_count).map(|i| format!("w{i}")).collect();
    start_tournament(&mut state).expect("start");
    assert_eq!(state.current_level, 1);

    let addon_try = process_addon(&mut state, "p0", spec.starting_stack / 2, spec.buy_in.max(1));
    assert!(
        addon_try.is_err(),
        "{} addon deveria falhar no catálogo: {addon_try:?}",
        spec.name
    );

    let mut stacks: HashMap<String, u64> = (0..field)
        .map(|i| (format!("p{i}"), spec.starting_stack))
        .collect();
    let mut out: HashSet<String> = HashSet::new();
    let mut waiter_idx = 0usize;
    let mut replacements = 0u32;
    let mut rebuys = 0u32;
    let mut orbits = 0u32;
    let mut hands = 0u32;
    let t0 = Instant::now();

    while stacks.values().filter(|s| **s > 0).count() >= 2 && orbits < 2_000 {
        let alive: Vec<(String, u64)> = stacks
            .iter()
            .filter(|(_, s)| **s > 0)
            .map(|(id, s)| (id.clone(), *s))
            .collect();
        if alive.len() < 2 {
            break;
        }

        let switched = spec.final_table_variant.is_some() && alive.len() <= spec.final_table_max;
        let variant = if switched {
            spec.final_table_variant.unwrap()
        } else {
            spec.variant
        };
        let table_max = if switched {
            spec.final_table_max
        } else {
            spec.table_max
        };

        let blinds = state
            .config
            .blind_levels
            .get((state.current_level.saturating_sub(1)) as usize)
            .cloned()
            .unwrap_or_else(|| state.config.blind_levels.last().unwrap().clone());

        let tables = seat_tables(&alive, table_max);
        if tables.is_empty() {
            break;
        }
        for seated in tables {
            hands += 1;
            if let Some(next) = play_one_hand(spec, variant, &seated, &blinds, hands) {
                for (id, stack) in next {
                    stacks.insert(id.clone(), stack);
                    if let Some(entry) = state.players.get_mut(&id) {
                        entry.stack = stack;
                    }
                }
            }
        }

        let busted: Vec<String> = stacks
            .iter()
            .filter(|(id, s)| **s == 0 && !out.contains(*id))
            .map(|(id, _)| id.clone())
            .collect();
        for id in busted {
            out.insert(id.clone());
            let _ = eliminate_player(&mut state, &id, None);
            if state.current_level <= state.config.rebuy_max_level {
                match process_rebuy(&mut state, &id) {
                    Ok(()) => {
                        stacks.insert(id.clone(), spec.rebuy_chips);
                        out.remove(&id);
                        rebuys += 1;
                    }
                    Err(_) => {}
                }
            } else if waiter_idx < waiter_ids.len() {
                let wid = waiter_ids[waiter_idx].clone();
                waiter_idx += 1;
                match register_player(&mut state, &wid, &format!("Reserva {wid}")) {
                    Ok(()) => {
                        stacks.insert(wid, spec.starting_stack);
                        replacements += 1;
                    }
                    Err(_) => {}
                }
            }
        }

        orbits += 1;
        if orbits % HANDS_PER_LEVEL == 0 {
            let _ = advance_blinds(&mut state);
        }
        if orbits % 50 == 0 {
            eprintln!(
                "[{}] orbit={orbits} hands={hands} alive={} level={} rebuys={rebuys} waiters_in={replacements}",
                spec.name,
                stacks.values().filter(|s| **s > 0).count(),
                state.current_level
            );
        }
    }

    let remaining: Vec<(String, u64)> = stacks
        .into_iter()
        .filter(|(_, s)| *s > 0)
        .collect();
    assert_eq!(
        remaining.len(),
        1,
        "{} deveria ter 1 campeão, restaram {} (orbits={orbits} hands={hands} level={})",
        spec.name,
        remaining.len(),
        state.current_level
    );
    if let Some((champ_id, champ_stack)) = remaining.first() {
        if let Some(entry) = state.players.get_mut(champ_id) {
            entry.stack = *champ_stack;
        }
    }

    let clock_min = u64::from(state.current_level.max(1)) * 5;
    let wall = t0.elapsed();
    let itm_2 = state.eliminated_order.iter().rev().next().cloned();
    let itm_3 = state.eliminated_order.iter().rev().nth(1).cloned();
    let result = finish_tournament(&mut state);
    println!(
        "[{}] champion={:?} stack={} hands={hands} orbits={orbits} rebuys={rebuys} waiters_in={replacements}/{} level={} clock≈{clock_min}min wall={wall:?} itm2={:?} itm3={:?} finish={:?}",
        spec.name,
        remaining.first().map(|(id, _)| id.clone()),
        remaining.first().map(|(_, s)| *s).unwrap_or(0),
        waiter_count,
        state.current_level,
        itm_2,
        itm_3,
        result.as_ref().map(|r| (
            r.winners.first().map(|w| w.player_id.clone()),
            r.winners.len(),
            r.total_prize_pool,
            r.duration_seconds
        ))
    );
    let Ok(res) = result else {
        panic!("{} finish: {result:?}", spec.name);
    };
    assert!(!res.winners.is_empty(), "{} sem premiados", spec.name);
    assert!(res.total_prize_pool >= spec.gtd, "{} GTD", spec.name);
    assert_eq!(
        res.winners[0].player_id,
        remaining[0].0,
        "{} campeão diverge do finish",
        spec.name
    );
}

#[test]
fn play_money_mtts_run_to_a_champion() {
    for spec in SPECS {
        run_spec(spec);
    }
}
