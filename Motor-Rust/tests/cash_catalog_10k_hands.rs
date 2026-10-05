//! Authorized cash campaign. Fixed entries, legal stack evolution, deterministic replay.
//! The filename is kept for existing callers; batches now stop on coverage saturation.
use poker_engine::deck::{create_deck, create_short_deck, Card};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::side_pots::{find_winners_for_pot, precompute_hands_for_variant, PlayerForPots};
use poker_engine::types::{PokerVariant, RakeCapSchedule, TableConfig};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn output() -> PathBuf {
    PathBuf::from(std::env::var("FULL_VALIDATION_REPORT_DIR").expect("campaign output required"))
}
fn event(out: &mut impl Write, value: Value) {
    serde_json::to_writer(&mut *out, &value).unwrap();
    writeln!(out).unwrap();
    out.flush().unwrap();
}
fn variant(value: &str) -> PokerVariant {
    match value {
        "holdem" => PokerVariant::Holdem,
        "short_deck" => PokerVariant::ShortDeck,
        "omaha" => PokerVariant::Omaha,
        "brazilian_pineapple" => PokerVariant::BrazilianPineapple,
        _ => panic!("unknown variant"),
    }
}
fn cards_valid(game: &GameLoop, short: bool) {
    let cards: Vec<Card> = game
        .state
        .players
        .iter()
        .flat_map(|p| p.hole_cards.iter())
        .chain(&game.state.community_cards)
        .chain(&game.state.burn_pile)
        .chain(&game.state.deck)
        .copied()
        .collect();
    let expected = if short {
        create_short_deck()
    } else {
        create_deck()
    };
    assert_eq!(cards.len(), expected.len(), "card count");
    for card in expected {
        assert_eq!(
            cards.iter().filter(|c| **c == card).count(),
            1,
            "impossible card"
        );
    }
}
fn expired() -> bool {
    let deadline: u64 = std::env::var("FULL_VALIDATION_DEADLINE")
        .unwrap()
        .parse()
        .unwrap();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        >= deadline
}

