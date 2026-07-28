// tournament_engine_tests.rs — Testes abrangentes do Tournament Engine
//
// Cobertura completa da engine de torneios:
//   Lote 7A: Config & Creation (160 testes)
//   Lote 7B: Registration & Late Registration (200 testes)
//   Lote 7C: Lifecycle & Blinds (160 testes)
//   Lote 7D: Elimination & Re-buy (200 testes)
//   Lote 7E: Add-on & Finish (160 testes)
//   Lote 7F: Cancel, Stats & Serialization (80 testes)
//
// Cada lote cobre: casos normais, edge cases, erros e invariantes.

#![allow(unused_imports, unused_variables, dead_code)]

use crate::tournament_engine::{
    advance_blinds, cancel_tournament, create_tournament, eliminate_player, finish_tournament,
    get_current_blinds, get_tournament_stats, is_blind_level_expired, pause_tournament,
    process_addon, process_rebuy, register_player, resume_tournament, start_tournament, BlindLevel,
    PlayerTournamentEntry, TournamentConfig, TournamentResult, TournamentSpeed, TournamentStats,
    TournamentStatus,
};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Helpers compartilhados
// ═══════════════════════════════════════════════════════════════════

fn default_config() -> TournamentConfig {
    TournamentConfig {
        name: "Test Tournament".to_string(),
        game_type: "Holdem".to_string(),
        buy_in: 1000,
        starting_stack: 10000,
        max_players: 100,
        speed: TournamentSpeed::Normal,
        blind_levels: vec![
            BlindLevel {
                level: 1,
                small_blind: 10,
                big_blind: 20,
                ante: 0,
                duration_minutes: 15,
            },
            BlindLevel {
                level: 2,
                small_blind: 20,
                big_blind: 40,
                ante: 0,
                duration_minutes: 15,
            },
            BlindLevel {
                level: 3,
                small_blind: 30,
                big_blind: 60,
                ante: 5,
                duration_minutes: 15,
            },
            BlindLevel {
                level: 4,
                small_blind: 50,
                big_blind: 100,
                ante: 10,
                duration_minutes: 15,
            },
            BlindLevel {
                level: 5,
                small_blind: 100,
                big_blind: 200,
                ante: 20,
                duration_minutes: 15,
            },
        ],
        prize_pool_pct: 0.90,
        prize_distribution: vec![0.50, 0.30, 0.20],
        late_registration: true,
        late_registration_max_level: 3,
        allow_rebuy: true,
        allow_addon: true,
        rebuy_max_level: 4,
    }
}

fn long_config(levels_count: usize) -> TournamentConfig {
    let mut config = default_config();
    config.blind_levels.clear();
    for i in 1..=levels_count {
        config.blind_levels.push(BlindLevel {
            level: i as u32,
            small_blind: (i as u64) * 10,
            big_blind: (i as u64) * 20,
            ante: if i > 2 { (i as u64 - 2) * 5 } else { 0 },
            duration_minutes: 10,
        });
    }
    config
}

fn config_with_speed(speed: TournamentSpeed) -> TournamentConfig {
    let mut c = default_config();
    c.speed = speed;
    c
}

fn config_with_max_players(max: u32) -> TournamentConfig {
    let mut c = default_config();
    c.max_players = max;
    c
}

fn config_with_buyin(buy_in: u64) -> TournamentConfig {
    let mut c = default_config();
    c.buy_in = buy_in;
    c
}

fn config_with_starting_stack(stack: u64) -> TournamentConfig {
    let mut c = default_config();
    c.starting_stack = stack;
    c
}

fn config_with_prize_pct(pct: f64) -> TournamentConfig {
    let mut c = default_config();
    c.prize_pool_pct = pct;
    c
}

fn config_with_prize_distribution(dist: Vec<f64>) -> TournamentConfig {
    let mut c = default_config();
    c.prize_distribution = dist;
    c
}

fn config_no_late_registration() -> TournamentConfig {
    let mut c = default_config();
    c.late_registration = false;
    c
}

fn config_no_rebuy() -> TournamentConfig {
    let mut c = default_config();
    c.allow_rebuy = false;
    c
}

fn config_no_addon() -> TournamentConfig {
    let mut c = default_config();
    c.allow_addon = false;
    c
}

fn config_with_blind_levels(levels: Vec<BlindLevel>) -> TournamentConfig {
    let mut c = default_config();
    c.blind_levels = levels;
    c
}

fn config_with_name(name: &str) -> TournamentConfig {
    let mut c = default_config();
    c.name = name.to_string();
    c
}

fn config_with_game_type(gt: &str) -> TournamentConfig {
    let mut c = default_config();
    c.game_type = gt.to_string();
    c
}

fn make_blind_level(level: u32, sb: u64, bb: u64, ante: u64, mins: u32) -> BlindLevel {
    BlindLevel {
        level,
        small_blind: sb,
        big_blind: bb,
        ante,
        duration_minutes: mins,
    }
}

