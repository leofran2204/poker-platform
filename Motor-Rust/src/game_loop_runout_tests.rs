// game_loop_runout_tests.rs — run_out_stalled_hand: ninguém pode agir
// (all-in geral, rotina em torneios quando blinds superam stacks).
// Determinísticos, CI.

use crate::game_loop::GameLoop;
use crate::hand_history::GameType;
use crate::types::TableConfig;

fn micro_table() -> GameLoop {
    let mut gl = GameLoop::new(
        TableConfig::new(1000, 0, 0),
        "hand-runout-1".to_string(),
        "MTT".to_string(),
        GameType::Tournament,
    )
    .with_skip_loss_deflator(true);
    // Stacks menores que os blinds: todos all-in na largada.
    gl.add_player("alice".to_string(), 300);
    gl.add_player("bob".to_string(), 200);
    gl.set_dealer(0);
    gl
}

#[test]
fn stalled_allin_hand_runs_out_to_showdown() {
    let mut gl = micro_table();
    assert!(gl.start_hand().is_ok());
    assert!(!gl.state.is_finished);
    assert_eq!(gl.state.active_players_count(), 0);
    assert!(gl.run_out_stalled_hand());
    assert!(gl.state.is_finished);
    assert_eq!(gl.state.community_cards.len(), 5);
    let res = gl.resolve_hand().expect("showdown all-in resolve");
    assert_eq!(res.rake, 0);
    let paid: u64 = res.payouts.values().sum();
    assert_eq!(paid, 500); // conservação total
}

#[test]
fn healthy_hand_is_untouched() {
    let mut gl = GameLoop::new(
        TableConfig::new(100, 0, 0),
        "hand-runout-2".to_string(),
        "MTT".to_string(),
        GameType::Tournament,
    );
    gl.add_player("alice".to_string(), 10000);
    gl.add_player("bob".to_string(), 10000);
    gl.set_dealer(0);
    assert!(gl.start_hand().is_ok());
    assert!(!gl.run_out_stalled_hand());
    assert!(!gl.state.is_finished);
    assert!(gl.state.community_cards.is_empty());
}

#[test]
fn finished_hand_is_untouched() {
    let mut gl = micro_table();
    assert!(gl.start_hand().is_ok());
    assert!(gl.run_out_stalled_hand());
    assert!(!gl.run_out_stalled_hand());
}
