use poker_engine::deck::{Card, Rank, Suit};
use poker_engine::game_loop::{GameLoop, PlayerState};
use poker_engine::hand_history::GameType;
use poker_engine::loss_deflator::LossDeflatorTier;
use poker_engine::types::{GamePhase, TableConfig};

fn card(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

fn player(
    id: &str,
    stack: u64,
    total_bet: u64,
    hole_cards: Vec<Card>,
    is_all_in: bool,
    all_in_phase: Option<GamePhase>,
    seat_index: usize,
) -> PlayerState {
    PlayerState {
        id: id.to_string(),
        stack,
        hole_cards,
        current_bet: total_bet,
        total_bet,
        has_folded: false,
        is_all_in,
        all_in_phase,
        has_acted: true,
        seat_index,
    }
}

#[test]
fn preflop_seven_percent_uses_only_the_post_rake_pot() {
    let mut game = GameLoop::new(
        TableConfig::new(2, 500, 100),
        "preflop-seven-percent".to_string(),
        "Regression Table".to_string(),
        GameType::Cash,
    );
    game.state.players = vec![
        player(
            "loser",
            0,
            100,
            vec![
                card(Rank::Ace, Suit::Spades),
                card(Rank::Queen, Suit::Diamonds),
            ],
            true,
            Some(GamePhase::Preflop),
            0,
        ),
        player(
            "winner",
            900,
            100,
            vec![
                card(Rank::King, Suit::Hearts),
                card(Rank::Jack, Suit::Hearts),
            ],
            false,
            None,
            1,
        ),
    ];
    game.state.community_cards = vec![
        card(Rank::Two, Suit::Clubs),
        card(Rank::Five, Suit::Diamonds),
        card(Rank::Seven, Suit::Spades),
        card(Rank::Jack, Suit::Clubs),
        card(Rank::Nine, Suit::Diamonds),
    ];
    game.state.is_finished = true;

    let resolution = game.resolve_hand().unwrap();
    let deflator = resolution
        .loss_deflator
        .as_ref()
        .expect("o perdedor all-in pré-flop deve receber o Loss Deflator");
    let payout_total: u64 = resolution.payouts.values().sum();

    assert_eq!(
        resolution.pots.iter().map(|pot| pot.amount).sum::<u64>(),
        200
    );
    assert_eq!(resolution.rake, 10);
    assert_eq!(deflator.tier, LossDeflatorTier::SevenPercent);
    assert_eq!(deflator.eligible_pot_total, 190);
    assert_eq!(deflator.cashback, 13);
    assert_eq!(resolution.payouts.get("loser"), Some(&13));
    assert_eq!(resolution.payouts.get("winner"), Some(&177));
    assert_eq!(payout_total + resolution.rake, 200);
}
