//! Exact teaching examples from the home/Academy hybrid-rule film.
use poker_engine::deck::{
    compare_hands, create_deck, evaluate_hand_brazilian_pineapple, Card, HandRank, Rank, Suit,
};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::side_pots::{calculate_side_pots, PlayerForPots};
use poker_engine::types::{GamePhase, PokerVariant, TableConfig};
use serde_json::Value;

fn card(code: &str) -> Card {
    let rank = match code.as_bytes()[0] {
        b'A' => Rank::Ace,
        b'K' => Rank::King,
        b'Q' => Rank::Queen,
        b'J' => Rank::Jack,
        b'T' => Rank::Ten,
        n => Rank::try_from(n - b'0').unwrap(),
    };
    let suit = match code.as_bytes()[1] {
        b'h' => Suit::Hearts,
        b'd' => Suit::Diamonds,
        b'c' => Suit::Clubs,
        b's' => Suit::Spades,
        _ => panic!("Unknown suit"),
    };
    Card { rank, suit }
}

#[test]
fn film_card_examples_match_the_actual_evaluator() {
    let episodes: Value =
        serde_json::from_str(include_str!("../../ZeroTiltCurso/editorial/episodes.json")).unwrap();
    let scenes = episodes["pineapple"]["scenes"].as_array().unwrap();
    for scene in scenes.iter().filter(|scene| scene["mode"] == "cards") {
        let cards = |key: &str| {
            scene[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| card(v.as_str().unwrap()))
                .collect::<Vec<_>>()
        };
        let hole = cards("hole");
        let board = cards("board");
        let all = [hole.clone(), board.clone()].concat();
        for (i, c) in all.iter().enumerate() {
            assert!(!all[i + 1..].contains(c), "Duplicate in film");
        }
        let result = evaluate_hand_brazilian_pineapple(&hole, &board);
        if scene["invalid"] == true {
            assert_ne!(result.rank, HandRank::Flush);
        } else {
            assert_eq!(result.rank, HandRank::Straight);
            let high = if scene["title"].as_str().unwrap().starts_with("Ás baixo") {
                Rank::Five
            } else {
                Rank::Ace
            };
            assert_eq!(result.cards[0].rank, high);
            if high == Rank::Five {
                let other = ["Ac", "2s", "Qd", "7d", "7h"].map(card);
                let tied = evaluate_hand_brazilian_pineapple(&other, &board);
                assert_eq!(compare_hands(&result, &tied), std::cmp::Ordering::Equal);
            }
            assert_eq!(scene["selectHole"].as_array().unwrap().len(), 2);
            assert_eq!(scene["selectBoard"].as_array().unwrap().len(), 3);
        }
    }
}

#[test]
fn postflop_film_short_allins_reopen_ana_but_not_caio() {
    let mut game = GameLoop::new(
        TableConfig::new(100, 0, 0)
            .with_small_blind(100)
            .with_poker_variant(PokerVariant::BrazilianPineapple),
        "film".into(),
        "Film examples".into(),
        GameType::Tournament,
    );
    // Dealer Dora: postflop order Ana, Beto, Caio, Dora.
    for (id, stack) in [
        ("Ana", 10000),
        ("Beto", 250),
        ("Caio", 10000),
        ("Dora", 300),
    ] {
        game.add_player(id.into(), stack);
    }
    game.set_dealer(3);
    game.start_hand_with_deck(create_deck()).unwrap();
    while game.state.phase == GamePhase::Preflop {
        let id = game.state.active_player().unwrap().id.clone();
        let action = if game.legal_actions(&id).allows("check") {
            PlayerMove::Check
        } else {
            PlayerMove::Call
        };
        game.player_action(&id, action).unwrap();
    }
    game.player_action("Ana", PlayerMove::Bet(100)).unwrap();
    game.player_action("Beto", PlayerMove::AllIn).unwrap();
    assert_eq!(game.state.min_raise, 100);
    assert_eq!(game.legal_actions("Caio").minimum_wager, 250);
    assert!(!game.can_raise("Ana"));
    game.player_action("Caio", PlayerMove::Call).unwrap();
    game.player_action("Dora", PlayerMove::AllIn).unwrap();
    assert!(game.can_raise("Ana"));
    assert_eq!(game.legal_actions("Ana").minimum_wager, 300);
    game.player_action("Ana", PlayerMove::Call).unwrap();
    assert!(!game.can_raise("Caio"));
    assert!(!game.legal_actions("Caio").allows("raise"));
}

#[test]
fn film_sidepots_separate_covered_and_unmatched_contributions() {
    for contributions in [[1000, 2000, 2000], [1000, 2000, 3000]] {
        let players: Vec<_> = contributions
            .iter()
            .enumerate()
            .map(|(i, amount)| PlayerForPots {
                id: i.to_string(),
                total_bet: *amount,
                has_folded: false,
                cards: vec![],
            })
            .collect();
        let pots = calculate_side_pots(&players);
        assert_eq!(pots[0].amount, 3000);
        assert_eq!(pots[0].eligible_players.len(), 3);
        assert_eq!(pots[1].amount, 2000);
        assert_eq!(pots[1].eligible_players, ["1", "2"]);
        if contributions[2] == 3000 {
            // A single eligible owner receives this unmatched layer back.
            assert_eq!(pots[2].amount, 1000);
            assert_eq!(pots[2].eligible_players, ["2"]);
        } else {
            assert_eq!(pots.len(), 2);
        }
    }
}
