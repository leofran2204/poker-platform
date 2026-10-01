use poker_engine::deck::{
    compare_hands, create_short_deck, evaluate_hand, evaluate_hand_brazilian_pineapple,
    evaluate_hand_omaha, evaluate_hand_short_deck, Card, HandRank, Rank, Suit,
};
use poker_engine::loss_deflator::get_multiway_win_probability_for_variant as equity;
use poker_engine::types::PokerVariant;
use std::cmp::Ordering;

fn c(s: &str) -> Card {
    let rank = match s.as_bytes()[0] {
        b'A' => Rank::Ace,
        b'K' => Rank::King,
        b'Q' => Rank::Queen,
        b'J' => Rank::Jack,
        b'T' => Rank::Ten,
        n => Rank::try_from(n - b'0').unwrap(),
    };
    let suit = match s.as_bytes()[1] {
        b'h' => Suit::Hearts,
        b'd' => Suit::Diamonds,
        b'c' => Suit::Clubs,
        _ => Suit::Spades,
    };
    Card { rank, suit }
}

#[test]
fn low_straight_cannot_take_a_pot_from_a_higher_straight() {
    use poker_engine::side_pots::{distribute_pots_with_seat_order_for_variant, PlayerForPots};
    use poker_engine::types::Pot;
    for (variant, board, low, high) in [
        (
            PokerVariant::Holdem,
            cards(&["2c", "3d", "4s", "Kh", "Qh"]),
            cards(&["Ah", "5d"]),
            cards(&["6h", "5c"]),
        ),
        (
            PokerVariant::ShortDeck,
            cards(&["7c", "8d", "9s", "Qh", "Kh"]),
            cards(&["Ah", "6h"]),
            cards(&["Th", "6d"]),
        ),
    ] {
        let players = vec![
            PlayerForPots {
                id: "low".into(),
                total_bet: 100,
                has_folded: false,
                cards: low,
            },
            PlayerForPots {
                id: "high".into(),
                total_bet: 100,
                has_folded: false,
                cards: high,
            },
        ];
        let order = vec!["low".into(), "high".into()];
        let pot = Pot::new(200, order.clone());
        let payouts =
            distribute_pots_with_seat_order_for_variant(&[pot], &players, &board, &order, variant);
        assert_eq!(payouts.get("high"), Some(&200));
        assert_eq!(payouts.get("low").copied().unwrap_or(0), 0);
        assert_eq!(payouts.values().sum::<u64>(), 200);
    }
}

#[test]
fn academy_examples_use_the_actual_omaha_and_pineapple_evaluators() {
    let result =
        evaluate_hand_brazilian_pineapple(&cards(&["As", "Ks", "Ah"]), &cards(&["Ad", "Qc", "2s"]));
    assert_eq!(result.rank, HandRank::ThreeOfAKind);
    assert_eq!(
        result.kickers.iter().map(|c| c.rank).collect::<Vec<_>>(),
        vec![Rank::Queen, Rank::Two]
    );
    let draw = evaluate_hand_omaha(
        &cards(&["Ah", "Kh", "Qc", "Jc"]),
        &cards(&["Th", "9h", "8c"]),
    );
    assert_eq!(draw.rank, HandRank::Straight);
    assert_eq!(draw.cards[0].rank, Rank::Queen);
    let royal_board = evaluate_hand_brazilian_pineapple(
        &cards(&["9c", "9d", "8c", "7d", "6s"]),
        &cards(&["Ah", "Kh", "Qh", "Jh", "Th"]),
    );
    assert_eq!(royal_board.rank, HandRank::Straight);
    assert_eq!(royal_board.cards[0].rank, Rank::Queen);
}

#[test]
fn documented_turn_equity_is_exact_and_selects_twenty_five_percent() {
    use poker_engine::loss_deflator::{
        calculate_progressive_loss_deflator, LossDeflatorTier, ProgressiveLossDeflatorParams,
    };
    use poker_engine::types::{GamePhase, Pot};
    let value = equity(
        &cards(&["Ad", "Ac"]),
        &[&cards(&["Jh", "Th"])],
        &cards(&["Qs", "9c", "4d", "2s"]),
        PokerVariant::Holdem,
    )
    .unwrap();
    assert!((value - 36.0 / 44.0).abs() < 1e-12);
    let deflator = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
        pots: vec![Pot::new(20000, vec!["hero".into(), "villain".into()])],
        loser_id: "hero".into(),
        winner_id: "villain".into(),
        phase: GamePhase::Turn,
        loser_equity: value,
    })
    .unwrap();
    assert_eq!(deflator.tier, LossDeflatorTier::TwentyFivePercent);
    assert_eq!(deflator.cashback, 5000);
}
fn cards(ss: &[&str]) -> Vec<Card> {
    ss.iter().map(|s| c(s)).collect()
}