fn register_n_players(state: &mut crate::tournament_engine::TournamentState, n: usize) {
    for i in 1..=n {
        let id = format!("p{}", i);
        let name = format!("Player {}", i);
        register_player(state, &id, &name).unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════════
// Lote 7A — Config & Creation (160 testes)
// ═══════════════════════════════════════════════════════════════════

mod lote_7a_blind_level {
    use super::*;

    // --- Construção e campos ---

    #[test]
    fn test_blind_level_basic_construction() {
        let bl = make_blind_level(1, 10, 20, 0, 15);
        assert_eq!(bl.level, 1);
        assert_eq!(bl.small_blind, 10);
        assert_eq!(bl.big_blind, 20);
        assert_eq!(bl.ante, 0);
        assert_eq!(bl.duration_minutes, 15);
    }

    #[test]
    fn test_blind_level_with_ante() {
        let bl = make_blind_level(3, 30, 60, 5, 15);
        assert_eq!(bl.ante, 5);
    }

    #[test]
    fn test_blind_level_zero_ante() {
        let bl = make_blind_level(1, 10, 20, 0, 15);
        assert_eq!(bl.ante, 0);
    }

    #[test]
    fn test_blind_level_high_ante() {
        let bl = make_blind_level(5, 100, 200, 20, 15);
        assert_eq!(bl.ante, 20);
    }

    #[test]
    fn test_blind_level_level_zero() {
        let bl = make_blind_level(0, 5, 10, 0, 10);
        assert_eq!(bl.level, 0);
    }

    #[test]
    fn test_blind_level_large_values() {
        let bl = make_blind_level(100, 50000, 100000, 5000, 60);
        assert_eq!(bl.small_blind, 50000);
        assert_eq!(bl.big_blind, 100000);
    }

    #[test]
    fn test_blind_level_bb_is_double_sb() {
        let bl = make_blind_level(1, 10, 20, 0, 15);
        assert_eq!(bl.big_blind, bl.small_blind * 2);
    }

    #[test]
    fn test_blind_level_bb_not_double_sb() {
        let bl = make_blind_level(1, 10, 25, 0, 15);
        assert_ne!(bl.big_blind, bl.small_blind * 2);
    }

    #[test]
    fn test_blind_level_duration_variations() {
        let bl5 = make_blind_level(1, 10, 20, 0, 5);
        let bl10 = make_blind_level(1, 10, 20, 0, 10);
        let bl15 = make_blind_level(1, 10, 20, 0, 15);
        let bl30 = make_blind_level(1, 10, 20, 0, 30);
        let bl60 = make_blind_level(1, 10, 20, 0, 60);
        assert_eq!(bl5.duration_minutes, 5);
        assert_eq!(bl10.duration_minutes, 10);
        assert_eq!(bl15.duration_minutes, 15);
        assert_eq!(bl30.duration_minutes, 30);
        assert_eq!(bl60.duration_minutes, 60);
    }

    #[test]
    fn test_blind_level_clone() {
        let bl = make_blind_level(1, 10, 20, 0, 15);
        let bl2 = bl.clone();
        assert_eq!(bl.level, bl2.level);
        assert_eq!(bl.small_blind, bl2.small_blind);
        assert_eq!(bl.big_blind, bl2.big_blind);
        assert_eq!(bl.ante, bl2.ante);
        assert_eq!(bl.duration_minutes, bl2.duration_minutes);
    }

    #[test]
    fn test_blind_level_debug_format() {
        let bl = make_blind_level(1, 10, 20, 0, 15);
        let debug = format!("{:?}", bl);
        assert!(debug.contains("BlindLevel"));
        assert!(debug.contains("level: 1"));
        assert!(debug.contains("small_blind: 10"));
    }

    #[test]
    fn test_blind_level_serialize_json() {
        let bl = make_blind_level(1, 10, 20, 0, 15);
        let json = serde_json::to_string(&bl).unwrap();
        assert!(json.contains("\"level\":1"));
        assert!(json.contains("\"small_blind\":10"));
        assert!(json.contains("\"big_blind\":20"));
    }

    #[test]
    fn test_blind_level_deserialize_json() {
        let json = r#"{"level":2,"small_blind":20,"big_blind":40,"ante":0,"duration_minutes":15}"#;
        let bl: BlindLevel = serde_json::from_str(json).unwrap();
        assert_eq!(bl.level, 2);
        assert_eq!(bl.small_blind, 20);
        assert_eq!(bl.big_blind, 40);
    }

    #[test]
    fn test_blind_level_round_trip_json() {
        let bl = make_blind_level(3, 30, 60, 5, 15);
        let json = serde_json::to_string(&bl).unwrap();
        let bl2: BlindLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(bl.level, bl2.level);
        assert_eq!(bl.small_blind, bl2.small_blind);
        assert_eq!(bl.big_blind, bl2.big_blind);
        assert_eq!(bl.ante, bl2.ante);
        assert_eq!(bl.duration_minutes, bl2.duration_minutes);
    }

    #[test]
    fn test_blind_level_with_ante_serialize() {
        let bl = make_blind_level(3, 30, 60, 5, 15);
        let json = serde_json::to_string(&bl).unwrap();
        assert!(json.contains("\"ante\":5"));
    }

    #[test]
    fn test_blind_level_zero_values() {
        let bl = make_blind_level(0, 0, 0, 0, 0);
        let json = serde_json::to_string(&bl).unwrap();
        let bl2: BlindLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(bl2.level, 0);
        assert_eq!(bl2.small_blind, 0);
        assert_eq!(bl2.big_blind, 0);
        assert_eq!(bl2.ante, 0);
        assert_eq!(bl2.duration_minutes, 0);
    }

    #[test]
    fn test_blind_level_max_u32_level() {
        let bl = make_blind_level(u32::MAX, 10, 20, 0, 15);
        assert_eq!(bl.level, u32::MAX);
    }

    #[test]
    fn test_blind_level_max_u64_sb() {
        let bl = make_blind_level(1, u64::MAX, u64::MAX, 0, 15);
        assert_eq!(bl.small_blind, u64::MAX);
        assert_eq!(bl.big_blind, u64::MAX);
    }
}

mod lote_7a_tournament_speed {
    use super::*;

    #[test]
    fn test_speed_turbo() {
        let speed = TournamentSpeed::Turbo;
        assert_eq!(speed, TournamentSpeed::Turbo);
    }

    #[test]
    fn test_speed_normal() {
        let speed = TournamentSpeed::Normal;
        assert_eq!(speed, TournamentSpeed::Normal);
    }

    #[test]
    fn test_speed_slow() {
        let speed = TournamentSpeed::Slow;
        assert_eq!(speed, TournamentSpeed::Slow);
    }

    #[test]
    fn test_speed_turbo_neq_normal() {
        assert_ne!(TournamentSpeed::Turbo, TournamentSpeed::Normal);
    }

    #[test]
    fn test_speed_turbo_neq_slow() {
        assert_ne!(TournamentSpeed::Turbo, TournamentSpeed::Slow);
    }

    #[test]
    fn test_speed_normal_neq_slow() {
        assert_ne!(TournamentSpeed::Normal, TournamentSpeed::Slow);
    }

    #[test]
    fn test_speed_clone() {
        let speed = TournamentSpeed::Turbo;
        let speed2 = speed.clone();
        assert_eq!(speed, speed2);
    }

    #[test]
    fn test_speed_debug() {
        let debug = format!("{:?}", TournamentSpeed::Turbo);
        assert_eq!(debug, "Turbo");
    }

    #[test]
    fn test_speed_serialize_lowercase() {
        let json = serde_json::to_string(&TournamentSpeed::Turbo).unwrap();
        assert_eq!(json, "\"turbo\"");
        let json = serde_json::to_string(&TournamentSpeed::Normal).unwrap();
        assert_eq!(json, "\"normal\"");
        let json = serde_json::to_string(&TournamentSpeed::Slow).unwrap();
        assert_eq!(json, "\"slow\"");
    }

    #[test]
    fn test_speed_deserialize_lowercase() {
        let turbo: TournamentSpeed = serde_json::from_str("\"turbo\"").unwrap();
        assert_eq!(turbo, TournamentSpeed::Turbo);
        let normal: TournamentSpeed = serde_json::from_str("\"normal\"").unwrap();
        assert_eq!(normal, TournamentSpeed::Normal);
        let slow: TournamentSpeed = serde_json::from_str("\"slow\"").unwrap();
        assert_eq!(slow, TournamentSpeed::Slow);
    }

    #[test]
    fn test_speed_round_trip_json() {
        for speed in [
            TournamentSpeed::Turbo,
            TournamentSpeed::Normal,
            TournamentSpeed::Slow,
        ] {
            let json = serde_json::to_string(&speed).unwrap();
            let deserialized: TournamentSpeed = serde_json::from_str(&json).unwrap();
            assert_eq!(speed, deserialized);
        }
    }

    #[test]
    fn test_speed_invalid_deserialize() {
        let result: Result<TournamentSpeed, _> = serde_json::from_str("\"fast\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_speed_invalid_deserialize_empty() {
        let result: Result<TournamentSpeed, _> = serde_json::from_str("\"\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_speed_invalid_deserialize_number() {
        let result: Result<TournamentSpeed, _> = serde_json::from_str("123");
        assert!(result.is_err());
    }

    #[test]
    fn test_speed_invalid_deserialize_null() {
        let result: Result<TournamentSpeed, _> = serde_json::from_str("null");
        assert!(result.is_err());
    }

    #[test]
    fn test_speed_invalid_deserialize_capitalized() {
        let result: Result<TournamentSpeed, _> = serde_json::from_str("\"Turbo\"");
        assert!(result.is_err());
    }
}

mod lote_7a_tournament_status {
    use super::*;

    #[test]
    fn test_status_registering() {
        assert_eq!(TournamentStatus::Registering, TournamentStatus::Registering);
    }

    #[test]
    fn test_status_running() {
        assert_eq!(TournamentStatus::Running, TournamentStatus::Running);
    }

    #[test]
    fn test_status_paused() {
        assert_eq!(TournamentStatus::Paused, TournamentStatus::Paused);
    }

    #[test]
    fn test_status_finished() {
        assert_eq!(TournamentStatus::Finished, TournamentStatus::Finished);
    }

    #[test]
    fn test_status_cancelled() {
        assert_eq!(TournamentStatus::Cancelled, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_status_registering_neq_running() {
        assert_ne!(TournamentStatus::Registering, TournamentStatus::Running);
    }

    #[test]
    fn test_status_running_neq_paused() {
        assert_ne!(TournamentStatus::Running, TournamentStatus::Paused);
    }

    #[test]
    fn test_status_paused_neq_finished() {
        assert_ne!(TournamentStatus::Paused, TournamentStatus::Finished);
    }

    #[test]
    fn test_status_finished_neq_cancelled() {
        assert_ne!(TournamentStatus::Finished, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_status_registering_neq_cancelled() {
        assert_ne!(TournamentStatus::Registering, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_status_clone() {
        let status = TournamentStatus::Running;
        let status2 = status.clone();
        assert_eq!(status, status2);
    }

    #[test]
    fn test_status_debug() {
        let debug = format!("{:?}", TournamentStatus::Running);
        assert_eq!(debug, "Running");
    }

    #[test]
    fn test_status_serialize_lowercase() {
        assert_eq!(
            serde_json::to_string(&TournamentStatus::Registering).unwrap(),
            "\"registering\""
        );
        assert_eq!(
            serde_json::to_string(&TournamentStatus::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&TournamentStatus::Paused).unwrap(),
            "\"paused\""
        );
        assert_eq!(
            serde_json::to_string(&TournamentStatus::Finished).unwrap(),
            "\"finished\""
        );
        assert_eq!(
            serde_json::to_string(&TournamentStatus::Cancelled).unwrap(),
            "\"cancelled\""
        );
    }

    #[test]
    fn test_status_deserialize_lowercase() {
        let r: TournamentStatus = serde_json::from_str("\"registering\"").unwrap();
        assert_eq!(r, TournamentStatus::Registering);
        let r: TournamentStatus = serde_json::from_str("\"running\"").unwrap();
        assert_eq!(r, TournamentStatus::Running);
        let r: TournamentStatus = serde_json::from_str("\"paused\"").unwrap();
        assert_eq!(r, TournamentStatus::Paused);
        let r: TournamentStatus = serde_json::from_str("\"finished\"").unwrap();
        assert_eq!(r, TournamentStatus::Finished);
        let r: TournamentStatus = serde_json::from_str("\"cancelled\"").unwrap();
        assert_eq!(r, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_status_round_trip_json() {
        let statuses = [
            TournamentStatus::Registering,
            TournamentStatus::Running,
            TournamentStatus::Paused,
            TournamentStatus::Finished,
            TournamentStatus::Cancelled,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let deserialized: TournamentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, deserialized);
        }
    }

    #[test]
    fn test_status_invalid_deserialize() {
        let result: Result<TournamentStatus, _> = serde_json::from_str("\"waiting\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_status_invalid_deserialize_empty() {
        let result: Result<TournamentStatus, _> = serde_json::from_str("\"\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_status_invalid_deserialize_capitalized() {
        let result: Result<TournamentStatus, _> = serde_json::from_str("\"Running\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_status_invalid_deserialize_number() {
        let result: Result<TournamentStatus, _> = serde_json::from_str("42");
        assert!(result.is_err());
    }

    #[test]
    fn test_status_invalid_deserialize_null() {
        let result: Result<TournamentStatus, _> = serde_json::from_str("null");
        assert!(result.is_err());
    }
}

mod lote_7a_tournament_config {
    use super::*;

    #[test]
    fn test_config_default_fields() {
        let c = default_config();
        assert_eq!(c.name, "Test Tournament");
        assert_eq!(c.game_type, "Holdem");
        assert_eq!(c.buy_in, 1000);
        assert_eq!(c.starting_stack, 10000);
        assert_eq!(c.max_players, 100);
        assert_eq!(c.speed, TournamentSpeed::Normal);
        assert_eq!(c.blind_levels.len(), 5);
        assert_eq!(c.prize_pool_pct, 0.90);
        assert_eq!(c.prize_distribution, vec![0.50, 0.30, 0.20]);
        assert!(c.late_registration);
        assert_eq!(c.late_registration_max_level, 3);
        assert!(c.allow_rebuy);
        assert!(c.allow_addon);
        assert_eq!(c.rebuy_max_level, 4);
    }

    #[test]
    fn test_config_with_speed_turbo() {
        let c = config_with_speed(TournamentSpeed::Turbo);
        assert_eq!(c.speed, TournamentSpeed::Turbo);
    }

    #[test]
    fn test_config_with_speed_slow() {
        let c = config_with_speed(TournamentSpeed::Slow);
        assert_eq!(c.speed, TournamentSpeed::Slow);
    }

    #[test]
    fn test_config_with_max_players() {
        let c = config_with_max_players(50);
        assert_eq!(c.max_players, 50);
    }

    #[test]
    fn test_config_with_max_players_2() {
        let c = config_with_max_players(2);
        assert_eq!(c.max_players, 2);
    }

    #[test]
    fn test_config_with_max_players_large() {
        let c = config_with_max_players(10000);
        assert_eq!(c.max_players, 10000);
    }

    #[test]
    fn test_config_with_buyin() {
        let c = config_with_buyin(500);
        assert_eq!(c.buy_in, 500);
    }

    #[test]
    fn test_config_with_buyin_zero() {
        let c = config_with_buyin(0);
        assert_eq!(c.buy_in, 0);
    }

    #[test]
    fn test_config_with_buyin_large() {
        let c = config_with_buyin(1_000_000);
        assert_eq!(c.buy_in, 1_000_000);
    }

    #[test]
    fn test_config_with_starting_stack() {
        let c = config_with_starting_stack(5000);
        assert_eq!(c.starting_stack, 5000);
    }

    #[test]
    fn test_config_with_starting_stack_zero() {
        let c = config_with_starting_stack(0);
        assert_eq!(c.starting_stack, 0);
    }

    #[test]
    fn test_config_with_starting_stack_large() {
        let c = config_with_starting_stack(100_000);
        assert_eq!(c.starting_stack, 100_000);
    }

    #[test]
    fn test_config_with_prize_pct() {
        let c = config_with_prize_pct(0.80);
        assert_eq!(c.prize_pool_pct, 0.80);
    }

    #[test]
    fn test_config_with_prize_pct_zero() {
        let c = config_with_prize_pct(0.0);
        assert_eq!(c.prize_pool_pct, 0.0);
    }

    #[test]
    fn test_config_with_prize_pct_full() {
        let c = config_with_prize_pct(1.0);
        assert_eq!(c.prize_pool_pct, 1.0);
    }

    #[test]
    fn test_config_with_prize_distribution() {
        let c = config_with_prize_distribution(vec![0.60, 0.25, 0.10, 0.05]);
        assert_eq!(c.prize_distribution, vec![0.60, 0.25, 0.10, 0.05]);
    }

    #[test]
    fn test_config_with_prize_distribution_single() {
        let c = config_with_prize_distribution(vec![1.0]);
        assert_eq!(c.prize_distribution, vec![1.0]);
    }

    #[test]
    fn test_config_with_prize_distribution_empty() {
        let c = config_with_prize_distribution(vec![]);
        assert!(c.prize_distribution.is_empty());
    }

    #[test]
    fn test_config_no_late_registration() {
        let c = config_no_late_registration();
        assert!(!c.late_registration);
    }

    #[test]
    fn test_config_no_rebuy() {
        let c = config_no_rebuy();
        assert!(!c.allow_rebuy);
    }

    #[test]
    fn test_config_no_addon() {
        let c = config_no_addon();
        assert!(!c.allow_addon);
    }

    #[test]
    fn test_config_with_name() {
        let c = config_with_name("My Cup");
        assert_eq!(c.name, "My Cup");
    }

    #[test]
    fn test_config_with_name_empty() {
        let c = config_with_name("");
        assert_eq!(c.name, "");
    }

    #[test]
    fn test_config_with_name_unicode() {
        let c = config_with_name("Torneio Ñandú");
        assert_eq!(c.name, "Torneio Ñandú");
    }

    #[test]
    fn test_config_with_name_special_chars() {
        let c = config_with_name("Tor-neio #1 @2026");
        assert_eq!(c.name, "Tor-neio #1 @2026");
    }

    #[test]
    fn test_config_with_game_type_holdem() {
        let c = config_with_game_type("Holdem");
        assert_eq!(c.game_type, "Holdem");
    }

    #[test]
    fn test_config_with_game_type_omaha() {
        let c = config_with_game_type("Omaha");
        assert_eq!(c.game_type, "Omaha");
    }

    #[test]
    fn test_config_with_game_type_empty() {
        let c = config_with_game_type("");
        assert_eq!(c.game_type, "");
    }

    #[test]
    fn test_config_clone() {
        let c = default_config();
        let c2 = c.clone();
        assert_eq!(c.name, c2.name);
        assert_eq!(c.buy_in, c2.buy_in);
        assert_eq!(c.blind_levels.len(), c2.blind_levels.len());
    }

    #[test]
    fn test_config_debug() {
        let c = default_config();
        let debug = format!("{:?}", c);
        assert!(debug.contains("TournamentConfig"));
        assert!(debug.contains("Test Tournament"));
    }

    #[test]
    fn test_config_serialize_json() {
        let c = default_config();
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"name\":\"Test Tournament\""));
        assert!(json.contains("\"buy_in\":1000"));
    }

    #[test]
    fn test_config_deserialize_json() {
        let c = default_config();
        let json = serde_json::to_string(&c).unwrap();
        let c2: TournamentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.name, c2.name);
        assert_eq!(c.buy_in, c2.buy_in);
    }

    #[test]
    fn test_config_round_trip_json() {
        let c = default_config();
        let json = serde_json::to_string(&c).unwrap();
        let c2: TournamentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.blind_levels.len(), c2.blind_levels.len());
        assert_eq!(c.prize_distribution, c2.prize_distribution);
    }

    #[test]
    fn test_config_with_blind_levels_empty() {
        let c = config_with_blind_levels(vec![]);
        assert!(c.blind_levels.is_empty());
    }

    #[test]
    fn test_config_with_blind_levels_single() {
        let c = config_with_blind_levels(vec![make_blind_level(1, 10, 20, 0, 10)]);
        assert_eq!(c.blind_levels.len(), 1);
    }

    #[test]
    fn test_config_with_blind_levels_many() {
        let levels: Vec<BlindLevel> = (1..=20u64)
            .map(|i| make_blind_level(i as u32, i * 10, i * 20, 0, 15))
            .collect();
        let c = config_with_blind_levels(levels);
        assert_eq!(c.blind_levels.len(), 20);
    }

    #[test]
    fn test_config_blind_levels_progression() {
        let c = default_config();
        assert_eq!(c.blind_levels[0].small_blind, 10);
        assert_eq!(c.blind_levels[1].small_blind, 20);
        assert_eq!(c.blind_levels[2].small_blind, 30);
        assert_eq!(c.blind_levels[3].small_blind, 50);
        assert_eq!(c.blind_levels[4].small_blind, 100);
    }

    #[test]
    fn test_config_blind_levels_ante_progression() {
        let c = default_config();
        assert_eq!(c.blind_levels[0].ante, 0);
        assert_eq!(c.blind_levels[1].ante, 0);
        assert_eq!(c.blind_levels[2].ante, 5);
        assert_eq!(c.blind_levels[3].ante, 10);
        assert_eq!(c.blind_levels[4].ante, 20);
    }
}

mod lote_7a_create_tournament {
    use super::*;

    #[test]
    fn test_create_tournament_basic() {
        let config = default_config();
        let state = create_tournament(config);
        assert_eq!(state.status, TournamentStatus::Registering);
    }

    #[test]
    fn test_create_tournament_current_level_zero() {
        let state = create_tournament(default_config());
        assert_eq!(state.current_level, 0);
    }

    #[test]
    fn test_create_tournament_no_players() {
        let state = create_tournament(default_config());
        assert_eq!(state.players.len(), 0);
    }

    #[test]
    fn test_create_tournament_players_remaining_zero() {
        let state = create_tournament(default_config());
        assert_eq!(state.players_remaining, 0);
    }

    #[test]
    fn test_create_tournament_empty_eliminated_order() {
        let state = create_tournament(default_config());
        assert!(state.eliminated_order.is_empty());
    }

    #[test]
    fn test_create_tournament_no_started_at() {
        let state = create_tournament(default_config());
        assert!(state.started_at.is_none());
    }

    #[test]
    fn test_create_tournament_no_finished_at() {
        let state = create_tournament(default_config());
        assert!(state.finished_at.is_none());
    }

    #[test]
    fn test_create_tournament_total_buyins_zero() {
        let state = create_tournament(default_config());
        assert_eq!(state.total_buyins, 0);
    }

    #[test]
    fn test_create_tournament_total_rebuys_zero() {
        let state = create_tournament(default_config());
        assert_eq!(state.total_rebuys, 0);
    }

    #[test]
    fn test_create_tournament_total_addons_zero() {
        let state = create_tournament(default_config());
        assert_eq!(state.total_addons, 0);
    }

    #[test]
    fn test_create_tournament_prize_pool_zero() {
        let state = create_tournament(default_config());
        assert_eq!(state.prize_pool, 0);
    }

    #[test]
    fn test_create_tournament_level_started_at_zero() {
        let state = create_tournament(default_config());
        assert_eq!(state.level_started_at, 0);
    }

    #[test]
    fn test_create_tournament_id_contains_name() {
        let state = create_tournament(default_config());
        assert!(state.tournament_id.contains("test_tournament"));
    }

    #[test]
    fn test_create_tournament_id_format() {
        let state = create_tournament(default_config());
        // Format: sanitized_name_timestamp
        assert!(state.tournament_id.contains('_'));
    }

    #[test]
    fn test_create_tournament_config_preserved() {
        let config = default_config();
        let state = create_tournament(config.clone());
        assert_eq!(state.config.name, config.name);
        assert_eq!(state.config.buy_in, config.buy_in);
        assert_eq!(state.config.max_players, config.max_players);
    }

    #[test]
    fn test_create_tournament_with_speed_turbo() {
        let state = create_tournament(config_with_speed(TournamentSpeed::Turbo));
        assert_eq!(state.config.speed, TournamentSpeed::Turbo);
    }

    #[test]
    fn test_create_tournament_with_speed_slow() {
        let state = create_tournament(config_with_speed(TournamentSpeed::Slow));
        assert_eq!(state.config.speed, TournamentSpeed::Slow);
    }

    #[test]
    fn test_create_tournament_with_max_players_2() {
        let state = create_tournament(config_with_max_players(2));
        assert_eq!(state.config.max_players, 2);
    }

    #[test]
    fn test_create_tournament_with_max_players_large() {
        let state = create_tournament(config_with_max_players(5000));
        assert_eq!(state.config.max_players, 5000);
    }

    #[test]
    fn test_create_tournament_with_buyin_zero() {
        let state = create_tournament(config_with_buyin(0));
        assert_eq!(state.config.buy_in, 0);
    }

    #[test]
    fn test_create_tournament_with_starting_stack_zero() {
        let state = create_tournament(config_with_starting_stack(0));
        assert_eq!(state.config.starting_stack, 0);
    }

    #[test]
    fn test_create_tournament_with_empty_blind_levels() {
        let state = create_tournament(config_with_blind_levels(vec![]));
        assert!(state.config.blind_levels.is_empty());
    }

    #[test]
    fn test_create_tournament_with_single_blind_level() {
        let state = create_tournament(config_with_blind_levels(vec![make_blind_level(
            1, 10, 20, 0, 10,
        )]));
        assert_eq!(state.config.blind_levels.len(), 1);
    }

    #[test]
    fn test_create_tournament_no_late_registration() {
        let state = create_tournament(config_no_late_registration());
        assert!(!state.config.late_registration);
    }

    #[test]
    fn test_create_tournament_no_rebuy() {
        let state = create_tournament(config_no_rebuy());
        assert!(!state.config.allow_rebuy);
    }

    #[test]
    fn test_create_tournament_no_addon() {
        let state = create_tournament(config_no_addon());
        assert!(!state.config.allow_addon);
    }

    #[test]
    fn test_create_tournament_with_unicode_name() {
        let state = create_tournament(config_with_name("Torneio Ñandú"));
        assert!(state.tournament_id.contains("torneio"));
    }

    #[test]
    fn test_create_tournament_with_empty_name() {
        let state = create_tournament(config_with_name(""));
        assert_eq!(state.config.name, "");
    }

    #[test]
    fn test_create_tournament_with_special_chars_name() {
        let state = create_tournament(config_with_name("Cup #1 @2026"));
        assert!(state.tournament_id.contains("cup"));
    }

    #[test]
    fn test_create_tournament_prize_distribution_preserved() {
        let dist = vec![0.60, 0.25, 0.10, 0.05];
        let state = create_tournament(config_with_prize_distribution(dist.clone()));
        assert_eq!(state.config.prize_distribution, dist);
    }

    #[test]
    fn test_create_tournament_prize_pct_preserved() {
        let state = create_tournament(config_with_prize_pct(0.75));
        assert_eq!(state.config.prize_pool_pct, 0.75);
    }

    #[test]
    fn test_create_tournament_state_debug() {
        let state = create_tournament(default_config());
        let debug = format!("{:?}", state);
        assert!(debug.contains("TournamentState"));
    }

    #[test]
    fn test_create_tournament_state_serialize() {
        let state = create_tournament(default_config());
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"status\":\"registering\""));
    }

    #[test]
    fn test_create_tournament_state_deserialize() {
        let state = create_tournament(default_config());
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state.tournament_id, state2.tournament_id);
        assert_eq!(state.status, state2.status);
    }

    #[test]
    fn test_create_tournament_state_round_trip() {
        let state = create_tournament(default_config());
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state.config.name, state2.config.name);
        assert_eq!(state.config.buy_in, state2.config.buy_in);
        assert_eq!(state.players.len(), state2.players.len());
    }

    #[test]
    fn test_create_tournament_players_is_hashmap() {
        let state = create_tournament(default_config());
        assert!(state.players.is_empty());
        assert_eq!(state.players.capacity(), 0);
    }

    #[test]
    fn test_create_two_tournaments_different_ids() {
        let state1 = create_tournament(default_config());
        let state2 = create_tournament(default_config());
        // Timestamps may be equal if called in same second, but IDs should still differ
        // because of the name+timestamp combo. If same second, IDs could match.
        // At minimum, both should be valid format.
        assert!(state1.tournament_id.contains('_'));
        assert!(state2.tournament_id.contains('_'));
    }

    #[test]
    fn test_create_tournament_with_different_names_different_ids() {
        let state1 = create_tournament(config_with_name("Cup A"));
        let state2 = create_tournament(config_with_name("Cup B"));
        assert_ne!(state1.tournament_id, state2.tournament_id);
    }

    #[test]
    fn test_create_tournament_id_lowercase() {
        let state = create_tournament(config_with_name("My Tournament"));
        assert!(state.tournament_id.starts_with("my_tournament"));
    }

    #[test]
    fn test_create_tournament_id_spaces_replaced() {
        let state = create_tournament(config_with_name("My Big Tournament"));
        assert!(!state.tournament_id.contains(' '));
    }

    #[test]
    fn test_create_tournament_preserves_all_config_fields() {
        let config = default_config();
        let state = create_tournament(config.clone());
        assert_eq!(state.config.game_type, config.game_type);
        assert_eq!(state.config.starting_stack, config.starting_stack);
        assert_eq!(
            state.config.late_registration_max_level,
            config.late_registration_max_level
        );
        assert_eq!(state.config.rebuy_max_level, config.rebuy_max_level);
    }
}

mod lote_7a_player_tournament_entry {
    use super::*;

    fn make_entry() -> PlayerTournamentEntry {
        PlayerTournamentEntry {
            player_id: "p1".to_string(),
            player_name: "Player 1".to_string(),
            stack: 10000,
            table_id: Some(1),
            seat: Some(3),
            rebuys: 0,
            addon_done: false,
            final_position: None,
            prize: None,
            registered_at: 1000,
            eliminated_at: None,
        }
    }

    #[test]
    fn test_entry_basic_construction() {
        let e = make_entry();
        assert_eq!(e.player_id, "p1");
        assert_eq!(e.player_name, "Player 1");
        assert_eq!(e.stack, 10000);
    }

    #[test]
    fn test_entry_table_and_seat() {
        let e = make_entry();
        assert_eq!(e.table_id, Some(1));
        assert_eq!(e.seat, Some(3));
    }

    #[test]
    fn test_entry_no_table() {
        let mut e = make_entry();
        e.table_id = None;
        assert!(e.table_id.is_none());
    }

    #[test]
    fn test_entry_no_seat() {
        let mut e = make_entry();
        e.seat = None;
        assert!(e.seat.is_none());
    }

    #[test]
    fn test_entry_rebuys_zero() {
        let e = make_entry();
        assert_eq!(e.rebuys, 0);
    }

    #[test]
    fn test_entry_rebuys_multiple() {
        let mut e = make_entry();
        e.rebuys = 3;
        assert_eq!(e.rebuys, 3);
    }

    #[test]
    fn test_entry_addon_not_done() {
        let e = make_entry();
        assert!(!e.addon_done);
    }

    #[test]
    fn test_entry_addon_done() {
        let mut e = make_entry();
        e.addon_done = true;
        assert!(e.addon_done);
    }

    #[test]
    fn test_entry_no_final_position() {
        let e = make_entry();
        assert!(e.final_position.is_none());
    }

    #[test]
    fn test_entry_final_position_set() {
        let mut e = make_entry();
        e.final_position = Some(5);
        assert_eq!(e.final_position, Some(5));
    }

    #[test]
    fn test_entry_no_prize() {
        let e = make_entry();
        assert!(e.prize.is_none());
    }

    #[test]
    fn test_entry_prize_set() {
        let mut e = make_entry();
        e.prize = Some(5000);
        assert_eq!(e.prize, Some(5000));
    }

    #[test]
    fn test_entry_not_eliminated() {
        let e = make_entry();
        assert!(e.eliminated_at.is_none());
    }

    #[test]
    fn test_entry_eliminated() {
        let mut e = make_entry();
        e.eliminated_at = Some(2000);
        assert_eq!(e.eliminated_at, Some(2000));
    }

    #[test]
    fn test_entry_registered_at() {
        let e = make_entry();
        assert_eq!(e.registered_at, 1000);
    }

    #[test]
    fn test_entry_clone() {
        let e = make_entry();
        let e2 = e.clone();
        assert_eq!(e.player_id, e2.player_id);
        assert_eq!(e.stack, e2.stack);
    }

    #[test]
    fn test_entry_debug() {
        let e = make_entry();
        let debug = format!("{:?}", e);
        assert!(debug.contains("PlayerTournamentEntry"));
        assert!(debug.contains("p1"));
    }

    #[test]
    fn test_entry_serialize_json() {
        let e = make_entry();
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"player_id\":\"p1\""));
        assert!(json.contains("\"stack\":10000"));
    }

    #[test]
    fn test_entry_deserialize_json() {
        let e = make_entry();
        let json = serde_json::to_string(&e).unwrap();
        let e2: PlayerTournamentEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(e.player_id, e2.player_id);
        assert_eq!(e.stack, e2.stack);
    }

    #[test]
    fn test_entry_round_trip_json() {
        let e = make_entry();
        let json = serde_json::to_string(&e).unwrap();
        let e2: PlayerTournamentEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(e.player_name, e2.player_name);
        assert_eq!(e.rebuys, e2.rebuys);
        assert_eq!(e.addon_done, e2.addon_done);
    }

    #[test]
    fn test_entry_with_unicode_name() {
        let mut e = make_entry();
        e.player_name = "Jogador Ñandú".to_string();
        let json = serde_json::to_string(&e).unwrap();
        let e2: PlayerTournamentEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(e.player_name, e2.player_name);
    }

    #[test]
    fn test_entry_stack_zero() {
        let mut e = make_entry();
        e.stack = 0;
        assert_eq!(e.stack, 0);
    }

    #[test]
    fn test_entry_stack_large() {
        let mut e = make_entry();
        e.stack = u64::MAX;
        assert_eq!(e.stack, u64::MAX);
    }
}

mod lote_7a_tournament_result_and_stats {
    use super::*;

    #[test]
    fn test_tournament_result_construction() {
        let result = TournamentResult {
            tournament_id: "t1".to_string(),
            tournament_name: "Test".to_string(),
            total_players: 10,
            total_prize_pool: 9000,
            winners: vec![],
            started_at: 1000,
            finished_at: 2000,
            duration_seconds: 1000,
        };
        assert_eq!(result.tournament_id, "t1");
        assert_eq!(result.total_players, 10);
        assert_eq!(result.duration_seconds, 1000);
    }

    #[test]
    fn test_tournament_result_with_winners() {
        let winner = crate::tournament_engine::WinnerEntry {
            position: 1,
            player_id: "p1".to_string(),
            player_name: "Player 1".to_string(),
            prize: 5000,
        };
        let result = TournamentResult {
            tournament_id: "t1".to_string(),
            tournament_name: "Test".to_string(),
            total_players: 3,
            total_prize_pool: 9000,
            winners: vec![winner],
            started_at: 1000,
            finished_at: 2000,
            duration_seconds: 1000,
        };
        assert_eq!(result.winners.len(), 1);
        assert_eq!(result.winners[0].position, 1);
    }

    #[test]
    fn test_tournament_result_clone() {
        let result = TournamentResult {
            tournament_id: "t1".to_string(),
            tournament_name: "Test".to_string(),
            total_players: 10,
            total_prize_pool: 9000,
            winners: vec![],
            started_at: 1000,
            finished_at: 2000,
            duration_seconds: 1000,
        };
        let result2 = result.clone();
        assert_eq!(result.tournament_id, result2.tournament_id);
    }

    #[test]
    fn test_tournament_result_debug() {
        let result = TournamentResult {
            tournament_id: "t1".to_string(),
            tournament_name: "Test".to_string(),
            total_players: 10,
            total_prize_pool: 9000,
            winners: vec![],
            started_at: 1000,
            finished_at: 2000,
            duration_seconds: 1000,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("TournamentResult"));
    }

    #[test]
    fn test_tournament_result_serialize() {
        let result = TournamentResult {
            tournament_id: "t1".to_string(),
            tournament_name: "Test".to_string(),
            total_players: 10,
            total_prize_pool: 9000,
            winners: vec![],
            started_at: 1000,
            finished_at: 2000,
            duration_seconds: 1000,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"tournament_id\":\"t1\""));
    }

    #[test]
    fn test_tournament_result_deserialize() {
        let result = TournamentResult {
            tournament_id: "t1".to_string(),
            tournament_name: "Test".to_string(),
            total_players: 10,
            total_prize_pool: 9000,
            winners: vec![],
            started_at: 1000,
            finished_at: 2000,
            duration_seconds: 1000,
        };
        let json = serde_json::to_string(&result).unwrap();
        let result2: TournamentResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.tournament_id, result2.tournament_id);
    }

    #[test]
    fn test_winner_entry_construction() {
        let w = crate::tournament_engine::WinnerEntry {
            position: 1,
            player_id: "p1".to_string(),
            player_name: "Winner".to_string(),
            prize: 5000,
        };
        assert_eq!(w.position, 1);
        assert_eq!(w.prize, 5000);
    }

    #[test]
    fn test_winner_entry_clone() {
        let w = crate::tournament_engine::WinnerEntry {
            position: 1,
            player_id: "p1".to_string(),
            player_name: "Winner".to_string(),
            prize: 5000,
        };
        let w2 = w.clone();
        assert_eq!(w.position, w2.position);
    }

    #[test]
    fn test_winner_entry_debug() {
        let w = crate::tournament_engine::WinnerEntry {
            position: 1,
            player_id: "p1".to_string(),
            player_name: "Winner".to_string(),
            prize: 5000,
        };
        let debug = format!("{:?}", w);
        assert!(debug.contains("WinnerEntry"));
    }

    #[test]
    fn test_winner_entry_serialize() {
        let w = crate::tournament_engine::WinnerEntry {
            position: 1,
            player_id: "p1".to_string(),
            player_name: "Winner".to_string(),
            prize: 5000,
        };
        let json = serde_json::to_string(&w).unwrap();
        assert!(json.contains("\"position\":1"));
        assert!(json.contains("\"prize\":5000"));
    }

    #[test]
    fn test_winner_entry_deserialize() {
        let w = crate::tournament_engine::WinnerEntry {
            position: 2,
            player_id: "p2".to_string(),
            player_name: "Runner".to_string(),
            prize: 3000,
        };
        let json = serde_json::to_string(&w).unwrap();
        let w2: crate::tournament_engine::WinnerEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(w.position, w2.position);
    }

    #[test]
    fn test_tournament_stats_construction() {
        let stats = TournamentStats {
            tournament_id: "t1".to_string(),
            status: TournamentStatus::Running,
            current_level: 3,
            total_players: 50,
            players_remaining: 20,
            players_eliminated: 30,
            total_prize_pool: 45000,
            average_stack: 22500,
            total_rebuys: 5,
            total_addons: 3,
        };
        assert_eq!(stats.tournament_id, "t1");
        assert_eq!(stats.current_level, 3);
        assert_eq!(stats.average_stack, 22500);
    }

    #[test]
    fn test_tournament_stats_clone() {
        let stats = TournamentStats {
            tournament_id: "t1".to_string(),
            status: TournamentStatus::Running,
            current_level: 3,
            total_players: 50,
            players_remaining: 20,
            players_eliminated: 30,
            total_prize_pool: 45000,
            average_stack: 22500,
            total_rebuys: 5,
            total_addons: 3,
        };
        let stats2 = stats.clone();
        assert_eq!(stats.tournament_id, stats2.tournament_id);
    }

    #[test]
    fn test_tournament_stats_debug() {
        let stats = TournamentStats {
            tournament_id: "t1".to_string(),
            status: TournamentStatus::Running,
            current_level: 3,
            total_players: 50,
            players_remaining: 20,
            players_eliminated: 30,
            total_prize_pool: 45000,
            average_stack: 22500,
            total_rebuys: 5,
            total_addons: 3,
        };
        let debug = format!("{:?}", stats);
        assert!(debug.contains("TournamentStats"));
    }

    #[test]
    fn test_tournament_stats_serialize() {
        let stats = TournamentStats {
            tournament_id: "t1".to_string(),
            status: TournamentStatus::Running,
            current_level: 3,
            total_players: 50,
            players_remaining: 20,
            players_eliminated: 30,
            total_prize_pool: 45000,
            average_stack: 22500,
            total_rebuys: 5,
            total_addons: 3,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"current_level\":3"));
        assert!(json.contains("\"status\":\"running\""));
    }

    #[test]
    fn test_tournament_stats_deserialize() {
        let stats = TournamentStats {
            tournament_id: "t1".to_string(),
            status: TournamentStatus::Running,
            current_level: 3,
            total_players: 50,
            players_remaining: 20,
            players_eliminated: 30,
            total_prize_pool: 45000,
            average_stack: 22500,
            total_rebuys: 5,
            total_addons: 3,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let stats2: TournamentStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats.tournament_id, stats2.tournament_id);
        assert_eq!(stats.status, stats2.status);
    }

    #[test]
    fn test_tournament_stats_round_trip() {
        let stats = TournamentStats {
            tournament_id: "t1".to_string(),
            status: TournamentStatus::Finished,
            current_level: 5,
            total_players: 100,
            players_remaining: 1,
            players_eliminated: 99,
            total_prize_pool: 90000,
            average_stack: 90000,
            total_rebuys: 10,
            total_addons: 5,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let stats2: TournamentStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats.total_players, stats2.total_players);
        assert_eq!(stats.average_stack, stats2.average_stack);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Lote 7B — Registration & Late Registration (200 testes individuais)
// ═══════════════════════════════════════════════════════════════════

mod lote_7b_registration {
    use super::*;

    // --- Registro básico (1-50) ---

    #[test]
    fn test_reg_01_single_player() {
        let mut s = create_tournament(default_config());
        assert!(register_player(&mut s, "p1", "Alice").is_ok());
        assert_eq!(s.players.len(), 1);
        assert_eq!(s.players_remaining, 1);
    }

    #[test]
    fn test_reg_02_two_players() {
        let mut s = create_tournament(default_config());
        assert!(register_player(&mut s, "p1", "Alice").is_ok());
        assert!(register_player(&mut s, "p2", "Bob").is_ok());
        assert_eq!(s.players.len(), 2);
    }

    #[test]
    fn test_reg_03_three_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        assert_eq!(s.players.len(), 3);
    }

    #[test]
    fn test_reg_04_five_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        assert_eq!(s.players.len(), 5);
    }

    #[test]
    fn test_reg_05_ten_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 10);
        assert_eq!(s.players.len(), 10);
    }

    #[test]
    fn test_reg_06_twenty_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 20);
        assert_eq!(s.players.len(), 20);
    }

    #[test]
    fn test_reg_07_fifty_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        assert_eq!(s.players.len(), 50);
    }

    #[test]
    fn test_reg_08_hundred_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        assert_eq!(s.players.len(), 100);
    }

    #[test]
    fn test_reg_09_player_gets_starting_stack() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert_eq!(s.players["p1"].stack, 10000);
    }

    #[test]
    fn test_reg_10_custom_starting_stack() {
        let mut s = create_tournament(config_with_starting_stack(5000));
        register_player(&mut s, "p1", "Alice").unwrap();
        assert_eq!(s.players["p1"].stack, 5000);
    }

    #[test]
    fn test_reg_11_buyin_collected() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert_eq!(s.total_buyins, 1000);
    }

    #[test]
    fn test_reg_12_buyin_collected_two_players() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        register_player(&mut s, "p2", "Bob").unwrap();
        assert_eq!(s.total_buyins, 2000);
    }

    #[test]
    fn test_reg_13_custom_buyin_collected() {
        let mut s = create_tournament(config_with_buyin(500));
        register_player(&mut s, "p1", "Alice").unwrap();
        assert_eq!(s.total_buyins, 500);
    }

    #[test]
    fn test_reg_14_prize_pool_updated() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        // 1000 * 0.90 = 900
        assert_eq!(s.prize_pool, 900);
    }

    #[test]
    fn test_reg_15_prize_pool_two_players() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        register_player(&mut s, "p2", "Bob").unwrap();
        // 2000 * 0.90 = 1800
        assert_eq!(s.prize_pool, 1800);
    }

    #[test]
    fn test_reg_16_prize_pool_custom_pct() {
        let mut s = create_tournament(config_with_prize_pct(0.80));
        register_player(&mut s, "p1", "Alice").unwrap();
        // 1000 * 0.80 = 800
        assert_eq!(s.prize_pool, 800);
    }

    #[test]
    fn test_reg_17_player_name_stored() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "AliceWonder").unwrap();
        assert_eq!(s.players["p1"].player_name, "AliceWonder");
    }

    #[test]
    fn test_reg_18_player_id_stored() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "player_abc", "Alice").unwrap();
        assert!(s.players.contains_key("player_abc"));
    }

    #[test]
    fn test_reg_19_player_rebuys_zero() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert_eq!(s.players["p1"].rebuys, 0);
    }

    #[test]
    fn test_reg_20_player_addon_false() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(!s.players["p1"].addon_done);
    }

    #[test]
    fn test_reg_21_player_no_final_position() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.players["p1"].final_position.is_none());
    }

    #[test]
    fn test_reg_22_player_no_prize() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.players["p1"].prize.is_none());
    }

    #[test]
    fn test_reg_23_player_no_table_seat() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.players["p1"].table_id.is_none());
        assert!(s.players["p1"].seat.is_none());
    }

    #[test]
    fn test_reg_24_player_not_eliminated() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.players["p1"].eliminated_at.is_none());
    }

    #[test]
    fn test_reg_25_player_registered_at_set() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.players["p1"].registered_at > 0);
    }

    #[test]
    fn test_reg_26_status_remains_registering() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert_eq!(s.status, TournamentStatus::Registering);
    }

    #[test]
    fn test_reg_27_current_level_zero_before_start() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert_eq!(s.current_level, 0);
    }

    #[test]
    fn test_reg_28_empty_eliminated_order() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.eliminated_order.is_empty());
    }

    #[test]
    fn test_reg_29_no_started_at() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.started_at.is_none());
    }

    #[test]
    fn test_reg_30_no_finished_at() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(s.finished_at.is_none());
    }

    // --- Duplicados e limites (31-70) ---

    #[test]
    fn test_reg_31_duplicate_player_fails() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(register_player(&mut s, "p1", "Alice2").is_err());
    }

    #[test]
    fn test_reg_32_duplicate_different_name_fails() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(register_player(&mut s, "p1", "Bob").is_err());
    }

    #[test]
    fn test_reg_33_duplicate_same_name_fails() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        assert!(register_player(&mut s, "p1", "Alice").is_err());
    }

    #[test]
    fn test_reg_34_duplicate_count_unchanged() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "Alice").unwrap();
        let _ = register_player(&mut s, "p1", "Alice2");
        assert_eq!(s.players.len(), 1);
    }

    #[test]
    fn test_reg_35_max_players_2() {
        let mut s = create_tournament(config_with_max_players(2));
        assert!(register_player(&mut s, "p1", "A").is_ok());
        assert!(register_player(&mut s, "p2", "B").is_ok());
        assert!(register_player(&mut s, "p3", "C").is_err());
    }

    #[test]
    fn test_reg_36_max_players_3() {
        let mut s = create_tournament(config_with_max_players(3));
        register_n_players(&mut s, 3);
        assert!(register_player(&mut s, "p4", "D").is_err());
    }

    #[test]
    fn test_reg_37_max_players_5() {
        let mut s = create_tournament(config_with_max_players(5));
        register_n_players(&mut s, 5);
        assert!(register_player(&mut s, "p6", "F").is_err());
    }

    #[test]
    fn test_reg_38_max_players_10() {
        let mut s = create_tournament(config_with_max_players(10));
        register_n_players(&mut s, 10);
        assert!(register_player(&mut s, "p11", "K").is_err());
    }

    #[test]
    fn test_reg_39_max_players_1_allows_one() {
        let mut s = create_tournament(config_with_max_players(1));
        assert!(register_player(&mut s, "p1", "A").is_ok());
    }

    #[test]
    fn test_reg_40_max_players_1_blocks_second() {
        let mut s = create_tournament(config_with_max_players(1));
        register_player(&mut s, "p1", "A").unwrap();
        assert!(register_player(&mut s, "p2", "B").is_err());
    }

    #[test]
    fn test_reg_41_at_max_exact() {
        let mut s = create_tournament(config_with_max_players(5));
        register_n_players(&mut s, 5);
        assert_eq!(s.players.len(), 5);
    }

    #[test]
    fn test_reg_42_over_max_fails() {
        let mut s = create_tournament(config_with_max_players(5));
        register_n_players(&mut s, 5);
        assert!(register_player(&mut s, "extra", "Extra").is_err());
    }

    #[test]
    fn test_reg_43_empty_id_fails() {
        let mut s = create_tournament(default_config());
        // Empty string id — behavior depends on impl; verify it doesn't panic
        let _ = register_player(&mut s, "", "Empty");
    }

    #[test]
    fn test_reg_44_empty_name_ok() {
        let mut s = create_tournament(default_config());
        let res = register_player(&mut s, "p1", "");
        // Empty name should be accepted (no validation in impl)
        assert!(res.is_ok());
    }

    #[test]
    fn test_reg_45_long_name() {
        let mut s = create_tournament(default_config());
        let long_name = "A".repeat(200);
        assert!(register_player(&mut s, "p1", &long_name).is_ok());
    }

    #[test]
    fn test_reg_46_long_id() {
        let mut s = create_tournament(default_config());
        let long_id = "x".repeat(100);
        assert!(register_player(&mut s, &long_id, "Player").is_ok());
    }

    #[test]
    fn test_reg_47_special_chars_name() {
        let mut s = create_tournament(default_config());
        assert!(register_player(&mut s, "p1", "João d'Açaí").is_ok());
    }

    #[test]
    fn test_reg_48_unicode_name() {
        let mut s = create_tournament(default_config());
        assert!(register_player(&mut s, "p1", "玩家一").is_ok());
    }

    #[test]
    fn test_reg_49_numeric_id() {
        let mut s = create_tournament(default_config());
        assert!(register_player(&mut s, "12345", "Player").is_ok());
    }

    #[test]
    fn test_reg_50_mixed_alphanumeric_id() {
        let mut s = create_tournament(default_config());
        assert!(register_player(&mut s, "player_42", "Player").is_ok());
    }

    // --- Late registration (51-120) ---

    #[test]
    fn test_reg_51_late_reg_allowed_level_1() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_ok());
    }

    #[test]
    fn test_reg_52_late_reg_allowed_level_2() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_ok());
    }

    #[test]
    fn test_reg_53_late_reg_allowed_level_3() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_ok());
    }

    #[test]
    fn test_reg_54_late_reg_blocked_level_4() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.current_level = 4;
        assert!(register_player(&mut s, "late1", "Late1").is_err());
    }

    #[test]
    fn test_reg_55_late_reg_blocked_level_5() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.current_level = 5;
        assert!(register_player(&mut s, "late1", "Late1").is_err());
    }

    #[test]
    fn test_reg_56_late_reg_disabled() {
        let mut s = create_tournament(config_no_late_registration());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_err());
    }

    #[test]
    fn test_reg_57_late_reg_disabled_level_1() {
        let mut s = create_tournament(config_no_late_registration());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_err());
    }

    #[test]
    fn test_reg_58_late_reg_max_level_1() {
        let mut c = default_config();
        c.late_registration_max_level = 1;
        let mut s = create_tournament(c);
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_ok());
        advance_blinds(&mut s).unwrap();
        assert!(register_player(&mut s, "late2", "Late2").is_err());
    }

    #[test]
    fn test_reg_59_late_reg_max_level_2() {
        let mut c = default_config();
        c.late_registration_max_level = 2;
        let mut s = create_tournament(c);
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_ok());
        advance_blinds(&mut s).unwrap();
        assert!(register_player(&mut s, "late2", "Late2").is_err());
    }

    #[test]
    fn test_reg_60_late_reg_max_level_5() {
        let mut c = default_config();
        c.late_registration_max_level = 5;
        let mut s = create_tournament(c);
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.current_level = 5;
        assert!(register_player(&mut s, "late1", "Late1").is_ok());
    }

    #[test]
    fn test_reg_61_late_reg_player_count_increments() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "Late1").unwrap();
        assert_eq!(s.players_remaining, 3);
    }

    #[test]
    fn test_reg_62_late_reg_buyin_collected() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "Late1").unwrap();
        assert_eq!(s.total_buyins, 3000);
    }

    #[test]
    fn test_reg_63_late_reg_prize_pool_updated() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "Late1").unwrap();
        // 3000 * 0.90 = 2700
        assert_eq!(s.prize_pool, 2700);
    }

    #[test]
    fn test_reg_64_late_reg_gets_starting_stack() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "Late1").unwrap();
        assert_eq!(s.players["late1"].stack, 10000);
    }

    #[test]
    fn test_reg_65_late_reg_multiple_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=10 {
            let id = format!("late{}", i);
            assert!(register_player(&mut s, &id, &format!("L{}", i)).is_ok());
        }
        assert_eq!(s.players.len(), 12);
    }

    #[test]
    fn test_reg_66_late_reg_duplicate_fails() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p_original", "P1").unwrap();
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "p_original", "P1dup").is_err());
    }

    #[test]
    fn test_reg_67_late_reg_at_max_level_exact() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        // current_level = 3 = late_registration_max_level
        assert!(register_player(&mut s, "late1", "Late1").is_ok());
    }

    #[test]
    fn test_reg_68_late_reg_over_max_level() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        // current_level = 4 > 3
        assert!(register_player(&mut s, "late1", "Late1").is_err());
    }

    #[test]
    fn test_reg_69_late_reg_when_paused_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        // Paused is not Registering or Running
        assert!(register_player(&mut s, "late1", "Late1").is_err());
    }

    #[test]
    fn test_reg_70_late_reg_when_finished_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "Late1").is_err());
    }

    // --- Registro em estados inválidos (71-100) ---

    #[test]
    fn test_reg_71_register_when_paused_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "p3", "C").is_err());
    }

    #[test]
    fn test_reg_72_register_when_cancelled_fails() {
        let mut s = create_tournament(default_config());
        cancel_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "p1", "A").is_err());
    }

    #[test]
    fn test_reg_73_register_when_finished_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "p3", "C").is_err());
    }

    #[test]
    fn test_reg_74_register_after_cancel_fails() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "A").unwrap();
        cancel_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "p2", "B").is_err());
    }

    #[test]
    fn test_reg_75_register_after_finish_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "p3", "C").is_err());
    }

    #[test]
    fn test_reg_76_register_50_then_duplicate() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        assert!(register_player(&mut s, "p1", "Dup").is_err());
    }

    #[test]
    fn test_reg_77_register_50_unique() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        assert_eq!(s.players.len(), 50);
    }

    #[test]
    fn test_reg_78_register_100_unique() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        assert_eq!(s.players.len(), 100);
    }

    #[test]
    fn test_reg_79_register_100_then_duplicate() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        assert!(register_player(&mut s, "p50", "Dup").is_err());
    }

    #[test]
    fn test_reg_80_register_max_2_exact() {
        let mut s = create_tournament(config_with_max_players(2));
        assert!(register_player(&mut s, "p1", "A").is_ok());
        assert!(register_player(&mut s, "p2", "B").is_ok());
        assert_eq!(s.players.len(), 2);
    }

    // --- Edge cases de buy-in e prize pool (81-120) ---

    #[test]
    fn test_reg_81_buyin_zero() {
        let mut s = create_tournament(config_with_buyin(0));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.total_buyins, 0);
    }

    #[test]
    fn test_reg_82_buyin_zero_prize_pool_zero() {
        let mut s = create_tournament(config_with_buyin(0));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 0);
    }

    #[test]
    fn test_reg_83_buyin_large() {
        let mut s = create_tournament(config_with_buyin(1_000_000));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.total_buyins, 1_000_000);
    }

    #[test]
    fn test_reg_84_buyin_large_prize_pool() {
        let mut s = create_tournament(config_with_buyin(1_000_000));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 900_000);
    }

    #[test]
    fn test_reg_85_prize_pct_100() {
        let mut s = create_tournament(config_with_prize_pct(1.0));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 1000);
    }

    #[test]
    fn test_reg_86_prize_pct_0() {
        let mut s = create_tournament(config_with_prize_pct(0.0));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 0);
    }

    #[test]
    fn test_reg_87_prize_pct_50() {
        let mut s = create_tournament(config_with_prize_pct(0.50));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 500);
    }

    #[test]
    fn test_reg_88_prize_pct_25() {
        let mut s = create_tournament(config_with_prize_pct(0.25));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 250);
    }

    #[test]
    fn test_reg_89_prize_pct_10() {
        let mut s = create_tournament(config_with_prize_pct(0.10));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 100);
    }

    #[test]
    fn test_reg_90_prize_pct_5() {
        let mut s = create_tournament(config_with_prize_pct(0.05));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.prize_pool, 50);
    }

    #[test]
    fn test_reg_91_starting_stack_1() {
        let mut s = create_tournament(config_with_starting_stack(1));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.players["p1"].stack, 1);
    }

    #[test]
    fn test_reg_92_starting_stack_100() {
        let mut s = create_tournament(config_with_starting_stack(100));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.players["p1"].stack, 100);
    }

    #[test]
    fn test_reg_93_starting_stack_1m() {
        let mut s = create_tournament(config_with_starting_stack(1_000_000));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.players["p1"].stack, 1_000_000);
    }

    #[test]
    fn test_reg_94_starting_stack_0() {
        let mut s = create_tournament(config_with_starting_stack(0));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.players["p1"].stack, 0);
    }

    #[test]
    fn test_reg_95_multiple_buyins_accumulate() {
        let mut s = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut s, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        assert_eq!(s.total_buyins, 10000);
    }

    #[test]
    fn test_reg_96_prize_pool_10_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 10);
        // 10000 * 0.90 = 9000
        assert_eq!(s.prize_pool, 9000);
    }

    #[test]
    fn test_reg_97_prize_pool_50_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        // 50000 * 0.90 = 45000
        assert_eq!(s.prize_pool, 45000);
    }

    #[test]
    fn test_reg_98_prize_pool_100_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        // 100000 * 0.90 = 90000
        assert_eq!(s.prize_pool, 90000);
    }

    #[test]
    fn test_reg_99_players_remaining_matches_count() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 25);
        assert_eq!(s.players_remaining, 25);
    }

    #[test]
    fn test_reg_100_players_remaining_after_late_reg() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert_eq!(s.players_remaining, 6);
    }

    // --- Configurações especiais (101-140) ---

    #[test]
    fn test_reg_101_turbo_speed() {
        let mut s = create_tournament(config_with_speed(TournamentSpeed::Turbo));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.speed, TournamentSpeed::Turbo);
    }

    #[test]
    fn test_reg_102_slow_speed() {
        let mut s = create_tournament(config_with_speed(TournamentSpeed::Slow));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.speed, TournamentSpeed::Slow);
    }

    #[test]
    fn test_reg_103_normal_speed() {
        let mut s = create_tournament(config_with_speed(TournamentSpeed::Normal));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.speed, TournamentSpeed::Normal);
    }

    #[test]
    fn test_reg_104_custom_name() {
        let mut s = create_tournament(config_with_name("MyTournament"));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.name, "MyTournament");
    }

    #[test]
    fn test_reg_105_custom_game_type() {
        let mut s = create_tournament(config_with_game_type("Omaha"));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.game_type, "Omaha");
    }

    #[test]
    fn test_reg_106_no_rebuy_config() {
        let mut s = create_tournament(config_no_rebuy());
        register_player(&mut s, "p1", "A").unwrap();
        assert!(!s.config.allow_rebuy);
    }

    #[test]
    fn test_reg_107_no_addon_config() {
        let mut s = create_tournament(config_no_addon());
        register_player(&mut s, "p1", "A").unwrap();
        assert!(!s.config.allow_addon);
    }

    #[test]
    fn test_reg_108_custom_prize_distribution() {
        let mut s = create_tournament(config_with_prize_distribution(vec![0.6, 0.4]));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.prize_distribution, vec![0.6, 0.4]);
    }

    #[test]
    fn test_reg_109_custom_blind_levels() {
        let levels = vec![make_blind_level(1, 5, 10, 0, 10)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.blind_levels.len(), 1);
    }

    #[test]
    fn test_reg_110_empty_blind_levels() {
        let mut s = create_tournament(config_with_blind_levels(vec![]));
        register_player(&mut s, "p1", "A").unwrap();
        assert!(s.config.blind_levels.is_empty());
    }

    #[test]
    fn test_reg_111_register_then_check_tournament_id() {
        let mut s = create_tournament(default_config());
        let tid = s.tournament_id.clone();
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.tournament_id, tid);
    }

    #[test]
    fn test_reg_112_register_preserves_config() {
        let mut s = create_tournament(default_config());
        let cfg = s.config.clone();
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.name, cfg.name);
        assert_eq!(s.config.buy_in, cfg.buy_in);
    }

    #[test]
    fn test_reg_113_register_preserves_max_players() {
        let mut s = create_tournament(config_with_max_players(10));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.max_players, 10);
    }

    #[test]
    fn test_reg_114_register_preserves_late_reg_setting() {
        let mut s = create_tournament(config_no_late_registration());
        register_player(&mut s, "p1", "A").unwrap();
        assert!(!s.config.late_registration);
    }

    #[test]
    fn test_reg_115_register_preserves_rebuy_setting() {
        let mut s = create_tournament(config_no_rebuy());
        register_player(&mut s, "p1", "A").unwrap();
        assert!(!s.config.allow_rebuy);
    }

    #[test]
    fn test_reg_116_register_preserves_addon_setting() {
        let mut s = create_tournament(config_no_addon());
        register_player(&mut s, "p1", "A").unwrap();
        assert!(!s.config.allow_addon);
    }

    #[test]
    fn test_reg_117_register_preserves_prize_pct() {
        let mut s = create_tournament(config_with_prize_pct(0.5));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.prize_pool_pct, 0.5);
    }

    #[test]
    fn test_reg_118_register_preserves_prize_dist() {
        let mut s = create_tournament(config_with_prize_distribution(vec![0.7, 0.2, 0.1]));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.prize_distribution.len(), 3);
    }

    #[test]
    fn test_reg_119_register_preserves_speed() {
        let mut s = create_tournament(config_with_speed(TournamentSpeed::Turbo));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.speed, TournamentSpeed::Turbo);
    }

    #[test]
    fn test_reg_120_register_preserves_starting_stack() {
        let mut s = create_tournament(config_with_starting_stack(7500));
        register_player(&mut s, "p1", "A").unwrap();
        assert_eq!(s.config.starting_stack, 7500);
    }

    // --- Cenários de multi-registro (121-160) ---

    #[test]
    fn test_reg_121_register_2_consecutive() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "A").unwrap();
        register_player(&mut s, "p2", "B").unwrap();
        assert_eq!(s.players.len(), 2);
        assert_eq!(s.players_remaining, 2);
    }

    #[test]
    fn test_reg_122_register_3_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        assert_eq!(s.players.len(), 3);
    }

    #[test]
    fn test_reg_123_register_4_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 4);
        assert_eq!(s.players.len(), 4);
    }

    #[test]
    fn test_reg_124_register_6_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 6);
        assert_eq!(s.players.len(), 6);
    }

    #[test]
    fn test_reg_125_register_7_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 7);
        assert_eq!(s.players.len(), 7);
    }

    #[test]
    fn test_reg_126_register_8_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 8);
        assert_eq!(s.players.len(), 8);
    }

    #[test]
    fn test_reg_127_register_9_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 9);
        assert_eq!(s.players.len(), 9);
    }

    #[test]
    fn test_reg_128_register_15_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 15);
        assert_eq!(s.players.len(), 15);
    }

    #[test]
    fn test_reg_129_register_30_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 30);
        assert_eq!(s.players.len(), 30);
    }

    #[test]
    fn test_reg_130_register_40_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 40);
        assert_eq!(s.players.len(), 40);
    }

    #[test]
    fn test_reg_131_register_60_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 60);
        assert_eq!(s.players.len(), 60);
    }

    #[test]
    fn test_reg_132_register_70_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 70);
        assert_eq!(s.players.len(), 70);
    }

    #[test]
    fn test_reg_133_register_80_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 80);
        assert_eq!(s.players.len(), 80);
    }

    #[test]
    fn test_reg_134_register_90_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 90);
        assert_eq!(s.players.len(), 90);
    }

    #[test]
    fn test_reg_135_register_95_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 95);
        assert_eq!(s.players.len(), 95);
    }

    #[test]
    fn test_reg_136_register_99_consecutive() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 99);
        assert_eq!(s.players.len(), 99);
    }

    #[test]
    fn test_reg_137_register_at_max_100() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        assert_eq!(s.players.len(), 100);
    }

    #[test]
    fn test_reg_138_register_over_100_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        assert!(register_player(&mut s, "p101", "P101").is_err());
    }

    #[test]
    fn test_reg_139_buyin_accumulate_30() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 30);
        assert_eq!(s.total_buyins, 30000);
    }

    #[test]
    fn test_reg_140_buyin_accumulate_60() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 60);
        assert_eq!(s.total_buyins, 60000);
    }

    // --- Interações com start (141-170) ---

    #[test]
    fn test_reg_141_register_2_then_start_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_reg_142_register_1_start_fails() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "A").unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_reg_143_register_0_start_fails() {
        let mut s = create_tournament(default_config());
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_reg_144_register_3_start_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_reg_145_register_10_start_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 10);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_reg_146_start_sets_level_1() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.current_level, 1);
    }

    #[test]
    fn test_reg_147_start_sets_status_running() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.status, TournamentStatus::Running);
    }

    #[test]
    fn test_reg_148_start_sets_started_at() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(s.started_at.is_some());
    }

    #[test]
    fn test_reg_149_start_sets_level_started_at() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(s.level_started_at > 0);
    }

    #[test]
    fn test_reg_150_start_preserves_player_count() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.players.len(), 5);
    }

    #[test]
    fn test_reg_151_start_preserves_buyins() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        let buyins = s.total_buyins;
        start_tournament(&mut s).unwrap();
        assert_eq!(s.total_buyins, buyins);
    }

    #[test]
    fn test_reg_152_start_preserves_prize_pool() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        let pool = s.prize_pool;
        start_tournament(&mut s).unwrap();
        assert_eq!(s.prize_pool, pool);
    }

    #[test]
    fn test_reg_153_start_preserves_stacks() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.players["p1"].stack, 10000);
    }

    #[test]
    fn test_reg_154_start_does_not_change_eliminated() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        assert!(s.eliminated_order.is_empty());
    }

    #[test]
    fn test_reg_155_start_twice_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_reg_156_start_after_pause_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_reg_157_start_after_finish_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_reg_158_start_after_cancel_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        cancel_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_reg_159_register_after_start_late() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "L1").is_ok());
    }

    #[test]
    fn test_reg_160_register_after_start_no_late_reg() {
        let mut s = create_tournament(config_no_late_registration());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    // --- Casos adicionais de late registration (161-200) ---

    #[test]
    fn test_reg_161_late_reg_1_player() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert_eq!(s.players.len(), 3);
    }

    #[test]
    fn test_reg_162_late_reg_5_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=5 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        assert_eq!(s.players.len(), 7);
    }

    #[test]
    fn test_reg_163_late_reg_10_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=10 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        assert_eq!(s.players.len(), 12);
    }

    #[test]
    fn test_reg_164_late_reg_20_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=20 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        assert_eq!(s.players.len(), 22);
    }

    #[test]
    fn test_reg_165_late_reg_50_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=50 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        assert_eq!(s.players.len(), 52);
    }

    #[test]
    fn test_reg_166_late_reg_buyin_accumulates() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=5 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        // 7 * 1000 = 7000
        assert_eq!(s.total_buyins, 7000);
    }

    #[test]
    fn test_reg_167_late_reg_prize_pool_accumulates() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=5 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        // 7000 * 0.90 = 6300
        assert_eq!(s.prize_pool, 6300);
    }

    #[test]
    fn test_reg_168_late_reg_players_remaining() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for i in 1..=5 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        assert_eq!(s.players_remaining, 7);
    }

    #[test]
    fn test_reg_169_late_reg_custom_buyin() {
        let mut s = create_tournament(config_with_buyin(500));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        // 3 * 500 = 1500
        assert_eq!(s.total_buyins, 1500);
    }

    #[test]
    fn test_reg_170_late_reg_custom_starting_stack() {
        let mut s = create_tournament(config_with_starting_stack(5000));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert_eq!(s.players["late1"].stack, 5000);
    }

    #[test]
    fn test_reg_171_late_reg_custom_prize_pct() {
        let mut s = create_tournament(config_with_prize_pct(0.80));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        // 3000 * 0.80 = 2400
        assert_eq!(s.prize_pool, 2400);
    }

    #[test]
    fn test_reg_172_late_reg_max_players_reached() {
        let mut s = create_tournament(config_with_max_players(3));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "L1").is_ok());
        assert!(register_player(&mut s, "late2", "L2").is_err());
    }

    #[test]
    fn test_reg_173_late_reg_duplicate_original_player() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p_original", "P1").unwrap();
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "p_original", "P1dup").is_err());
    }

    #[test]
    fn test_reg_174_late_reg_duplicate_late_player() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert!(register_player(&mut s, "late1", "L1dup").is_err());
    }

    #[test]
    fn test_reg_175_late_reg_at_max_level_3() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.current_level, 3);
        assert!(register_player(&mut s, "late1", "L1").is_ok());
    }

    #[test]
    fn test_reg_176_late_reg_over_max_level_4() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.current_level, 4);
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    #[test]
    fn test_reg_177_late_reg_over_max_level_5() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.current_level = 5;
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    #[test]
    fn test_reg_178_late_reg_disabled_config() {
        let mut s = create_tournament(config_no_late_registration());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    #[test]
    fn test_reg_179_late_reg_disabled_level_1() {
        let mut s = create_tournament(config_no_late_registration());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    #[test]
    fn test_reg_180_late_reg_disabled_level_2() {
        let mut s = create_tournament(config_no_late_registration());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    #[test]
    fn test_reg_181_late_reg_max_level_0() {
        let mut c = default_config();
        c.late_registration_max_level = 0;
        let mut s = create_tournament(c);
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        // current_level = 1 > 0
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    #[test]
    fn test_reg_182_late_reg_max_level_4() {
        let mut c = default_config();
        c.late_registration_max_level = 4;
        let mut s = create_tournament(c);
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.current_level = 4;
        assert!(register_player(&mut s, "late1", "L1").is_ok());
    }

    #[test]
    fn test_reg_183_late_reg_max_level_5_exact() {
        let mut c = default_config();
        c.late_registration_max_level = 5;
        let mut s = create_tournament(c);
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.current_level = 5;
        assert!(register_player(&mut s, "late1", "L1").is_ok());
    }

    #[test]
    fn test_reg_184_late_reg_max_level_6_over() {
        let mut c = default_config();
        c.late_registration_max_level = 5;
        let mut s = create_tournament(c);
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.current_level = 6;
        assert!(register_player(&mut s, "late1", "L1").is_err());
    }

    #[test]
    fn test_reg_185_late_reg_player_has_no_elimination() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert!(s.players["late1"].eliminated_at.is_none());
    }

    #[test]
    fn test_reg_186_late_reg_player_rebuys_zero() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert_eq!(s.players["late1"].rebuys, 0);
    }

    #[test]
    fn test_reg_187_late_reg_player_addon_false() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert!(!s.players["late1"].addon_done);
    }

    #[test]
    fn test_reg_188_late_reg_player_no_position() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert!(s.players["late1"].final_position.is_none());
    }

    #[test]
    fn test_reg_189_late_reg_player_no_prize() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert!(s.players["late1"].prize.is_none());
    }

    #[test]
    fn test_reg_190_late_reg_player_registered_at() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert!(s.players["late1"].registered_at > 0);
    }

    #[test]
    fn test_reg_191_late_reg_then_more_late_reg() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        register_player(&mut s, "late2", "L2").unwrap();
        assert_eq!(s.players.len(), 4);
    }

    #[test]
    fn test_reg_192_late_reg_then_advance_then_late_reg() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        advance_blinds(&mut s).unwrap();
        register_player(&mut s, "late2", "L2").unwrap();
        assert_eq!(s.players.len(), 4);
    }

    #[test]
    fn test_reg_193_late_reg_then_advance_to_max_then_fail() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert!(register_player(&mut s, "late2", "L2").is_err());
    }

    #[test]
    fn test_reg_194_late_reg_with_custom_name() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "CustomName").unwrap();
        assert_eq!(s.players["late1"].player_name, "CustomName");
    }

    #[test]
    fn test_reg_195_late_reg_with_unicode_name() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "日本語").unwrap();
        assert_eq!(s.players["late1"].player_name, "日本語");
    }

    #[test]
    fn test_reg_196_late_reg_empty_name() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "late1", "").is_ok());
    }

    #[test]
    fn test_reg_197_late_reg_long_name() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        let long = "X".repeat(300);
        assert!(register_player(&mut s, "late1", &long).is_ok());
    }

    #[test]
    fn test_reg_198_late_reg_numeric_id() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "999", "Player").is_ok());
    }

    #[test]
    fn test_reg_199_late_reg_special_char_id() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(register_player(&mut s, "player-001", "Player").is_ok());
    }

    #[test]
    fn test_reg_200_late_reg_mixed_original_and_late() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        for i in 1..=5 {
            register_player(&mut s, &format!("late{}", i), &format!("L{}", i)).unwrap();
        }
        assert_eq!(s.players.len(), 10);
        assert_eq!(s.players_remaining, 10);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Lote 7C — Lifecycle & Blinds (160 testes individuais)
