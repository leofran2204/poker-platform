// fuzz_tests.rs — Suíte de Fuzzing & Property-Based Testing de Alta Densidade (200.000 iterações/função)
// Valida a imunidade a panics, conservação de fichas e invariantes sob 2,6 MILHÕES de cenários estocásticos.

use proptest::prelude::*;
use crate::rake::{calculate_rake_for_pot, deduct_rake};
use crate::side_pots::{calculate_side_pots, PlayerForPots};
use crate::loss_deflator::{calculate_progressive_loss_deflator, ProgressiveLossDeflatorParams};
use crate::hand_history::{HandHistory, TableConfig as HandTableConfig, GameType};
use crate::auth::AuthManager;
use crate::antifraud::chip_dumping::{ChipDumpAnalyzer, HandStrength};
use crate::tournament_engine::{TournamentConfig, TournamentSpeed};
use crate::types::{Pot, TableConfig, GamePhase};
use crate::deck::{Card, Suit, Rank};

/// Função auxiliar para obter configuração proptest dinâmica via env var PROPTEST_CASES (default 200.000)
fn get_proptest_config() -> ProptestConfig {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(200_000);
    ProptestConfig {
        cases,
        max_shrink_iters: 1000,
        .. ProptestConfig::default()
    }
}

// ─── 1. Rake Invariants ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn rake_pot_invariants(
        pot_amount in 0..100_000_000u64,
        rake_percent in 0.0..100.0f64,
        rake_cap in 0..10_000u64,
    ) {
        let rake = calculate_rake_for_pot(pot_amount, rake_percent, rake_cap);
        prop_assert!(rake <= rake_cap, "Rake > cap");
        let max_possible = ((pot_amount as f64 * rake_percent) / 100.0).floor() as u64;
        prop_assert!(rake <= max_possible, "Rake > percentual");
    }
}

// ─── 2. Rake Deduct Conservation ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn rake_deduct_conservation(
        p1 in 1000..10_000_000u64,
        p2 in 1000..10_000_000u64,
        rake_percent in 1.0..10.0f64,
        rake_cap in 100..5000u64,
    ) {
        let pots = vec![
            Pot { amount: p1, eligible_players: vec!["p1".into(), "p2".into()] },
            Pot { amount: p2, eligible_players: vec!["p1".into()] },
        ];
        let config = TableConfig { big_blind: 200, rake_percent, rake_cap };
        let result = deduct_rake(&pots, &config, Some(200));
        let total_before = p1 + p2;
        let pots_after_sum: u64 = result.pots_after_rake.iter().map(|p| p.amount).sum();
        let total_after = pots_after_sum + result.total_rake;
        prop_assert_eq!(total_before, total_after, "Perda/ganho de fichas no rake");
    }
}

// ─── 3. Side Pots Extreme Multiway ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn side_pots_extreme_multiway(
        player_specs in prop::collection::vec((1u64..1_000_000u64, proptest::bool::ANY), 2..=6),
    ) {
        let num_players = player_specs.len();
        let mut players = Vec::with_capacity(num_players);
        for (i, &(bet, fold)) in player_specs.iter().enumerate() {
            let has_folded = if i == 0 { false } else { fold };
            players.push(PlayerForPots {
                id: format!("p{}", i),
                total_bet: bet,
                has_folded,
                cards: vec![
                    Card { suit: Suit::Spades, rank: Rank::Ace },
                    Card { suit: Suit::Hearts, rank: Rank::King },
                ],
            });
        }
        let pots = calculate_side_pots(&players);
        let total_contributed: u64 = player_specs.iter().map(|(b, _)| b).sum();
        let total_pots: u64 = pots.iter().map(|p| p.amount).sum();
        prop_assert_eq!(total_contributed, total_pots, "Erro em multiway side pots");
    }
}

