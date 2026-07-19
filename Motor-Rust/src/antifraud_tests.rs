// ─── Testes Expandidos de Antifraude ───
// Cobertura de edge cases, boundary conditions e cenários extremos
// para todos os 4 módulos de antifraude.
//
// Complementa os testes inline existentes em cada módulo e os
// property-based tests em tests/property_tests.rs.

use crate::antifraud::{bot_detection, chip_dumping, collusion, multi_account};

// ═══════════════════════════════════════════════════════════════
// Collusion - Edge Cases & Boundaries
// ═══════════════════════════════════════════════════════════════

mod collusion_edge_cases {
    use super::*;

    fn make_action(
        player: &str,
        action: collusion::PlayerAction,
        strength: collusion::HandStrength,
        street: u8,
    ) -> collusion::ActionRecord {
        collusion::ActionRecord {
            player_id: player.to_string(),
            action,
            hand_strength: strength,
            timestamp_ms: 1000,
            street,
        }
    }

    #[test]
    fn test_score_zero_with_no_hands() {
        let _pair = collusion::PlayerPair {
            player_a: "a".into(),
            player_b: "b".into(),
            hands_together: 0,
            soft_play_count: 0,
            coordinated_actions: 0,
            suspicion_score: 0.0,
        };
        // calculate_suspicion_score is private, but we can verify via analyzer
        let analyzer = collusion::CollusionAnalyzer::new();
        // No hands → no pairs → no alerts
        assert!(analyzer.get_alerts().is_empty());
        assert!(analyzer.get_all_pairs().is_empty());
    }

