use poker_engine::analytics::{AiCoach, SimpleAction};
use poker_engine::engine::evaluator::{Card, Rank, Suit};

#[test]
fn test_pot_odds_and_ev_calculation() {
    let pot_odds = AiCoach::calculate_pot_odds(50.0, 100.0);
    assert!((pot_odds - 33.333).abs() < 0.01);

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
fn test_friendly_coach_advice_and_opponent_range() {
    let player_cards = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
    ];
    let board = vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Two, Suit::Spades),
    ];

    let advice = AiCoach::analyze_hand_friendly(&player_cards, &board, 1, 100.0, 20.0, 1000);

    // Deve ser uma recomendação amigável clara
    assert!(advice.headline.contains("Dica do Coach"));
    assert!(matches!(advice.simple_action, SimpleAction::AumentarBlefe(_)) || matches!(advice.simple_action, SimpleAction::PagarAposta));
    assert!(advice.win_chance_label.contains("Excelente"));
    assert!(!advice.opponent_range.likely_hand_types.is_empty());
}

#[test]
fn test_friendly_coach_fold_recommendation() {
    let weak_cards = vec![
        Card::new(Rank::Two, Suit::Spades),
        Card::new(Rank::Seven, Suit::Hearts),
    ];
    let board = vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::Queen, Suit::Diamonds),
        Card::new(Rank::Jack, Suit::Spades),
    ];

    let advice = AiCoach::analyze_hand_friendly(&weak_cards, &board, 1, 100.0, 200.0, 1000);

    assert_eq!(advice.simple_action, SimpleAction::Desistir);
    assert!(advice.headline.contains("Desistir"));
}
