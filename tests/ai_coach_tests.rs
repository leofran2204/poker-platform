use poker_engine::analytics::{AiCoach, EquityCalculator, GtoRecommendation};
use poker_engine::engine::evaluator::{Card, Rank, Suit};

#[test]
fn test_pot_odds_and_ev_calculation() {
    // Pote de R$ 100, aposta a pagar de R$ 50 -> Pot Odds = 50 / (100 + 50) = 33.33%
    let pot_odds = AiCoach::calculate_pot_odds(50.0, 100.0);
    assert!((pot_odds - 33.333).abs() < 0.01);

    // Com 50% de equidade, EV = (0.5 * 100) - (0.5 * 50) = +R$ 25,00
    let equity_result = poker_engine::analytics::EquityResult {
        win_percentage: 50.0,
        tie_percentage: 0.0,
        loss_percentage: 50.0,
        total_simulations: 1000,
    };
    let ev = AiCoach::calculate_expected_value(&equity_result, 100.0, 50.0);
    assert_eq!(ev, 25.0);
}

#[test]
fn test_pocket_aces_preflop_equity() {
    let hole_cards = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
    ];
    let community_cards = vec![]; // Preflop

    // Simulação Monte Carlo contra 1 oponente aleatório
    let result = EquityCalculator::calculate_equity(&hole_cards, &community_cards, 1, 2_000);
    
    // Pocket Aces deve ter aproximadamente 85% de vitoria preflop contra 1 oponente aleatório
    assert!(result.win_percentage > 78.0, "AA Preflop deve ter equidade > 78% (Obtido: {})", result.win_percentage);
}

#[test]
fn test_gto_fold_recommendation_negative_ev() {
    let weak_cards = vec![
        Card::new(Rank::Two, Suit::Spades),
        Card::new(Rank::Seven, Suit::Hearts),
    ];
    let board = vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::Queen, Suit::Diamonds),
        Card::new(Rank::Jack, Suit::Spades),
    ];

    // Pote R$ 100, aposta pesada de R$ 200 a pagar
    let advice = AiCoach::analyze_hand(&weak_cards, &board, 1, 100.0, 200.0, 1000);
    
    assert_eq!(advice.recommendation, GtoRecommendation::Fold);
    assert!(advice.expected_value < 0.0);
}

#[test]
fn test_gto_value_bet_recommendation() {
    let strong_cards = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
    ];
    let board = vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Two, Suit::Spades),
    ]; // Trinca de Ases

    // Pote R$ 100, sem aposta a pagar (Call amount = 0.0)
    let advice = AiCoach::analyze_hand(&strong_cards, &board, 1, 100.0, 0.0, 1000);

    assert!(matches!(advice.recommendation, GtoRecommendation::ValueBet(_)));
}
