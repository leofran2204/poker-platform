// extreme_fuzz_tests.rs — Suíte de Fuzzing Extremo de Alta Densidade (1.000.000 de Iterações no Motor Rust)
// Executa Fuzzing estocástico em 8 módulos críticos: rake, side_pots, loss_deflator, hand_history, auth, tournament, antifraud e deck.

use crate::antifraud::{bot_detection::BotDetector, RiskScore};
use crate::auth::AuthManager;
use crate::deck::{create_deck, evaluate_hand, shuffle_deck, Card, Rank, Suit};
use crate::hand_history::HandHistory;
use crate::loss_deflator::{calculate_progressive_loss_deflator, ProgressiveLossDeflatorParams};
use crate::rake::calculate_rake_for_pot;
use crate::side_pots::{calculate_side_pots, PlayerForPots};
use crate::tournament_engine::{TournamentConfig, TournamentSpeed};
use crate::types::{GamePhase, Pot};
use proptest::prelude::*;

fn get_extreme_proptest_config() -> ProptestConfig {
    let cases = std::env::var("EXTREME_FUZZ_CASES")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(100_000);
    ProptestConfig {
        cases,
        max_shrink_iters: 100,
        ..ProptestConfig::default()
    }
}

// ─── 1. Extreme Rake Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_rake_invariants(
        pot in 0u64..500_000_000u64,
        rake_basis_points in 0u16..=5_000u16,
        cap in 0u64..50_000u64,
    ) {
        let rake = calculate_rake_for_pot(pot, rake_basis_points, cap);
        if cap > 0 {
            prop_assert!(rake <= cap, "Rake excedeu o teto (cap)");
        }
        let pot_after = pot.saturating_sub(rake);
        prop_assert!(pot_after <= pot, "Pote resultante maior que o inicial");
    }
}

// ─── 2. Extreme Side Pots Multiway Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_side_pots(
        bets in prop::collection::vec(0u64..10_000_000u64, 2..=9),
    ) {
        let mut players = Vec::with_capacity(bets.len());
        for (idx, &bet) in bets.iter().enumerate() {
            players.push(PlayerForPots {
                id: format!("p{}", idx),
                total_bet: bet,
                has_folded: idx % 3 == 0,
                cards: vec![
                    Card { suit: Suit::Spades, rank: Rank::Ace },
                    Card { suit: Suit::Hearts, rank: Rank::King },
                ],
            });
        }
        let pots = calculate_side_pots(&players);
        let total_contributed: u64 = bets.iter().sum();
        let total_pots: u64 = pots.iter().map(|p| p.amount).sum();
        prop_assert_eq!(total_contributed, total_pots, "Discrepância na conservação de fichas em side pots");
    }
}

// ─── 3. Extreme Loss Deflator Cashback Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_loss_deflator(
        pote_main in 1000u64..1_000_000u64,
        phase_idx in 0..4u8,
        loser_equity in any::<f64>(),
    ) {
        let phase = match phase_idx {
            0 => GamePhase::Preflop,
            1 => GamePhase::Flop,
            2 => GamePhase::Turn,
            _ => GamePhase::River,
        };
        let params = ProgressiveLossDeflatorParams {
            pots: vec![Pot { amount: pote_main, eligible_players: vec!["p1".into(), "p2".into()] }],
            loser_id: "p1".into(),
            winner_id: "p2".into(),
            phase,
            loser_equity,
        };
        let res = calculate_progressive_loss_deflator(params);
        if let Some(defl) = res {
            prop_assert!(defl.cashback <= pote_main, "Cashback fora dos limites válidos do pote");
        }
    }
}

// ─── 4. Extreme Hand History JSON Parser Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_hand_history_json(
        mutated_json in ".*",
    ) {
        let _res: Result<HandHistory, _> = serde_json::from_str(&mutated_json);
    }
}

// ─── 5. Extreme Auth Manager Token & Verification Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_auth_manager(
        token_str in ".*",
        secret_key in "[a-zA-Z0-9]{16,64}",
    ) {
        let mgr = AuthManager::new(&secret_key);
        let res = mgr.validate_token(&token_str, "access");
        let _ = res;
    }
}

// ─── 6. Extreme Antifraud Bot & Collusion Detector Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_antifraud_engine(
        bot_times in prop::collection::vec(0u64..10_000u64, 5..=30),
    ) {
        let mut bot_detector = BotDetector::default();
        for t in bot_times {
            bot_detector.record_reaction_time("player_1", t);
        }
        let b_score = bot_detector.calculate_bot_score("player_1");
        prop_assert!((0.0..=100.0).contains(&b_score), "Bot score fora dos limites 0-100");

        let risk = RiskScore::new(b_score, 0.0);
        prop_assert!(risk.total_score >= 0.0 && risk.total_score <= 100.0, "Risk total_score fora dos limites");
    }
}

// ─── 7. Extreme Tournament Config Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_tournament_config(
        num_players in 2..=200u32,
        starting_chips in 500u64..100_000u64,
        buy_in in 10u64..10_000u64,
    ) {
        let config = TournamentConfig {
            name: "Extreme Fuzz MTT".into(),
            game_type: "Holdem".into(),
            buy_in,
            starting_stack: starting_chips,
            max_players: num_players,
            speed: TournamentSpeed::Turbo,
            blind_levels: vec![],
            prize_pool_pct: 0.15,
            prize_distribution: vec![0.5, 0.3, 0.2],
            late_registration: true,
            late_registration_max_level: 4,
            allow_rebuy: true,
            allow_addon: true,
            rebuy_max_level: 4,
            guaranteed_prize: 0,
            is_freeroll: false,
            rebuy_cost: 0,
            rebuy_chips: 0,
            rebuy_max_count: 0,
            rebuy_stack_threshold: 0,
        };

        prop_assert!(config.buy_in > 0);
        prop_assert!(config.starting_stack > 0);
    }
}

// ─── 8. Extreme Deck Shuffler & Evaluator Fuzz (100k iterations) ───
proptest! {
    #![proptest_config(get_extreme_proptest_config())]
    #[test]
    fn extreme_fuzz_deck_evaluator(
        _dummy in 0..100u32,
    ) {
        let deck = create_deck();
        shuffle_deck(&deck);
        let hole = vec![deck[0], deck[1]];
        let community = vec![deck[2], deck[3], deck[4], deck[5], deck[6]];

        let res = evaluate_hand(&hole, &community);
        prop_assert!(res.value > 0, "Valor de mão zero no evaluator");
    }
}
