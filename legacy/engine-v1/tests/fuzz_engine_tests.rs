use poker_engine::engine::{
    calculate_loss_deflators, calculate_side_pots, Contribution, GameLoop, GameState, Player,
    PlayerLossStats,
};
use rand::Rng;

#[test]
fn test_massive_fuzz_side_pots_and_pot_conservation_10k_iterations() {
    let mut rng = rand::thread_rng();

    for iteration in 0..10_000 {
        let num_players = rng.gen_range(2..=9);
        let mut contributions = Vec::new();
        let mut total_bets_sum = 0.0;

        for p_idx in 0..num_players {
            let bet = (rng.gen_range(0..=1000) as f64) * 5.0; // 0.0 a 5000.0 em incrementos
            let has_folded = rng.gen_bool(0.35); // 35% de chance de fold

            total_bets_sum += bet;
            contributions.push(Contribution {
                player_id: format!("Player_{}", p_idx),
                total_bet: bet,
                has_folded,
            });
        }

        let side_pots = calculate_side_pots(&contributions);

        // INVARIANTE 1: Nenhum pote pode ter valor negativo ou zero
        for pot in &side_pots {
            assert!(
                pot.amount > 0.0,
                "Pote com valor <= 0 na iteração {}",
                iteration
            );

            // INVARIANTE 2: Nenhum jogador que deu fold pode ser elegível
            for eligible_id in &pot.eligible_players {
                let player_contrib = contributions
                    .iter()
                    .find(|c| &c.player_id == eligible_id)
                    .unwrap();
                assert!(
                    !player_contrib.has_folded,
                    "VIOLAÇÃO CRÍTICA: Jogador folded {} listado como elegível no pote!",
                    eligible_id
                );
            }
        }

        // INVARIANTE 3: Soma de todos os potes criados para quem não foldou não pode exceder o total das apostas
        let total_pots_sum: f64 = side_pots.iter().map(|p| p.amount).sum();
        assert!(
            total_pots_sum <= total_bets_sum + 1e-6,
            "Soma dos potes ({}) excedeu apostas totais ({}) na iteração {}",
            total_pots_sum,
            total_bets_sum,
            iteration
        );
    }
}

#[test]
fn test_massive_fuzz_loss_deflator_10k_iterations() {
    let mut rng = rand::thread_rng();

    for _ in 0..10_000 {
        let mut stats_list = Vec::new();
        let num_players = rng.gen_range(2..=9);

        for p in 0..num_players {
            let eligible_loss_after_rake = (rng.gen_range(0..=1000) as f64) * 10.0;
            let loser_equity = rng.gen_range(0.0..=1.0);

            stats_list.push(PlayerLossStats {
                player_id: format!("P_{}", p),
                eligible_loss_after_rake,
                loser_equity,
            });
        }

        let deflators = calculate_loss_deflators(&stats_list);
        assert_eq!(deflators.len(), num_players);

        for d in deflators {
            // INVARIANTE 1: Cashback jamais pode ser negativo
            assert!(
                d.cashback_amount >= 0.0,
                "VIOLAÇÃO CRÍTICA: Cashback negativo ({}) para jogador {}",
                d.cashback_amount,
                d.player_id
            );

            // INVARIANTE 2: Perda líquida é estritamente >= 0
            assert!(d.net_loss >= 0.0);

            // INVARIANTE 3: Se o jogador não perdeu dinheiro, cashback DEVE ser 0.0
            let original_player = stats_list
                .iter()
                .find(|s| s.player_id == d.player_id)
                .unwrap();
            if original_player.eligible_loss_after_rake == 0.0 {
                assert_eq!(
                    d.cashback_amount, 0.0,
                    "Jogador vitorioso/empatado recebeu cashback indevido!"
                );
            }
        }
    }
}

#[test]
fn test_massive_fuzz_game_loop_state_machine_transitions() {
    let mut rng = rand::thread_rng();

    for _ in 0..1_000 {
        let num_players = rng.gen_range(2..=9);
        let mut players = Vec::new();

        for i in 0..num_players {
            let stack = (rng.gen_range(0..=500) as f64) * 10.0;
            let mut p = Player::new(format!("P_{}", i), format!("Player {}", i), stack);
            if stack == 0.0 {
                p.is_all_in = true;
            } else if rng.gen_bool(0.2) {
                p.has_folded = true;
            }
            players.push(p);
        }

        let button_idx = rng.gen_range(0..num_players);
        let state = GameState::new(players, button_idx, 10.0);
        let mut game_loop = GameLoop::new(state);

        // Executar transições repetidas garantindo ausência de deadlocks ou loops infinitos
        for _ in 0..50 {
            let advanced = game_loop.advance_turn();
            if !advanced {
                // Chegou ao fim da mão ou Showdown
                break;
            }
            assert!(
                game_loop.state.current_player_idx < num_players,
                "Índice de jogador inválido na mesa"
            );
        }
    }
}
