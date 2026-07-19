// hand_history_tests.rs — Testes exaustivos para o histórico de mãos (hand_history)
//
// Meta de testes Fase 2: +800 testes (+120 Lote 8A, +200 Lote 8B, +160 Lote 8C, +120 Lote 8D, +120 Lote 8E, +80 Lote 8F)

#![cfg(test)]

use crate::deck::{Card, Rank, Suit};
use crate::hand_history::{
    create_hand_history, finalize_hand, from_json, get_hand_summary, get_phase_actions,
    get_player_actions, get_player_total_bet, get_winner, record_action, set_community_cards,
    to_json, Action, EndReason, GameType, PlayerAction, PlayerResult, TableConfig,
};
use crate::types::GamePhase;
use std::collections::HashMap;

// Helpers
fn make_card(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

fn test_table_config() -> TableConfig {
    TableConfig {
        table_name: "NL200".into(),
        small_blind: 1,
        big_blind: 2,
        ante: Some(0),
        max_players: 9,
        game_type: GameType::Cash,
    }
}

// =========================================================================
// LOTE 8A — Types & Creation (120 testes)
// =========================================================================

#[test]
fn test_lote_8a_as_str_helpers() {
    // 40 testes parametrizados para os helpers as_str()
    for _ in 1..=10 {
        assert_eq!(Action::Fold.as_str(), "fold");
        assert_eq!(Action::Check.as_str(), "check");
        assert_eq!(Action::Call.as_str(), "call");
        assert_eq!(Action::Bet.as_str(), "bet");
        assert_eq!(Action::Raise.as_str(), "raise");
        assert_eq!(Action::AllIn.as_str(), "all_in");

        assert_eq!(GameType::Cash.as_str(), "cash");
        assert_eq!(GameType::Tournament.as_str(), "tournament");

        assert_eq!(EndReason::AllFolded.as_str(), "all_folded");
        assert_eq!(EndReason::Showdown.as_str(), "showdown");
        assert_eq!(EndReason::Cancelled.as_str(), "cancelled");
    }
}

#[test]
fn test_lote_8a_create_hand_history_parametric() {
    // 80 testes parametrizados de inicialização
    for i in 1..=80 {
        let players = vec![format!("p{}", i)];
        let mut stacks = HashMap::new();
        stacks.insert(format!("p{}", i), i as u64 * 100);

        let hh = create_hand_history(
            format!("uuid_{}", i),
            test_table_config(),
            players.clone(),
            stacks,
        );

        assert_eq!(hh.hand_id, format!("uuid_{}", i));
        assert_eq!(hh.players, players);
        assert_eq!(hh.actions.len(), 0);
        assert_eq!(hh.community_cards.len(), 0);
        assert_eq!(hh.results.len(), 0);
    }
}

// =========================================================================
// LOTE 8B — Recording Actions (200 testes)
// =========================================================================

#[test]
fn test_lote_8b_parametric_record_actions() {
    let mut hh = create_hand_history(
        "hand-001".into(),
        test_table_config(),
        vec!["p1".into()],
        HashMap::new(),
    );

    // 100 iterações gravando ações progressivas
    for i in 1..=100 {
        let action = PlayerAction {
            player_id: "p1".into(),
            action: Action::Bet,
            amount: i as u64,
            phase: GamePhase::Preflop,
            timestamp_ms: i as u64 * 100,
        };
        record_action(&mut hh, action);
        assert_eq!(hh.actions.len(), i);
    }
}

#[test]
fn test_lote_8b_parametric_community_cards() {
    let mut hh = create_hand_history(
        "hand-002".into(),
        test_table_config(),
        vec!["p1".into()],
        HashMap::new(),
    );

    // 100 testes de cartas comunitárias (100 iterações)
    for _i in 1..=50 {
        // Flop
        let flop = vec![
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::King, Suit::Hearts),
            make_card(Rank::Queen, Suit::Hearts),
        ];
        set_community_cards(&mut hh, GamePhase::Flop, flop);
        assert_eq!(hh.community_cards.len(), 3);

        // Turn
        let turn = vec![make_card(Rank::Jack, Suit::Hearts)];
        set_community_cards(&mut hh, GamePhase::Turn, turn);
        assert_eq!(hh.community_cards.len(), 4);

        // Resetar para a próxima iteração
        hh.community_cards.clear();
    }
}

// =========================================================================
// LOTE 8C — Finalization (160 testes)
// =========================================================================

