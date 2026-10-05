//! Directed acceptance checks. Assert business rules using legal entries and complete action traces.
use poker_engine::deck::{create_deck, Card};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::types::TableConfig;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn pineapple(stacks: &[u64], ante: u64) -> GameLoop {
    let mut g = GameLoop::new(
        TableConfig::new(100, 0, 0)
            .with_small_blind(100)
            .with_poker_variant(poker_engine::types::PokerVariant::BrazilianPineapple),
        "hybrid-v1".into(),
        "Pineapple".into(),
        GameType::Tournament,
    )
    .with_ante(ante)
    .with_skip_loss_deflator(true);
    for (i, stack) in stacks.iter().enumerate() {
        g.add_player(format!("p{i}"), *stack);
    }
    g.set_dealer(stacks.len().saturating_sub(3));
    g.start_hand_with_deck(create_deck()).unwrap();
    g
}

fn act(g: &mut GameLoop, action: PlayerMove) {
    let id = g.state.active_player().unwrap().id.clone();
    g.player_action(&id, action).unwrap();
}

fn rejected_unchanged(g: &mut GameLoop, action: PlayerMove) {
    let id = g.state.active_player().unwrap().id.clone();
    let before = format!("{:?}", g.state);
    let history = serde_json::to_value(&g.history).unwrap();
    let legal = g.legal_actions(&id);
    assert!(g.player_action(&id, action).is_err());
    assert_eq!(before, format!("{:?}", g.state));
    assert_eq!(history, serde_json::to_value(&g.history).unwrap());
    assert_eq!(legal, g.legal_actions(&id));
}

fn pineapple_flop(n: usize) -> GameLoop {
    // BBA supplies the remainder: all paid contributions total exactly R$10.
    let mut g = pineapple(&vec![10000; n], 1000 - 100 * n as u64);
    while g.state.phase == poker_engine::types::GamePhase::Preflop {
        let p = g.state.active_player().unwrap();
        let mv = if p.current_bet < g.state.current_bet_to_match {
            PlayerMove::Call
        } else {
            PlayerMove::Check
        };
        act(&mut g, mv);
    }
    assert_eq!(g.state.total_pot(), 1000);
    g
}

#[test]
fn pineapple_fixed_levels_cap_heads_up_and_transition() {
    for n in [2, 3, 6] {
        let mut g = pineapple(&vec![10000; n], 0);
        for level in [200, 300, 400] {
            let legal = g.legal_actions(&g.state.active_player().unwrap().id);
            assert_eq!((legal.minimum_wager, legal.maximum_wager), (level, level));
            assert!(!legal.allows("allin"));
            rejected_unchanged(&mut g, PlayerMove::Raise(level + 1));
            act(&mut g, PlayerMove::Raise(level));
            assert_eq!(g.state.min_raise, 100);
        }
        rejected_unchanged(&mut g, PlayerMove::Raise(500));
        rejected_unchanged(&mut g, PlayerMove::AllIn);
        while g.state.phase == poker_engine::types::GamePhase::Preflop {
            act(&mut g, PlayerMove::Call);
        }
        let legal = g.legal_actions(&g.state.active_player().unwrap().id);
        assert_eq!(
            (legal.minimum_wager, legal.maximum_wager),
            (100, 400 * n as u64)
        );
        assert!(g.state.players.iter().all(|p| p.hole_cards.len() == 3));
    }
}

#[test]
fn pineapple_short_allins_completion_and_individual_reopening() {
    let mut g = pineapple(&[10000, 250, 10000, 10000], 0);
    act(&mut g, PlayerMove::Raise(200));
    act(&mut g, PlayerMove::AllIn); // 2.5 BB
    act(&mut g, PlayerMove::Call); // p2 completed at 250
    let legal = g.legal_actions("p3");
    assert_eq!((legal.minimum_wager, legal.maximum_wager), (300, 300));
    act(&mut g, PlayerMove::Raise(300)); // complete the level, not a new 100 increment
    assert!(g.can_raise("p0")); // faces a full BB cumulatively
    act(&mut g, PlayerMove::Call);
    assert!(!g.can_raise("p2")); // only 50 more than p2's last action
    rejected_unchanged(&mut g, PlayerMove::Raise(400));
    rejected_unchanged(&mut g, PlayerMove::AllIn);

    let mut short = pineapple(&[10000, 225, 300, 10000], 0);
    act(&mut short, PlayerMove::Raise(200));
    act(&mut short, PlayerMove::AllIn);
    act(&mut short, PlayerMove::AllIn);
    act(&mut short, PlayerMove::Call);
    assert!(short.can_raise("p0"));
    act(&mut short, PlayerMove::Raise(400));
}

