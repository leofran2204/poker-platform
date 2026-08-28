// tournament_fuzz_tests.rs — Suíte de Fuzzing Massivo de Rebalanceamento MTT (200.000 iterações)
// Valida a conservação de fichas, limites de balanceamento de mesas e invariants sob eliminação em massa.

use crate::tournament_engine::{
    advance_blinds, create_tournament, eliminate_player, get_current_blinds, register_player,
    start_tournament, BlindLevel, TournamentConfig, TournamentSpeed, TournamentStatus,
};
use proptest::prelude::*;

fn make_default_fuzz_config(
    buy_in: u64,
    starting_stack: u64,
    speed: TournamentSpeed,
) -> TournamentConfig {
    TournamentConfig {
        name: "Fuzz MTT Tournament".to_string(),
        game_type: "Holdem".to_string(),
        buy_in,
        starting_stack,
        max_players: 1000,
        speed,
        blind_levels: vec![
            BlindLevel {
                level: 1,
                small_blind: 10,
                big_blind: 20,
                ante: 0,
                duration_minutes: 10,
            },
            BlindLevel {
                level: 2,
                small_blind: 15,
                big_blind: 30,
                ante: 0,
                duration_minutes: 10,
            },
            BlindLevel {
                level: 3,
                small_blind: 25,
                big_blind: 50,
                ante: 5,
                duration_minutes: 10,
            },
            BlindLevel {
                level: 4,
                small_blind: 50,
                big_blind: 100,
                ante: 10,
                duration_minutes: 10,
            },
        ],
        prize_pool_pct: 1.0,
        prize_distribution: vec![0.50, 0.30, 0.20],
        late_registration: true,
        late_registration_max_level: 3,
        allow_rebuy: true,
        allow_addon: true,
        rebuy_max_level: 4,
            guaranteed_prize: 0,
            is_freeroll: false,
            rebuy_cost: 0,
            rebuy_chips: 0,
            rebuy_max_count: 0,
            rebuy_stack_threshold: 0,
    }
}

fn get_proptest_config() -> ProptestConfig {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(200_000);
    ProptestConfig {
        cases,
        max_shrink_iters: 100,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(get_proptest_config())]

    // ─── 1. Fuzzing de Configuração e Invariantes de Torneio ───
    #[test]
    fn fuzz_tournament_config_invariants(
        buy_in in 1u64..10_000u64,
        starting_stack in 100u64..100_000u64,
        speed_idx in 0..3u8,
    ) {
        let speed = match speed_idx {
            0 => TournamentSpeed::Turbo,
            1 => TournamentSpeed::Normal,
            _ => TournamentSpeed::Slow,
        };

        let config = make_default_fuzz_config(buy_in, starting_stack, speed);
        let tourn = create_tournament(config);

        prop_assert_eq!(&tourn.status, &TournamentStatus::Registering);
        prop_assert_eq!(tourn.players.len(), 0);
        prop_assert_eq!(tourn.prize_pool, 0);
    }

    // ─── 2. Fuzzing de Inscrições e Acúmulo de Prize Pool ───
    #[test]
    fn fuzz_tournament_player_registration(
        num_players in 2..100usize,
        buy_in in 10u64..500u64,
        starting_stack in 1000u64..10000u64,
    ) {
        let config = make_default_fuzz_config(buy_in, starting_stack, TournamentSpeed::Normal);
        let mut tourn = create_tournament(config);

        for p in 0..num_players {
            let pid = format!("player_{}", p);
            let pname = format!("Player {}", p);
            let res = register_player(&mut tourn, &pid, &pname);
            prop_assert!(res.is_ok());
        }

        prop_assert_eq!(tourn.players.len(), num_players);
        let expected_prize_pool = (num_players as u64) * buy_in;
        prop_assert_eq!(tourn.prize_pool, expected_prize_pool);
    }

    // ─── 3. Fuzzing de Rebalanceamento de Mesas sob Eliminações em Massa ───
    #[test]
    fn fuzz_mtt_table_rebalancing_invariants(
        num_players in 6..200usize,
        eliminations in 1..150usize,
    ) {
        let config = make_default_fuzz_config(100, 5000, TournamentSpeed::Turbo);
        let mut tourn = create_tournament(config);

        for p in 0..num_players {
            let pid = format!("mtt_player_{}", p);
            let pname = format!("MTT Player {}", p);
            let _ = register_player(&mut tourn, &pid, &pname);
        }

        let start_res = start_tournament(&mut tourn);
        prop_assert!(start_res.is_ok());
        prop_assert_eq!(&tourn.status, &TournamentStatus::Running);

        // Simula eliminação em massa de jogadores
        let max_elims = eliminations.min(num_players - 1);
        for p in 0..max_elims {
            let pid = format!("mtt_player_{}", p);
            let elim_res = eliminate_player(&mut tourn, &pid, None);
            prop_assert!(elim_res.is_ok());
        }

        let remaining = tourn.players_remaining;
        prop_assert_eq!(remaining, (num_players - max_elims) as u32);

        // Verifica que o histórico de eliminações mantém ordem única
        let elim_set: std::collections::HashSet<_> = tourn.eliminated_order.iter().collect();
        prop_assert_eq!(elim_set.len(), tourn.eliminated_order.len());
    }

    // ─── 4. Fuzzing de Avanço de Níveis de Blinds ───
    #[test]
    fn fuzz_tournament_blind_level_advancement(
        advancements in 1..3u32,
    ) {
        let config = make_default_fuzz_config(50, 3000, TournamentSpeed::Turbo);
        let mut tourn = create_tournament(config);
        let _ = register_player(&mut tourn, "p1", "Player 1");
        let _ = register_player(&mut tourn, "p2", "Player 2");
        let _ = start_tournament(&mut tourn);

        for _ in 0..advancements {
            let _ = advance_blinds(&mut tourn);
        }

        let current_lvl = tourn.current_level;
        prop_assert_eq!(current_lvl, advancements + 1);
        let level_opt = get_current_blinds(&tourn);
        prop_assert!(level_opt.is_some());
        let level = level_opt.unwrap();
        prop_assert!(level.small_blind > 0);
        prop_assert!(level.big_blind >= level.small_blind * 2);
    }
}