#[test]
#[ignore = "manual campaign: requires authorization, deadline and artifact directory"]
fn cash_catalog_coverage_batches() {
    assert_eq!(
        std::env::var("FULL_VALIDATION_APPROVED").as_deref(),
        Ok("1")
    );
    let catalog: Value =
        serde_json::from_str(include_str!("../../Documentacao/STATUS_OPERACIONAL.json")).unwrap();
    let index: usize = std::env::var("FULL_VALIDATION_CASH_INDEX")
        .unwrap()
        .parse()
        .unwrap();
    let cfg = &catalog["cash_tables"][index];
    let detail: Value =
        serde_json::from_slice(&fs::read(output().join("catalog-db.json")).unwrap()).unwrap();
    let row = &detail["cash"][index];
    let v = variant(cfg["variant"].as_str().unwrap());
    let bb = cfg["big_blind_cents"].as_u64().unwrap();
    let sb = cfg["small_blind_cents"].as_u64().unwrap();
    let entry = cfg["buy_in_cents"].as_u64().unwrap();
    let cap = cfg["max_players"].as_u64().unwrap() as usize;
    // Independent sessions for every legal occupancy. Only a busted player is replaced,
    // with a NEW synthetic identity and exactly the fixed catalogue entry.
    let mut sessions: Vec<Vec<(String, u64)>> = (2..=cap)
        .map(|n| {
            (0..n)
                .map(|seat| (format!("c{index}-n{n}-p{seat}"), entry))
                .collect()
        })
        .collect();
    let mut replacements = 0;
    let mut trace =
        BufWriter::new(File::create(output().join(format!("cash-{index}-events.jsonl"))).unwrap());
    let mut coverage = BTreeSet::<String>::new();
    let mut batches = Vec::new();
    let mut stagnant = 0;
    let mut hands = 0;
    let mut stop = "ceiling";
    for batch in 0..20u64 {
        let prior = coverage.len();
        // Each batch rotates through occupancies and target streets. Subsequent batches
        // favour aggressive/passive policies missing from this configuration's evidence.
        for offset in 0..1000u64 {
            if expired() {
                stop = "deadline";
                break;
            }
            let hand = batch * 1000 + offset;
            let seed = 0x26_1001 + index as u64 * 1_000_000 + hand;
            let mut rng = StdRng::seed_from_u64(seed);
            let n = 2 + hand as usize % (cap - 1);
            let stacks = &mut sessions[n - 2];
            for (id, stack) in stacks.iter_mut() {
                if *stack == 0 {
                    replacements += 1;
                    *id = format!("c{index}-replacement-{replacements}");
                    *stack = entry;
                    event(
                        &mut trace,
                        json!({"event":"entry","player":id,"cents":entry}),
                    );
                }
            }
            let before: u64 = stacks.iter().map(|(_, s)| s).sum();
            // Deflator is exercised in directed fixtures and a separate precision campaign:
            // calculating 500k boards on every preflop all-in would duplicate that evidence.
            let mut game = GameLoop::new(
                TableConfig::new(
                    bb,
                    row["rake_basis_points"].as_u64().unwrap() as u16,
                    row["rake_cap"].as_u64().unwrap(),
                )
                .with_rake_cap_schedule(RakeCapSchedule {
                    heads_up: row["rake_cap_heads_up"].as_u64().unwrap(),
                    three_to_four: row["rake_cap_three_to_four"].as_u64().unwrap(),
                    five_plus: row["rake_cap_five_plus"].as_u64().unwrap(),
                })
                .with_small_blind(sb)
                .with_poker_variant(v),
                format!("cash-{index}-{hand}"),
                cfg["name"].as_str().unwrap().into(),
                GameType::Cash,
            )
            .with_skip_loss_deflator(true);
            for (id, stack) in stacks.iter() {
                game.add_player(id.clone(), *stack);
            }
            let dealer = hand as usize / (cap - 1) % n;
            game.set_dealer(dealer);
            let mut deck = if v.uses_short_deck() {
                create_short_deck()
            } else {
                create_deck()
            };
            deck.shuffle(&mut rng);
            event(
                &mut trace,
                json!({"event":"start","betting_rule_version":game.betting_structure(),"hand":hand,"seed":seed,"config":cfg,
                "stacks":stacks,"dealer":dealer,"deck":deck,"deflator":"directed_only"}),
            );
            game.start_hand_with_deck(deck).unwrap();
            game.run_out_stalled_hand();
            cards_valid(&game, v.uses_short_deck());
            let mut contexts = Vec::new();
            let target = (hand / (cap - 1) as u64 % 4) as usize;
            let mut policy = (hand / ((cap - 1) * 4) as u64 % 5) as usize;
            if batch > 0 {
                if !coverage.contains("sequence=allin")
                    || !coverage.contains("sequence=fold_after_allin")
                {
                    policy = 0;
                } else if !coverage.contains("sequence=check_raise") {
                    policy = 2;
                }
            }
            let mut steps = 0;
            let mut raises = 0;
            let mut checked = BTreeSet::new();
            while !game.state.is_finished {
                steps += 1;
                assert!(steps < 500, "hand stalled: seed {seed}");
                let p = game.state.active_player().unwrap();
                let id = p.id.clone();
                let seat = p.seat_index;
                let to_call = game
                    .state
                    .current_bet_to_match
                    .saturating_sub(p.current_bet);
                let street = match game.state.community_cards.len() {
                    0 => 0,
                    3 => 1,
                    4 => 2,
                    _ => 3,
                };
                let phase = format!("{:?}", game.state.phase);
                let seen_allin = game.state.players.iter().any(|p| p.is_all_in);
                let roll: u32 = rng.gen_range(0..100);
                let min_total = game.state.current_bet_to_match + game.state.min_raise;
                let legal = game.legal_actions(&id);
                let mv = if v == PokerVariant::BrazilianPineapple
                    && street == 0
                    && legal.allows("allin")
                {
                    // Evolved short stacks can now witness a preflop all-in;
                    // deep stacks remain limited to the next fixed level.
                    PlayerMove::AllIn
                } else if hand == 0 && n == 2 && index != 4 {
                    // Two legal HU hands witness medium/short stacks and a
                    // check-raise followed by one covered all-in. No chip injection.
                    match steps {
                        1 => PlayerMove::Raise(entry - 50 * bb),
                        2 => PlayerMove::AllIn,
                        _ => PlayerMove::Fold,
                    }
                } else if hand == (cap - 1) as u64 && n == 2 && index != 4 {
                    match steps {
                        1 => PlayerMove::Raise(40 * bb),
                        2 | 6 | 8 => PlayerMove::Call,
                        3 => PlayerMove::Check,
                        4 => PlayerMove::Bet(bb),
                        5 => PlayerMove::Raise(2 * bb),
                        7 => PlayerMove::AllIn,
                        _ => panic!("unexpected directed HU step {steps}"),
                    }
                } else if policy == 0 && seen_allin && to_call > 0 && roll < 30 {
                    PlayerMove::Fold
                } else if (policy == 2 && raises >= 4 && game.can_raise(&id))
                    || (street == target
                        && policy == 0
                        && (!seen_allin || roll < 35)
                        && (game.can_raise(&id) || p.stack <= to_call))
                {
                    PlayerMove::AllIn
                } else if to_call > 0 && matches!(policy, 0 | 1 | 3) && roll < 30 {
                    PlayerMove::Fold
                } else if street >= target
                    // Bound the policy's raises, not the legal size of a stack.
                    // Deep stacks can otherwise make 500 perfectly legal min-raises.
                    && raises < 8
                    && (policy == 2 || (matches!(policy, 1 | 3) && raises < 2))
                    && game.can_raise(&id)
                    && roll < 40
                    && p.current_bet + p.stack >= min_total
                {
                    let increment = if policy == 3 {
                        game.state.min_raise.saturating_mul(1 + u64::from(roll % 5))
                    } else {
                        game.state.min_raise
                    };
                    let total =
                        (game.state.current_bet_to_match + increment).min(p.current_bet + p.stack);
                    if game.state.current_bet_to_match == 0 {
                        PlayerMove::Bet(total)
                    } else {
                        PlayerMove::Raise(total)
                    }
                } else if to_call == 0 {
                    PlayerMove::Check
                } else {
                    PlayerMove::Call
                };
                let mv = legal.constrain_move(mv);
                let sequence = match mv {
                    PlayerMove::Fold if seen_allin => "fold_after_allin",
                    PlayerMove::Fold => "fold",
                    PlayerMove::AllIn => "allin",
                    PlayerMove::Bet(_) | PlayerMove::Raise(_)
                        if checked.contains(&(phase.clone(), seat)) =>
                    {
                        "check_raise"
                    }
                    PlayerMove::Bet(_) | PlayerMove::Raise(_) if raises > 0 => "reraise",
                    PlayerMove::Bet(_) | PlayerMove::Raise(_) => "open",
                    _ => "passive",
                };
                let position = if seat == dealer {
                    "button"
                } else if n == 2 {
                    "bb"
                } else if seat == (dealer + 1) % n {
                    "sb"
                } else if seat == (dealer + 2) % n {
                    "bb"
                } else {
                    "other"
                };
                let stack = if p.stack <= 20 * bb {
                    "short"
                } else if p.stack <= 100 * bb {
                    "medium"
                } else {
                    "deep"
                };
                contexts.push(vec![
                    format!("variant={}", cfg["variant"].as_str().unwrap()),
                    format!(
                        "occupancy={}",
                        if n == 2 {
                            "hu"
                        } else if n == cap {
                            "full"
                        } else {
                            "partial"
                        }
                    ),
                    format!("position={position}"),
                    format!("street={phase}"),
                    format!("stack={stack}"),
                    format!("sequence={sequence}"),
                ]);
                event(
                    &mut trace,
                    json!({"event":"action","hand":hand,"step":steps,"player":id,
                    "phase":phase,"move":format!("{mv:?}"),"to_call":to_call}),
                );
                if matches!(mv, PlayerMove::Check) {
                    checked.insert((phase, seat));
                }
                if matches!(mv, PlayerMove::Bet(_) | PlayerMove::Raise(_)) {
                    raises += 1;
                }
                game.player_action(&id, mv).unwrap();
                cards_valid(&game, v.uses_short_deck());
                assert_eq!(
                    game.state
                        .players
                        .iter()
                        .map(|p| p.stack + p.total_bet)
                        .sum::<u64>(),
                    before,
                    "financial divergence seed {seed}"
                );
            }
            if v == PokerVariant::BrazilianPineapple && game.state.community_cards.len() == 5 {
                for p in game.state.players.iter().filter(|p| p.is_in_hand()) {
                    assert_eq!(p.hole_cards.len(), 5);
                }
            }
            let result = game.resolve_hand().unwrap();
            assert_eq!(
                game.state.players.iter().map(|p| p.stack).sum::<u64>()
                    + result.payouts.values().sum::<u64>()
                    + result.rake,
                before,
                "settlement divergence seed {seed}"
            );
            let players: Vec<_> = game
                .state
                .players
                .iter()
                .map(|p| PlayerForPots {
                    id: p.id.clone(),
                    total_bet: p.total_bet,
                    has_folded: p.has_folded,
                    cards: p.hole_cards.clone(),
                })
                .collect();
            let evaluated = precompute_hands_for_variant(&players, &game.state.community_cards, v);
            let tied = result
                .pots
                .iter()
                .any(|pot| find_winners_for_pot(pot, &players, &evaluated).len() > 1);
            let allins = game.state.players.iter().filter(|p| p.is_all_in).count();
            // Distinct contested eligibility sets define actual side pots.
            // Uncalled returns and segments with the same contenders do not.
            let contested: BTreeSet<Vec<String>> = result
                .pots
                .iter()
                .filter_map(|pot| {
                    let mut eligible: Vec<_> = pot
                        .eligible_players
                        .iter()
                        .filter(|id| {
                            game.state
                                .players
                                .iter()
                                .any(|p| p.id == **id && !p.has_folded)
                        })
                        .cloned()
                        .collect();
                    eligible.sort();
                    (eligible.len() > 1).then_some(eligible)
                })
                .collect();
            for mut row in contexts {
                row.extend([
                    format!(
                        "allin={}",
                        if allins == 0 {
                            "none"
                        } else if allins == 1 {
                            "single"
                        } else {
                            "multi"
                        }
                    ),
                    format!("tie={tied}"),
                    format!(
                        "pots={}",
                        if contested.len() > 1 {
                            "multiple"
                        } else {
                            "single"
                        }
                    ),
                ]);
                for a in 0..row.len() {
                    coverage.insert(row[a].clone());
                    for b in a + 1..row.len() {
                        let mut pair = [row[a].clone(), row[b].clone()];
                        pair.sort();
                        coverage.insert(pair.join("|"));
                    }
                }
            }
            for (id, stack) in stacks.iter_mut() {
                *stack = game
                    .state
                    .players
                    .iter()
                    .find(|p| p.id == *id)
                    .unwrap()
                    .stack
                    + result.payouts.get(id).copied().unwrap_or(0);
            }
            event(
                &mut trace,
                json!({"event":"settlement","hand":hand,"payouts":result.payouts,
                "rake":result.rake,"pots":result.pots,"stacks":stacks}),
            );
            hands += 1;
        }
        let gained = coverage.len() - prior;
        stagnant = if gained == 0 { stagnant + 1 } else { 0 };
        batches.push(json!({"batch":batch,"hands_completed":hands,"new_coverage":gained}));
        let report = json!({"config":cfg,"hands":hands,"batches":batches,"coverage":coverage,
            "stop":if stop=="deadline" {"deadline"} else if stagnant>=3 {"saturation"} else if batch == 19 {"ceiling"} else {"running"},
            "replacements":replacements,"deflator":"directed_only"});
        fs::write(
            output().join(format!("cash-{index}.json")),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        if stop == "deadline" || stagnant >= 3 {
            if stagnant >= 3 {
                stop = "saturation";
            }
            break;
        }
    }
    event(
        &mut trace,
        json!({"event":"stop","reason":stop,"hands":hands}),
    );
    assert_ne!(stop, "deadline", "incomplete campaign: time limit");
}