#[test]
fn pineapple_short_blinds_and_ante_do_not_change_live_levels() {
    let mut g = pineapple(&[10000, 40, 70], 500);
    assert_eq!(g.state.current_bet_to_match, 100);
    assert_eq!(g.state.total_pot(), 110); // incomplete BB has priority, no ante collected
    assert_eq!(g.legal_actions("p0").call_amount, 100);
    assert_eq!(g.legal_actions("p0").minimum_wager, 200);
    act(&mut g, PlayerMove::Call);
    g.run_out_stalled_hand();
    assert!(g.state.is_finished);
    let result = g.resolve_hand().unwrap();
    assert_eq!(
        result.payouts.values().sum::<u64>() + g.state.players.iter().map(|p| p.stack).sum::<u64>(),
        10110
    );
    assert!(g.state.players.iter().all(|p| p.hole_cards.len() == 5));
}

#[test]
fn pineapple_pot_before_call_required_examples_and_reraise() {
    for n in [2, 3, 4] {
        let mut g = pineapple_flop(n);
        act(&mut g, PlayerMove::Bet(500));
        assert_eq!(g.state.total_pot(), 1500);
        let legal = g.legal_actions(&g.state.active_player().unwrap().id);
        assert_eq!(
            (legal.call_amount, legal.minimum_wager, legal.maximum_wager),
            (500, 1000, 2000)
        );
        rejected_unchanged(&mut g, PlayerMove::Raise(2001));
        rejected_unchanged(&mut g, PlayerMove::AllIn);
        act(&mut g, PlayerMove::Raise(2000));
        assert_eq!(g.state.total_pot(), 3500);
        if n > 2 {
            let legal = g.legal_actions(&g.state.active_player().unwrap().id);
            assert_eq!(
                (legal.call_amount, legal.minimum_wager, legal.maximum_wager),
                (2000, 3500, 5500)
            );
        }
        while g.state.phase == poker_engine::types::GamePhase::Flop {
            act(&mut g, PlayerMove::Call);
        }
        assert_eq!(g.state.total_pot(), 1000 + 2000 * n as u64); // 50, 70, 90 reais
    }
    let mut g = pineapple_flop(3);
    act(&mut g, PlayerMove::Bet(500));
    act(&mut g, PlayerMove::Raise(2000));
    rejected_unchanged(&mut g, PlayerMove::Raise(5501));
    act(&mut g, PlayerMove::Raise(5500));
    assert_eq!(g.state.min_raise, 3500);
}

#[test]
fn pineapple_postflop_short_allin_preserves_minimum_and_sidepots() {
    let mut g = pineapple(&[10000, 250, 10000, 10000], 0);
    while g.state.phase == poker_engine::types::GamePhase::Preflop {
        let legal = g.legal_actions(&g.state.active_player().unwrap().id);
        act(
            &mut g,
            if legal.allows("check") {
                PlayerMove::Check
            } else {
                PlayerMove::Call
            },
        );
    }
    // Postflop p2, p3, p0, p1. p1 has 150 left, short raise over 100.
    act(&mut g, PlayerMove::Bet(100));
    act(&mut g, PlayerMove::Call);
    act(&mut g, PlayerMove::Call);
    assert!(g.legal_actions("p1").allows("allin"));
    act(&mut g, PlayerMove::AllIn);
    assert_eq!(g.state.min_raise, 100);
    assert!(!g.can_raise("p2"));
    rejected_unchanged(&mut g, PlayerMove::Raise(250));
    while !g.state.is_finished {
        let legal = g.legal_actions(&g.state.active_player().unwrap().id);
        let mv = if legal.allows("bet") {
            PlayerMove::Bet(100)
        } else if legal.allows("check") {
            PlayerMove::Check
        } else {
            PlayerMove::Call
        };
        act(&mut g, mv);
    }
    let result = g.resolve_hand().unwrap();
    assert!(result.pots.len() > 1);
    assert_eq!(
        result.payouts.values().sum::<u64>() + g.state.players.iter().map(|p| p.stack).sum::<u64>(),
        30250
    );
}

