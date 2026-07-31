use poker_engine::rake::{deduct_rake, deduct_rake_with_rounding_for_players, RakeRounding};
use poker_engine::types::{Pot, RakeCapSchedule, TableConfig};

fn contested_pot(amount: u64) -> Pot {
    Pot::new(amount, vec!["alice".into(), "bob".into()])
}

#[test]
fn cap_schedule_uses_the_number_of_players_dealt_into_the_hand() {
    let config = TableConfig::new(200, 500, 9_999).with_rake_cap_schedule(RakeCapSchedule {
        heads_up: 100,
        three_to_four: 200,
        five_plus: 300,
    });
    let pots = vec![contested_pot(100_000)];

    for (players_dealt, expected_cap) in [(2, 100), (4, 200), (9, 300)] {
        let result = deduct_rake_with_rounding_for_players(
            &pots,
            &config,
            None,
            RakeRounding::HalfToEven,
            players_dealt,
        );
        assert_eq!(result.total_rake, expected_cap);
    }
}

#[test]
fn legacy_table_cap_remains_the_fallback_without_a_schedule() {
    let config = TableConfig::new(200, 500, 250);
    let pots = vec![contested_pot(100_000)];

    let result = deduct_rake(&pots, &config, None);

    assert_eq!(result.total_rake, 250);
}
