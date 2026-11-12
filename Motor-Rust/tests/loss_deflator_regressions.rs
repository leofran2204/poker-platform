use poker_engine::deck::{Card, Rank, Suit};
use poker_engine::game_loop::{GameLoop, PlayerMove, PlayerState};
use poker_engine::hand_history::GameType;
use poker_engine::loss_deflator::{get_multiway_win_probability_for_variant, LossDeflatorTier};
use poker_engine::types::{GamePhase, PokerVariant, TableConfig};

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
        draw_count: None,
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

/// Baralho de 52 cartas com o prefixo dado e o restante na ordem padrão.
fn deck_with_prefix(text: &str) -> Vec<Card> {
    let mut prefix: Vec<Card> = text.split_whitespace().map(parse_card).collect();
    let rest: Vec<Card> = poker_engine::deck::create_full_deck()
        .into_iter()
        .filter(|card| !prefix.contains(card))
        .collect();
    prefix.extend(rest);
    assert_eq!(prefix.len(), 52);
    prefix
}

fn parse_card(text: &str) -> Card {
    let mut chars = text.chars();
    let rank = match chars.next().unwrap() {
        'A' => Rank::Ace,
        'K' => Rank::King,
        'Q' => Rank::Queen,
        'J' => Rank::Jack,
        'T' => Rank::Ten,
        digit => Rank::try_from(digit.to_digit(10).unwrap() as u8).unwrap(),
    };
    let suit = match chars.next().unwrap() {
        'h' => Suit::Hearts,
        'd' => Suit::Diamonds,
        'c' => Suit::Clubs,
        _ => Suit::Spades,
    };
    card(rank, suit)
}

fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace().map(parse_card).collect()
}

fn variant_game(variant: PokerVariant, stacks: &[u64], deck: &str) -> GameLoop {
    let mut game = GameLoop::new(
        TableConfig::new(10, 0, 0).with_poker_variant(variant),
        "deflator-variant".to_string(),
        "Regression Table".to_string(),
        GameType::Cash,
    );
    for (index, stack) in stacks.iter().enumerate() {
        game.add_player(format!("p{index}"), *stack);
    }
    game.set_dealer(0);
    game.start_hand_with_deck(deck_with_prefix(deck)).unwrap();
    game
}

/// Omaha 4: favorito no flop perde para quadra no river e recebe o deflator,
/// com equity exata (C(41, 2) turn/river) calculada no pagamento do all-in.
#[test]
fn omaha_flop_all_in_bad_beat_pays_deflator() {
    // Privadas em rodízio p0/p1 (4 cada), queima, flop, queima, turn, queima, river.
    let mut game = variant_game(
        PokerVariant::Omaha,
        &[100, 200],
        "Ah 7s Ad 6d Kc 3c Qc 2h Td Ac 7h 6c Th 6h Ts 6s",
    );
    game.player_action("p0", PlayerMove::Call).unwrap();
    game.player_action("p1", PlayerMove::Check).unwrap();
    game.player_action("p1", PlayerMove::Check).unwrap();
    game.player_action("p0", PlayerMove::AllIn).unwrap();
    game.player_action("p1", PlayerMove::Call).unwrap();

    let expected = get_multiway_win_probability_for_variant(
        &cards("Ah Ad Kc Qc"),
        &[&cards("7s 6d 3c 2h")],
        &cards("Ac 7h 6c"),
        PokerVariant::Omaha,
    )
    .unwrap();
    assert!(expected >= 0.86, "favorito claro no flop: {expected}");

    let resolution = game.resolve_hand().unwrap();
    let audit = resolution
        .loss_deflators
        .iter()
        .find(|entry| entry.loser_id == "p0")
        .expect("deflator do perdedor all-in");
    assert_eq!(audit.phase, GamePhase::Flop);
    assert_eq!(audit.loser_equity, expected);
    assert_eq!(audit.tier, LossDeflatorTier::ThirtyFivePercent);
    // Pote líquido de 200 (sem rake): 35% = 70 centavos de volta ao perdedor.
    assert_eq!(audit.cashback, 70);
    assert_eq!(resolution.payouts.get("p0"), Some(&70));
    assert_eq!(
        resolution.payouts.values().sum::<u64>() + resolution.rake,
        200
    );
}

/// Brazilian Pineapple: o perdedor troca uma carta depois das apostas do flop
/// e vai all-in no turn. O snapshot usa exatamente as 3 privadas que ele tinha
/// no pagamento; depois do all-in ele não troca mais, e a equity exata só
/// completa o river. No showdown ele fica com as duas melhores.
#[test]
fn pineapple_draw_turn_all_in_uses_cards_held_at_payment() {
    // Privadas em rodízio p0/p1 (3 cada); queima + flop; carta recebida na
    // troca de p0; queima + turn; queima + river.
    let mut game = variant_game(
        PokerVariant::PineappleDraw,
        &[40, 100],
        "As 6c Ad 5s 7d 9c Tc Ah 7c 2d Kd Td 3h Th 4h",
    );
    game.player_action("p0", PlayerMove::Call).unwrap();
    game.player_action("p1", PlayerMove::Check).unwrap();
    // Flop: apostas (p1, p0) e depois a troca; p0 troca o 7♦ e recebe o K♦.
    game.player_action("p1", PlayerMove::Check).unwrap();
    game.player_action("p0", PlayerMove::Check).unwrap();
    game.player_action("p1", PlayerMove::Keep).unwrap();
    game.player_action("p0", PlayerMove::Swap(2)).unwrap();
    assert_eq!(game.state.players[0].hole_cards, cards("As Ad Kd"));
    // Turn: p0 vai all-in com as três cartas que tem agora.
    assert_eq!(game.state.phase, GamePhase::Turn);
    game.player_action("p1", PlayerMove::Check).unwrap();
    game.player_action("p0", PlayerMove::AllIn).unwrap();
    game.player_action("p1", PlayerMove::Call).unwrap();
    assert!(game.state.is_finished);

    let expected = get_multiway_win_probability_for_variant(
        &cards("As Ad Kd"),
        &[&cards("6c 5s 9c")],
        &cards("Ah 7c 2d 3h"),
        PokerVariant::PineappleDraw,
    )
    .unwrap();
    assert!(
        expected >= 0.85,
        "trinca de ases favorita no turn: {expected}"
    );

    let resolution = game.resolve_hand().unwrap();
    // Sem rodada de descarte (all-in): fica com o par de ases, que faz a trinca.
    assert_eq!(game.state.players[0].hole_cards, cards("As Ad"));
    assert_eq!(game.state.community_cards, cards("Ah 7c 2d 3h 4h"));
    let audit = resolution
        .loss_deflators
        .iter()
        .find(|entry| entry.loser_id == "p0")
        .expect("deflator do perdedor all-in");
    assert_eq!(audit.phase, GamePhase::Turn);
    assert_eq!(audit.loser_equity, expected);
    assert!(audit.cashback > 0);
    assert_eq!(resolution.payouts.get("p0"), Some(&audit.cashback));
    let total_bet: u64 = game.state.players.iter().map(|p| p.total_bet).sum();
    assert_eq!(
        resolution.payouts.values().sum::<u64>() + resolution.rake,
        total_bet
    );
}