#[test]
fn pineapple_rule_history_is_versioned_and_old_json_stays_unversioned() {
    let g = pineapple(&[10000, 10000], 0);
    let mut value = serde_json::to_value(g.history.unwrap()).unwrap();
    assert_eq!(
        value["betting_rule_version"],
        "brazilian_pineapple_hybrid_v1"
    );
    value
        .as_object_mut()
        .unwrap()
        .remove("betting_rule_version");
    let old: poker_engine::hand_history::HandHistory =
        serde_json::from_value(value.clone()).unwrap();
    assert!(old.betting_rule_version.is_none());
    assert_eq!(serde_json::to_value(old).unwrap(), value);
}

#[test]
fn pineapple_overflow_is_rejected_before_start_and_limits_saturate() {
    let mut g = GameLoop::new(
        TableConfig::new(100, 0, 0),
        "overflow".into(),
        "overflow".into(),
        GameType::Cash,
    );
    g.add_player("p0".into(), u64::MAX);
    g.add_player("p1".into(), 1000);
    let before = format!("{:?}", g.state);
    assert!(g.start_hand().is_err());
    assert_eq!(before, format!("{:?}", g.state));
    let mut g = pineapple(&[u64::MAX / 2, u64::MAX / 2], 0);
    act(&mut g, PlayerMove::Raise(200));
    act(&mut g, PlayerMove::Call);
    rejected_unchanged(&mut g, PlayerMove::Bet(u64::MAX));
    assert_eq!(
        g.legal_actions(&g.state.active_player().unwrap().id)
            .maximum_wager,
        400
    );
}