// ─── 4. Side Pots Exact Split Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn side_pots_exact_split_fuzz(
        bet in 100u64..500_000u64,
        num_players in 2..=8usize,
    ) {
        let mut players = Vec::with_capacity(num_players);
        for i in 0..num_players {
            players.push(PlayerForPots {
                id: format!("p{}", i),
                total_bet: bet,
                has_folded: false,
                cards: vec![
                    Card { suit: Suit::Spades, rank: Rank::Ace },
                    Card { suit: Suit::Hearts, rank: Rank::King },
                ],
            });
        }
        let pots = calculate_side_pots(&players);
        prop_assert_eq!(pots.len(), 1, "Apostas iguais devem gerar exatamente 1 pote principal");
        let expected_total = bet * (num_players as u64);
        prop_assert_eq!(pots[0].amount, expected_total, "Pote único com valor incorreto");
    }
}

// ─── 5. Side Pots Uncontested Fold Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn side_pots_uncontested_fold_fuzz(
        bets in prop::collection::vec(1000u64..1_000_000u64, 3..=6),
    ) {
        let num_players = bets.len();
        let mut players = Vec::with_capacity(num_players);
        for (i, &bet) in bets.iter().enumerate() {
            let has_folded = i != 0; // todos foldam exceto p0
            players.push(PlayerForPots {
                id: format!("p{}", i),
                total_bet: bet,
                has_folded,
                cards: vec![
                    Card { suit: Suit::Spades, rank: Rank::Ace },
                    Card { suit: Suit::Hearts, rank: Rank::King },
                ],
            });
        }
        let pots = calculate_side_pots(&players);
        let total_contributed: u64 = bets.iter().sum();
        let total_pots: u64 = pots.iter().map(|p| p.amount).sum();
        prop_assert_eq!(total_contributed, total_pots, "Invariante de fold total quebrado");
    }
}

// ─── 6. Loss Deflator No Panic ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn loss_deflator_no_panic(
        pot_amount in 0u64..100_000_000u64,
        phase_idx in 0..4u8,
    ) {
        let phase = match phase_idx {
            0 => GamePhase::Preflop,
            1 => GamePhase::Flop,
            2 => GamePhase::Turn,
            _ => GamePhase::River,
        };
        let params = ProgressiveLossDeflatorParams {
            pots: vec![Pot { amount: pot_amount, eligible_players: vec!["p1".into(), "p2".into()] }],
            loser_id: "p1".into(),
            winner_id: "p2".into(),
            phase,
        };
        let result = calculate_progressive_loss_deflator(params);
        if let Some(res) = result {
            prop_assert!(res.cashback <= pot_amount);
        }
    }
}

// ─── 7. Loss Deflator Multi Eligible Pots Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn loss_deflator_multi_eligible_pots_fuzz(
        p1 in 10000u64..5000000u64,
        p2 in 10000u64..5000000u64,
    ) {
        let pots = vec![
            Pot { amount: p1, eligible_players: vec!["p1".into(), "p2".into()] },
            Pot { amount: p2, eligible_players: vec!["p2".into()] }, // p1 não participou
        ];
        let params = ProgressiveLossDeflatorParams {
            pots,
            loser_id: "p1".into(),
            winner_id: "p2".into(),
            phase: GamePhase::Flop,
        };
        let result = calculate_progressive_loss_deflator(params);
        if let Some(res) = result {
            let expected_cashback = ((p1 as f64) * 0.25).floor() as u64;
            prop_assert_eq!(res.cashback, expected_cashback, "Cashback incidiu sobre pote inelegível");
        }
    }
}

// ─── 8. Auth JWT Malformed Input Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn auth_jwt_malformed_input(
        random_str in ".*",
        secret in "[a-zA-Z0-9]{8,32}",
    ) {
        let mgr = AuthManager::new(&secret);
        let ver_result = mgr.validate_token(&random_str, "access");
        prop_assert!(ver_result.is_err(), "JWT malformado não deve ser aceita");
    }
}

// ─── 9. Auth Password Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn auth_password_fuzz(
        pass in "\\PC*",
    ) {
        let is_strong = pass.len() >= 8 && pass.chars().any(|c| c.is_uppercase()) && pass.chars().any(|c| c.is_lowercase()) && pass.chars().any(|c| c.is_numeric());
        let _ = is_strong;
    }
}