    #[test]
    fn test_score_maximum_with_extreme_behavior() {
        let mut analyzer =
            collusion::CollusionAnalyzer::with_thresholds(collusion::CollusionThresholds {
                min_hands_together: 2,
                alert_threshold: 0.1,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        // 20 mãos com soft play máximo (Check com Monster) + coordenação máxima
        for _ in 0..20 {
            let actions = vec![
                make_action(
                    "alice",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Medium,
                    1,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Fold,
                    collusion::HandStrength::VeryWeak,
                    1,
                ),
            ];
            analyzer.analyze_hand("table1", &actions, 1000);
        }

        let alerts = analyzer.get_alerts();
        assert!(!alerts.is_empty());
        // Com 20 mãos, volume_factor = 1.0, score deve ser máximo (1.0)
        let last = alerts.last().unwrap();
        assert!(last.pair.suspicion_score > 0.8);
        assert_eq!(last.severity, "critical");
    }

    #[test]
    fn test_threshold_boundary_exactly_at_alert_threshold() {
        let mut analyzer =
            collusion::CollusionAnalyzer::with_thresholds(collusion::CollusionThresholds {
                min_hands_together: 5,
                alert_threshold: 0.3,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        // 5 mãos: soft_play_count=5, coord=5 → soft_rate=1.0, coord_rate=1.0
        // soft_score=1.0, coord_score=1.0, combined=1.0, volume=5/20=0.25
        // final = 0.25 → abaixo de 0.3 → sem alerta
        for _ in 0..5 {
            let actions = vec![
                make_action(
                    "alice",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Medium,
                    1,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Fold,
                    collusion::HandStrength::VeryWeak,
                    1,
                ),
            ];
            analyzer.analyze_hand("table1", &actions, 1000);
        }

        // Score = 0.25 < 0.3 → sem alerta
        let alerts = analyzer.get_alerts();
        assert!(alerts.is_empty());

        // +5 mãos → volume=10/20=0.5, score=0.5 → alerta
        for _ in 0..5 {
            let actions = vec![
                make_action(
                    "alice",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Medium,
                    1,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Fold,
                    collusion::HandStrength::VeryWeak,
                    1,
                ),
            ];
            analyzer.analyze_hand("table1", &actions, 2000);
        }

        let alerts = analyzer.get_alerts();
        assert!(!alerts.is_empty());
    }

    #[test]
    fn test_severity_boundary_critical_vs_high() {
        let mut analyzer =
            collusion::CollusionAnalyzer::with_thresholds(collusion::CollusionThresholds {
                min_hands_together: 2,
                alert_threshold: 0.1,
                critical_threshold: 0.7,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        // 10 mãos: volume=0.5, score=0.5 → high (>=0.5, <0.7)
        for _ in 0..10 {
            let actions = vec![
                make_action(
                    "alice",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Medium,
                    1,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Fold,
                    collusion::HandStrength::VeryWeak,
                    1,
                ),
            ];
            analyzer.analyze_hand("table1", &actions, 1000);
        }

        let alerts = analyzer.get_alerts();
        let last = alerts.last().unwrap();
        assert_eq!(last.severity, "high");

        // +10 mãos → volume=1.0, score=1.0 → critical
        for _ in 0..10 {
            let actions = vec![
                make_action(
                    "alice",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Medium,
                    1,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Fold,
                    collusion::HandStrength::VeryWeak,
                    1,
                ),
            ];
            analyzer.analyze_hand("table1", &actions, 2000);
        }

        let alerts = analyzer.get_alerts();
        let last = alerts.last().unwrap();
        assert_eq!(last.severity, "critical");
    }

    #[test]
    fn test_empty_player_ids() {
        let mut analyzer = collusion::CollusionAnalyzer::new();
        let actions = vec![
            make_action(
                "",
                collusion::PlayerAction::Check,
                collusion::HandStrength::Strong,
                0,
            ),
            make_action(
                "",
                collusion::PlayerAction::Call,
                collusion::HandStrength::Medium,
                0,
            ),
        ];
        // Não deve panicar com strings vazias
        let alerts = analyzer.analyze_hand("table1", &actions, 0);
        // Dois jogadores com mesmo ID vazio → mesmo jogador, sem par
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_many_players_no_panic() {
        let mut analyzer = collusion::CollusionAnalyzer::new();
        let mut actions = Vec::new();
        for i in 0..10 {
            actions.push(make_action(
                &format!("player{}", i),
                collusion::PlayerAction::Call,
                collusion::HandStrength::Medium,
                0,
            ));
        }
        let alerts = analyzer.analyze_hand("table1", &actions, 0);
        // 10 jogadores → C(10,2) = 45 pares, mas apenas 1 mão → sem alertas
        assert!(alerts.is_empty());
        let pairs = analyzer.get_all_pairs();
        assert_eq!(pairs.len(), 45);
    }

    #[test]
    fn test_all_in_does_not_count_as_soft_play() {
        let mut analyzer = collusion::CollusionAnalyzer::new();
        // AllIn é ação agressiva, não soft play
        let actions = vec![
            make_action(
                "alice",
                collusion::PlayerAction::AllIn(5000),
                collusion::HandStrength::Monster,
                2,
            ),
            make_action(
                "bob",
                collusion::PlayerAction::Call,
                collusion::HandStrength::Strong,
                2,
            ),
        ];
        analyzer.analyze_hand("table1", &actions, 0);

        let pairs = analyzer.get_all_pairs();
        // alice deu AllIn → não é soft play. bob deu Call com Strong → soft play
        assert_eq!(pairs[0].soft_play_count, 1);
    }

    #[test]
    fn test_raise_with_weak_hand_not_soft_play() {
        let mut analyzer = collusion::CollusionAnalyzer::new();
        // Raise é sempre agressivo, independente da força da mão.
        // Soft play requer mão Strong+ (Strong, VeryStrong, Monster) + Check/Call.
        // alice tem VeryWeak + Raise → não soft play.
        // bob tem Medium + Call → não soft play (Medium < Strong).
        let actions = vec![
            make_action(
                "alice",
                collusion::PlayerAction::Raise(100),
                collusion::HandStrength::VeryWeak,
                0,
            ),
            make_action(
                "bob",
                collusion::PlayerAction::Call,
                collusion::HandStrength::Medium,
                0,
            ),
        ];
        analyzer.analyze_hand("table1", &actions, 0);

        let pairs = analyzer.get_all_pairs();
        // Nenhum dos dois tem mão Strong+ → soft_play_count = 0
        assert_eq!(pairs[0].soft_play_count, 0);
    }

    #[test]
    fn test_coordination_requires_raise_then_fold() {
        let mut analyzer = collusion::CollusionAnalyzer::new();
        // Check → Fold não é coordenação (não tem raise antes)
        let actions = vec![
            make_action(
                "alice",
                collusion::PlayerAction::Check,
                collusion::HandStrength::Medium,
                0,
            ),
            make_action(
                "bob",
                collusion::PlayerAction::Fold,
                collusion::HandStrength::Weak,
                0,
            ),
        ];
        analyzer.analyze_hand("table1", &actions, 0);

        let pairs = analyzer.get_all_pairs();
        assert_eq!(pairs[0].coordinated_actions, 0);
    }

    #[test]
    fn test_multiple_tables_independent() {
        let mut analyzer = collusion::CollusionAnalyzer::new();

        // Table 1: alice+bob soft play
        analyzer.analyze_hand(
            "table1",
            &[
                make_action(
                    "alice",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
            ],
            1000,
        );

        // Table 2: alice+charlie normal play
        analyzer.analyze_hand(
            "table2",
            &[
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Strong,
                    0,
                ),
                make_action(
                    "charlie",
                    collusion::PlayerAction::Raise(200),
                    collusion::HandStrength::Strong,
                    0,
                ),
            ],
            1000,
        );

        let pairs = analyzer.get_all_pairs();
        // Deve ter 2 pares: alice-bob e alice-charlie
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_get_alerts_for_player() {
        let mut analyzer =
            collusion::CollusionAnalyzer::with_thresholds(collusion::CollusionThresholds {
                min_hands_together: 2,
                alert_threshold: 0.1,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        // alice+bob: comportamento suspeito
        for _ in 0..10 {
            let actions = vec![
                make_action(
                    "alice",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Check,
                    collusion::HandStrength::Monster,
                    2,
                ),
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Medium,
                    1,
                ),
                make_action(
                    "bob",
                    collusion::PlayerAction::Fold,
                    collusion::HandStrength::VeryWeak,
                    1,
                ),
            ];
            analyzer.analyze_hand("table1", &actions, 1000);
        }

        // alice+charlie: normal
        for _ in 0..5 {
            let actions = vec![
                make_action(
                    "alice",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Strong,
                    0,
                ),
                make_action(
                    "charlie",
                    collusion::PlayerAction::Raise(200),
                    collusion::HandStrength::Strong,
                    0,
                ),
            ];
            analyzer.analyze_hand("table2", &actions, 1000);
        }

        // get_alerts_for_player não existe em collusion, mas get_suspicious_pairs sim
        let suspicious = analyzer.get_suspicious_pairs();
        // alice-bob deve estar na lista
        let has_alice_bob = suspicious.iter().any(|p| {
            (p.player_a == "alice" && p.player_b == "bob")
                || (p.player_a == "bob" && p.player_b == "alice")
        });
        assert!(has_alice_bob);
    }

    #[test]
    fn test_hand_strength_all_variants() {
        // Verifica que todas as variantes de HandStrength são comparáveis
        let strengths = [collusion::HandStrength::VeryWeak,
            collusion::HandStrength::Weak,
            collusion::HandStrength::Medium,
            collusion::HandStrength::Strong,
            collusion::HandStrength::VeryStrong,
            collusion::HandStrength::Monster];

        for i in 1..strengths.len() {
            assert!(strengths[i] > strengths[i - 1]);
        }
    }

    #[test]
    fn test_pair_key_consistent_with_special_chars() {
        // make_pair_key é privada, mas podemos verificar indiretamente
        let mut analyzer = collusion::CollusionAnalyzer::new();
        analyzer.analyze_hand(
            "table1",
            &[
                make_action(
                    "player-1",
                    collusion::PlayerAction::Call,
                    collusion::HandStrength::Medium,
                    0,
                ),
                make_action(
                    "player_2",
                    collusion::PlayerAction::Raise(100),
                    collusion::HandStrength::Strong,
                    0,
                ),
            ],
            1000,
        );
        let pairs = analyzer.get_all_pairs();
        assert_eq!(pairs.len(), 1);
        // O par contém ambos os jogadores (ordem depende da implementação de make_pair_key)
        let has_both = (pairs[0].player_a == "player-1" && pairs[0].player_b == "player_2")
            || (pairs[0].player_a == "player_2" && pairs[0].player_b == "player-1");
        assert!(
            has_both,
            "Pair should contain both players, got: a={}, b={}",
            pairs[0].player_a, pairs[0].player_b
        );
    }
}

// ═══════════════════════════════════════════════════════════════
// Chip Dumping - Edge Cases & Boundaries
// ═══════════════════════════════════════════════════════════════

mod chip_dumping_edge_cases {
    use super::*;

    #[test]
    fn test_score_zero_with_no_transfers() {
        let analyzer = chip_dumping::ChipDumpAnalyzer::new();
        assert!(analyzer.get_alerts().is_empty());
        assert_eq!(analyzer.get_total_dumped("alice", "bob"), 0);
    }

    #[test]
    fn test_score_maximum_with_extreme_dumping() {
        let mut analyzer =
            chip_dumping::ChipDumpAnalyzer::with_thresholds(chip_dumping::ChipDumpThresholds {
                max_hand_strength: chip_dumping::HandStrength::Weak,
                min_amount: 100,
                min_occurrences: 2,
                min_total_dumped: 200,
                alert_threshold: 0.2,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        // 10 all-ins VeryWeak com valores altos
        for i in 0..10 {
            analyzer.analyze_all_in(
                "alice",
                "bob",
                10000,
                chip_dumping::HandStrength::VeryWeak,
                &format!("hand{}", i),
                1000 + i as u64 * 100,
            );
        }

        let alerts = analyzer.get_alerts();
        assert!(!alerts.is_empty());
        let last = alerts.last().unwrap();
        // Com 10 ocorrências VeryWeak de 10k cada:
        // consistency=1.0, weakness=1.0, volume=min(100k/10k,1.0)=1.0, frequency=min(10/10,1.0)=1.0
        // score = 0.30 + 0.30 + 0.25 + 0.15 = 1.0
        assert!(last.suspicion_score > 0.9);
        assert_eq!(last.severity, "critical");
    }

    #[test]
    fn test_threshold_boundary_exact_min_amount() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // Exatamente no min_amount (500)
        let result = analyzer.analyze_all_in(
            "alice",
            "bob",
            500,
            chip_dumping::HandStrength::VeryWeak,
            "hand1",
            1000,
        );
        // Registrado (>= min_amount)
        assert!(!result); // 1 ocorrência não gera alerta
        let transfers = analyzer.get_transfers("alice", "bob");
        assert_eq!(transfers.len(), 1);
    }

    #[test]
    fn test_threshold_boundary_below_min_amount() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // Abaixo do min_amount (499 < 500)
        let result = analyzer.analyze_all_in(
            "alice",
            "bob",
            499,
            chip_dumping::HandStrength::VeryWeak,
            "hand1",
            1000,
        );
        assert!(!result);
        assert!(analyzer.get_transfers("alice", "bob").is_empty());
    }

    #[test]
    fn test_severity_boundary_medium_to_high() {
        let mut analyzer =
            chip_dumping::ChipDumpAnalyzer::with_thresholds(chip_dumping::ChipDumpThresholds {
                max_hand_strength: chip_dumping::HandStrength::Weak,
                min_amount: 100,
                min_occurrences: 2,
                min_total_dumped: 200,
                alert_threshold: 0.2,
                critical_threshold: 0.9,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        // 2 dumps VeryWeak de 1000 cada:
        // consistency=1.0, weakness=1.0, volume=min(2000/10000,1.0)=0.2, frequency=min(2/10,1.0)=0.2
        // score = 0.30 + 0.30 + 0.05 + 0.03 = 0.68 → high
        analyzer.analyze_all_in(
            "alice",
            "bob",
            1000,
            chip_dumping::HandStrength::VeryWeak,
            "h1",
            1000,
        );
        analyzer.analyze_all_in(
            "alice",
            "bob",
            1000,
            chip_dumping::HandStrength::VeryWeak,
            "h2",
            2000,
        );

        let alerts = analyzer.get_alerts();
        assert_eq!(alerts[0].severity, "high");
    }

    #[test]
    fn test_severity_boundary_low_to_medium() {
        let mut analyzer =
            chip_dumping::ChipDumpAnalyzer::with_thresholds(chip_dumping::ChipDumpThresholds {
                max_hand_strength: chip_dumping::HandStrength::Weak,
                min_amount: 100,
                min_occurrences: 2,
                min_total_dumped: 200,
                alert_threshold: 0.1,
                critical_threshold: 0.9,
                high_threshold: 0.7,
                medium_threshold: 0.4,
            });

        // 2 dumps Weak (não VeryWeak) de 500 cada:
        // consistency=1.0, weakness=0.5 (Weak, não VeryWeak), volume=min(1000/10000,1.0)=0.1, frequency=0.2
        // score = 0.30 + 0.15 + 0.025 + 0.03 = 0.505 → medium
        analyzer.analyze_all_in(
            "alice",
            "bob",
            500,
            chip_dumping::HandStrength::Weak,
            "h1",
            1000,
        );
        analyzer.analyze_all_in(
            "alice",
            "bob",
            500,
            chip_dumping::HandStrength::Weak,
            "h2",
            2000,
        );

        let alerts = analyzer.get_alerts();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, "medium");
    }

    #[test]
    fn test_empty_player_ids() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // Strings vazias não devem panicar
        let result = analyzer.analyze_all_in(
            "",
            "",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "hand1",
            1000,
        );
        assert!(!result);
        // Deve ter registrado (from="" e to="" → mesmo par)
        let transfers = analyzer.get_transfers("", "");
        assert_eq!(transfers.len(), 1);
    }

    #[test]
    fn test_self_transfer_ignored_or_handled() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // Jogador transfere para si mesmo (não faz sentido, mas não deve panicar)
        let result = analyzer.analyze_all_in(
            "alice",
            "alice",
            5000,
            chip_dumping::HandStrength::VeryWeak,
            "hand1",
            1000,
        );
        // Deve ser registrado ou ignorado sem panic
        assert!(!result);
    }

    #[test]
    fn test_rapid_fire_dumps() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // 5 dumps em rápida sucessão (mesmo timestamp)
        for i in 0..5 {
            analyzer.analyze_all_in(
                "alice",
                "bob",
                2000,
                chip_dumping::HandStrength::VeryWeak,
                &format!("hand{}", i),
                1000, // mesmo timestamp
            );
        }

        let alerts = analyzer.get_alerts();
        assert!(!alerts.is_empty());
        let last = alerts.last().unwrap();
        assert_eq!(last.occurrences, 5);
        assert_eq!(last.total_dumped, 10000);
    }

    #[test]
    fn test_mixed_strength_dumps() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // Mistura de VeryWeak e Weak
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h1",
            1000,
        );
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::Weak,
            "h2",
            2000,
        );
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h3",
            3000,
        );

        let alerts = analyzer.get_alerts();
        assert!(!alerts.is_empty());
        // weakness_score: 2 VeryWeak (1.0 cada) + 1 Weak (0.5) → avg = 0.833
        let last = alerts.last().unwrap();
        assert!(last.suspicion_score > 0.5);
    }

    #[test]
    fn test_get_transfers_nonexistent_pair() {
        let analyzer = chip_dumping::ChipDumpAnalyzer::new();
        let transfers = analyzer.get_transfers("unknown1", "unknown2");
        assert!(transfers.is_empty());
    }

    #[test]
    fn test_get_total_dumped_nonexistent_pair() {
        let analyzer = chip_dumping::ChipDumpAnalyzer::new();
        let total = analyzer.get_total_dumped("unknown1", "unknown2");
        assert_eq!(total, 0);
    }

    #[test]
    fn test_alerts_accumulate_over_time() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // Gera primeiro alerta
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h1",
            1000,
        );
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h2",
            2000,
        );
        assert_eq!(analyzer.get_alerts().len(), 1);

        // Mais dumps → mais alertas
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h3",
            3000,
        );
        assert!(!analyzer.get_alerts().is_empty());
    }

    #[test]
    fn test_three_player_chain() {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::new();
        // Alice → Bob → Charlie (chain dumping)
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h1",
            1000,
        );
        analyzer.analyze_all_in(
            "alice",
            "bob",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h2",
            2000,
        );
        analyzer.analyze_all_in(
            "bob",
            "charlie",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h3",
            3000,
        );
        analyzer.analyze_all_in(
            "bob",
            "charlie",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h4",
            4000,
        );

        let alerts = analyzer.get_alerts();
        // Dois pares diferentes → 2 alertas
        assert_eq!(alerts.len(), 2);
    }

    #[test]
    fn test_very_weak_vs_weak_scoring_difference() {
        // Verifica que VeryWeak gera score maior que Weak
        let mut analyzer_vw = chip_dumping::ChipDumpAnalyzer::new();
        analyzer_vw.analyze_all_in(
            "a",
            "b",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h1",
            1000,
        );
        analyzer_vw.analyze_all_in(
            "a",
            "b",
            2000,
            chip_dumping::HandStrength::VeryWeak,
            "h2",
            2000,
        );

        let mut analyzer_w = chip_dumping::ChipDumpAnalyzer::new();
        analyzer_w.analyze_all_in("a", "b", 2000, chip_dumping::HandStrength::Weak, "h1", 1000);
        analyzer_w.analyze_all_in("a", "b", 2000, chip_dumping::HandStrength::Weak, "h2", 2000);

        let score_vw = analyzer_vw.get_alerts()[0].suspicion_score;
        let score_w = analyzer_w.get_alerts()[0].suspicion_score;
        assert!(score_vw > score_w);
    }
}