#[test]
fn classic_wheel_has_five_cards_and_loses_to_six_high() {
    let board = cards(&["2c", "3d", "4s", "Kh", "Qh"]);
    let wheel = evaluate_hand(&cards(&["Ah", "5d"]), &board);
    let higher = evaluate_hand(&cards(&["6h", "5c"]), &board);
    assert_eq!(wheel.rank, HandRank::Straight);
    assert_eq!(
        wheel.cards.iter().map(|c| c.rank as u8).collect::<Vec<_>>(),
        vec![5, 4, 3, 2, 14]
    );
    assert_eq!(compare_hands(&wheel, &higher), Ordering::Less);
    let tied = evaluate_hand(&cards(&["As", "5s"]), &board);
    assert_eq!(compare_hands(&wheel, &tied), Ordering::Equal);
}

#[test]
fn wheel_order_applies_to_flush_omaha_and_pineapple() {
    let low = evaluate_hand(&cards(&["Ah", "2h"]), &cards(&["3h", "4h", "5h"]));
    let high = evaluate_hand(&cards(&["6h", "2h"]), &cards(&["3h", "4h", "5h"]));
    assert_eq!(low.rank, HandRank::StraightFlush);
    assert_eq!(compare_hands(&low, &high), Ordering::Less);
    let board = cards(&["2c", "3d", "4s", "Kh", "Qh"]);
    for evaluate in [evaluate_hand_omaha, evaluate_hand_brazilian_pineapple] {
        let low = evaluate(&cards(&["Ah", "5d", "8c", "9s"]), &board);
        let high = evaluate(&cards(&["6h", "5c", "8d", "9h"]), &board);
        assert_eq!(low.cards.len(), 5);
        assert_eq!(compare_hands(&low, &high), Ordering::Less);
    }
}

#[test]
fn short_deck_wheel_loses_to_ten_high_including_straight_flush() {
    for board in [
        cards(&["7c", "8d", "9s", "Qh", "Kh"]),
        cards(&["7h", "8h", "9h"]),
    ] {
        let low = evaluate_hand_short_deck(&cards(&["Ah", "6h"]), &board);
        let high = evaluate_hand_short_deck(&cards(&["Th", "6d"]), &board);
        // The suited case compares to a suited ten-high separately.
        if board.len() == 5 {
            assert_eq!(compare_hands(&low, &high), Ordering::Less);
        } else {
            let high = evaluate_hand_short_deck(&cards(&["Th", "6h"]), &board);
            assert_eq!(low.rank, HandRank::StraightFlush);
            assert_eq!(compare_hands(&low, &high), Ordering::Less);
        }
        assert_eq!(low.cards[0].rank, Rank::Nine);
        assert_eq!(low.cards.len(), 5);
    }
}

#[test]
fn settlement_equity_uses_variant_flush_and_full_house_order() {
    let hero = cards(&["Jh", "9h"]);
    let villain = cards(&["Ac", "Kc"]);
    let board = cards(&["Ah", "Kh", "Qh", "As", "Kd"]);
    assert_eq!(
        equity(&hero, &[&villain], &board, PokerVariant::ShortDeck),
        Some(1.0)
    );
    assert_eq!(
        equity(&hero, &[&villain], &board, PokerVariant::Holdem),
        Some(0.0)
    );
}

#[test]
fn short_deck_turn_equity_matches_exhaustive_legal_rivers() {
    let hero = cards(&["9c", "9d"]);
    let villain = cards(&["Th", "6c"]);
    let board = cards(&["9s", "8h", "7d", "Jc"]);
    let known: Vec<Card> = hero.iter().chain(&villain).chain(&board).copied().collect();
    let mut total = 0;
    let mut wins = 0.0;
    for river in create_short_deck()
        .into_iter()
        .filter(|c| !known.contains(c))
    {
        assert!(river.rank as u8 >= 6);
        let mut final_board = board.clone();
        final_board.push(river);
        wins += match compare_hands(
            &evaluate_hand_short_deck(&hero, &final_board),
            &evaluate_hand_short_deck(&villain, &final_board),
        ) {
            Ordering::Greater => 1.0,
            Ordering::Equal => 0.5,
            Ordering::Less => 0.0,
        };
        total += 1;
    }
    assert_eq!(total, 28);
    assert_eq!(
        equity(&hero, &[&villain], &board, PokerVariant::ShortDeck),
        Some(wins / 28.0)
    );
}

#[test]
fn variant_equity_splits_three_way_ties_and_rejects_invalid_input() {
    let hero = cards(&["9c", "9d"]);
    let a = cards(&["8c", "8d"]);
    let b = cards(&["7c", "7d"]);
    let board = cards(&["Ah", "Kh", "Qh", "Jh", "Th"]);
    assert_eq!(
        equity(&hero, &[&a, &b], &board, PokerVariant::ShortDeck),
        Some(1.0 / 3.0)
    );
    assert_eq!(
        equity(&hero, &[&hero], &board, PokerVariant::ShortDeck),
        None
    );
    assert_eq!(
        equity(
            &cards(&["2c", "3c"]),
            &[&a],
            &board,
            PokerVariant::ShortDeck
        ),
        None
    );
    assert_eq!(
        equity(&hero, &[&a], &cards(&["Ah", "Kh"]), PokerVariant::ShortDeck),
        None
    );
    assert_eq!(
        equity(&hero, &[&a], &board, PokerVariant::BrazilianPineapple),
        None
    );
    assert_eq!(equity(&hero, &[&a], &board, PokerVariant::Omaha), None);
}
