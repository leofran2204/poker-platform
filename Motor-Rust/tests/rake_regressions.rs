use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::rake::{
    calculate_rake_for_pot, calculate_rake_for_pot_with_rounding, deduct_rake,
    deduct_rake_for_hand, RakeRounding,
};
use poker_engine::types::{GamePhase, Pot, TableConfig};

fn contested_pot(amount: u64) -> Pot {
    Pot::new(amount, vec!["alice".into(), "bob".into()])
}

#[test]
fn true_percentage_rake_uses_half_to_even_and_allows_floor_policy() {
    // 5% de 10 = 0,5: o inteiro par mais próximo é 0.
    assert_eq!(calculate_rake_for_pot(10, 500, 100), 0);
    // 5% de 30 = 1,5: o inteiro par mais próximo é 2.
    assert_eq!(calculate_rake_for_pot(30, 500, 100), 2);
    assert_eq!(
        calculate_rake_for_pot_with_rounding(30, 500, 100, RakeRounding::Floor),
        1
    );
}

#[test]
fn single_cap_is_consumed_from_main_pot_then_side_pots() {
    let pots = vec![contested_pot(10_000), contested_pot(5_000)];
    let result = deduct_rake(&pots, &TableConfig::new(200, 500, 600), None);

    assert_eq!(result.total_rake, 600);
    assert_eq!(result.per_pot[0].rake, 500);
    assert_eq!(result.per_pot[1].rake, 100);
    assert_eq!(result.pots_after_rake[0].amount, 9_500);
    assert_eq!(result.pots_after_rake[1].amount, 4_900);
}

#[test]
fn uncalled_wager_is_returned_without_rake() {
    let pots = vec![
        contested_pot(20_000),
        Pot::new(10_000, vec!["alice".into()]),
    ];
    let result = deduct_rake(&pots, &TableConfig::new(200, 500, 2_000), None);

    assert_eq!(result.total_rakeable_pot, 20_000);
    assert_eq!(result.uncalled_amount, 10_000);
    assert_eq!(result.total_rake, 1_000);
    assert_eq!(result.per_pot[0].rake, 1_000);
    assert_eq!(result.per_pot[1].rake, 0);
    assert_eq!(result.pots_after_rake[1].amount, 10_000);
}

#[test]
fn no_flop_no_drop_waives_rake_before_flop() {
    let pots = vec![contested_pot(20_000)];
    let config = TableConfig::new(200, 500, 2_000);
    let result = deduct_rake_for_hand(&pots, &config, None, false, RakeRounding::HalfToEven);

    assert_eq!(result.total_rake, 0);
    assert_eq!(result.pots_after_rake[0].amount, 20_000);
}

#[test]
fn preflop_fold_has_no_rake_and_returns_uncalled_blind() {
    let mut game = GameLoop::new(
        TableConfig::new(10, 500, 500),
        "preflop-fold".into(),
        "HU".into(),
        GameType::Cash,
    );
    game.add_player("alice".into(), 1_000);
    game.add_player("bob".into(), 1_000);
    game.set_dealer(0);
    game.start_hand().unwrap();

    game.player_action("alice", PlayerMove::Fold).unwrap();
    let resolution = game.resolve_hand().unwrap();

    assert_eq!(resolution.end_phase, GamePhase::Preflop);
    assert_eq!(resolution.rake, 0);
    assert_eq!(resolution.pots.len(), 2);
    assert_eq!(resolution.pots[0].amount, 10);
    assert_eq!(resolution.pots[1].amount, 5);
    assert_eq!(resolution.payouts["bob"], 15);
}

#[test]
fn postflop_fold_rakes_only_the_called_pot() {
    let mut game = GameLoop::new(
        TableConfig::new(10, 500, 500),
        "postflop-fold".into(),
        "HU".into(),
        GameType::Cash,
    );
    game.add_player("alice".into(), 1_000);
    game.add_player("bob".into(), 1_000);
    game.set_dealer(0);
    game.start_hand().unwrap();

    game.player_action("alice", PlayerMove::Call).unwrap();
    game.player_action("bob", PlayerMove::Check).unwrap();
    assert_eq!(game.state.phase, GamePhase::Flop);

    game.player_action("bob", PlayerMove::Bet(20)).unwrap();
    game.player_action("alice", PlayerMove::Fold).unwrap();
    let resolution = game.resolve_hand().unwrap();

    // Pote chamado: 20; aposta não coberta de Bob: 20; rake: 5% de 20 = 1.
    assert_eq!(resolution.end_phase, GamePhase::Flop);
    assert_eq!(resolution.rake, 1);
    assert_eq!(resolution.pots.len(), 2);
    assert_eq!(resolution.pots[0].amount, 20);
    assert_eq!(resolution.pots[1].amount, 20);
    assert_eq!(resolution.payouts["bob"], 39);
}
