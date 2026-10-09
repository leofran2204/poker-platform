//! Exact teaching examples from the home/Academy Pineapple film.
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
        ("Beto", 350),
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

fn v2_game(game_type: GameType, stacks: &[u64], bb: u64) -> GameLoop {
    let mut game = GameLoop::new(
        TableConfig::new(bb, 0, 0)
            .with_small_blind(bb / 2)
            .with_poker_variant(PokerVariant::BrazilianPineapple),
        "v2-film".into(),
        "Film".into(),
        game_type,
    )
    .with_ante(999)
    .with_skip_loss_deflator(true); // actor cannot duplicate/override the variant ante
    for (i, stack) in stacks.iter().enumerate() {
        game.add_player(format!("p{i}"), *stack);
    }
    game.start_hand_with_deck(create_deck()).unwrap();
    game
}

#[test]
fn approved_preflop_examples_cash_and_tournament() {
    for kind in [GameType::Cash, GameType::Tournament] {
        let mut calls = v2_game(kind, &[10000; 3], 100);
        assert_eq!(calls.nominal_ante(), 100);
        assert_eq!(calls.ante_paid(), 100);
        assert_eq!(calls.ante_player_id(), Some("p2"));
        assert_eq!(calls.state.total_pot(), 250);
        assert_eq!(calls.legal_actions("p0").call_amount, 100);
        assert_eq!(calls.legal_actions("p0").maximum_wager, 350);
        assert_eq!(calls.state.players[2].current_bet, 100);
        calls.player_action("p0", PlayerMove::Call).unwrap();
        calls.player_action("p1", PlayerMove::Call).unwrap();
        calls.player_action("p2", PlayerMove::Check).unwrap();
        assert_eq!(calls.state.total_pot(), 400);

        let mut raised = v2_game(kind, &[10000; 3], 100);
        assert!(raised.player_action("p0", PlayerMove::Raise(351)).is_err());
        raised.player_action("p0", PlayerMove::Raise(350)).unwrap();
        assert_eq!(raised.state.min_raise, 250);
        assert_eq!(raised.legal_actions("p1").minimum_wager, 600);
        assert_eq!(raised.legal_actions("p1").maximum_wager, 950);
        raised.player_action("p1", PlayerMove::Call).unwrap();
        raised.player_action("p2", PlayerMove::Call).unwrap();
        assert_eq!(raised.state.total_pot(), 1150);
        assert_eq!(
            raised.history.as_ref().unwrap().table_config.ante,
            Some(100)
        );
    }
}

