//! Five live catalogue events, legal entries, chip/money ledgers and full replay.
//! Reads the isolated migrated database snapshot, not stale duplicated constants.
use poker_engine::deck::{create_deck, create_short_deck};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::tournament_engine::*;
use poker_engine::types::{PokerVariant, TableConfig};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

fn emit(out: &mut File, value: Value) {
    writeln!(out, "{value}").unwrap();
    out.flush().unwrap();
}
#[test]
#[ignore = "authorized campaign with isolated migrated catalogue"]
fn catalog_mtts_to_champion() {
    assert_eq!(
        std::env::var("FULL_VALIDATION_APPROVED").as_deref(),
        Ok("1")
    );
    let dir = PathBuf::from(std::env::var("FULL_VALIDATION_REPORT_DIR").unwrap());
    let detail: Value =
        serde_json::from_slice(&fs::read(dir.join("catalog-db.json")).unwrap()).unwrap();
    let index: usize = std::env::var("FULL_VALIDATION_MTT_INDEX")
        .unwrap()
        .parse()
        .unwrap();
    let row = &detail["mtt"][index];
    let u = |key: &str| row[key].as_u64().unwrap();
    let variant = match row["poker_variant"].as_str().unwrap() {
        "holdem" => PokerVariant::Holdem,
        "omaha" => PokerVariant::Omaha,
        "brazilian_pineapple" => PokerVariant::BrazilianPineapple,
        _ => panic!("catalog variant"),
    };
    let cap = u("table_max_players") as usize;
    let field = u("max_players") as usize;
    let cfg = TournamentConfig {
        name: row["name"].as_str().unwrap().into(),
        game_type: variant.as_str().into(),
        buy_in: u("buy_in"),
        starting_stack: u("starting_stack"),
        max_players: field as u32,
        blind_levels: serde_json::from_value(row["blind_levels"].clone()).unwrap(),
        late_registration: row["late_registration"].as_bool().unwrap(),
        late_registration_max_level: u("late_reg_max_level") as u32,
        allow_rebuy: row["allow_rebuy"].as_bool().unwrap(),
        rebuy_max_level: u("rebuy_max_level") as u32,
        guaranteed_prize: u("guaranteed_prize"),
        is_freeroll: row["is_freeroll"].as_bool().unwrap(),
        rebuy_cost: u("rebuy_cost"),
        rebuy_chips: u("rebuy_chips"),
        rebuy_max_count: u("rebuy_max_count") as u32,
        rebuy_stack_threshold: u("rebuy_stack_threshold"),
        ..Default::default()
    };
    let mut trace = File::create(dir.join(format!("mtt-{index}-events.jsonl"))).unwrap();
    emit(
        &mut trace,
        json!({"event":"catalog","row":row,"units":{"stack":"tournament_chips","prize":"cents"}}),
    );
    let mut cancelled = create_tournament(cfg.clone());
    register_player(&mut cancelled, "cancel", "Cancel").unwrap();
    assert_eq!(
        unregister_player(&mut cancelled, "cancel").unwrap(),
        cfg.buy_in + entry_fee_cents(cfg.buy_in)
    );
    assert_eq!(cancelled.total_buyins, 0);
    cancel_tournament(&mut cancelled).unwrap();
    assert_eq!(cancelled.status, TournamentStatus::Cancelled);
    let mut state = create_tournament(cfg.clone());
    let mut stacks = BTreeMap::new();
    for i in 0..field {
        let id = format!("p{i:02}");
        register_player(&mut state, &id, &id).unwrap();
        stacks.insert(id, cfg.starting_stack);
    }
    assert!(register_player(&mut state, "overflow", "overflow").is_err());
    start_tournament(&mut state).unwrap();
    assert!(unregister_player(&mut state, "p00").is_err());
    assert!(process_addon(&mut state, "p00", 100, 100).is_err());
    let mut tables = vec![Vec::<String>::new(); 3];
    for (i, t, _) in assign_initial_tables(field, cap as u32) {
        tables[t as usize].push(format!("p{i:02}"));
    }
    let mut chip_supply = field as u64 * cfg.starting_stack;
    let mut reentries = 0;
    let mut moves = 0;
    let mut hands = 0u64;
    let mut orbit = 0;
    while stacks.values().filter(|s| **s > 0).count() > 1 {
        orbit += 1;
        assert!(orbit < 2000, "MTT did not reach champion");
        for table in &mut tables {
            table.retain(|id| stacks[id] > 0);
        }
        let alive = stacks.values().filter(|s| **s > 0).count();
        if should_consolidate(alive as u32, cap as u32) {
            let merged: Vec<_> = tables.iter().flatten().cloned().collect();
            if tables.iter().skip(1).any(|t| !t.is_empty()) {
                moves += 1;
            }
            tables = vec![merged, vec![], vec![]];
        } else {
            while let Some((from, to)) =
                rebalance_move(&tables.iter().map(|t| t.len() as u32).collect::<Vec<_>>())
            {
                let id = tables[from].pop().unwrap();
                emit(
                    &mut trace,
                    json!({"event":"move","player":id,"from":from,"to":to}),
                );
                tables[to].push(id);
                moves += 1;
            }
        }
        let blinds = get_current_blinds(&state).unwrap().clone();
        for (table_idx, seats) in tables.iter().enumerate().filter(|(_, s)| s.len() >= 2) {
            assert!(seats.len() <= cap);
            hands += 1;
            let seed = 0x261001 + index as u64 * 100000 + hands;
            let mut rng = StdRng::seed_from_u64(seed);
            let mut deck = if variant.uses_short_deck() {
                create_short_deck()
            } else {
                create_deck()
            };
            deck.shuffle(&mut rng);
            let initial: u64 = seats.iter().map(|id| stacks[id]).sum();
            let mut game = GameLoop::new(
                TableConfig::new(blinds.big_blind, 0, 0)
                    .with_small_blind(blinds.small_blind)
                    .with_poker_variant(variant),
                format!("mtt-{index}-{hands}"),
                cfg.name.clone(),
                GameType::Tournament,
            )
            .with_ante(blinds.ante)
            .with_skip_loss_deflator(true);
            for id in seats {
                game.add_player(id.clone(), stacks[id]);
            }
            let dealer = hands as usize % seats.len();
            game.set_dealer(dealer);
            emit(
                &mut trace,
                json!({"event":"hand","hand":hands,"seed":seed,"table":table_idx,
                "dealer":dealer,"deck":deck,"blinds":blinds,"seats":seats,
                "stacks":seats.iter().map(|id|(id,stacks[id])).collect::<BTreeMap<_,_>>()}),
            );
            game.start_hand_with_deck(deck).unwrap();
            let mut steps = 0;
            while !game.state.is_finished {
                steps += 1;
                assert!(steps < 500);
                let p = game.state.active_player().unwrap();
                let id = p.id.clone();
                let call = game
                    .state
                    .current_bet_to_match
                    .saturating_sub(p.current_bet);
                let action = if hands.is_multiple_of(3) && (game.can_raise(&id) || p.stack <= call)
                {
                    PlayerMove::AllIn
                } else if call > 0 {
                    PlayerMove::Call
                } else {
                    PlayerMove::Check
                };
                let action = game.legal_actions(&id).constrain_move(action);
                emit(
                    &mut trace,
                    json!({"event":"action","hand":hands,"player":id,"move":format!("{action:?}")}),
                );
                game.player_action(&id, action).unwrap();
            }
            let result = game.resolve_hand().unwrap();
            assert_eq!(result.rake, 0);
            for p in &game.state.players {
                stacks.insert(
                    p.id.clone(),
                    p.stack + result.payouts.get(&p.id).copied().unwrap_or(0),
                );
            }
            assert_eq!(
                seats.iter().map(|id| stacks[id]).sum::<u64>(),
                initial,
                "tournament chips lost"
            );
            emit(
                &mut trace,
                json!({"event":"settlement","hand":hands,"payouts":result.payouts,"pots":result.pots}),
            );
        }
        let busted: Vec<_> = stacks
            .iter()
            .filter(|(id, s)| **s == 0 && state.players[*id].eliminated_at.is_none())
            .map(|(id, _)| id.clone())
            .collect();
        // Equal-stack simultaneous busts: deterministic seat/id tie-break in the
        // harness; production coordinator tie-break remains separately audited.
        for id in busted {
            eliminate_player(&mut state, &id, None).unwrap();
            if state.current_level <= cfg.rebuy_max_level
                && state.players[&id].rebuys < cfg.rebuy_max_count
            {
                process_rebuy(&mut state, &id).unwrap();
                stacks.insert(id.clone(), state.players[&id].stack);
                chip_supply += state.players[&id].stack;
                reentries += 1;
                assert!(
                    process_rebuy(&mut state, &id).is_err(),
                    "more than catalogue reentry limit"
                );
                if !tables.iter().any(|t| t.contains(&id)) {
                    let t = tables.iter().position(|t| t.len() < cap).unwrap();
                    tables[t].push(id.clone());
                }
                emit(
                    &mut trace,
                    json!({"event":"reentry","player":id,"chips":state.players[&id].stack}),
                );
            } else {
                emit(&mut trace, json!({"event":"elimination","player":id}));
            }
        }
        for (id, stack) in &stacks {
            state.players.get_mut(id).unwrap().stack = *stack;
        }
        assert_eq!(stacks.values().sum::<u64>(), chip_supply);
        if orbit % 4 == 0 && state.current_level < (cfg.blind_levels.len() as u32) {
            advance_blinds(&mut state).unwrap();
            emit(
                &mut trace,
                json!({"event":"blind_level","level":state.current_level}),
            );
        }
        fs::write(dir.join(format!("mtt-{index}-progress.json")),json!({"hands":hands,
            "players_remaining":state.players_remaining,"level":state.current_level,"reentries":reentries,
            "moves":moves,"chip_supply":chip_supply}).to_string()).unwrap();
    }
    let result = finish_tournament(&mut state).unwrap();
    fs::write(dir.join(format!("mtt-{index}.json")),serde_json::to_vec_pretty(&json!({
        "result":result,"hands":hands,"reentries":reentries,"moves":moves,
        "chip_supply":chip_supply,"awarded_cents":result.winners.iter().map(|w|w.prize).sum::<u64>(),
        "elimination_order":state.eliminated_order})).unwrap()).unwrap();
    assert_eq!(
        result.winners[0].player_id,
        *stacks.iter().find(|(_, s)| **s > 0).unwrap().0
    );
    assert!(reentries > 0, "reentry not covered");
    assert!(moves > 0, "table movement not covered");
    assert_eq!(
        result.winners.iter().map(|w| w.prize).sum::<u64>(),
        result.total_prize_pool,
        "unawarded prize money: eliminated paid places must receive their prizes"
    );
    assert_eq!(
        result.total_prize_pool,
        (state.total_buyins + state.total_rebuys).max(cfg.guaranteed_prize)
    );
}