fn game(stacks: &[u64], dealer: usize, id: &str) -> GameLoop {
    let mut game = GameLoop::new(
        TableConfig::new(25, 500, 250).with_small_blind(25),
        id.into(),
        "NL 0,25".into(),
        GameType::Cash,
    )
    .with_skip_loss_deflator(true);
    for (i, stack) in stacks.iter().enumerate() {
        game.add_player(format!("p{i}"), *stack);
    }
    game.set_dealer(dealer);
    game.start_hand_with_deck(create_deck()).unwrap();
    game
}
fn save(name: &str, value: serde_json::Value) {
    if let Ok(dir) = std::env::var("FULL_VALIDATION_REPORT_DIR") {
        fs::write(
            PathBuf::from(dir).join(format!("{name}.json")),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn short_allin_does_not_reopen_a_completed_action() {
    // All three enter at exactly the published fixed front. A legal previous hand
    // leaves p0 short; no direct assignment of an impossible initial stack.
    let mut first = game(&[2500, 2500, 2500], 0, "setup-short");
    first.player_action("p0", PlayerMove::Raise(2400)).unwrap();
    first.player_action("p1", PlayerMove::AllIn).unwrap();
    first.player_action("p2", PlayerMove::Fold).unwrap();
    first.player_action("p0", PlayerMove::Fold).unwrap();
    let result = first.resolve_hand().unwrap();
    let stacks: Vec<u64> = first
        .state
        .players
        .iter()
        .map(|p| p.stack + result.payouts.get(&p.id).copied().unwrap_or(0))
        .collect();
    assert_eq!(stacks.iter().sum::<u64>() + result.rake, 7500);
    assert_eq!(stacks[0], 100);
    let mut second = game(&stacks, 1, "short-reopen");
    second.player_action("p1", PlayerMove::Raise(75)).unwrap(); // full increment 50
    second.player_action("p2", PlayerMove::Call).unwrap();
    second.player_action("p0", PlayerMove::AllIn).unwrap(); // increment 25 < 50
    let before = format!("{:?}", second.state);
    let rejected = second.player_action("p1", PlayerMove::Raise(150));
    save(
        "short-allin-reopening",
        json!({"entry_cents":[2500,2500,2500],
        "deck":create_deck(),"setup_actions":[["p0","Raise(2400)"],["p1","AllIn"],["p2","Fold"],["p0","Fold"]],
        "setup_dealer":0,"stacks_after_setup":stacks,"setup_rake":result.rake,
        "dealer":1,"actions":[["p1","Raise(75)"],["p2","Call"],["p0","AllIn"],["p1","Raise(150)"]],
        "expected":"reject final raise without changing state","actual":format!("{rejected:?}"),
        "before":before,"after":format!("{:?}",second.state)}),
    );
    assert!(
        rejected.is_err(),
        "short all-in reopened an already completed action"
    );
    assert_eq!(
        before,
        format!("{:?}", second.state),
        "rejection mutated chips"
    );
    assert!(!second.can_raise("p1"));
    assert!(second.player_action("p1", PlayerMove::AllIn).is_err());
    assert_eq!(before, format!("{:?}", second.state));
    second.player_action("p1", PlayerMove::Call).unwrap();
    assert!(!second.can_raise("p2"));
    second.player_action("p2", PlayerMove::Call).unwrap();
    assert!(second.can_raise("p1"), "new street restores raise rights");
}

#[test]
fn cumulative_short_allins_reopen_only_players_facing_a_full_increment() {
    let mut g = game(&[2500, 100, 125, 2500], 1, "cumulative-reopen");
    g.player_action("p0", PlayerMove::Raise(75)).unwrap();
    g.player_action("p1", PlayerMove::AllIn).unwrap();
    g.player_action("p2", PlayerMove::AllIn).unwrap();
    g.player_action("p3", PlayerMove::Call).unwrap();
    assert_eq!(g.state.min_raise, 50);
    assert!(g.can_raise("p0"));
    assert!(!g.can_raise("p3"));
    g.player_action("p0", PlayerMove::Raise(175)).unwrap();
    assert!(g.can_raise("p3"));
    g.player_action("p3", PlayerMove::Call).unwrap();
}

#[test]
fn checked_player_can_raise_a_short_opening_allin() {
    let mut g = game(&[2500, 40, 2500], 2, "check-then-short-bet");
    g.player_action("p2", PlayerMove::Call).unwrap();
    g.player_action("p0", PlayerMove::Check).unwrap();
    g.player_action("p1", PlayerMove::Check).unwrap();
    g.player_action("p0", PlayerMove::Check).unwrap();
    g.player_action("p1", PlayerMove::AllIn).unwrap();
    g.player_action("p2", PlayerMove::Call).unwrap();
    assert!(g.can_raise("p0"));
    g.player_action("p0", PlayerMove::Raise(40)).unwrap();
}

#[test]
fn capped_raise_uses_the_actual_full_increment() {
    let mut g = game(&[2500, 150, 2500], 0, "capped-raise");
    g.player_action("p0", PlayerMove::Raise(75)).unwrap();
    g.player_action("p1", PlayerMove::Raise(1000)).unwrap();
    assert_eq!(g.state.current_bet_to_match, 150);
    assert_eq!(g.state.min_raise, 75);
    assert!(g.player_action("p2", PlayerMove::Raise(200)).is_err());
    g.player_action("p2", PlayerMove::Raise(225)).unwrap();
}

#[test]
fn allin_small_blind_does_not_keep_the_turn() {
    let mut setup = game(&[2500, 2500], 0, "evolve-micro-stack");
    setup.player_action("p0", PlayerMove::Raise(2477)).unwrap();
    setup.player_action("p1", PlayerMove::AllIn).unwrap();
    setup.player_action("p0", PlayerMove::Fold).unwrap();
    let paid = setup.resolve_hand().unwrap();
    let stacks: Vec<_> = setup
        .state
        .players
        .iter()
        .map(|p| p.stack + paid.payouts.get(&p.id).copied().unwrap_or(0))
        .collect();
    assert_eq!(stacks, vec![23, 4977]);
    let mut g = game(&stacks, 0, "allin-small-blind");
    assert!(g.state.players[0].is_all_in);
    assert_eq!(g.state.active_player().unwrap().id, "p1");
    g.player_action("p1", PlayerMove::Check).unwrap();
    assert!(g.state.is_finished);
    let result = g.resolve_hand().unwrap();
    assert_eq!(
        g.state.players.iter().map(|p| p.stack).sum::<u64>()
            + result.payouts.values().sum::<u64>()
            + result.rake,
        5000
    );
}

#[test]
fn tournament_prizes_include_eliminated_places_and_conserve_odd_cents() {
    use poker_engine::tournament_engine::*;
    let mut state = create_tournament(TournamentConfig {
        guaranteed_prize: 10_003,
        ..Default::default()
    });
    for i in 0..5 {
        register_player(&mut state, &format!("p{i}"), &format!("P{i}")).unwrap();
    }
    start_tournament(&mut state).unwrap();
    for i in (1..5).rev() {
        eliminate_player(&mut state, &format!("p{i}"), None).unwrap();
    }
    let result = finish_tournament(&mut state).unwrap();
    assert_eq!(
        result
            .winners
            .iter()
            .map(|w| (w.player_id.as_str(), w.position, w.prize))
            .collect::<Vec<_>>(),
        vec![("p0", 1, 5001), ("p1", 2, 3001), ("p2", 3, 2001)]
    );
    assert_eq!(result.winners.iter().map(|w| w.prize).sum::<u64>(), 10_003);
    assert_eq!(state.players["p4"].final_position, Some(5));
    assert!(state.players["p4"].prize.is_none());
    assert!(finish_tournament(&mut state).is_err());
}

#[test]
fn tournament_two_entrants_redistribute_proportionally_without_float_money() {
    use poker_engine::tournament_engine::*;
    // Also tests amounts above the exact-integer range of f64.
    for pool in [8_000, u64::MAX] {
        let mut state = create_tournament(TournamentConfig {
            guaranteed_prize: pool,
            ..Default::default()
        });
        register_player(&mut state, "a", "A").unwrap();
        register_player(&mut state, "b", "B").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "b", None).unwrap();
        let result = finish_tournament(&mut state).unwrap();
        assert_eq!(result.winners.len(), 2);
        assert_eq!(result.winners.iter().map(|w| w.prize).sum::<u64>(), pool);
        assert_eq!(
            result.winners[0].prize,
            ((u128::from(pool) * 5 + 4) / 8) as u64
        );
    }
}

#[test]
fn invalid_prize_distribution_does_not_finish_tournament() {
    use poker_engine::tournament_engine::*;
    for distribution in [vec![], vec![0.5, 0.4], vec![f64::NAN], vec![1.1, -0.1]] {
        let mut state = create_tournament(TournamentConfig {
            guaranteed_prize: 1000,
            prize_distribution: distribution,
            ..Default::default()
        });
        register_player(&mut state, "a", "A").unwrap();
        register_player(&mut state, "b", "B").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(finish_tournament(&mut state).is_err());
        assert_eq!(state.status, TournamentStatus::Running);
        assert!(state.finished_at.is_none());
        assert!(state.players.values().all(|p| p.prize.is_none()));
    }
}
#[test]
fn uncalled_flop_wager_is_returned_before_rake() {
    let mut g = game(&[2500, 2500], 0, "uncalled");
    g.player_action("p0", PlayerMove::Check).unwrap();
    g.player_action("p1", PlayerMove::Check).unwrap();
    g.player_action("p1", PlayerMove::Bet(2000)).unwrap();
    g.player_action("p0", PlayerMove::Fold).unwrap();
    let result = g.resolve_hand().unwrap();
    // Only the two 25-cent blinds were contested. 5% of 50 with banker's rounding = 2.
    save(
        "uncalled-flop",
        json!({"deck":create_deck(),"entry":[2500,2500],"dealer":0,
        "actions":[["p0","Check"],["p1","Check"],["p1","Bet(2000)"],["p0","Fold"]],
        "expected_rake":2,"actual_rake":result.rake,"payouts":result.payouts,"pots":result.pots}),
    );
    assert_eq!(result.rake, 2, "rake charged on uncalled wager");
    assert_eq!(
        g.state.players.iter().map(|p| p.stack).sum::<u64>()
            + result.payouts.values().sum::<u64>()
            + result.rake,
        5000
    );
}
#[test]
fn invalid_deck_and_under_minimum_raise_are_rejected_without_mutation() {
    let mut g = game(&[2500, 2500, 2500], 0, "rejection");
    let before = format!("{:?}", g.state);
    assert!(g.player_action("p0", PlayerMove::Raise(26)).is_err());
    assert_eq!(before, format!("{:?}", g.state));
    let mut bad: Vec<Card> = create_deck();
    bad[0] = bad[1];
    let mut fresh = GameLoop::new(
        TableConfig::new(25, 500, 250),
        "invalid-deck".into(),
        "NL 0,25".into(),
        GameType::Cash,
    );
    fresh.add_player("a".into(), 2500);
    fresh.add_player("b".into(), 2500);
    let before = format!("{:?}", fresh.state);
    let error = fresh.start_hand_with_deck(bad).unwrap_err();
    assert!(error.to_string().contains("Baralho"));
    assert_eq!(before, format!("{:?}", fresh.state));
}

fn legal_evolved_stacks() -> Vec<u64> {
    let mut first = game(&[2500, 2500, 2500], 0, "setup-different-stacks");
    for (id, action) in [
        ("p0", PlayerMove::Raise(2400)),
        ("p1", PlayerMove::AllIn),
        ("p2", PlayerMove::Fold),
        ("p0", PlayerMove::Fold),
    ] {
        first.player_action(id, action).unwrap();
    }
    let result = first.resolve_hand().unwrap();
    first
        .state
        .players
        .iter()
        .map(|p| p.stack + result.payouts.get(&p.id).copied().unwrap_or(0))
        .collect()
}
fn deck_prefix(text: &str) -> Vec<Card> {
    use poker_engine::deck::{Rank, Suit};
    let mut prefix: Vec<Card> = text
        .split_whitespace()
        .map(|s| Card {
            rank: match s.as_bytes()[0] {
                b'A' => Rank::Ace,
                b'K' => Rank::King,
                b'Q' => Rank::Queen,
                b'J' => Rank::Jack,
                b'T' => Rank::Ten,
                n => Rank::try_from(n - b'0').unwrap(),
            },
            suit: match s.as_bytes()[1] {
                b'h' => Suit::Hearts,
                b'd' => Suit::Diamonds,
                b'c' => Suit::Clubs,
                _ => Suit::Spades,
            },
        })
        .collect();
    let remaining: Vec<_> = create_deck()
        .into_iter()
        .filter(|c| !prefix.contains(c))
        .collect();
    prefix.extend(remaining);
    assert_eq!(prefix.len(), 52);
    prefix
}
fn fixture(stacks: &[u64], deck: Vec<Card>, id: &str, deflator: bool) -> GameLoop {
    let mut g = GameLoop::new(
        TableConfig::new(25, 500, 250)
            .with_small_blind(25)
            .with_rake_cap_schedule(poker_engine::types::RakeCapSchedule {
                heads_up: 75,
                three_to_four: 150,
                five_plus: 250,
            }),
        id.into(),
        "NL 0,25".into(),
        GameType::Cash,
    )
    .with_skip_loss_deflator(!deflator);
    for (i, s) in stacks.iter().enumerate() {
        g.add_player(format!("p{i}"), *s);
    }
    g.set_dealer(1);
    g.start_hand_with_deck(deck).unwrap();
    g
}
#[test]
fn distinct_side_pot_winners_from_legal_entries() {
    let stacks = legal_evolved_stacks();
    let deck = deck_prefix("As Qs Ks Ah Qh Kh 4c 2c 3d 7s 5c 9h 6c Tc");
    let mut g = fixture(&stacks, deck.clone(), "distinct-pots", false);
    g.player_action("p1", PlayerMove::AllIn).unwrap();
    g.player_action("p2", PlayerMove::AllIn).unwrap();
    g.player_action("p0", PlayerMove::Call).unwrap();
    let result = g.resolve_hand().unwrap();
    save(
        "distinct-pots",
        json!({"entry":[2500,2500,2500],"evolution":"setup-different-stacks",
        "stacks":stacks,"deck":deck,"dealer":1,"actions":[["p1","AllIn"],["p2","AllIn"],["p0","Call"]],
        "pots":result.pots,"payouts":result.payouts,"rake":result.rake}),
    );
    assert_eq!(
        result.pots.iter().map(|p| p.amount).collect::<Vec<_>>(),
        vec![300, 4750, 2450]
    );
    assert_eq!(result.payouts.get("p0"), Some(&285));
    assert_eq!(result.payouts.get("p2"), Some(&4615));
    assert_eq!(result.payouts.get("p1"), Some(&2450));
    assert_eq!(result.rake, 150);
    assert_eq!(result.payouts.values().sum::<u64>() + result.rake, 7500);
}
#[test]
fn deflator_timing_with_later_fold_is_auditable() {
    use poker_engine::loss_deflator::get_multiway_win_probability_for_variant as equity;
    use poker_engine::types::{GamePhase, PokerVariant};
    let stacks = legal_evolved_stacks();
    let deck = deck_prefix("As Jh 8c Ah Th 7d 2c Qs 9c 4d 3c 2s 5c Kh");
    let mut g = fixture(&stacks, deck.clone(), "temporal-fold", true);
    let actions = [
        ("p1", PlayerMove::Check),
        ("p2", PlayerMove::Check),
        ("p0", PlayerMove::Check),
        ("p2", PlayerMove::Check),
        ("p0", PlayerMove::Check),
        ("p1", PlayerMove::Check),
        ("p2", PlayerMove::Check),
        ("p0", PlayerMove::AllIn),
        ("p1", PlayerMove::Call),
        ("p2", PlayerMove::Call),
        ("p2", PlayerMove::Check),
        ("p1", PlayerMove::Bet(25)),
        ("p2", PlayerMove::Fold),
    ];
    // p1 is the button, not a blind, so its first action is a call.
    let mut events = Vec::new();
    let mut snapshot = None;
    for (step, (id, action)) in actions.into_iter().enumerate() {
        let action = if step == 0 { PlayerMove::Call } else { action };
        events.push(
            json!({"step":step,"player":id,"phase":g.state.phase,"move":format!("{action:?}")}),
        );
        g.player_action(id, action).unwrap();
        if step == 8 {
            let hero = &g.state.players[0].hole_cards;
            let a = &g.state.players[1].hole_cards;
            let b = &g.state.players[2].hole_cards;
            snapshot = Some(
                equity(
                    hero,
                    &[a, b],
                    &g.state.community_cards,
                    PokerVariant::Holdem,
                )
                .unwrap(),
            );
        }
    }
    let result = g.resolve_hand().unwrap();
    let audit = result
        .loss_deflators
        .iter()
        .find(|a| a.loser_id == "p0")
        .unwrap();
    assert_eq!(audit.phase, GamePhase::Turn);
    assert_eq!(audit.opponents_counted, 2);
    // Seven straight outs (four kings and three eights) among 42 unseen cards.
    assert!((audit.loser_equity - 35.0 / 42.0).abs() < 1e-12);
    assert_eq!(Some(audit.loser_equity), snapshot);
    assert_eq!(
        g.state.players.iter().map(|p| p.stack).sum::<u64>()
            + result.payouts.values().sum::<u64>()
            + result.rake,
        7500
    );
    save(
        "deflator-temporal-audit",
        json!({"status":"passed","entry":[2500,2500,2500],
        "stacks":stacks,"deck":deck,"events":events,"audit":audit,
        "acceptance_snapshot_equity":snapshot,"settlement_equity":audit.loser_equity,
        "rule":"Preserve live opponents at payment, including later folds; user confirmed 2026-10-02."}),
    );
}
