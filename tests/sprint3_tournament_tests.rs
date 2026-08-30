use poker_engine::ledger::LedgerAccount;
use poker_engine::tournament::{
    BlindStructure, TableBalancer, TableStateSummary, Tournament, TournamentState,
};
use std::collections::HashMap;

#[test]
fn test_blind_structure_progression() {
    let regular = BlindStructure::standard_regular();
    assert_eq!(regular.levels.len(), 26);
    assert_eq!(regular.levels[0].small_blind, 25.0);
    assert_eq!(regular.levels[0].big_blind, 50.0);
    assert_eq!(regular.levels[8].ante, 50.0);
    assert_eq!(regular.levels[25].level_number, 26);
    assert_eq!(regular.levels[25].small_blind, 25_000.0);
    assert_eq!(regular.levels[25].big_blind, 50_000.0);
    assert_eq!(regular.levels[25].ante, 8_000.0);

    let turbo = BlindStructure::turbo_fast();
    assert_eq!(turbo.levels.len(), 26);
    assert_eq!(turbo.levels[0].duration_seconds, 180);
}

#[test]
fn test_tournament_registration_and_rebuy() {
    // Saldo inicial suficiente: R$ 200,00 (20000 centavos)
    let account = LedgerAccount::new("Player_1", 20000);
    let blind_structure = BlindStructure::standard_regular();
    let mut tournament = Tournament::new(
        "MTT-001",
        "Sunday Million",
        5000,
        350,
        10000.0,
        blind_structure,
    );

    // Registra jogador (Custo: 5000 buyin + 350 rake = 5350 centavos)
    assert!(tournament
        .register_player("Player_1", "Alice", &account)
        .is_ok());
    assert_eq!(account.get_balance_cents().unwrap(), 14650); // 20000 - 5350 = 14650
    assert_eq!(tournament.prize_pool_cents, 5000);

    // Re-buy com stack zerado (Custo: 5000 centavos)
    tournament.players.get_mut("Player_1").unwrap().chip_stack = 0.0;
    assert!(tournament.rebuy_player("Player_1", &account).is_ok());
    assert_eq!(account.get_balance_cents().unwrap(), 9650); // 14650 - 5000 = 9650
    assert_eq!(tournament.prize_pool_cents, 10000);
}

#[test]
fn test_table_balancer_rebalancing() {
    let tables = vec![
        TableStateSummary {
            table_id: "Table_1".into(),
            active_player_ids: vec![
                "P1".into(),
                "P2".into(),
                "P3".into(),
                "P4".into(),
                "P5".into(),
                "P6".into(),
            ],
        },
        TableStateSummary {
            table_id: "Table_2".into(),
            active_player_ids: vec!["P7".into(), "P8".into(), "P9".into()],
        },
    ];

    let moves = TableBalancer::balance_tables(&tables);
    assert!(!moves.is_empty());
    assert_eq!(moves[0].from_table_id, "Table_1");
    assert_eq!(moves[0].to_table_id, "Table_2");
}

#[test]
fn test_tournament_elimination_and_prize_pool_distribution() {
    // Saldo inicial suficiente para BuyIn (10000) + Rake (700) = 10700 centavos
    let initial_bal = 20000i64;
    let acc_p1 = LedgerAccount::new("P1", initial_bal);
    let acc_p2 = LedgerAccount::new("P2", initial_bal);
    let acc_p3 = LedgerAccount::new("P3", initial_bal);

    let mut accounts = HashMap::new();
    accounts.insert("P1".to_string(), acc_p1.clone());
    accounts.insert("P2".to_string(), acc_p2.clone());
    accounts.insert("P3".to_string(), acc_p3.clone());

    let blind_structure = BlindStructure::standard_regular();
    let mut tournament = Tournament::new(
        "STG-001",
        "Sit & Go 3-Max",
        10000,
        700,
        5000.0,
        blind_structure,
    );

    assert!(tournament.register_player("P1", "Alice", &acc_p1).is_ok());
    assert!(tournament.register_player("P2", "Bob", &acc_p2).is_ok());
    assert!(tournament.register_player("P3", "Charlie", &acc_p3).is_ok());

    tournament.state = TournamentState::Running;
    assert_eq!(tournament.prize_pool_cents, 30000); // 3 * 10000

    // Eliminar P3 (3º lugar -> 20% = R$ 60,00 -> 6000 centavos)
    let rank_p3 = tournament.eliminate_player("P3");
    assert_eq!(rank_p3, Some(3));

    // Eliminar P2 (2º lugar -> 30% = R$ 90,00 -> 9000 centavos)
    let rank_p2 = tournament.eliminate_player("P2");
    assert_eq!(rank_p2, Some(2));

    // P1 é o campeão (1º lugar -> 50% = R$ 150,00 -> 15000 centavos)
    tournament.players.get_mut("P1").unwrap().finish_rank = Some(1);
    tournament.state = TournamentState::Finished;

    // Distribuir prêmios
    let payouts = tournament.distribute_prize_pool(&accounts);
    assert_eq!(payouts.len(), 3);

    // P1 (1º lugar): 20000 - 10700 + 15000 = 24300
    // P2 (2º lugar): 20000 - 10700 + 9000  = 18300
    // P3 (3º lugar): 20000 - 10700 + 6000  = 15300
    assert_eq!(acc_p1.get_balance_cents().unwrap(), 24300);
    assert_eq!(acc_p2.get_balance_cents().unwrap(), 18300);
    assert_eq!(acc_p3.get_balance_cents().unwrap(), 15300);
}
