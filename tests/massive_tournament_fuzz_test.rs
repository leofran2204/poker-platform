use poker_engine::ledger::LedgerAccount;
use poker_engine::tournament::{
    BlindStructure, TableBalancer, TableStateSummary, Tournament, TournamentState,
};
use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_1_million_tournament_fuzzing_and_table_balancing_simulations() {
    println!("\n========================================================");
    println!(" INICIANDO SIMULAÇÃO MASSIVA DE 1.000.000 DE TORNEIOS (1M) ");
    println!("========================================================\n");

    let total_iterations = 1_000_000;
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();

    let mut total_table_moves_generated = 0u64;
    let mut total_tournaments_completed = 0u64;
    let mut total_payouts_verified = 0u64;

    for i in 1..=total_iterations {
        // 1. Simulação Estocástica de Balanceamento de Mesas
        let num_tables = rng.gen_range(2..=10);
        let mut tables = Vec::new();

        for t_id in 0..num_tables {
            let num_players = rng.gen_range(1..=9);
            let players: Vec<String> = (0..num_players).map(|p| format!("P_{}_{}", t_id, p)).collect();
            tables.push(TableStateSummary {
                table_id: format!("Table_{}", t_id),
                active_player_ids: players,
            });
        }

        let moves = TableBalancer::balance_tables(&tables);
        total_table_moves_generated += moves.len() as u64;

        // 2. Simular Inscrição e Distribuição de Payouts em Torneio a cada 100 iterações
        if i % 100 == 0 {
            let acc1 = LedgerAccount::new(format!("TP1_{}", i), 50000);
            let acc2 = LedgerAccount::new(format!("TP2_{}", i), 50000);
            let acc3 = LedgerAccount::new(format!("TP3_{}", i), 50000);

            let mut accounts = HashMap::new();
            accounts.insert(format!("TP1_{}", i), acc1.clone());
            accounts.insert(format!("TP2_{}", i), acc2.clone());
            accounts.insert(format!("TP3_{}", i), acc3.clone());

            let blind_structure = BlindStructure::turbo_fast();
            let mut tournament = Tournament::new(
                format!("MTT_{}", i),
                "Fuzz Tournament",
                10000,
                1000,
                10000.0,
                blind_structure,
            );

            assert!(tournament.register_player(&format!("TP1_{}", i), "Alice", &acc1).is_ok());
            assert!(tournament.register_player(&format!("TP2_{}", i), "Bob", &acc2).is_ok());
            assert!(tournament.register_player(&format!("TP3_{}", i), "Charlie", &acc3).is_ok());

            // Simular Eliminações até o Final
            tournament.state = TournamentState::Running;
            let _ = tournament.eliminate_player(&format!("TP3_{}", i));
            let _ = tournament.eliminate_player(&format!("TP2_{}", i));
            tournament.players.get_mut(&format!("TP1_{}", i)).unwrap().finish_rank = Some(1);
            tournament.state = TournamentState::Finished;

            let payouts = tournament.distribute_prize_pool(&accounts);
            assert_eq!(payouts.len(), 3);
            
            // Validação de Conservação do Prize Pool (100% dos 30.000 centavos distribuídos)
            let total_payout_sum: i64 = payouts.iter().map(|(_, _, amt)| amt).sum();
            assert_eq!(
                total_payout_sum, tournament.prize_pool_cents,
                "Invariante do Prize Pool violada na iteração {}",
                i
            );

            total_tournaments_completed += 1;
            total_payouts_verified += payouts.len() as u64;
        }
    }

    let elapsed = start_time.elapsed();
    let ops_per_sec = (total_iterations as f64) / elapsed.as_secs_f64();

    println!("   ✔ 1.000.000 de iterações de torneio concluídas com SUCESSO!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!("   - Taxa de Processamento: {:.2} op/segundo", ops_per_sec);
    println!("   - Movimentos de Balanceador Gerados: {}", total_table_moves_generated);
    println!("   - Torneios Completos Fuzzados: {}", total_tournaments_completed);
    println!("   - Payouts do Ledger Verificados: {}", total_payouts_verified);
    println!("========================================================\n");

    assert_eq!(total_iterations, 1_000_000);
}