#[test]
fn published_preflop_animation_payments_match_engine_actions() {
    let episodes: Value =
        serde_json::from_str(include_str!("../../ZeroTiltCurso/editorial/episodes.json")).unwrap();
    for title in [
        "Se todos apenas acompanharem",
        "Abertura máxima para R$ 3,50",
    ] {
        let scene = episodes["pineapple"]["scenes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["title"] == title)
            .unwrap();
        let mut game = v2_game(GameType::Cash, &[10000; 3], 100);
        let mut pot = scene["table"]["pot"].as_u64().unwrap();
        let mut bets: Vec<u64> = scene["table"]["players"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["bet"].as_u64().unwrap())
            .collect();
        let mut stacks: Vec<u64> = scene["table"]["players"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["stack"].as_u64().unwrap())
            .collect();
        for action in scene["actions"].as_array().unwrap() {
            let i = action["player"].as_u64().unwrap() as usize;
            let total = action["to"].as_u64().unwrap();
            let payment = total - bets[i];
            assert_eq!(game.state.active_player().unwrap().id, format!("p{i}"));
            let movement = if total > game.state.current_bet_to_match {
                PlayerMove::Raise(total)
            } else if payment == 0 {
                PlayerMove::Check
            } else {
                PlayerMove::Call
            };
            game.player_action(&format!("p{i}"), movement).unwrap();
            bets[i] = total;
            stacks[i] -= payment;
            pot += payment;
            assert_eq!(game.state.total_pot(), pot, "{title}");
            assert_eq!(
                game.state
                    .players
                    .iter()
                    .map(|p| p.stack)
                    .collect::<Vec<_>>(),
                stacks,
                "{title}"
            );
        }
        assert_eq!(game.state.phase, GamePhase::Flop);
    }
}

#[test]
fn published_postflop_animation_matches_button_turns_and_payments() {
    let episodes: Value =
        serde_json::from_str(include_str!("../../ZeroTiltCurso/editorial/episodes.json")).unwrap();
    let scenes: Vec<_> = episodes["pineapple"]["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| {
            s["title"] == "R$ 10 no pote. Abertura de R$ 5."
                || s["title"] == "Beto aumenta para R$ 20"
        })
        .collect();
    let mut game = GameLoop::new(
        TableConfig::new(100, 0, 0)
            .with_small_blind(50)
            .with_poker_variant(PokerVariant::BrazilianPineapple),
        "film-postflop".into(),
        "Film".into(),
        GameType::Tournament,
    );
    for i in 0..3 {
        game.add_player(format!("p{i}"), 10000);
    }
    game.set_dealer(2);
    game.start_hand_with_deck(create_deck()).unwrap();
    game.player_action("p2", PlayerMove::Raise(300)).unwrap();
    game.player_action("p0", PlayerMove::Call).unwrap();
    game.player_action("p1", PlayerMove::Call).unwrap();
    assert_eq!(game.state.phase, GamePhase::Flop);
    for scene in scenes {
        assert_eq!(scene["dealer"], game.state.dealer_index);
        let mut pot = scene["table"]["pot"].as_u64().unwrap();
        assert_eq!(game.state.total_pot(), pot);
        let mut bets: Vec<_> = scene["table"]["players"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["bet"].as_u64().unwrap())
            .collect();
        let mut stacks: Vec<_> = scene["table"]["players"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["stack"].as_u64().unwrap())
            .collect();
        assert_eq!(
            game.state
                .players
                .iter()
                .map(|p| p.stack)
                .collect::<Vec<_>>(),
            stacks
        );
        for action in scene["actions"].as_array().unwrap() {
            let i = action["player"].as_u64().unwrap() as usize;
            let total = action["to"].as_u64().unwrap();
            let payment = total - bets[i];
            assert_eq!(game.state.active_player().unwrap().id, format!("p{i}"));
            let movement = if game.state.current_bet_to_match == 0 {
                PlayerMove::Bet(total)
            } else if total > game.state.current_bet_to_match {
                PlayerMove::Raise(total)
            } else {
                PlayerMove::Call
            };
            game.player_action(&format!("p{i}"), movement).unwrap();
            bets[i] = total;
            stacks[i] -= payment;
            pot += payment;
            assert_eq!(game.state.total_pot(), pot);
            assert_eq!(
                game.state
                    .players
                    .iter()
                    .map(|p| p.stack)
                    .collect::<Vec<_>>(),
                stacks
            );
        }
    }
    assert_eq!(game.state.total_pot(), 7000);
    assert_eq!(game.state.phase, GamePhase::Turn);
}

#[test]
fn partial_ante_heads_up_levels_sidepots_and_refund_conserve_chips() {
    for kind in [GameType::Cash, GameType::Tournament] {
        for (stack, paid) in [(70, 0), (100, 0), (150, 50), (200, 100)] {
            let mut game = v2_game(kind, &[10000, 10000, stack], 100);
            assert_eq!(game.ante_paid(), paid);
            assert_eq!(game.ante_player_id(), Some("p2"));
            assert_eq!(game.state.total_pot(), 50 + stack);
            let call = game.legal_actions("p0").call_amount;
            assert_eq!(game.eligible_pot_after_call("p0", call), 50 + stack + call);
            game.player_action("p0", PlayerMove::Raise(200)).unwrap();
            game.player_action("p1", PlayerMove::Call).unwrap();
            // Only p0/p1 can bet on the flop. An unmatched bet must be returned.
            game.player_action("p1", PlayerMove::Bet(100)).unwrap();
            game.player_action("p0", PlayerMove::Fold).unwrap();
            game.run_out_stalled_hand();
            let result = game.resolve_hand().unwrap();
            assert_eq!(result.pots[0].amount, 3 * stack.min(100) + paid);
            assert_eq!(result.pots[0].eligible_players.len(), 3); // contribution layers retain folded money
            assert_eq!(result.payouts.get("p0").copied().unwrap_or(0), 0);
            assert!(result.pots.iter().skip(1).all(|p| !p.is_eligible("p2")));
            assert_eq!(
                result.payouts.values().sum::<u64>()
                    + game.state.players.iter().map(|p| p.stack).sum::<u64>(),
                20000 + stack
            );
        }
        for bb in [100, 200, 1000] {
            let game = v2_game(kind, &[10000; 2], bb);
            assert_eq!(game.nominal_ante(), bb);
            assert_eq!(game.ante_paid(), bb);
            assert_eq!(game.ante_player_id(), Some("p1"));
            assert_eq!(game.state.total_pot(), bb * 5 / 2);
            assert_eq!(game.legal_actions("p0").call_amount, bb / 2);
        }
    }
}