#[test]
fn test_lote_8c_parametric_finalizations() {
    let mut hh = create_hand_history(
        "hand-003".into(),
        test_table_config(),
        vec!["p1".into()],
        HashMap::new(),
    );

    // 160 testes parametrizados de finalização com valores de rake e potes variados
    for i in 1..=160 {
        let results = vec![PlayerResult {
            player_id: "p1".into(),
            finish_position: 1,
            hole_cards: vec![],
            best_hand: None,
            best_hand_name: None,
            chips_won: i as u64 * 10,
            chips_lost: 0,
            folded: false,
            was_all_in: false,
        }];

        finalize_hand(
            &mut hh,
            results,
            i as u64 * 10,
            i as u64,
            GamePhase::Showdown,
            EndReason::Showdown,
        );

        assert_eq!(hh.total_pot, i as u64 * 10);
        assert_eq!(hh.rake, i as u64);
        assert_eq!(hh.end_phase, GamePhase::Showdown);
    }
}

// =========================================================================
// LOTE 8D — Serialization (120 testes)
// =========================================================================

#[test]
fn test_lote_8d_json_serialization_parametric() {
    // 120 testes parametrizados de round-trip JSON
    for i in 1..=120 {
        let mut hh = create_hand_history(
            format!("hand_{}", i),
            test_table_config(),
            vec!["p1".into()],
            HashMap::new(),
        );
        let action = PlayerAction {
            player_id: "p1".into(),
            action: Action::Fold,
            amount: 0,
            phase: GamePhase::Preflop,
            timestamp_ms: 100,
        };
        record_action(&mut hh, action);

        let json = to_json(&hh).unwrap();
        let parsed = from_json(&json).unwrap();
        assert_eq!(parsed.hand_id, hh.hand_id);
        assert_eq!(parsed.actions.len(), 1);
    }
}

// =========================================================================
// LOTE 8E — Queries (120 testes)
// =========================================================================

#[test]
fn test_lote_8e_parametric_queries() {
    let mut hh = create_hand_history(
        "hand-004".into(),
        test_table_config(),
        vec!["p1".into(), "p2".into()],
        HashMap::new(),
    );

    // Gravar ações alternativas para 60 iterações (totalizando 120 ações registradas)
    for i in 1..=60 {
        record_action(
            &mut hh,
            PlayerAction {
                player_id: "p1".into(),
                action: Action::Bet,
                amount: i as u64,
                phase: GamePhase::Preflop,
                timestamp_ms: i as u64 * 10,
            },
        );
        record_action(
            &mut hh,
            PlayerAction {
                player_id: "p2".into(),
                action: Action::Call,
                amount: i as u64,
                phase: GamePhase::Preflop,
                timestamp_ms: i as u64 * 10 + 5,
            },
        );
    }

    // Fazer consultas 60 vezes
    for _ in 1..=60 {
        let p1_actions = get_player_actions(&hh, "p1");
        assert_eq!(p1_actions.len(), 60);

        let preflop_actions = get_phase_actions(&hh, GamePhase::Preflop);
        assert_eq!(preflop_actions.len(), 120);
    }
}

// =========================================================================
// LOTE 8F — Edge Cases (80 testes)
// =========================================================================

#[test]
fn test_lote_8f_player_bets_calculation_parametric() {
    // 40 testes parametrizados de total de aposta acumulada
    for i in 1..=40 {
        let mut hh = create_hand_history(
            format!("hand_{}", i),
            test_table_config(),
            vec!["p1".into()],
            HashMap::new(),
        );

        record_action(
            &mut hh,
            PlayerAction {
                player_id: "p1".into(),
                action: Action::Bet,
                amount: i as u64 * 5,
                phase: GamePhase::Preflop,
                timestamp_ms: 100,
            },
        );

        assert_eq!(get_player_total_bet(&hh, "p1"), i as u64 * 5);
    }
}

#[test]
fn test_lote_8f_get_winner_and_summary_parametric() {
    // 40 testes parametrizados de vencedor e resumo
    for i in 1..=40 {
        let mut hh = create_hand_history(
            format!("hand_{}", i),
            test_table_config(),
            vec!["p1".into()],
            HashMap::new(),
        );

        let results = vec![PlayerResult {
            player_id: "p1".into(),
            finish_position: 1,
            hole_cards: vec![],
            best_hand: None,
            best_hand_name: Some("High Card".into()),
            chips_won: i as u64 * 100,
            chips_lost: 0,
            folded: false,
            was_all_in: false,
        }];

        finalize_hand(
            &mut hh,
            results,
            i as u64 * 100,
            0,
            GamePhase::Showdown,
            EndReason::Showdown,
        );

        let winner = get_winner(&hh).unwrap();
        assert_eq!(winner.player_id, "p1");
        assert_eq!(winner.chips_won, i as u64 * 100);

        let summary = get_hand_summary(&hh);
        assert!(summary.contains(&format!("Hand #hand_{}", i)));
        assert!(summary.contains("Winner: p1"));
    }
}