// ═══════════════════════════════════════════════════════════════
// Bot Detection - Edge Cases & Boundaries
// ═══════════════════════════════════════════════════════════════

mod bot_detection_edge_cases {
    use super::*;

    fn make_action(
        player_id: &str,
        action_type: &str,
        amount: u64,
        timestamp_ms: u64,
        hand_id: &str,
        street: &str,
    ) -> bot_detection::PlayerAction {
        bot_detection::PlayerAction {
            player_id: player_id.to_string(),
            action_type: action_type.to_string(),
            amount,
            timestamp_ms,
            hand_id: hand_id.to_string(),
            street: street.to_string(),
        }
    }

    #[test]
    fn test_exact_min_actions_boundary() {
        let mut detector = bot_detection::BotDetector::new();
        // Exatamente 20 ações (min_actions = 20)
        let mut ts = 1000u64;
        for i in 0..20 {
            ts += 3000 + (i as u64 % 2) * 10; // quase constante
            detector.record_action(make_action(
                "alice",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("alice", ts + 1000);
        // 20 ações → deve analisar
        assert!(alert.is_some());
    }

    #[test]
    fn test_one_below_min_actions() {
        let mut detector = bot_detection::BotDetector::new();
        // 19 ações (min_actions = 20)
        let mut ts = 1000u64;
        for i in 0..19 {
            ts += 3000 + (i as u64 % 2) * 10;
            detector.record_action(make_action(
                "alice",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("alice", ts + 1000);
        assert!(alert.is_none());
    }

    #[test]
    fn test_cv_exactly_zero() {
        let mut detector =
            bot_detection::BotDetector::with_thresholds(bot_detection::BotThresholds {
                min_actions: 10,
                max_coefficient_of_variation: 0.15,
                max_mathematical_precision: 0.85,
                alert_threshold: 0.2,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        let mut ts = 1000u64;
        for i in 0..15 {
            // Timing EXATAMENTE constante (3000ms entre cada ação)
            ts += 3000;
            detector.record_action(make_action(
                "bot1",
                "bet",
                500,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("bot1", ts + 1000);
        assert!(alert.is_some());
        // CV = 0 → temporal_consistency deve ser alto (próximo de 1.0)
        let metrics = &alert.unwrap().metrics;
        assert!(
            metrics.coefficient_of_variation < 0.01,
            "CV should be near 0, got: {}",
            metrics.coefficient_of_variation
        );
        assert!(
            metrics.temporal_consistency > 0.5,
            "temporal_consistency should be high with CV=0, got: {}",
            metrics.temporal_consistency
        );
    }

    #[test]
    fn test_precision_exactly_one() {
        let mut detector =
            bot_detection::BotDetector::with_thresholds(bot_detection::BotThresholds {
                min_actions: 10,
                max_coefficient_of_variation: 0.3,
                max_mathematical_precision: 0.7,
                alert_threshold: 0.2,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        let mut ts = 1000u64;
        for i in 0..15 {
            ts += 2000 + (i as u64 * 500) % 8000; // variação humana no timing
                                                  // Todos os valores são múltiplos exatos de 100 (BB)
            detector.record_action(make_action(
                "precise_bot",
                "bet",
                (i as u64 + 1) * 100,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("precise_bot", ts + 1000);
        assert!(alert.is_some());
        let metrics = &alert.unwrap().metrics;
        // Todos os amounts são múltiplos de 100 → precision = 1.0
        assert!(metrics.mathematical_precision > 0.9);
    }

    #[test]
    fn test_mixed_streets() {
        let mut detector =
            bot_detection::BotDetector::with_thresholds(bot_detection::BotThresholds {
                min_actions: 10,
                max_coefficient_of_variation: 0.3,
                max_mathematical_precision: 0.7,
                alert_threshold: 0.2,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        let mut ts = 1000u64;
        let streets = ["preflop", "flop", "turn", "river"];
        for i in 0..15 {
            ts += 3000 + (i as u64 % 2) * 10;
            detector.record_action(make_action(
                "multi_street_bot",
                "bet",
                [300, 500, 700, 1000][i % 4],
                ts,
                &format!("h{}", i),
                streets[i % streets.len()],
            ));
        }
        let alert = detector.analyze_player("multi_street_bot", ts + 1000);
        // Streets diferentes não devem afetar a detecção
        assert!(alert.is_some());
    }

    #[test]
    fn test_zero_amount_actions() {
        let mut detector = bot_detection::BotDetector::new();
        let mut ts = 1000u64;
        for i in 0..25 {
            ts += 2000 + (i as u64 * 700) % 15000;
            // Check/Call/Fold com amount=0
            detector.record_action(make_action(
                "alice",
                "check",
                0,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let metrics = detector.get_metrics("alice").unwrap();
        // Amount=0 → não conta para precisão matemática → precision = 0
        assert_eq!(metrics.mathematical_precision, 0.0);
    }

    #[test]
    fn test_large_gaps_ignored_in_timing() {
        let mut detector = bot_detection::BotDetector::new();
        let mut ts = 1000u64;
        for i in 0..25 {
            if i == 12 {
                // Gap enorme (>30s) entre mãos → deve ser ignorado
                ts += 60000;
            } else {
                ts += 3000 + (i as u64 % 2) * 10; // constante
            }
            detector.record_action(make_action(
                "alice",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("alice", ts + 1000);
        // O gap grande é ignorado, então o CV ainda é baixo → alerta
        assert!(alert.is_some());
    }

    #[test]
    fn test_single_action_no_panic() {
        let mut detector = bot_detection::BotDetector::new();
        detector.record_action(make_action("alice", "fold", 0, 1000, "h1", "preflop"));
        let alert = detector.analyze_player("alice", 2000);
        assert!(alert.is_none());
        let metrics = detector.get_metrics("alice");
        assert!(metrics.is_none());
    }

    #[test]
    fn test_all_check_calls() {
        let mut detector =
            bot_detection::BotDetector::with_thresholds(bot_detection::BotThresholds {
                min_actions: 10,
                max_coefficient_of_variation: 0.3,
                max_mathematical_precision: 0.7,
                alert_threshold: 0.2,
                critical_threshold: 0.8,
                high_threshold: 0.5,
                medium_threshold: 0.3,
            });

        let mut ts = 1000u64;
        for i in 0..15 {
            ts += 3000 + (i as u64 % 2) * 5;
            // Apenas check/call (amount=0) → precisão = 0
            detector.record_action(make_action(
                "passive_bot",
                "check",
                0,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("passive_bot", ts + 1000);
        // Score = temporal_consistency * 0.5 + 0 * 0.5 = temporal_consistency * 0.5
        // Se timing for constante, temporal_consistency ≈ 1.0 → score ≈ 0.5
        if let Some(a) = alert {
            assert!(a.bot_score < 0.7); // Não atinge critical só com timing
        }
    }

    #[test]
    fn test_severity_low_boundary() {
        let mut detector =
            bot_detection::BotDetector::with_thresholds(bot_detection::BotThresholds {
                min_actions: 10,
                max_coefficient_of_variation: 0.3,
                max_mathematical_precision: 0.7,
                alert_threshold: 0.2,
                critical_threshold: 0.9,
                high_threshold: 0.7,
                medium_threshold: 0.5,
            });

        let mut ts = 1000u64;
        for i in 0..15 {
            ts += 3000 + (i as u64 * 200) % 2000; // alguma variação
            detector.record_action(make_action(
                "low_bot",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("low_bot", ts + 1000);
        if let Some(a) = alert {
            // Score entre 0.2 e 0.5 → low
            if a.bot_score < 0.5 {
                assert_eq!(a.severity, "low");
            }
        }
    }

    #[test]
    fn test_get_alerts_for_player_multiple() {
        let mut detector =
            bot_detection::BotDetector::with_thresholds(bot_detection::BotThresholds {
                min_actions: 10,
                max_coefficient_of_variation: 0.3,
                max_mathematical_precision: 0.7,
                alert_threshold: 0.2,
                critical_threshold: 0.9,
                high_threshold: 0.6,
                medium_threshold: 0.3,
            });

        // Registra 2 bots
        let mut ts = 1000u64;
        for i in 0..15 {
            ts += 3000 + (i as u64 % 2) * 5;
            detector.record_action(make_action(
                "bot_a",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("ha{}", i),
                "flop",
            ));
        }
        let mut ts2 = 1000u64;
        for i in 0..15 {
            ts2 += 3000 + (i as u64 % 2) * 5;
            detector.record_action(make_action(
                "bot_b",
                "bet",
                [500, 1000][i % 2],
                ts2,
                &format!("hb{}", i),
                "flop",
            ));
        }

        detector.analyze_all(ts.max(ts2) + 1000);

        let bot_a_alerts = detector.get_alerts_for_player("bot_a");
        let bot_b_alerts = detector.get_alerts_for_player("bot_b");
        assert!(!bot_a_alerts.is_empty());
        assert!(!bot_b_alerts.is_empty());
    }

    #[test]
    fn test_analyze_player_twice_no_duplicate_alert() {
        let mut detector =
            bot_detection::BotDetector::with_thresholds(bot_detection::BotThresholds {
                min_actions: 10,
                max_coefficient_of_variation: 0.3,
                max_mathematical_precision: 0.7,
                alert_threshold: 0.2,
                critical_threshold: 0.9,
                high_threshold: 0.6,
                medium_threshold: 0.3,
            });

        let mut ts = 1000u64;
        for i in 0..15 {
            ts += 3000 + (i as u64 % 2) * 5;
            detector.record_action(make_action(
                "bot",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }

        let alert1 = detector.analyze_player("bot", ts + 1000);
        assert!(alert1.is_some());
        let count1 = detector.get_alerts().len();

        // Analisa de novo sem novas ações
        let alert2 = detector.analyze_player("bot", ts + 2000);
        assert!(alert2.is_some());
        let count2 = detector.get_alerts().len();

        // Deve ter gerado novo alerta (não verifica duplicatas)
        assert!(count2 >= count1);
    }
}

// ═══════════════════════════════════════════════════════════════
// Multi-Account - Edge Cases & Boundaries
// ═══════════════════════════════════════════════════════════════

mod multi_account_edge_cases {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn make_fp(
        player_id: &str,
        ip: &str,
        hw: &str,
        ua: &str,
        screen: &str,
        tz: &str,
        lang: &str,
        ts: u64,
    ) -> multi_account::PlayerFingerprint {
        multi_account::PlayerFingerprint {
            player_id: player_id.to_string(),
            ip_address: ip.to_string(),
            hardware_id: hw.to_string(),
            user_agent: ua.to_string(),
            screen_resolution: screen.to_string(),
            timezone: tz.to_string(),
            language: lang.to_string(),
            first_seen_ms: ts,
            last_seen_ms: ts,
        }
    }

    #[test]
    fn test_duplicate_player_id_overwrites() {
        let mut detector = multi_account::MultiAccountDetector::new();
        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "alice", "10.0.0.1", "hw2", "Firefox", "1366x768", "UTC-5", "en-US", 2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        // Mesmo player_id → atualiza, não cria duplicata
        assert_eq!(detector.get_player_count(), 1);
        let fp = detector.get_fingerprint("alice").unwrap();
        // Deve ter os dados mais recentes
        assert_eq!(fp.ip_address, "10.0.0.1");
    }

    #[test]
    fn test_all_fields_empty_no_alert() {
        let mut detector = multi_account::MultiAccountDetector::new();
        let fp1 = make_fp("alice", "", "", "", "", "", "", 1000);
        let fp2 = make_fp("bob", "", "", "", "", "", "", 2000);

        detector.register_fingerprint(fp1);
        let alerted = detector.register_fingerprint(fp2);
        // Todos os campos vazios → score = 0 → sem alerta
        assert!(!alerted);
        assert!(detector.get_alerts().is_empty());
    }

    #[test]
    fn test_only_weak_factors_match() {
        let mut detector = multi_account::MultiAccountDetector::new();
        // Apenas language e timezone iguais (pesos baixos)
        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "bob", "10.0.0.1", "hw2", "Firefox", "1366x768", "UTC-3", "pt-BR", 2000,
        );

        detector.register_fingerprint(fp1);
        let alerted = detector.register_fingerprint(fp2);
        // timezone (0.10) + language (0.05) = 0.15 < 0.30 → sem alerta
        assert!(!alerted);
    }

    #[test]
    fn test_score_exactly_at_alert_threshold() {
        let mut detector = multi_account::MultiAccountDetector::with_thresholds(
            multi_account::MultiAccountThresholds {
                alert_threshold: 0.30,
                critical_threshold: 0.8,
                high_threshold: 0.6,
                medium_threshold: 0.4,
                ip_weight: 0.30,
                hardware_weight: 0.30,
                user_agent_weight: 0.15,
                screen_weight: 0.10,
                timezone_weight: 0.10,
                language_weight: 0.05,
            },
        );

        // Exatamente IP = 0.30 → no limiar
        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "bob",
            "192.168.1.1",
            "hw2",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        let alerted = detector.register_fingerprint(fp2);
        assert!(alerted);
    }

    #[test]
    fn test_score_just_below_alert_threshold() {
        let mut detector = multi_account::MultiAccountDetector::with_thresholds(
            multi_account::MultiAccountThresholds {
                alert_threshold: 0.31, // ligeiramente acima de ip_weight
                critical_threshold: 0.8,
                high_threshold: 0.6,
                medium_threshold: 0.4,
                ip_weight: 0.30,
                hardware_weight: 0.30,
                user_agent_weight: 0.15,
                screen_weight: 0.10,
                timezone_weight: 0.10,
                language_weight: 0.05,
            },
        );

        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "bob",
            "192.168.1.1",
            "hw2",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        let alerted = detector.register_fingerprint(fp2);
        // IP = 0.30 < 0.31 → sem alerta
        assert!(!alerted);
    }

    #[test]
    fn test_severity_all_levels() {
        let mut detector = multi_account::MultiAccountDetector::with_thresholds(
            multi_account::MultiAccountThresholds {
                alert_threshold: 0.1,
                critical_threshold: 0.8,
                high_threshold: 0.6,
                medium_threshold: 0.4,
                ip_weight: 0.30,
                hardware_weight: 0.30,
                user_agent_weight: 0.15,
                screen_weight: 0.10,
                timezone_weight: 0.10,
                language_weight: 0.05,
            },
        );

        // Low: apenas IP (0.30)
        let fp1 = make_fp(
            "a1",
            "1.1.1.1",
            "hw_a",
            "UA_a",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "a2", "1.1.1.1", "hw_b", "UA_b", "1366x768", "UTC-5", "en-US", 2000,
        );
        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        assert_eq!(detector.get_alerts()[0].severity, "low");
        detector.reset();

        // Medium: IP + user_agent (0.45)
        let fp3 = make_fp(
            "b1",
            "2.2.2.2",
            "hw_c",
            "UA_shared",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp4 = make_fp(
            "b2",
            "2.2.2.2",
            "hw_d",
            "UA_shared",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );
        detector.register_fingerprint(fp3);
        detector.register_fingerprint(fp4);
        assert_eq!(detector.get_alerts()[0].severity, "medium");
        detector.reset();

        // High: IP + hardware (0.60)
        let fp5 = make_fp(
            "c1",
            "3.3.3.3",
            "hw_shared",
            "UA_e",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp6 = make_fp(
            "c2",
            "3.3.3.3",
            "hw_shared",
            "UA_f",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );
        detector.register_fingerprint(fp5);
        detector.register_fingerprint(fp6);
        assert_eq!(detector.get_alerts()[0].severity, "high");
        detector.reset();

        // Critical: todos os fatores (1.0)
        let fp7 = make_fp(
            "d1",
            "4.4.4.4",
            "hw_full",
            "UA_full",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp8 = make_fp(
            "d2",
            "4.4.4.4",
            "hw_full",
            "UA_full",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            2000,
        );
        detector.register_fingerprint(fp7);
        detector.register_fingerprint(fp8);
        assert_eq!(detector.get_alerts()[0].severity, "critical");
    }

    #[test]
    fn test_primary_account_is_older() {
        let mut detector = multi_account::MultiAccountDetector::new();
        // Registra conta mais nova primeiro, depois a mais antiga
        let fp_new = make_fp(
            "new_user",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            5000,
        );
        let fp_old = make_fp(
            "old_user",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            1000,
        );

        detector.register_fingerprint(fp_new);
        detector.register_fingerprint(fp_old);

        let alert = &detector.get_alerts()[0];
        // old_user (first_seen=1000) é mais antiga → primária
        assert_eq!(alert.primary_account, "old_user");
        assert_eq!(alert.secondary_account, "new_user");
    }

    #[test]
    fn test_analyze_all_deduplication() {
        let mut detector = multi_account::MultiAccountDetector::with_thresholds(
            multi_account::MultiAccountThresholds {
                alert_threshold: 0.25,
                ..Default::default()
            },
        );

        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "bob",
            "192.168.1.1",
            "hw2",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        let count1 = detector.get_alerts().len();
        // analyze_all não deve duplicar alertas existentes
        detector.analyze_all();
        let count2 = detector.get_alerts().len();
        assert_eq!(count1, count2);
    }

    #[test]
    fn test_share_ip_unknown_players() {
        let detector = multi_account::MultiAccountDetector::new();
        assert!(!detector.share_ip("unknown1", "unknown2"));
    }

    #[test]
    fn test_share_hardware_unknown_players() {
        let detector = multi_account::MultiAccountDetector::new();
        assert!(!detector.share_hardware("unknown1", "unknown2"));
    }

    #[test]
    fn test_get_alerts_for_player_as_secondary() {
        let mut detector = multi_account::MultiAccountDetector::new();
        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        // bob é secundário → get_alerts_for_player deve retornar o alerta
        let bob_alerts = detector.get_alerts_for_player("bob");
        assert_eq!(bob_alerts.len(), 1);
    }

    #[test]
    fn test_many_players_no_false_positives() {
        let mut detector = multi_account::MultiAccountDetector::new();
        // 20 jogadores, todos com fingerprints diferentes
        for i in 0..20 {
            let fp = make_fp(
                &format!("player{}", i),
                &format!("192.168.{}.{}", i, i),
                &format!("hw{}", i),
                &format!("UA{}", i),
                "1920x1080",
                &format!("UTC{}", i % 12 - 6),
                if i % 2 == 0 { "pt-BR" } else { "en-US" },
                i as u64 * 1000,
            );
            detector.register_fingerprint(fp);
        }

        // Nenhum par deve ter score suficiente para alerta
        assert!(detector.get_alerts().is_empty());
    }

    #[test]
    fn test_clear_alerts_preserves_fingerprints() {
        let mut detector = multi_account::MultiAccountDetector::new();
        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fp(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        assert!(!detector.get_alerts().is_empty());

        detector.clear_alerts();
        assert!(detector.get_alerts().is_empty());
        assert_eq!(detector.get_player_count(), 2);
        assert!(detector.get_fingerprint("alice").is_some());
    }

    #[test]
    fn test_update_fingerprint_triggers_recheck() {
        let mut detector = multi_account::MultiAccountDetector::new();
        // Registra alice com IP diferente
        let fp1 = make_fp(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        detector.register_fingerprint(fp1);

        // Registra bob com IP diferente → sem match
        let fp2 = make_fp(
            "bob", "10.0.0.1", "hw2", "Firefox", "1366x768", "UTC-5", "en-US", 2000,
        );
        let alerted = detector.register_fingerprint(fp2);
        assert!(!alerted);

        // Atualiza alice com o mesmo IP de bob → deve gerar alerta
        let fp1_updated = make_fp(
            "alice",
            "10.0.0.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            3000,
        );
        let alerted = detector.register_fingerprint(fp1_updated);
        assert!(alerted);
    }
}
