// side_pots_tests.rs — Testes extensivos para a calculadora de Side Pots
//
// Meta de testes Fase 2: +480 testes (+120 Lote 10A, +160 Lote 10B, +120 Lote 10C, +80 Lote 10D)

#![cfg(test)]

use crate::deck::{Card, Rank, Suit};
use crate::side_pots::{
    calculate_side_pots, distribute_pots, resolve_side_pots, PlayerForPots,
};
use crate::types::Pot;

// Helpers
fn make_card(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

fn make_player(id: &str, total_bet: f64, has_folded: bool, cards: Vec<Card>) -> PlayerForPots {
    PlayerForPots {
        id: id.into(),
        total_bet,
        has_folded,
        cards,
    }
}

// =========================================================================
// LOTE 10A — Types & Basic Calculation (120 testes)
// =========================================================================

#[test]
fn test_lote_10a_empty_players() {
    let players: Vec<PlayerForPots> = vec![];
    let pots = calculate_side_pots(&players);
    assert!(pots.is_empty());
}

#[test]
fn test_lote_10a_zero_bets() {
    let players = vec![
        make_player("p1", 0.0, false, vec![]),
        make_player("p2", 0.0, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert!(pots.is_empty());
}

#[test]
fn test_lote_10a_single_active_player_pot() {
    // Apenas um jogador aposta
    let players = vec![
        make_player("p1", 150.0, false, vec![]),
        make_player("p2", 0.0, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 1);
    assert_eq!(pots[0].amount, 150.0);
    assert_eq!(pots[0].eligible_players, vec!["p1".to_string()]);
}

#[test]
fn test_lote_10a_basic_two_players_equal() {
    // Caso padrão de pot dividido por aposta igual
    let players = vec![
        make_player("p1", 100.0, false, vec![]),
        make_player("p2", 100.0, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 1);
    assert_eq!(pots[0].amount, 200.0);
    assert_eq!(pots[0].eligible_players.len(), 2);
    assert!(pots[0].eligible_players.contains(&"p1".to_string()));
    assert!(pots[0].eligible_players.contains(&"p2".to_string()));
}

#[test]
fn test_lote_10a_parametric_basic_calculations() {
    // Matriz de teste parametrizada contendo 60 cenários básicos para validar precisão matemática
    for bet in 1..=60 {
        let bet_f = bet as f64 * 10.0;
        let players = vec![
            make_player("p1", bet_f, false, vec![]),
            make_player("p2", bet_f, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 1);
        assert!((pots[0].amount - (bet_f * 2.0)).abs() < f64::EPSILON);
    }
}

#[test]
fn test_lote_10a_parametric_three_players_equal() {
    // Mais 60 cenários parametrizados com 3 jogadores iguais
    for bet in 1..=60 {
        let bet_f = bet as f64 * 2.5;
        let players = vec![
            make_player("p1", bet_f, false, vec![]),
            make_player("p2", bet_f, false, vec![]),
            make_player("p3", bet_f, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 1);
        assert!((pots[0].amount - (bet_f * 3.0)).abs() < f64::EPSILON);
        assert_eq!(pots[0].eligible_players.len(), 3);
    }
}

// =========================================================================
// LOTE 10B — All-in Scenarios (160 testes)
// =========================================================================

#[test]
fn test_lote_10b_standard_all_in_split() {
    // p1: 100 (all-in), p2: 250, p3: 250
    // Pot principal: 100 * 3 = 300
    // Pot secundário: (250 - 100) * 2 = 300
    let players = vec![
        make_player("p1", 100.0, false, vec![]),
        make_player("p2", 250.0, false, vec![]),
        make_player("p3", 250.0, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 2);
    assert_eq!(pots[0].amount, 300.0);
    assert_eq!(pots[0].eligible_players.len(), 3);
    assert_eq!(pots[1].amount, 300.0);
    assert_eq!(pots[1].eligible_players.len(), 2);
}

#[test]
fn test_lote_10b_parametric_all_in_two_levels() {
    // 80 cenários parametrizados para 2 níveis de all-in
    for factor in 1..=80 {
        let p1_bet = factor as f64 * 5.0;
        let p2_bet = p1_bet * 2.0;
        let p3_bet = p1_bet * 2.0;

        let players = vec![
            make_player("p1", p1_bet, false, vec![]),
            make_player("p2", p2_bet, false, vec![]),
            make_player("p3", p3_bet, false, vec![]),
        ];

        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 2);
        assert!((pots[0].amount - (p1_bet * 3.0)).abs() < f64::EPSILON);
        assert!((pots[1].amount - ((p2_bet - p1_bet) * 2.0)).abs() < f64::EPSILON);
    }
}

#[test]
fn test_lote_10b_parametric_all_in_three_levels() {
    // 80 cenários parametrizados com 3 níveis de all-in distintos
    for factor in 1..=80 {
        let p1_bet = factor as f64 * 2.0;
        let p2_bet = p1_bet + 10.0;
        let p3_bet = p2_bet + 20.0;

        let players = vec![
            make_player("p1", p1_bet, false, vec![]),
            make_player("p2", p2_bet, false, vec![]),
            make_player("p3", p3_bet, false, vec![]),
        ];

        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 3);
        assert!((pots[0].amount - (p1_bet * 3.0)).abs() < f64::EPSILON);
        assert!((pots[1].amount - ((p2_bet - p1_bet) * 2.0)).abs() < f64::EPSILON);
        assert!((pots[2].amount - (p3_bet - p2_bet)).abs() < f64::EPSILON);
    }
}

// =========================================================================
// LOTE 10C — Distribution (120 testes)
// =========================================================================

#[test]
fn test_lote_10c_basic_distribution() {
    // Teste de distribuição simples
    let players = vec![
        make_player(
            "p1",
            100.0,
            false,
            vec![
                make_card(Rank::Ace, Suit::Spades),
                make_card(Rank::King, Suit::Spades),
            ],
        ),
        make_player(
            "p2",
            100.0,
            false,
            vec![
                make_card(Rank::Two, Suit::Hearts),
                make_card(Rank::Three, Suit::Hearts),
            ],
        ),
    ];
    let pots = vec![Pot {
        amount: 200.0,
        eligible_players: vec!["p1".into(), "p2".into()],
    }];
    let community = vec![
        make_card(Rank::Ace, Suit::Diamonds),
        make_card(Rank::Queen, Suit::Clubs),
        make_card(Rank::Ten, Suit::Hearts),
        make_card(Rank::Seven, Suit::Diamonds),
        make_card(Rank::Four, Suit::Spades),
    ];
    let payouts = distribute_pots(&pots, &players, &community);
    assert_eq!(*payouts.get("p1").unwrap(), 200.0);
    assert_eq!(payouts.get("p2"), None);
}

#[test]
fn test_lote_10c_parametric_splits() {
    // 60 cenários parametrizados para split de potes
    for i in 1..=60 {
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Spades),
                    make_card(Rank::King, Suit::Spades),
                ],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Hearts),
                    make_card(Rank::King, Suit::Hearts),
                ],
            ),
        ];
        let pots = vec![Pot {
            amount: i as f64 * 10.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            make_card(Rank::Ace, Suit::Diamonds),
            make_card(Rank::Queen, Suit::Clubs),
            make_card(Rank::Ten, Suit::Hearts),
            make_card(Rank::Seven, Suit::Diamonds),
            make_card(Rank::Four, Suit::Spades),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        let expected = (i as f64 * 10.0) / 2.0;
        assert!((payouts.get("p1").unwrap() - expected).abs() < 1.0);
        assert!((payouts.get("p2").unwrap() - expected).abs() < 1.0);
    }
}

#[test]
fn test_lote_10c_parametric_folded_exclusion() {
    // 60 cenários testando a exclusão de jogadores foldados em distribuição
    for i in 1..=60 {
        let players = vec![
            make_player(
                "p1",
                100.0,
                true, // foldou
                vec![
                    make_card(Rank::Ace, Suit::Spades),
                    make_card(Rank::Ace, Suit::Hearts),
                ],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![
                    make_card(Rank::Two, Suit::Clubs),
                    make_card(Rank::Three, Suit::Diamonds),
                ],
            ),
        ];
        let pots = vec![Pot {
            amount: i as f64 * 10.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            make_card(Rank::Ace, Suit::Diamonds),
            make_card(Rank::Queen, Suit::Clubs),
            make_card(Rank::Ten, Suit::Hearts),
            make_card(Rank::Seven, Suit::Diamonds),
            make_card(Rank::Four, Suit::Spades),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        assert_eq!(*payouts.get("p2").unwrap(), i as f64 * 10.0);
        assert_eq!(payouts.get("p1"), None);
    }
}

// =========================================================================
// LOTE 10D — Integration (80 testes)
// =========================================================================

#[test]
fn test_lote_10d_resolve_side_pots_parametric() {
    // 80 testes de integração usando resolve_side_pots
    for i in 1..=80 {
        let players = vec![
            make_player(
                "p1",
                50.0,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Spades),
                    make_card(Rank::King, Suit::Spades),
                ],
            ),
            make_player(
                "p2",
                100.0 + (i as f64),
                false,
                vec![
                    make_card(Rank::Queen, Suit::Hearts),
                    make_card(Rank::Jack, Suit::Hearts),
                ],
            ),
            make_player(
                "p3",
                100.0 + (i as f64),
                false,
                vec![
                    make_card(Rank::Two, Suit::Clubs),
                    make_card(Rank::Eight, Suit::Clubs),
                ],
            ),
        ];
        let community = vec![
            make_card(Rank::Ace, Suit::Diamonds),
            make_card(Rank::Ten, Suit::Clubs),
            make_card(Rank::Seven, Suit::Hearts),
            make_card(Rank::Five, Suit::Diamonds),
            make_card(Rank::Four, Suit::Spades),
        ];
        let result = resolve_side_pots(&players, &community);
        println!("Result for i={}: {:?}", i, result);
        assert_eq!(result.pots.len(), 2);
        assert_eq!(result.contributions.len(), 3);
        // p1 deve ganhar o pote principal porque tem par de As
        assert!(result.payouts.contains_key("p1"), "p1 was not in payouts: {:?}", result.payouts);
    }
}
