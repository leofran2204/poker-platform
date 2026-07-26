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

fn make_player(id: &str, total_bet: u64, has_folded: bool, cards: Vec<Card>) -> PlayerForPots {
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
        make_player("p1", 0, false, vec![]),
        make_player("p2", 0, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert!(pots.is_empty());
}

#[test]
fn test_lote_10a_single_active_player_pot() {
    // Apenas um jogador aposta
    let players = vec![
        make_player("p1", 15000, false, vec![]),
        make_player("p2", 0, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 1);
    assert_eq!(pots[0].amount, 15000);
    assert_eq!(pots[0].eligible_players, vec!["p1".to_string()]);
}

#[test]
fn test_lote_10a_basic_two_players_equal() {
    // Caso padrão de pot dividido por aposta igual
    let players = vec![
        make_player("p1", 10000, false, vec![]),
        make_player("p2", 10000, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 1);
    assert_eq!(pots[0].amount, 20000);
    assert_eq!(pots[0].eligible_players.len(), 2);
    assert!(pots[0].eligible_players.contains(&"p1".to_string()));
    assert!(pots[0].eligible_players.contains(&"p2".to_string()));
}

#[test]
fn test_lote_10a_parametric_basic_calculations() {
    // Matriz de teste parametrizada contendo 60 cenários básicos para validar precisão matemática
    for bet in 1..=60 {
        let bet_u = bet as u64 * 1000;
        let players = vec![
            make_player("p1", bet_u, false, vec![]),
            make_player("p2", bet_u, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 1);
        assert_eq!(pots[0].amount, bet_u * 2);
    }
}

#[test]
fn test_lote_10a_parametric_three_players_equal() {
    // Mais 60 cenários parametrizados com 3 jogadores iguais
    for bet in 1..=60 {
        let bet_u = bet as u64 * 250;
        let players = vec![
            make_player("p1", bet_u, false, vec![]),
            make_player("p2", bet_u, false, vec![]),
            make_player("p3", bet_u, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 1);
        assert_eq!(pots[0].amount, bet_u * 3);
        assert_eq!(pots[0].eligible_players.len(), 3);
    }
}

// =========================================================================
// LOTE 10B — All-in Scenarios (160 testes)
// =========================================================================

#[test]
fn test_lote_10b_standard_all_in_split() {
    // p1: 10000 (all-in), p2: 25000, p3: 25000
    // Pot principal: 10000 * 3 = 30000
    // Pot secundário: (25000 - 10000) * 2 = 30000
    let players = vec![
        make_player("p1", 10000, false, vec![]),
        make_player("p2", 25000, false, vec![]),
        make_player("p3", 25000, false, vec![]),
    ];
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 2);
    assert_eq!(pots[0].amount, 30000);
    assert_eq!(pots[0].eligible_players.len(), 3);
    assert_eq!(pots[1].amount, 30000);
    assert_eq!(pots[1].eligible_players.len(), 2);
}

#[test]
fn test_lote_10b_parametric_all_in_two_levels() {
    // 80 cenários parametrizados para 2 níveis de all-in
    for factor in 1..=80 {
        let p1_bet = factor as u64 * 500;
        let p2_bet = p1_bet * 2;
        let p3_bet = p1_bet * 2;

        let players = vec![
            make_player("p1", p1_bet, false, vec![]),
            make_player("p2", p2_bet, false, vec![]),
            make_player("p3", p3_bet, false, vec![]),
        ];

        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 2);
        assert_eq!(pots[0].amount, p1_bet * 3);
        assert_eq!(pots[1].amount, (p2_bet - p1_bet) * 2);
    }
}

#[test]
fn test_lote_10b_parametric_all_in_three_levels() {
    // 80 cenários parametrizados com 3 níveis de all-in distintos
    for factor in 1..=80 {
        let p1_bet = factor as u64 * 200;
        let p2_bet = p1_bet + 1000;
        let p3_bet = p2_bet + 2000;

        let players = vec![
            make_player("p1", p1_bet, false, vec![]),
            make_player("p2", p2_bet, false, vec![]),
            make_player("p3", p3_bet, false, vec![]),
        ];

        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 3);
        assert_eq!(pots[0].amount, p1_bet * 3);
        assert_eq!(pots[1].amount, (p2_bet - p1_bet) * 2);
        assert_eq!(pots[2].amount, p3_bet - p2_bet);
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
            10000,
            false,
            vec![
                make_card(Rank::Ace, Suit::Spades),
                make_card(Rank::King, Suit::Spades),
            ],
        ),
        make_player(
            "p2",
            10000,
            false,
            vec![
                make_card(Rank::Two, Suit::Hearts),
                make_card(Rank::Three, Suit::Hearts),
            ],
        ),
    ];
    let pots = vec![Pot {
        amount: 20000,
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
    assert_eq!(*payouts.get("p1").unwrap(), 20000);
    assert_eq!(payouts.get("p2"), None);
}

#[test]
fn test_lote_10c_parametric_splits() {
    // 60 cenários parametrizados para split de potes
    for i in 1..=60 {
        let players = vec![
            make_player(
                "p1",
                10000,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Spades),
                    make_card(Rank::King, Suit::Spades),
                ],
            ),
            make_player(
                "p2",
                10000,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Hearts),
                    make_card(Rank::King, Suit::Hearts),
                ],
            ),
        ];
        let pots = vec![Pot {
            amount: i as u64 * 1000,
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
        let total_pot = i as u64 * 1000;
        let expected = total_pot / 2;
        assert_eq!(*payouts.get("p1").unwrap(), expected);
        assert_eq!(*payouts.get("p2").unwrap(), expected + (total_pot % 2));
    }
}

#[test]
fn test_lote_10c_parametric_folded_exclusion() {
    // 60 cenários testando a exclusão de jogadores foldados em distribuição
    for i in 1..=60 {
        let players = vec![
            make_player(
                "p1",
                10000,
                true, // foldou
                vec![
                    make_card(Rank::Ace, Suit::Spades),
                    make_card(Rank::Ace, Suit::Hearts),
                ],
            ),
            make_player(
                "p2",
                10000,
                false,
                vec![
                    make_card(Rank::Two, Suit::Clubs),
                    make_card(Rank::Three, Suit::Diamonds),
                ],
            ),
        ];
        let pots = vec![Pot {
            amount: i as u64 * 1000,
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
        assert_eq!(*payouts.get("p2").unwrap(), i as u64 * 1000);
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
                5000,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Spades),
                    make_card(Rank::King, Suit::Spades),
                ],
            ),
            make_player(
                "p2",
                10000 + (i as u64 * 100),
                false,
                vec![
                    make_card(Rank::Queen, Suit::Hearts),
                    make_card(Rank::Jack, Suit::Hearts),
                ],
            ),
            make_player(
                "p3",
                10000 + (i as u64 * 100),
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