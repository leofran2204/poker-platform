// tournament_multitable_tests.rs — fee 15%, 3 mesas, balanceamento, FT.
// Testes determinísticos (rodam na CI); sem carga probabilística.

use crate::tournament_engine::{
    assign_initial_tables, entry_fee_cents, rebalance_move, register_player, should_consolidate,
    start_tournament, tournament_capacity, BlindLevel, TournamentConfig, TournamentSpeed,
    TournamentStatus, TOURNAMENT_FEE_BASIS_POINTS, TOURNAMENT_TABLE_COUNT,
};

fn config_10_plus() -> TournamentConfig {
    TournamentConfig {
        name: "MTT 10+fee".to_string(),
        game_type: "Holdem".to_string(),
        buy_in: 1000,
        starting_stack: 10_000,
        max_players: 27,
        speed: TournamentSpeed::Normal,
        blind_levels: vec![BlindLevel {
            level: 1,
            small_blind: 25,
            big_blind: 50,
            ante: 0,
            duration_minutes: 10,
        }],
        prize_pool_pct: 1.0,
        prize_distribution: vec![0.50, 0.30, 0.20],
        late_registration: false,
        late_registration_max_level: 0,
        allow_rebuy: false,
        allow_addon: false,
        rebuy_max_level: 0,
        guaranteed_prize: 0,
        is_freeroll: false,
        rebuy_cost: 0,
        rebuy_chips: 0,
        rebuy_max_count: 0,
        rebuy_stack_threshold: 0,
    }
}

// ─── Fee 15% por cima ───

#[test]
fn fee_const_is_1500_basis_points() {
    assert_eq!(TOURNAMENT_FEE_BASIS_POINTS, 1500);
}

#[test]
fn fee_math_per_buyin() {
    assert_eq!(entry_fee_cents(1000), 150); // R$ 10 -> R$ 1,50
    assert_eq!(entry_fee_cents(2500), 375);
    assert_eq!(entry_fee_cents(0), 0); // freeroll: zero
    assert_eq!(entry_fee_cents(1), 0); // truncamento p/ baixo
}

#[test]
fn register_tracks_fees_without_touching_prize() {
    let mut state = crate::tournament_engine::create_tournament(config_10_plus());
    register_player(&mut state, "p1", "P1").unwrap();
    register_player(&mut state, "p2", "P2").unwrap();
    assert_eq!(state.total_buyins, 2000);
    assert_eq!(state.total_fees, 300); // 2 x 150
    assert_eq!(state.prize_pool, 2000); // prize intacto: fee por cima
}

// ─── 3 mesas ───

#[test]
fn table_count_is_three() {
    assert_eq!(TOURNAMENT_TABLE_COUNT, 3);
}

#[test]
fn capacity_is_three_times_table_max() {
    assert_eq!(tournament_capacity(9), 27); // Texas
    assert_eq!(tournament_capacity(8), 24); // SD Texas
    assert_eq!(tournament_capacity(5), 15); // Omaha
    assert_eq!(tournament_capacity(6), 18); // Pineapple
}

#[test]
fn initial_seating_is_balanced_round_robin() {
    let seats = assign_initial_tables(20, 9);
    assert_eq!(seats.len(), 20);
    let mut per_table = [0u32; 3];
    for (i, table, seat) in &seats {
        assert_eq!(*seat, per_table[*table as usize]);
        per_table[*table as usize] += 1;
        assert!(*i < 20);
    }
    assert_eq!(per_table, [7, 7, 6]);
}

#[test]
fn initial_seating_single_player() {
    assert_eq!(assign_initial_tables(1, 9), vec![(0, 0, 0)]);
}

#[test]
fn initial_seating_empty() {
    assert!(assign_initial_tables(0, 9).is_empty());
}

// ─── Balanceamento ───

#[test]
fn balanced_tables_need_no_move() {
    assert_eq!(rebalance_move(&[7, 7, 6]), None);
    assert_eq!(rebalance_move(&[5, 5, 5]), None);
    assert_eq!(rebalance_move(&[0, 0, 0]), None);
}

#[test]
fn skewed_tables_move_from_fullest_to_emptiest() {
    assert_eq!(rebalance_move(&[9, 5, 5]), Some((0, 1)));
    assert_eq!(rebalance_move(&[4, 4, 9]), Some((2, 0)));
}

#[test]
fn wrong_table_count_is_rejected() {
    assert_eq!(rebalance_move(&[9, 5]), None);
    assert_eq!(rebalance_move(&[]), None);
}

// ─── Consolidação / FT ───

#[test]
fn consolidate_only_when_fitting_final_table() {
    assert!(!should_consolidate(9, 8));
    assert!(should_consolidate(8, 8));
    assert!(should_consolidate(5, 9));
    assert!(!should_consolidate(0, 8));
}

#[test]
fn start_still_needs_two_players() {
    let mut state = crate::tournament_engine::create_tournament(config_10_plus());
    assert!(start_tournament(&mut state).is_err());
    register_player(&mut state, "p1", "P1").unwrap();
    register_player(&mut state, "p2", "P2").unwrap();
    assert!(start_tournament(&mut state).is_ok());
    assert_eq!(state.status, TournamentStatus::Running);
}
