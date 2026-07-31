use poker_engine::deck::{Card, Rank, Suit};
use poker_engine::game_loop::{GameLoop, PlayerMove, PlayerState};
use poker_engine::hand_history::GameType;
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

fn game_with_stacks(stacks: &[u64]) -> GameLoop {
    let mut game = GameLoop::new(
        TableConfig::new(10, 0, 0),
        "phase-regression".to_string(),
        "Regression Table".to_string(),
        GameType::Cash,
    );
    for (index, stack) in stacks.iter().enumerate() {
        game.add_player(format!("p{index}"), *stack);
    }
    game.set_dealer(0);
    game
}

#[test]
fn explicit_all_in_records_preflop_phase() {
    let mut game = game_with_stacks(&[100, 100]);
    game.start_hand().unwrap();

    game.player_action("p0", PlayerMove::AllIn).unwrap();

    assert!(game.state.players[0].is_all_in);
    assert_eq!(game.state.players[0].all_in_phase, Some(GamePhase::Preflop));
}

#[test]
fn call_that_exhausts_stack_records_preflop_phase() {
    let mut game = game_with_stacks(&[100, 30]);
    game.start_hand().unwrap();

    game.player_action("p0", PlayerMove::Raise(30)).unwrap();
    game.player_action("p1", PlayerMove::Call).unwrap();

    assert!(game.state.players[1].is_all_in);
    assert_eq!(game.state.players[1].all_in_phase, Some(GamePhase::Preflop));
}

#[test]
fn raise_that_exhausts_stack_records_preflop_phase() {
    let mut game = game_with_stacks(&[30, 100]);
    game.start_hand().unwrap();

    game.player_action("p0", PlayerMove::Raise(30)).unwrap();

    assert!(game.state.players[0].is_all_in);
    assert_eq!(game.state.players[0].all_in_phase, Some(GamePhase::Preflop));
}

#[test]
fn bet_that_exhausts_stack_records_flop_phase() {
    let mut game = game_with_stacks(&[100, 100]);
    game.start_hand().unwrap();
    game.player_action("p0", PlayerMove::Call).unwrap();
    game.player_action("p1", PlayerMove::Check).unwrap();

    game.player_action("p1", PlayerMove::Bet(90)).unwrap();

    assert!(game.state.players[1].is_all_in);
    assert_eq!(game.state.players[1].all_in_phase, Some(GamePhase::Flop));
}

#[test]
fn ante_that_exhausts_non_blind_stack_records_preflop_phase() {
    let mut game = game_with_stacks(&[5, 100, 100]).with_ante(5);
    game.start_hand().unwrap();

    assert!(game.state.players[0].is_all_in);
    assert_eq!(game.state.players[0].all_in_phase, Some(GamePhase::Preflop));
}

#[test]
fn multiple_deflators_never_credit_more_than_winners_pay() {
    let mut game = GameLoop::new(
        TableConfig::new(10, 0, 0),
        "multi-deflator".to_string(),
        "Regression Table".to_string(),
        GameType::Cash,
    );
    game.state.players = vec![
        player(
            "loser-1",
            0,
            100,
            vec![
                card(Rank::Ace, Suit::Spades),
                card(Rank::Ace, Suit::Diamonds),
            ],
            true,
            Some(GamePhase::Turn),
            0,
        ),
        player(
            "loser-2",
            0,
            100,
            vec![
                card(Rank::King, Suit::Spades),
                card(Rank::King, Suit::Diamonds),
            ],
            true,
            Some(GamePhase::Turn),
            1,
        ),
        player(
            "loser-3",
            0,
            100,
            vec![
                card(Rank::Queen, Suit::Spades),
                card(Rank::Queen, Suit::Diamonds),
            ],
            true,
            Some(GamePhase::Turn),
            2,
        ),
        player(
            "winner",
            900,
            100,
            vec![
                card(Rank::Queen, Suit::Clubs),
                card(Rank::Jack, Suit::Clubs),
            ],
            false,
            None,
            3,
        ),
    ];
    game.state.community_cards = vec![
        card(Rank::Two, Suit::Hearts),
        card(Rank::Three, Suit::Hearts),
        card(Rank::Nine, Suit::Spades),
        card(Rank::Jack, Suit::Hearts),
        card(Rank::Jack, Suit::Spades),
    ];
    game.state.is_finished = true;

    let resolution = game.resolve_hand().unwrap();
    let payout_total: u64 = resolution.payouts.values().sum();
    let cashback_total: u64 = resolution
        .loss_deflators
        .iter()
        .map(|result| result.cashback)
        .sum();

    assert_eq!(
        resolution.pots.iter().map(|pot| pot.amount).sum::<u64>(),
        400
    );
    // Conservação: pagamentos + rake = pote. Cashback multiway pode ser
    // menor que no modelo HU-vs-único-vencedor (equity real é multiway).
    assert_eq!(payout_total + resolution.rake, 400);
    assert!(cashback_total <= 400);
    for entry in &resolution.loss_deflators {
        assert!(entry.opponents_counted >= 1);
        assert!(entry.cashback > 0);
        assert!(entry.loser_equity >= 0.56);
    }
    // Nenhum perdedor recebe mais cashback do que o vencedor tinha no pote.
    let winner_final = resolution.payouts.get("winner").copied().unwrap_or(0);
    assert!(winner_final + cashback_total <= 400);
}

#[test]
fn odd_cashback_cent_goes_to_first_winner_left_of_button() {
    let mut game = GameLoop::new(
        TableConfig::new(10, 0, 0),
        "odd-cashback".to_string(),
        "Regression Table".to_string(),
        GameType::Cash,
    );
    game.set_dealer(0);
    game.state.players = vec![
        player(
            "loser",
            0,
            101,
            vec![card(Rank::Ace, Suit::Spades), card(Rank::Ace, Suit::Hearts)],
            true,
            Some(GamePhase::Turn),
            0,
        ),
        player(
            "winner-left",
            899,
            101,
            vec![card(Rank::Ten, Suit::Clubs), card(Rank::Nine, Suit::Clubs)],
            false,
            None,
            1,
        ),
        player(
            "winner-right",
            899,
            101,
            vec![
                card(Rank::Ten, Suit::Diamonds),
                card(Rank::Nine, Suit::Diamonds),
            ],
            false,
            None,
            2,
        ),
    ];
    game.state.community_cards = vec![
        card(Rank::Queen, Suit::Hearts),
        card(Rank::Jack, Suit::Spades),
        card(Rank::Two, Suit::Clubs),
        card(Rank::Three, Suit::Diamonds),
        card(Rank::Eight, Suit::Hearts),
    ];
    game.state.is_finished = true;

    let resolution = game.resolve_hand().unwrap();
    let payout_total: u64 = resolution.payouts.values().sum();

    assert_eq!(resolution.loss_deflators.len(), 1);
    assert_eq!(resolution.loss_deflators[0].cashback, 75);
    assert_eq!(resolution.payouts.get("loser"), Some(&75));
    assert_eq!(resolution.payouts.get("winner-left"), Some(&114));
    assert_eq!(resolution.payouts.get("winner-right"), Some(&114));
    assert_eq!(payout_total + resolution.rake, 303);
}
