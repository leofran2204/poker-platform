//! Brazilian Pineapple: exemplos de aposta ensinados na Academy (aumento limitado ao
//! pote antes do call e ante de 1 BB pago pelo big blind), conferidos contra o
//! motor com a rodada de troca no caminho.
use poker_engine::deck::create_deck;
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::side_pots::{calculate_side_pots, PlayerForPots};
use poker_engine::types::{GamePhase, PokerVariant, TableConfig};

/// As situações abaixo tratam só das apostas: cada rodada de troca é concluída
/// com todos mantendo as cartas.
fn keep_all(game: &mut GameLoop) {
    while (game.state.drawing || game.state.discarding) && !game.state.is_finished {
        let id = game.state.active_player().unwrap().id.clone();
        let mv = if game.state.discarding {
            PlayerMove::Discard(None)
        } else {
            PlayerMove::Keep
        };
        game.player_action(&id, mv).unwrap();
    }
}

fn act(game: &mut GameLoop, id: &str, action: PlayerMove) {
    game.player_action(id, action).unwrap();
    keep_all(game);
}

#[test]
fn postflop_short_allins_reopen_ana_but_not_caio() {
    let mut game = GameLoop::new(
        TableConfig::new(100, 0, 0)
            .with_small_blind(100)
            .with_poker_variant(PokerVariant::PineappleDraw),
        "examples".into(),
        "Examples".into(),
        GameType::Tournament,
    );
    // Dealer Dora: ordem pós-flop Ana, Beto, Caio, Dora.
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
        act(&mut game, &id, action);
    }
    act(&mut game, "Ana", PlayerMove::Bet(100));
    act(&mut game, "Beto", PlayerMove::AllIn);
    assert_eq!(game.state.min_raise, 100);
    assert_eq!(game.legal_actions("Caio").minimum_wager, 250);
    assert!(!game.can_raise("Ana"));
    act(&mut game, "Caio", PlayerMove::Call);
    act(&mut game, "Dora", PlayerMove::AllIn);
    assert!(game.can_raise("Ana"));
    assert_eq!(game.legal_actions("Ana").minimum_wager, 300);
    act(&mut game, "Ana", PlayerMove::Call);
    assert!(!game.can_raise("Caio"));
    assert!(!game.legal_actions("Caio").allows("raise"));
}

#[test]
fn sidepots_separate_covered_and_unmatched_contributions() {
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
            // A camada sem cobertura volta ao único dono.
            assert_eq!(pots[2].amount, 1000);
            assert_eq!(pots[2].eligible_players, ["2"]);
        } else {
            assert_eq!(pots.len(), 2);
        }
    }
}

fn draw_game(game_type: GameType, stacks: &[u64], bb: u64) -> GameLoop {
    let mut game = GameLoop::new(
        TableConfig::new(bb, 0, 0)
            .with_small_blind(bb / 2)
            .with_poker_variant(PokerVariant::PineappleDraw),
        "draw-examples".into(),
        "Examples".into(),
        game_type,
    )
    .with_ante(999)
    .with_skip_loss_deflator(true); // o ator não duplica nem altera o ante da modalidade
    for (i, stack) in stacks.iter().enumerate() {
        game.add_player(format!("p{i}"), *stack);
    }
    game.start_hand_with_deck(create_deck()).unwrap();
    game
}

#[test]
fn approved_preflop_examples_cash_and_tournament() {
    // Blinds R$ 0,50/R$ 1, ante R$ 1: pote inicial R$ 2,50; todos acompanhando
    // formam R$ 4; abertura máxima para R$ 3,50; dois calls formam R$ 11,50.
    for kind in [GameType::Cash, GameType::Tournament] {
        let mut calls = draw_game(kind, &[10000; 3], 100);
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
        assert_eq!(calls.state.phase, GamePhase::Flop);
        assert!(!calls.state.drawing, "o flop abre direto nas apostas");

        let mut raised = draw_game(kind, &[10000; 3], 100);
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
fn postflop_pot_before_call_example_after_the_draw_round() {
    // Pote de R$ 10 no flop; aposta de R$ 5; aumento máximo para R$ 20;
    // três participantes com R$ 20 na rodada formam R$ 70.
    let mut game = GameLoop::new(
        TableConfig::new(100, 0, 0)
            .with_small_blind(50)
            .with_poker_variant(PokerVariant::PineappleDraw),
        "postflop".into(),
        "Examples".into(),
        GameType::Tournament,
    );
    for i in 0..3 {
        game.add_player(format!("p{i}"), 10000);
    }
    game.set_dealer(2);
    game.start_hand_with_deck(create_deck()).unwrap();
    act(&mut game, "p2", PlayerMove::Raise(300));
    act(&mut game, "p0", PlayerMove::Call);
    act(&mut game, "p1", PlayerMove::Call);
    assert_eq!(game.state.phase, GamePhase::Flop);
    assert!(!game.state.drawing);
    assert_eq!(game.state.total_pot(), 1000);
    assert_eq!(game.state.active_player().unwrap().id, "p0");
    act(&mut game, "p0", PlayerMove::Bet(500));
    let legal = game.legal_actions("p1");
    assert_eq!(
        (legal.call_amount, legal.minimum_wager, legal.maximum_wager),
        (500, 1000, 2000)
    );
    act(&mut game, "p1", PlayerMove::Raise(2000));
    act(&mut game, "p2", PlayerMove::Call);
    act(&mut game, "p0", PlayerMove::Call);
    assert_eq!(game.state.total_pot(), 7000);
    assert_eq!(game.state.phase, GamePhase::Turn);
}

#[test]
fn partial_ante_heads_up_levels_sidepots_and_refund_conserve_chips() {
    for kind in [GameType::Cash, GameType::Tournament] {
        for (stack, paid) in [(70, 0), (100, 0), (150, 50), (200, 100)] {
            let mut game = draw_game(kind, &[10000, 10000, stack], 100);
            assert_eq!(game.ante_paid(), paid);
            assert_eq!(game.ante_player_id(), Some("p2"));
            assert_eq!(game.state.total_pot(), 50 + stack);
            let call = game.legal_actions("p0").call_amount;
            assert_eq!(game.eligible_pot_after_call("p0", call), 50 + stack + call);
            act(&mut game, "p0", PlayerMove::Raise(200));
            act(&mut game, "p1", PlayerMove::Call);
            // Só p0/p1 apostam no flop. Uma aposta sem cobertura volta ao dono.
            act(&mut game, "p1", PlayerMove::Bet(100));
            act(&mut game, "p0", PlayerMove::Fold);
            game.run_out_stalled_hand();
            let result = game.resolve_hand().unwrap();
            assert_eq!(result.pots[0].amount, 3 * stack.min(100) + paid);
            assert_eq!(result.pots[0].eligible_players.len(), 3);
            assert_eq!(result.payouts.get("p0").copied().unwrap_or(0), 0);
            assert!(result.pots.iter().skip(1).all(|p| !p.is_eligible("p2")));
            assert_eq!(
                result.payouts.values().sum::<u64>()
                    + game.state.players.iter().map(|p| p.stack).sum::<u64>(),
                20000 + stack
            );
        }
        for bb in [100, 200, 1000] {
            let game = draw_game(kind, &[10000; 2], bb);
            assert_eq!(game.nominal_ante(), bb);
            assert_eq!(game.ante_paid(), bb);
            assert_eq!(game.ante_player_id(), Some("p1"));
            assert_eq!(game.state.total_pot(), bb * 5 / 2);
            assert_eq!(game.legal_actions("p0").call_amount, bb / 2);
        }
    }
}