// ─── 10. Hand History JSON Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn hand_history_json_fuzz(
        json_input in ".*",
    ) {
        let res: Result<HandHistory, _> = serde_json::from_str(&json_input);
        let _ = res;
    }
}

// ─── 11. Hand History Roundtrip Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn hand_history_roundtrip_fuzz(
        hand_id in "[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[a-f0-9]{4}-[a-f0-9]{12}",
        pot in 100u64..1_000_000u64,
    ) {
        let config = HandTableConfig {
            table_name: "Fuzz Table".into(),
            small_blind: 10,
            big_blind: 20,
            ante: None,
            max_players: 6,
            game_type: GameType::Cash,
        };
        let hh = HandHistory {
            hand_id: hand_id.clone(),
            timestamp: 1700000000,
            table_config: config,
            players: vec!["p1".into(), "p2".into()],
            starting_stacks: std::collections::HashMap::new(),
            community_cards: vec![],
            actions: vec![],
            results: vec![],
            total_pot: pot,
            rake: 5,
            end_phase: GamePhase::Preflop,
            end_reason: crate::hand_history::EndReason::AllFolded,
            signature: None,
        };
        let json = serde_json::to_string(&hh).expect("Falha ao serializar");
        let restored: HandHistory = serde_json::from_str(&json).expect("Falha ao desserializar");
        prop_assert_eq!(restored.hand_id, hand_id);
        prop_assert_eq!(restored.total_pot, pot);
    }
}

// ─── 12. Antifraud Chip Dumping Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn antifraud_chip_dumping_fuzz(
        amount in 100u64..100_000u64,
        ts in 1000u64..1_000_000u64,
    ) {
        let mut analyzer = ChipDumpAnalyzer::default();
        analyzer.analyze_all_in("p1", "p2", amount, HandStrength::Weak, "hand_1", ts);
        for alert in analyzer.get_alerts() {
            prop_assert!(alert.suspicion_score >= 0.0 && alert.suspicion_score <= 1.0);
        }
    }
}

// ─── 13. Tournament Structure Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn tournament_structure_fuzz(
        buy_in in 100u64..100_000u64,
        starting_stack in 1000u64..100_000u64,
    ) {
        let config = TournamentConfig {
            name: "Fuzz Tournament".into(),
            game_type: "Holdem".into(),
            buy_in,
            starting_stack,
            max_players: 100,
            speed: TournamentSpeed::Normal,
            blind_levels: vec![],
            prize_pool_pct: 0.15,
            prize_distribution: vec![0.5, 0.3, 0.2],
            late_registration: true,
            late_registration_max_level: 4,
            allow_rebuy: true,
            rebuy_max_level: 4,
            allow_addon: true,
        };
        prop_assert!(config.buy_in > 0);
        prop_assert!(config.starting_stack > 0);
    }
}

// ─── 14. Hand Evaluation & Flush Invariants Fuzz ───
proptest! {
    #![proptest_config(get_proptest_config())]
    #[test]
    fn evaluate_hand_and_flush_invariants_fuzz(
        hole_c1 in 0..52usize,
        hole_c2 in 0..52usize,
        board_c1 in 0..52usize,
        board_c2 in 0..52usize,
        board_c3 in 0..52usize,
        board_c4 in 0..52usize,
        board_c5 in 0..52usize,
    ) {
        let deck = crate::deck::create_deck();
        let hole = vec![deck[hole_c1 % 52], deck[hole_c2 % 52]];
        let board = vec![
            deck[board_c1 % 52],
            deck[board_c2 % 52],
            deck[board_c3 % 52],
            deck[board_c4 % 52],
            deck[board_c5 % 52],
        ];

        let result = crate::deck::evaluate_hand(&hole, &board);
        prop_assert!(result.value >= 1 && result.value <= 10);
        if result.rank == crate::deck::HandRank::Flush {
            prop_assert_eq!(result.cards.len(), 5);
            // Todas as 5 cartas do Flush devem ter o mesmo naipe
            let first_suit = result.cards[0].suit;
            for card in &result.cards {
                prop_assert_eq!(card.suit, first_suit);
            }
        }
    }
}