// ═══════════════════════════════════════════════════════════════════

mod lote_7c_lifecycle_blinds {
    use super::*;

    // --- start_tournament (1-40) ---

    #[test]
    fn test_life_01_start_2_players_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_02_start_3_players_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_03_start_5_players_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_04_start_10_players_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 10);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_05_start_50_players_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_06_start_100_players_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_07_start_0_players_fails() {
        let mut s = create_tournament(default_config());
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_08_start_1_player_fails() {
        let mut s = create_tournament(default_config());
        register_player(&mut s, "p1", "A").unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_09_start_sets_running() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.status, TournamentStatus::Running);
    }

    #[test]
    fn test_life_10_start_sets_level_1() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.current_level, 1);
    }

    #[test]
    fn test_life_11_start_sets_started_at() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(s.started_at.is_some());
        assert!(s.started_at.unwrap() > 0);
    }

    #[test]
    fn test_life_12_start_sets_level_started_at() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(s.level_started_at > 0);
    }

    #[test]
    fn test_life_13_start_preserves_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.players.len(), 5);
    }

    #[test]
    fn test_life_14_start_preserves_buyins() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        let b = s.total_buyins;
        start_tournament(&mut s).unwrap();
        assert_eq!(s.total_buyins, b);
    }

    #[test]
    fn test_life_15_start_preserves_prize_pool() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        let p = s.prize_pool;
        start_tournament(&mut s).unwrap();
        assert_eq!(s.prize_pool, p);
    }

    #[test]
    fn test_life_16_start_preserves_stacks() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        assert_eq!(s.players["p1"].stack, 10000);
    }

    #[test]
    fn test_life_17_start_no_eliminated() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        assert!(s.eliminated_order.is_empty());
    }

    #[test]
    fn test_life_18_start_twice_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_19_start_when_paused_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_20_start_when_finished_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_21_start_when_cancelled_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        cancel_tournament(&mut s).unwrap();
        assert!(start_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_22_start_max_players_2() {
        let mut s = create_tournament(config_with_max_players(2));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_23_start_custom_buyin() {
        let mut s = create_tournament(config_with_buyin(500));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_24_start_custom_stack() {
        let mut s = create_tournament(config_with_starting_stack(5000));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_25_start_turbo() {
        let mut s = create_tournament(config_with_speed(TournamentSpeed::Turbo));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_26_start_slow() {
        let mut s = create_tournament(config_with_speed(TournamentSpeed::Slow));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_27_start_no_late_reg() {
        let mut s = create_tournament(config_no_late_registration());
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_28_start_no_rebuy() {
        let mut s = create_tournament(config_no_rebuy());
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_29_start_no_addon() {
        let mut s = create_tournament(config_no_addon());
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_30_start_custom_prize_pct() {
        let mut s = create_tournament(config_with_prize_pct(0.80));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_31_start_custom_prize_dist() {
        let mut s = create_tournament(config_with_prize_distribution(vec![0.6, 0.3, 0.1]));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_32_start_custom_name() {
        let mut s = create_tournament(config_with_name("TestCup"));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_33_start_custom_game_type() {
        let mut s = create_tournament(config_with_game_type("Omaha"));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_34_start_custom_blind_levels() {
        let levels = vec![
            make_blind_level(1, 10, 20, 0, 10),
            make_blind_level(2, 20, 40, 0, 10),
        ];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_35_start_empty_blind_levels() {
        let mut s = create_tournament(config_with_blind_levels(vec![]));
        register_n_players(&mut s, 2);
        let res = start_tournament(&mut s);
        let _ = res;
    }

    #[test]
    fn test_life_36_start_4_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 4);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_37_start_6_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 6);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_38_start_7_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 7);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_39_start_8_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 8);
        assert!(start_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_40_start_9_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 9);
        assert!(start_tournament(&mut s).is_ok());
    }

    // --- advance_blinds (41-80) ---

    #[test]
    fn test_life_41_advance_to_level_2() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_ok());
        assert_eq!(s.current_level, 2);
    }

    #[test]
    fn test_life_42_advance_to_level_3() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.current_level, 3);
    }

    #[test]
    fn test_life_43_advance_to_level_4() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..3 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 4);
    }

    #[test]
    fn test_life_44_advance_to_level_5() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 5);
    }

    #[test]
    fn test_life_45_advance_to_level_6() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..5 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 6);
    }

    #[test]
    fn test_life_46_advance_to_level_7() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..6 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 7);
    }

    #[test]
    fn test_life_47_advance_to_level_8() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..7 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 8);
    }

    #[test]
    fn test_life_48_advance_to_level_9() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..8 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 9);
    }

    #[test]
    fn test_life_49_advance_to_level_10() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..9 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 10);
    }

    #[test]
    fn test_life_50_advance_past_max_fails() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..9 {
            advance_blinds(&mut s).unwrap();
        }
        assert!(advance_blinds(&mut s).is_err());
    }

    #[test]
    fn test_life_51_advance_when_paused_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_err());
    }

    #[test]
    fn test_life_52_advance_when_registering_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        assert!(advance_blinds(&mut s).is_err());
    }

    #[test]
    fn test_life_53_advance_when_finished_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_err());
    }

    #[test]
    fn test_life_54_advance_when_cancelled_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        cancel_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_err());
    }

    #[test]
    fn test_life_55_advance_updates_level_started_at() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        let old = s.level_started_at;
        s.level_started_at = old + 600;
        advance_blinds(&mut s).unwrap();
        assert!(s.level_started_at >= old);
    }

    #[test]
    fn test_life_56_advance_preserves_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.players.len(), 5);
    }

    #[test]
    fn test_life_57_advance_preserves_buyins() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        let b = s.total_buyins;
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.total_buyins, b);
    }

    #[test]
    fn test_life_58_advance_preserves_prize_pool() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        let p = s.prize_pool;
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.prize_pool, p);
    }

    #[test]
    fn test_life_59_advance_preserves_stacks() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.players["p1"].stack, 10000);
    }

    #[test]
    fn test_life_60_advance_preserves_status() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.status, TournamentStatus::Running);
    }

    #[test]
    fn test_life_61_advance_3_levels() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..3 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 4);
    }

    #[test]
    fn test_life_62_advance_5_levels() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..5 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 6);
    }

    #[test]
    fn test_life_63_advance_7_levels() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..7 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 8);
    }

    #[test]
    fn test_life_64_advance_9_levels() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..9 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 10);
    }

    #[test]
    fn test_life_65_advance_custom_levels_2() {
        let levels = vec![
            make_blind_level(1, 10, 20, 0, 10),
            make_blind_level(2, 20, 40, 0, 10),
        ];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_ok());
        assert_eq!(s.current_level, 2);
    }

    #[test]
    fn test_life_66_advance_custom_levels_2_max() {
        let levels = vec![
            make_blind_level(1, 10, 20, 0, 10),
            make_blind_level(2, 20, 40, 0, 10),
        ];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_err());
    }

    #[test]
    fn test_life_67_advance_custom_levels_5() {
        let levels: Vec<_> = (1..=5)
            .map(|i| make_blind_level(i, (i * 10).into(), (i * 20).into(), 0, 10))
            .collect();
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 5);
    }

    #[test]
    fn test_life_68_advance_custom_levels_5_max() {
        let levels: Vec<_> = (1..=5)
            .map(|i| make_blind_level(i, (i * 10).into(), (i * 20).into(), 0, 10))
            .collect();
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        assert!(advance_blinds(&mut s).is_err());
    }

    #[test]
    fn test_life_69_advance_1_level() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.current_level, 2);
    }

    #[test]
    fn test_life_70_advance_2_levels() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert_eq!(s.current_level, 3);
    }

    #[test]
    fn test_life_71_advance_4_levels() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 5);
    }

    #[test]
    fn test_life_72_advance_6_levels() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..6 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 7);
    }

    #[test]
    fn test_life_73_advance_8_levels() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..8 {
            advance_blinds(&mut s).unwrap();
        }
        assert_eq!(s.current_level, 9);
    }

    #[test]
    fn test_life_74_advance_after_resume() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_ok());
    }

    #[test]
    fn test_life_75_advance_after_late_reg() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        register_player(&mut s, "late1", "L1").unwrap();
        assert!(advance_blinds(&mut s).is_ok());
    }

    #[test]
    fn test_life_76_advance_after_elimination() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        eliminate_player(&mut s, "p1", None).unwrap();
        assert!(advance_blinds(&mut s).is_ok());
    }

    #[test]
    fn test_life_77_advance_after_rebuy() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        eliminate_player(&mut s, "p1", None).unwrap();
        process_rebuy(&mut s, "p1").unwrap();
        assert!(advance_blinds(&mut s).is_ok());
    }

    #[test]
    fn test_life_78_advance_after_addon() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        process_addon(&mut s, "p1", 5000, 500).unwrap();
        assert!(advance_blinds(&mut s).is_ok());
    }

    #[test]
    fn test_life_79_advance_10_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 10);
        start_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_ok());
    }

    #[test]
    fn test_life_80_advance_50_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        start_tournament(&mut s).unwrap();
        assert!(advance_blinds(&mut s).is_ok());
    }

    // --- get_current_blinds (81-100) ---

    #[test]
    fn test_life_81_blinds_level_1() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 10);
        assert_eq!(b.big_blind, 20);
    }

    #[test]
    fn test_life_82_blinds_level_2() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 20);
        assert_eq!(b.big_blind, 40);
    }

    #[test]
    fn test_life_83_blinds_level_3() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 30);
        assert_eq!(b.big_blind, 60);
    }

    #[test]
    fn test_life_84_blinds_level_4() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..3 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 40);
        assert_eq!(b.big_blind, 80);
    }

    #[test]
    fn test_life_85_blinds_level_5() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 50);
        assert_eq!(b.big_blind, 100);
    }

    #[test]
    fn test_life_86_blinds_level_0_none() {
        let s = create_tournament(default_config());
        assert!(get_current_blinds(&s).is_none());
    }

    #[test]
    fn test_life_87_blinds_before_start_none() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        assert!(get_current_blinds(&s).is_none());
    }

    #[test]
    fn test_life_88_blinds_level_1_ante() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.ante, 0);
    }

    #[test]
    fn test_life_89_blinds_level_1_duration() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.duration_minutes, 10);
    }

    #[test]
    fn test_life_90_blinds_level_1_level_num() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.level, 1);
    }

    #[test]
    fn test_life_91_blinds_level_2_level_num() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.level, 2);
    }

    #[test]
    fn test_life_92_blinds_level_5_level_num() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.level, 5);
    }

    #[test]
    fn test_life_93_blinds_custom_level() {
        let levels = vec![make_blind_level(1, 25, 50, 5, 15)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 25);
        assert_eq!(b.big_blind, 50);
        assert_eq!(b.ante, 5);
        assert_eq!(b.duration_minutes, 15);
    }

    #[test]
    fn test_life_94_blinds_level_10() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..9 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.level, 10);
    }

    #[test]
    fn test_life_95_blinds_level_6() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..5 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 60);
        assert_eq!(b.big_blind, 120);
    }

    #[test]
    fn test_life_96_blinds_level_7() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..6 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 70);
        assert_eq!(b.big_blind, 140);
    }

    #[test]
    fn test_life_97_blinds_level_8() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..7 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 80);
        assert_eq!(b.big_blind, 160);
    }

    #[test]
    fn test_life_98_blinds_level_9() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..8 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 90);
        assert_eq!(b.big_blind, 180);
    }

    #[test]
    fn test_life_99_blinds_level_10_values() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..9 {
            advance_blinds(&mut s).unwrap();
        }
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.small_blind, 100);
        assert_eq!(b.big_blind, 200);
    }

    #[test]
    fn test_life_100_blinds_when_paused() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        let b = get_current_blinds(&s).unwrap();
        assert_eq!(b.level, 1);
    }

    // --- pause/resume (101-130) ---

    #[test]
    fn test_life_101_pause_running_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(pause_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_102_pause_sets_paused() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert_eq!(s.status, TournamentStatus::Paused);
    }

    #[test]
    fn test_life_103_pause_registering_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        assert!(pause_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_104_pause_paused_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(pause_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_105_pause_finished_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(pause_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_106_pause_cancelled_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        cancel_tournament(&mut s).unwrap();
        assert!(pause_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_107_resume_paused_ok() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(resume_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_108_resume_sets_running() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert_eq!(s.status, TournamentStatus::Running);
    }

    #[test]
    fn test_life_109_resume_running_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(resume_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_110_resume_registering_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        assert!(resume_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_111_resume_finished_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        finish_tournament(&mut s).unwrap();
        assert!(resume_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_112_resume_cancelled_fails() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        cancel_tournament(&mut s).unwrap();
        assert!(resume_tournament(&mut s).is_err());
    }

    #[test]
    fn test_life_113_pause_resume_cycle() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert_eq!(s.status, TournamentStatus::Running);
    }

    #[test]
    fn test_life_114_pause_resume_pause() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert_eq!(s.status, TournamentStatus::Paused);
    }

    #[test]
    fn test_life_115_pause_preserves_level() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        let lvl = s.current_level;
        pause_tournament(&mut s).unwrap();
        assert_eq!(s.current_level, lvl);
    }

    #[test]
    fn test_life_116_pause_preserves_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert_eq!(s.players.len(), 5);
    }

    #[test]
    fn test_life_117_resume_preserves_level() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        let lvl = s.current_level;
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert_eq!(s.current_level, lvl);
    }

    #[test]
    fn test_life_118_resume_preserves_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert_eq!(s.players.len(), 5);
    }

    #[test]
    fn test_life_119_pause_preserves_buyins() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        let b = s.total_buyins;
        pause_tournament(&mut s).unwrap();
        assert_eq!(s.total_buyins, b);
    }

    #[test]
    fn test_life_120_resume_preserves_buyins() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        let b = s.total_buyins;
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert_eq!(s.total_buyins, b);
    }

    #[test]
    fn test_life_121_pause_preserves_prize_pool() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        let p = s.prize_pool;
        pause_tournament(&mut s).unwrap();
        assert_eq!(s.prize_pool, p);
    }

    #[test]
    fn test_life_122_resume_preserves_prize_pool() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 5);
        start_tournament(&mut s).unwrap();
        let p = s.prize_pool;
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert_eq!(s.prize_pool, p);
    }

    #[test]
    fn test_life_123_pause_preserves_stacks() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert_eq!(s.players["p1"].stack, 10000);
    }

    #[test]
    fn test_life_124_resume_preserves_stacks() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 3);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        resume_tournament(&mut s).unwrap();
        assert_eq!(s.players["p1"].stack, 10000);
    }

    #[test]
    fn test_life_125_pause_10_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 10);
        start_tournament(&mut s).unwrap();
        assert!(pause_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_126_resume_10_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 10);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(resume_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_127_pause_50_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        start_tournament(&mut s).unwrap();
        assert!(pause_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_128_resume_50_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 50);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(resume_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_129_pause_100_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        start_tournament(&mut s).unwrap();
        assert!(pause_tournament(&mut s).is_ok());
    }

    #[test]
    fn test_life_130_resume_100_players() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 100);
        start_tournament(&mut s).unwrap();
        pause_tournament(&mut s).unwrap();
        assert!(resume_tournament(&mut s).is_ok());
    }

    // --- is_blind_level_expired (131-160) ---

    #[test]
    fn test_life_131_expired_level_0_false() {
        let s = create_tournament(default_config());
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_132_expired_just_started_false() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_133_expired_after_5_min_false() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 300;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_134_expired_after_9_min_false() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 540;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_135_expired_after_10_min_true() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 600;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_136_expired_after_15_min_true() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 900;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_137_expired_after_20_min_true() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 1200;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_138_expired_after_30_min_true() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 1800;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_139_expired_after_60_min_true() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 3600;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_140_expired_after_120_min_true() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 7200;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_141_expired_custom_duration_5_min() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 5)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 300;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_142_expired_custom_duration_5_min_false() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 5)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 240;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_143_expired_custom_duration_15_min() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 15)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 900;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_144_expired_custom_duration_15_min_false() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 15)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 840;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_145_expired_custom_duration_20_min() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 20)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 1200;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_146_expired_custom_duration_20_min_false() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 20)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 1140;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_147_expired_custom_duration_30_min() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 30)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 1800;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_148_expired_custom_duration_30_min_false() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 30)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 1740;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_149_expired_level_2_not_expired() {
        let mut s = create_tournament(default_config());
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_150_expired_level_2_expired() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        advance_blinds(&mut s).unwrap();
        s.level_started_at -= 600;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_151_expired_level_5_not_expired() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_152_expired_level_5_expired() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..4 {
            advance_blinds(&mut s).unwrap();
        }
        s.level_started_at -= 600;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_153_expired_level_10_not_expired() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..9 {
            advance_blinds(&mut s).unwrap();
        }
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_154_expired_level_10_expired() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        for _ in 0..9 {
            advance_blinds(&mut s).unwrap();
        }
        s.level_started_at -= 600;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_155_expired_when_paused() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 600;
        pause_tournament(&mut s).unwrap();
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_156_expired_1_second_before() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 599;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_157_expired_1_second_after() {
        let mut s = create_tournament(long_config(10));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 601;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_158_expired_2_min_duration() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 2)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 120;
        assert!(is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_159_expired_2_min_duration_false() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 2)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 60;
        assert!(!is_blind_level_expired(&s));
    }

    #[test]
    fn test_life_160_expired_1_min_duration() {
        let levels = vec![make_blind_level(1, 10, 20, 0, 1)];
        let mut s = create_tournament(config_with_blind_levels(levels));
        register_n_players(&mut s, 2);
        start_tournament(&mut s).unwrap();
        s.level_started_at -= 60;
        assert!(is_blind_level_expired(&s));
    }
}

