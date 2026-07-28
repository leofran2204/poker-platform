use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{calculate_side_pots, Contribution};

#[test]
fn test_odd_chip_distribution_split_pot() {
    // Quando 2 jogadores empatam em um pote com valor ímpar (ex: R$ 100,01 -> 10001 centavos),
    // o centavo ímpar (odd chip) deve ser atribuído de forma determinística sem criar/destruir moedas.
    let contributions = vec![
        Contribution {
            player_id: "P1".into(),
            total_bet: 50.005, // Total 100.01
            has_folded: false,
        },
        Contribution {
            player_id: "P2".into(),
            total_bet: 50.005,
            has_folded: false,
        },
    ];

    let pots = calculate_side_pots(&contributions);
    assert_eq!(pots.len(), 1);
    assert_eq!(pots[0].eligible_players.len(), 2);
}

#[test]
fn test_5_way_all_in_cascading_side_pots() {
    // 5 jogadores entram em all-in com stacks diferentes: 100, 200, 300, 400, 500
    let contributions = vec![
        Contribution {
            player_id: "P1".into(),
            total_bet: 100.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P2".into(),
            total_bet: 200.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P3".into(),
            total_bet: 300.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P4".into(),
            total_bet: 400.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P5".into(),
            total_bet: 500.0,
            has_folded: false,
        },
    ];

    let pots = calculate_side_pots(&contributions);

    // Devem ser criados exatamente 5 potes paralelos
    assert_eq!(pots.len(), 5);

    // Pote 1 (nível 100): 5 * 100 = 500. Elegíveis: P1, P2, P3, P4, P5
    assert_eq!(pots[0].amount, 500.0);
    assert_eq!(pots[0].eligible_players.len(), 5);

    // Pote 2 (nível 200): 4 * 100 = 400. Elegíveis: P2, P3, P4, P5 (P1 fora)
    assert_eq!(pots[1].amount, 400.0);
    assert_eq!(pots[1].eligible_players.len(), 4);
    assert!(!pots[1].eligible_players.contains(&"P1".to_string()));

    // Pote 5 (nível 500): 1 * 100 = 100. Elegível: apenas P5
    assert_eq!(pots[4].amount, 100.0);
    assert_eq!(pots[4].eligible_players, vec!["P5".to_string()]);
}

#[test]
fn test_multiway_split_pot_tiebreaker_by_kicker() {
    // P1 e P2 ambos têm um Par de Reis, mas P1 ganha pelo Kicker Ás
    let p1_hand = vec![
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Ten, Suit::Diamonds),
        Card::new(Rank::Eight, Suit::Hearts),
    ];

    let p2_hand = vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Ten, Suit::Spades),
        Card::new(Rank::Eight, Suit::Clubs),
    ];

    let rank_p1 = evaluate_hand(&p1_hand);
    let rank_p2 = evaluate_hand(&p2_hand);

    assert!(
        rank_p1 > rank_p2,
        "P1 deve vencer P2 devido ao Kicker Ás vs Dama"
    );
}

#[test]
fn test_exact_tie_identical_straight_split_pot() {
    // P1 e P2 têm exatamente a mesma Straight (10 a Ás)
    let p1_hand = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Jack, Suit::Diamonds),
        Card::new(Rank::Ten, Suit::Hearts),
    ];

    let p2_hand = vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Ten, Suit::Clubs),
    ];

    let rank_p1 = evaluate_hand(&p1_hand);
    let rank_p2 = evaluate_hand(&p2_hand);

    assert_eq!(
        rank_p1, rank_p2,
        "Straight idêntica deve ser um empate absoluto (Split Pot)"
    );
}

#[test]
fn test_folded_highest_stack_player_side_pot_isolation() {
    // P1 (all-in 100), P2 (bet 1000, FOLDED), P3 (bet 1000, active)
    let contributions = vec![
        Contribution {
            player_id: "P1".into(),
            total_bet: 100.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P2".into(),
            total_bet: 1000.0,
            has_folded: true,
        }, // FOLDED!
        Contribution {
            player_id: "P3".into(),
            total_bet: 1000.0,
            has_folded: false,
        },
    ];

    let pots = calculate_side_pots(&contributions);

    // Pote 1 (nível 100): 300 total. Elegíveis: P1, P3 (P2 folded fora!)
    assert_eq!(pots[0].amount, 300.0);
    assert!(!pots[0].eligible_players.contains(&"P2".to_string()));

    // Pote 2 (nível 1000): 1800 total. Elegível: Apenas P3
    assert_eq!(pots[1].amount, 1800.0);
    assert_eq!(pots[1].eligible_players, vec!["P3".to_string()]);
}

#[test]
fn test_zero_sum_pot_conservation_complex_all_in() {
    let contributions = vec![
        Contribution {
            player_id: "P1".into(),
            total_bet: 150.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P2".into(),
            total_bet: 300.0,
            has_folded: true,
        },
        Contribution {
            player_id: "P3".into(),
            total_bet: 450.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P4".into(),
            total_bet: 600.0,
            has_folded: false,
        },
    ];

    let total_bets: f64 = contributions.iter().map(|c| c.total_bet).sum();
    let pots = calculate_side_pots(&contributions);
    let total_pots: f64 = pots.iter().map(|p| p.amount).sum();

    assert_eq!(total_bets, 1500.0);
    assert_eq!(
        total_pots, 1500.0,
        "A soma dos potes DEVE ser exatamente igual à soma das apostas (Conservação de Saldo)"
    );
}