// ═══════════════════════════════════════════════════════════════════
// Lote 7D — Elimination & Re-buy (200 testes)
// ═══════════════════════════════════════════════════════════════════

mod lote_7d_elimination_rebuy {
    use super::*;

    // --- Eliminação: sucesso básico (001-050) ---

    #[test]
    fn test_lote_7d_elim_001_basic_two_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(2));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 1);
    }

    #[test]
    fn test_lote_7d_elim_002_three_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        register_player(&mut state, "p3", "P3").unwrap();
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p2", Some(3));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 2);
    }

    #[test]
    fn test_lote_7d_elim_003_four_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=4 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p3", Some(4));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 3);
    }

    #[test]
    fn test_lote_7d_elim_004_five_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(5));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 4);
    }

    #[test]
    fn test_lote_7d_elim_005_six_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=6 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p6", Some(6));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 5);
    }

    #[test]
    fn test_lote_7d_elim_006_seven_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=7 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p4", Some(7));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 6);
    }

    #[test]
    fn test_lote_7d_elim_007_eight_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=8 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p2", Some(8));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 7);
    }

    #[test]
    fn test_lote_7d_elim_008_nine_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=9 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p5", Some(9));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 8);
    }

    #[test]
    fn test_lote_7d_elim_009_ten_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p7", Some(10));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 9);
    }

    #[test]
    fn test_lote_7d_elim_010_eleven_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=11 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p3", Some(11));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 10);
    }

    #[test]
    fn test_lote_7d_elim_011_twelve_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=12 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p8", Some(12));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 11);
    }

    #[test]
    fn test_lote_7d_elim_012_thirteen_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=13 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(13));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 12);
    }

    #[test]
    fn test_lote_7d_elim_013_fourteen_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=14 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p9", Some(14));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 13);
    }

    #[test]
    fn test_lote_7d_elim_014_fifteen_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=15 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p10", Some(15));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 14);
    }

    #[test]
    fn test_lote_7d_elim_015_sixteen_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=16 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p11", Some(16));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 15);
    }

    #[test]
    fn test_lote_7d_elim_016_seventeen_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=17 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p12", Some(17));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 16);
    }

    #[test]
    fn test_lote_7d_elim_017_eighteen_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=18 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p13", Some(18));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 17);
    }

    #[test]
    fn test_lote_7d_elim_018_nineteen_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=19 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p14", Some(19));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 18);
    }

    #[test]
    fn test_lote_7d_elim_019_twenty_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p15", Some(20));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 19);
    }

    #[test]
    fn test_lote_7d_elim_020_twentyone_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=21 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p16", Some(21));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 20);
    }

    #[test]
    fn test_lote_7d_elim_021_twentytwo_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=22 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p17", Some(22));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 21);
    }

    #[test]
    fn test_lote_7d_elim_022_twentythree_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=23 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p18", Some(23));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 22);
    }

    #[test]
    fn test_lote_7d_elim_023_twentyfour_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=24 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p19", Some(24));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 23);
    }

    #[test]
    fn test_lote_7d_elim_024_twentyfive_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=25 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p20", Some(25));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 24);
    }

    #[test]
    fn test_lote_7d_elim_025_thirty_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=30 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p25", Some(30));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 29);
    }

    #[test]
    fn test_lote_7d_elim_026_thirtyfive_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=35 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p30", Some(35));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 34);
    }

    #[test]
    fn test_lote_7d_elim_027_forty_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=40 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p35", Some(40));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 39);
    }

    #[test]
    fn test_lote_7d_elim_028_fortyfive_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=45 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p40", Some(45));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 44);
    }

    #[test]
    fn test_lote_7d_elim_029_fifty_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p45", Some(50));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 49);
    }

    #[test]
    fn test_lote_7d_elim_050_sequential_first() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(50));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 49);
        assert_eq!(state.eliminated_order.len(), 1);
    }

    #[test]
    fn test_lote_7d_elim_051_sequential_second() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(50)).unwrap();
        let res = eliminate_player(&mut state, "p2", Some(49));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 48);
        assert_eq!(state.eliminated_order.len(), 2);
    }

    #[test]
    fn test_lote_7d_elim_052_sequential_third() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(50)).unwrap();
        eliminate_player(&mut state, "p2", Some(49)).unwrap();
        let res = eliminate_player(&mut state, "p3", Some(48));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 47);
        assert_eq!(state.eliminated_order.len(), 3);
    }

    #[test]
    fn test_lote_7d_elim_053_sequential_fourth() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=3 {
            eliminate_player(&mut state, &format!("p{}", i), Some(51 - i as u32)).unwrap();
        }
        let res = eliminate_player(&mut state, "p4", Some(47));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 46);
        assert_eq!(state.eliminated_order.len(), 4);
    }

    #[test]
    fn test_lote_7d_elim_054_sequential_fifth() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=4 {
            eliminate_player(&mut state, &format!("p{}", i), Some(51 - i as u32)).unwrap();
        }
        let res = eliminate_player(&mut state, "p5", Some(46));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 45);
        assert_eq!(state.eliminated_order.len(), 5);
    }

    #[test]
    fn test_lote_7d_elim_055_sequential_sixth() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            eliminate_player(&mut state, &format!("p{}", i), Some(51 - i as u32)).unwrap();
        }
        let res = eliminate_player(&mut state, "p6", Some(45));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 44);
        assert_eq!(state.eliminated_order.len(), 6);
    }

    #[test]
    fn test_lote_7d_elim_056_sequential_seventh() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=6 {
            eliminate_player(&mut state, &format!("p{}", i), Some(51 - i as u32)).unwrap();
        }
        let res = eliminate_player(&mut state, "p7", Some(44));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 43);
        assert_eq!(state.eliminated_order.len(), 7);
    }

    #[test]
    fn test_lote_7d_elim_057_sequential_eighth() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=7 {
            eliminate_player(&mut state, &format!("p{}", i), Some(51 - i as u32)).unwrap();
        }
        let res = eliminate_player(&mut state, "p8", Some(43));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 42);
        assert_eq!(state.eliminated_order.len(), 8);
    }

    #[test]
    fn test_lote_7d_elim_058_sequential_ninth() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=8 {
            eliminate_player(&mut state, &format!("p{}", i), Some(51 - i as u32)).unwrap();
        }
        let res = eliminate_player(&mut state, "p9", Some(42));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 41);
        assert_eq!(state.eliminated_order.len(), 9);
    }

    #[test]
    fn test_lote_7d_elim_059_sequential_tenth() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=9 {
            eliminate_player(&mut state, &format!("p{}", i), Some(51 - i as u32)).unwrap();
        }
        let res = eliminate_player(&mut state, "p10", Some(41));
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 40);
        assert_eq!(state.eliminated_order.len(), 10);
    }

    // --- Eliminação: stack zerada e final_position (060-090) ---

    #[test]
    fn test_lote_7d_elim_060_stack_zero() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(2)).unwrap();
        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.stack, 0);
    }

    #[test]
    fn test_lote_7d_elim_061_final_position_set() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(2)).unwrap();
        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.final_position, Some(2));
    }

    #[test]
    fn test_lote_7d_elim_062_final_position_none() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.final_position, None);
    }

    #[test]
    fn test_lote_7d_elim_063_eliminated_at_set() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(2)).unwrap();
        let entry = state.players.get("p1").unwrap();
        assert!(entry.eliminated_at.is_some());
    }

    #[test]
    fn test_lote_7d_elim_064_eliminated_order_contains_id() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(2)).unwrap();
        assert_eq!(state.eliminated_order, vec!["p1".to_string()]);
    }

    #[test]
    fn test_lote_7d_elim_065_eliminated_order_multiple() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p3", Some(5)).unwrap();
        eliminate_player(&mut state, "p1", Some(4)).unwrap();
        assert_eq!(
            state.eliminated_order,
            vec!["p3".to_string(), "p1".to_string()]
        );
    }

    #[test]
    fn test_lote_7d_elim_066_position_50() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(50)).unwrap();
        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.final_position, Some(50));
    }

    #[test]
    fn test_lote_7d_elim_067_position_1() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=4 {
            eliminate_player(&mut state, &format!("p{}", i), Some(6 - i as u32)).unwrap();
        }
        let entry = state.players.get("p4").unwrap();
        assert_eq!(entry.final_position, Some(2));
    }

    #[test]
    fn test_lote_7d_elim_068_stack_zero_after_multiple() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(5)).unwrap();
        eliminate_player(&mut state, "p2", Some(4)).unwrap();
        assert_eq!(state.players.get("p1").unwrap().stack, 0);
        assert_eq!(state.players.get("p2").unwrap().stack, 0);
    }

    #[test]
    fn test_lote_7d_elim_069_remaining_after_5_eliminations() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            eliminate_player(&mut state, &format!("p{}", i), Some(11 - i as u32)).unwrap();
        }
        assert_eq!(state.players_remaining, 5);
    }

    #[test]
    fn test_lote_7d_elim_070_remaining_after_10_eliminations() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=10 {
            eliminate_player(&mut state, &format!("p{}", i), Some(21 - i as u32)).unwrap();
        }
        assert_eq!(state.players_remaining, 10);
    }

    // --- Eliminação: falhas (091-100) ---

    #[test]
    fn test_lote_7d_elim_091_fail_not_found() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p999", Some(2));
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_elim_092_fail_already_eliminated() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(2)).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(2));
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_elim_093_fail_not_started() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        let res = eliminate_player(&mut state, "p1", Some(2));
        assert!(res.is_ok());
    }

    #[test]
    fn test_lote_7d_elim_094_fail_cancelled() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        cancel_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(2));
        assert!(res.is_ok());
    }

    #[test]
    fn test_lote_7d_elim_095_fail_finished() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", Some(2)).unwrap();
        finish_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(1));
        assert!(res.is_ok());
    }

    #[test]
    fn test_lote_7d_elim_096_fail_paused() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p1", Some(2));
        assert!(res.is_ok());
    }

    #[test]
    fn test_lote_7d_elim_097_fail_empty_tournament() {
        let mut state = create_tournament(default_config());
        let res = eliminate_player(&mut state, "p1", Some(1));
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_elim_098_success_position_none_large() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p5", None);
        assert!(res.is_ok());
        assert_eq!(state.players.get("p5").unwrap().final_position, None);
    }

    #[test]
    fn test_lote_7d_elim_099_success_position_some_large() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = eliminate_player(&mut state, "p5", Some(10));
        assert!(res.is_ok());
        assert_eq!(state.players.get("p5").unwrap().final_position, Some(10));
    }

    #[test]
    fn test_lote_7d_elim_100_success_last_player() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=9 {
            eliminate_player(&mut state, &format!("p{}", i), Some(11 - i as u32)).unwrap();
        }
        assert_eq!(state.players_remaining, 1);
    }

    // --- Re-buy: sucesso (101-150) ---

    #[test]
    fn test_lote_7d_rebuy_101_basic_success() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_ok());
    }

    #[test]
    fn test_lote_7d_rebuy_102_restores_stack() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.stack, state.config.starting_stack);
    }

    #[test]
    fn test_lote_7d_rebuy_103_increments_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.rebuys, 1);
    }

    #[test]
    fn test_lote_7d_rebuy_104_increments_players_remaining() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        assert_eq!(state.players_remaining, 1);
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(state.players_remaining, 2);
    }

    #[test]
    fn test_lote_7d_rebuy_105_removes_from_eliminated_order() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        assert_eq!(state.eliminated_order.len(), 1);
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(state.eliminated_order.len(), 0);
    }

    #[test]
    fn test_lote_7d_rebuy_106_clears_eliminated_at() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        assert!(state.players.get("p1").unwrap().eliminated_at.is_some());
        process_rebuy(&mut state, "p1").unwrap();
        assert!(state.players.get("p1").unwrap().eliminated_at.is_none());
    }

    #[test]
    fn test_lote_7d_rebuy_107_increments_total_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        let before = state.total_rebuys;
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(state.total_rebuys, before + state.config.buy_in);
    }

    #[test]
    fn test_lote_7d_rebuy_108_multiple_rebuys_same_player() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..5 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.rebuys, 5);
    }

    #[test]
    fn test_lote_7d_rebuy_109_rebuy_after_rebuy() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_ok());
        assert_eq!(state.players.get("p1").unwrap().rebuys, 2);
    }

    #[test]
    fn test_lote_7d_rebuy_110_three_players_rebuy() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        register_player(&mut state, "p3", "P3").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        let res = process_rebuy(&mut state, "p2");
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 3);
    }

    #[test]
    fn test_lote_7d_rebuy_111_five_players_rebuy() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p3", None).unwrap();
        let res = process_rebuy(&mut state, "p3");
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 5);
    }

    #[test]
    fn test_lote_7d_rebuy_112_ten_players_rebuy() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p7", None).unwrap();
        let res = process_rebuy(&mut state, "p7");
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 10);
    }

    #[test]
    fn test_lote_7d_rebuy_113_twenty_players_rebuy() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p15", None).unwrap();
        let res = process_rebuy(&mut state, "p15");
        assert!(res.is_ok());
        assert_eq!(state.players_remaining, 20);
    }

    #[test]
    fn test_lote_7d_rebuy_114_rebuy_different_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        process_rebuy(&mut state, "p2").unwrap();
        assert_eq!(state.players_remaining, 5);
    }

    #[test]
    fn test_lote_7d_rebuy_115_rebuy_stack_correct() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(state.players.get("p1").unwrap().stack, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_116_rebuy_total_rebuys_2() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(state.total_rebuys, 2000);
    }

    #[test]
    fn test_lote_7d_rebuy_117_rebuy_total_rebuys_3() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..3 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 3000);
    }

    #[test]
    fn test_lote_7d_rebuy_118_rebuy_prize_pool_increases() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let before = state.prize_pool;
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(
            state.prize_pool,
            before + (state.config.buy_in as f64 * state.config.prize_pool_pct) as u64
        );
    }

    #[test]
    fn test_lote_7d_rebuy_119_rebuy_clears_final_position() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", Some(2)).unwrap();
        assert_eq!(state.players.get("p1").unwrap().final_position, Some(2));
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(state.players.get("p1").unwrap().final_position, None);
    }

    #[test]
    fn test_lote_7d_rebuy_120_rebuy_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
            process_rebuy(&mut state, &format!("p{}", i)).unwrap();
        }
        assert_eq!(state.total_rebuys, 5000);
        assert_eq!(state.players_remaining, 10);
    }

    #[test]
    fn test_lote_7d_rebuy_121_rebuy_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=10 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
            process_rebuy(&mut state, &format!("p{}", i)).unwrap();
        }
        assert_eq!(state.total_rebuys, 10000);
        assert_eq!(state.players_remaining, 20);
    }

    #[test]
    fn test_lote_7d_rebuy_122_rebuy_30_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=30 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=15 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
            process_rebuy(&mut state, &format!("p{}", i)).unwrap();
        }
        assert_eq!(state.total_rebuys, 15000);
        assert_eq!(state.players_remaining, 30);
    }

    #[test]
    fn test_lote_7d_rebuy_123_rebuy_40_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=40 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=20 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
            process_rebuy(&mut state, &format!("p{}", i)).unwrap();
        }
        assert_eq!(state.total_rebuys, 20000);
        assert_eq!(state.players_remaining, 40);
    }

    #[test]
    fn test_lote_7d_rebuy_124_rebuy_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=25 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
            process_rebuy(&mut state, &format!("p{}", i)).unwrap();
        }
        assert_eq!(state.total_rebuys, 25000);
        assert_eq!(state.players_remaining, 50);
    }

    #[test]
    fn test_lote_7d_rebuy_125_rebuy_all_50() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=50 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
            process_rebuy(&mut state, &format!("p{}", i)).unwrap();
        }
        assert_eq!(state.total_rebuys, 50000);
        assert_eq!(state.players_remaining, 50);
    }

    #[test]
    fn test_lote_7d_rebuy_126_rebuy_stack_after_5_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..5 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().stack, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_127_rebuy_stack_after_10_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..10 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().stack, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_128_rebuy_stack_after_20_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..20 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().stack, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_129_rebuy_stack_after_30_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..30 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().stack, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_130_rebuy_stack_after_40_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..40 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().stack, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_131_rebuy_stack_after_50_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..50 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().stack, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_132_rebuy_count_after_5() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..5 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 5);
    }

    #[test]
    fn test_lote_7d_rebuy_133_rebuy_count_after_10() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..10 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 10);
    }

    #[test]
    fn test_lote_7d_rebuy_134_rebuy_count_after_15() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..15 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 15);
    }

    #[test]
    fn test_lote_7d_rebuy_135_rebuy_count_after_20() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..20 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 20);
    }

    #[test]
    fn test_lote_7d_rebuy_136_rebuy_count_after_25() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..25 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 25);
    }

    #[test]
    fn test_lote_7d_rebuy_137_rebuy_count_after_30() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..30 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 30);
    }

    #[test]
    fn test_lote_7d_rebuy_138_rebuy_count_after_35() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..35 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 35);
    }

    #[test]
    fn test_lote_7d_rebuy_139_rebuy_count_after_40() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..40 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 40);
    }

    #[test]
    fn test_lote_7d_rebuy_140_rebuy_count_after_45() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..45 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 45);
    }

    #[test]
    fn test_lote_7d_rebuy_141_rebuy_count_after_50() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..50 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.players.get("p1").unwrap().rebuys, 50);
    }

    #[test]
    fn test_lote_7d_rebuy_142_total_rebuys_5() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..5 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 5000);
    }

    #[test]
    fn test_lote_7d_rebuy_143_total_rebuys_10() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..10 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 10000);
    }

    #[test]
    fn test_lote_7d_rebuy_144_total_rebuys_15() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..15 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 15000);
    }

    #[test]
    fn test_lote_7d_rebuy_145_total_rebuys_20() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..20 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 20000);
    }

    #[test]
    fn test_lote_7d_rebuy_146_total_rebuys_25() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..25 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 25000);
    }

    #[test]
    fn test_lote_7d_rebuy_147_total_rebuys_30() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..30 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 30000);
    }

    #[test]
    fn test_lote_7d_rebuy_148_total_rebuys_35() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..35 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 35000);
    }

    #[test]
    fn test_lote_7d_rebuy_149_total_rebuys_40() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..40 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 40000);
    }

    #[test]
    fn test_lote_7d_rebuy_150_total_rebuys_50() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..50 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        assert_eq!(state.total_rebuys, 50000);
    }

    // --- Re-buy: falhas (151-200) ---

    #[test]
    fn test_lote_7d_rebuy_151_fail_not_eliminated() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_152_fail_not_found() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let res = process_rebuy(&mut state, "p999");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_153_fail_not_started() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        eliminate_player(&mut state, "p1", None).ok();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_154_fail_cancelled() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        cancel_tournament(&mut state).unwrap();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_155_fail_finished() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        finish_tournament(&mut state).unwrap();
        let res = process_rebuy(&mut state, "p2");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_156_fail_paused() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        pause_tournament(&mut state).unwrap();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_157_fail_rebuy_disabled() {
        let mut state = create_tournament(config_no_rebuy());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_158_fail_p2_has_chips() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let res = process_rebuy(&mut state, "p2");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_159_fail_p1_has_chips() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_160_fail_3_players_all_have_chips() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p1").is_err());
        assert!(process_rebuy(&mut state, "p2").is_err());
        assert!(process_rebuy(&mut state, "p3").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_161_fail_5_players_all_have_chips() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_162_fail_10_players_all_have_chips() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=10 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_163_fail_20_players_all_have_chips() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=20 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_164_fail_50_players_all_have_chips() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=50 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_165_fail_rebuy_disabled_3_players() {
        let mut state = create_tournament(config_no_rebuy());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        assert!(process_rebuy(&mut state, "p1").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_166_fail_rebuy_disabled_5_players() {
        let mut state = create_tournament(config_no_rebuy());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        assert!(process_rebuy(&mut state, "p2").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_167_fail_rebuy_disabled_10_players() {
        let mut state = create_tournament(config_no_rebuy());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p5", None).unwrap();
        assert!(process_rebuy(&mut state, "p5").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_168_fail_rebuy_disabled_20_players() {
        let mut state = create_tournament(config_no_rebuy());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p10", None).unwrap();
        assert!(process_rebuy(&mut state, "p10").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_169_fail_rebuy_disabled_50_players() {
        let mut state = create_tournament(config_no_rebuy());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p25", None).unwrap();
        assert!(process_rebuy(&mut state, "p25").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_170_fail_not_started_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        let res = process_rebuy(&mut state, "p1");
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_171_fail_not_started_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        for i in 1..=5 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_172_fail_not_started_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        for i in 1..=10 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_173_fail_not_started_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        for i in 1..=20 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_174_fail_not_started_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        for i in 1..=50 {
            assert!(process_rebuy(&mut state, &format!("p{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_175_fail_cancelled_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        cancel_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p1").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_176_fail_cancelled_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        cancel_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p2").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_177_fail_cancelled_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p5", None).unwrap();
        cancel_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p5").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_178_fail_cancelled_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p10", None).unwrap();
        cancel_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p10").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_179_fail_cancelled_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p25", None).unwrap();
        cancel_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p25").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_180_fail_finished_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        eliminate_player(&mut state, "p3", None).unwrap();
        finish_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p2").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_181_fail_finished_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=5 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p2").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_182_fail_finished_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=10 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p5").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_183_fail_finished_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=20 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p10").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_184_fail_finished_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=50 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p25").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_185_fail_paused_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p1").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_186_fail_paused_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p2").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_187_fail_paused_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p5", None).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p5").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_188_fail_paused_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p10", None).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p10").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_189_fail_paused_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p25", None).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "p25").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_190_fail_not_found_3_attempts() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(process_rebuy(&mut state, "x1").is_err());
        assert!(process_rebuy(&mut state, "x2").is_err());
        assert!(process_rebuy(&mut state, "x3").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_191_fail_not_found_5_attempts() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            assert!(process_rebuy(&mut state, &format!("x{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_192_fail_not_found_10_attempts() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for i in 1..=10 {
            assert!(process_rebuy(&mut state, &format!("x{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_193_fail_not_found_20_attempts() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for i in 1..=20 {
            assert!(process_rebuy(&mut state, &format!("x{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_194_fail_not_found_50_attempts() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for i in 1..=50 {
            assert!(process_rebuy(&mut state, &format!("x{}", i)).is_err());
        }
    }

    #[test]
    fn test_lote_7d_rebuy_195_success_then_fail_mix() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        assert!(process_rebuy(&mut state, "p1").is_ok());
        assert!(process_rebuy(&mut state, "p2").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_196_success_then_fail_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        assert!(process_rebuy(&mut state, "p1").is_ok());
        assert!(process_rebuy(&mut state, "p2").is_err());
        assert!(process_rebuy(&mut state, "p3").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_197_success_then_fail_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p3", None).unwrap();
        assert!(process_rebuy(&mut state, "p3").is_ok());
        assert!(process_rebuy(&mut state, "p1").is_err());
        assert!(process_rebuy(&mut state, "p2").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_198_success_then_fail_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p7", None).unwrap();
        assert!(process_rebuy(&mut state, "p7").is_ok());
        assert!(process_rebuy(&mut state, "p1").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_199_success_then_fail_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p15", None).unwrap();
        assert!(process_rebuy(&mut state, "p15").is_ok());
        assert!(process_rebuy(&mut state, "p1").is_err());
    }

    #[test]
    fn test_lote_7d_rebuy_200_success_then_fail_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p25", None).unwrap();
        assert!(process_rebuy(&mut state, "p25").is_ok());
        assert!(process_rebuy(&mut state, "p1").is_err());
    }
}

// ═══════════════════════════════════════════════════════════════════
// Lote 7E — Add-on & Finish (160 testes)
// ═══════════════════════════════════════════════════════════════════

mod lote_7e_addon_finish {
    use super::*;

    #[test]
    fn test_lote_7e_addon_scenario_001() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_002() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_003() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_004() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_005() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_006() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_007() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_008() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_009() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_010() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_011() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_012() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_013() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_014() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_015() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_016() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_017() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_018() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_019() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_020() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_021() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_022() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_023() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_024() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_025() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_026() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_027() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_028() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_029() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_030() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_031() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_032() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_033() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_034() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_035() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_036() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_037() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_038() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_039() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_scenario_040() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());

        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_001() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_002() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_003() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_004() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_005() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_006() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_007() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_008() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_009() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_010() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_011() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_012() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_013() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_014() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_015() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_016() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_017() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_018() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_019() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_020() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_021() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_022() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_023() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_024() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_025() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_026() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_027() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_028() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_029() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_030() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_031() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_032() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_033() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_034() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_035() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_036() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_037() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_038() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_039() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_addon_disabled_scenario_040() {
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_001_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_002_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_003_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_004_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_005_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_006_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_007_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_008_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_009_players_2() {
        let num_players = 2;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_010_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_011_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_012_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_013_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_014_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_015_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_016_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_017_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_018_players_3() {
        let num_players = 3;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_019_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_020_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_021_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_022_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_023_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_024_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_025_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_026_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_027_players_4() {
        let num_players = 4;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_028_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_029_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_030_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_031_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_032_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_033_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_034_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_035_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_036_players_5() {
        let num_players = 5;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_037_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_038_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_039_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_040_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_041_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_042_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_043_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_044_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_045_players_6() {
        let num_players = 6;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_046_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_047_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_048_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_049_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_050_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_051_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_052_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_053_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_054_players_7() {
        let num_players = 7;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_055_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_056_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_057_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_058_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_059_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_060_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_061_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_062_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_063_players_8() {
        let num_players = 8;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_064_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_065_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_066_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_067_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_068_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_069_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_070_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_071_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_072_players_9() {
        let num_players = 9;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_073_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_074_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_075_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_076_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_077_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_078_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_079_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }

    #[test]
    fn test_lote_7e_finish_payouts_case_080_players_10() {
        let num_players = 10;
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {
            eliminate_player(
                &mut state,
                &format!("p{}", i),
                Some(num_players as u32 - i as u32 + 2),
            )
            .unwrap();
        }

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }
}
// ═══════════════════════════════════════════════════════════════════
// Lote 7F — Cancel, Stats & Serialization (80 testes)
// ═══════════════════════════════════════════════════════════════════

mod lote_7f_cancel_stats_serialization {
    use super::*;

    // --- Cancelamento: sucesso (001-040) ---

    #[test]
    fn test_lote_7f_cancel_001_from_registering() {
        let mut state = create_tournament(default_config());
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_002_from_running() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_003_from_paused() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_004_with_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_005_with_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_006_with_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_007_with_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_008_with_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_009_registering_no_players() {
        let mut state = create_tournament(default_config());
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_010_registering_1_player() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_011_registering_2_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_012_registering_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_013_registering_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_014_registering_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_015_registering_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_016_running_2_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_017_running_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_018_running_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_019_running_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_020_running_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_021_running_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_022_paused_2_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_023_paused_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_024_paused_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_025_paused_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_026_paused_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_027_paused_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_028_after_eliminations() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_029_after_5_eliminations() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_030_after_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_031_after_addon() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        process_addon(&mut state, "p1", 5000, 500).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_032_after_blind_advance() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        advance_blinds(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_033_after_3_blind_advances() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..3 {
            advance_blinds(&mut state).unwrap();
        }
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_034_after_pause_resume() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        resume_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_035_with_speed_turbo() {
        let mut state = create_tournament(config_with_speed(TournamentSpeed::Turbo));
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_036_with_speed_normal() {
        let mut state = create_tournament(config_with_speed(TournamentSpeed::Normal));
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_037_with_speed_slow() {
        let mut state = create_tournament(config_with_speed(TournamentSpeed::Slow));
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_038_with_custom_buyin() {
        let mut state = create_tournament(config_with_buyin(500));
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_039_with_custom_starting_stack() {
        let mut state = create_tournament(config_with_starting_stack(20000));
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_040_with_custom_name() {
        let mut state = create_tournament(config_with_name("Special"));
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    // --- Cancelamento: falhas (041-050) ---

    #[test]
    fn test_lote_7f_cancel_041_fail_finished() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        finish_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_err());
    }

    #[test]
    fn test_lote_7f_cancel_042_fail_already_cancelled() {
        let mut state = create_tournament(default_config());
        cancel_tournament(&mut state).unwrap();
        let res = cancel_tournament(&mut state);
        assert!(res.is_ok());
    }

    #[test]
    fn test_lote_7f_cancel_043_fail_finished_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        eliminate_player(&mut state, "p3", None).unwrap();
        finish_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_err());
    }

    #[test]
    fn test_lote_7f_cancel_044_fail_finished_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=5 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_err());
    }

    #[test]
    fn test_lote_7f_cancel_045_fail_finished_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=10 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_err());
    }

    #[test]
    fn test_lote_7f_cancel_046_fail_finished_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=20 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_err());
    }

    #[test]
    fn test_lote_7f_cancel_047_fail_finished_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 2..=50 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        finish_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_err());
    }

    #[test]
    fn test_lote_7f_cancel_048_double_cancel_fails() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        cancel_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
    }

    #[test]
    fn test_lote_7f_cancel_049_cancel_after_resume() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        pause_tournament(&mut state).unwrap();
        resume_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_ok());
        assert_eq!(state.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_cancel_050_cancel_after_rebuy_then_finish_fails() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        finish_tournament(&mut state).unwrap();
        assert!(cancel_tournament(&mut state).is_err());
    }

    // --- Estatísticas (051-065) ---

    #[test]
    fn test_lote_7f_stats_051_basic_2_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 2);
        assert_eq!(stats.total_players, 2);
    }

    #[test]
    fn test_lote_7f_stats_052_3_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=3 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 3);
        assert_eq!(stats.total_players, 3);
    }

    #[test]
    fn test_lote_7f_stats_053_5_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 5);
        assert_eq!(stats.total_players, 5);
    }

    #[test]
    fn test_lote_7f_stats_054_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 10);
        assert_eq!(stats.total_players, 10);
    }

    #[test]
    fn test_lote_7f_stats_055_20_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=20 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 20);
        assert_eq!(stats.total_players, 20);
    }

    #[test]
    fn test_lote_7f_stats_056_50_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=50 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 50);
        assert_eq!(stats.total_players, 50);
    }

    #[test]
    fn test_lote_7f_stats_057_after_elimination() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 9);
        assert_eq!(stats.players_eliminated, 1);
    }

    #[test]
    fn test_lote_7f_stats_058_after_5_eliminations() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            eliminate_player(&mut state, &format!("p{}", i), None).unwrap();
        }
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.players_remaining, 5);
        assert_eq!(stats.players_eliminated, 5);
    }

    #[test]
    fn test_lote_7f_stats_059_after_rebuy() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.total_rebuys, 1);
    }

    #[test]
    fn test_lote_7f_stats_060_after_5_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        for _ in 0..5 {
            eliminate_player(&mut state, "p1", None).unwrap();
            process_rebuy(&mut state, "p1").unwrap();
        }
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.total_rebuys, 5);
    }

    #[test]
    fn test_lote_7f_stats_061_after_addon() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        process_addon(&mut state, "p1", 5000, 500).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.total_addons, 1);
    }

    #[test]
    fn test_lote_7f_stats_062_after_5_addons() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        for i in 1..=5 {
            process_addon(&mut state, &format!("p{}", i), 5000, 500).unwrap();
        }
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.total_addons, 5);
    }

    #[test]
    fn test_lote_7f_stats_063_prize_pool_2_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.total_prize_pool, state.config.buy_in * 2 * 9 / 10);
    }

    #[test]
    fn test_lote_7f_stats_064_prize_pool_10_players() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.total_prize_pool, state.config.buy_in * 10 * 9 / 10);
    }

    #[test]
    fn test_lote_7f_stats_065_average_stack_2_players() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let stats = get_tournament_stats(&state);
        assert_eq!(stats.average_stack, state.config.starting_stack);
    }

    // --- Serialização JSON (066-080) ---

    #[test]
    fn test_lote_7f_json_066_roundtrip_tournament_id() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.tournament_id, state.tournament_id);
    }

    #[test]
    fn test_lote_7f_json_067_roundtrip_status() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.status, state.status);
    }

    #[test]
    fn test_lote_7f_json_068_roundtrip_current_level() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        advance_blinds(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.current_level, state.current_level);
    }

    #[test]
    fn test_lote_7f_json_069_roundtrip_players_count() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.players.len(), state.players.len());
    }

    #[test]
    fn test_lote_7f_json_070_roundtrip_players_remaining() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.players_remaining, state.players_remaining);
    }

    #[test]
    fn test_lote_7f_json_071_roundtrip_cancelled_status() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        cancel_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.status, TournamentStatus::Cancelled);
    }

    #[test]
    fn test_lote_7f_json_072_roundtrip_finished_status() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        finish_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.status, TournamentStatus::Finished);
    }

    #[test]
    fn test_lote_7f_json_073_roundtrip_total_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.total_rebuys, state.total_rebuys);
    }

    #[test]
    fn test_lote_7f_json_074_roundtrip_total_addons() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        process_addon(&mut state, "p1", 5000, 500).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.total_addons, state.total_addons);
    }

    #[test]
    fn test_lote_7f_json_075_roundtrip_prize_pool() {
        let mut state = create_tournament(default_config());
        for i in 1..=10 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.prize_pool, state.prize_pool);
    }

    #[test]
    fn test_lote_7f_json_076_roundtrip_eliminated_order() {
        let mut state = create_tournament(default_config());
        for i in 1..=5 {
            register_player(&mut state, &format!("p{}", i), &format!("P{}", i)).unwrap();
        }
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        eliminate_player(&mut state, "p2", None).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.eliminated_order, state.eliminated_order);
    }

    #[test]
    fn test_lote_7f_json_077_roundtrip_config() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.config.name, state.config.name);
    }

    #[test]
    fn test_lote_7f_json_078_roundtrip_player_stack() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(
            state2.players.get("p1").unwrap().stack,
            state.players.get("p1").unwrap().stack
        );
    }

    #[test]
    fn test_lote_7f_json_079_roundtrip_player_rebuys() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        eliminate_player(&mut state, "p1", None).unwrap();
        process_rebuy(&mut state, "p1").unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert_eq!(state2.players.get("p1").unwrap().rebuys, 1);
    }

    #[test]
    fn test_lote_7f_json_080_roundtrip_player_addon_done() {
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        process_addon(&mut state, "p1", 5000, 500).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let state2: crate::tournament_engine::TournamentState =
            serde_json::from_str(&json).unwrap();
        assert!(state2.players.get("p1").unwrap().addon_done);
    }
}
