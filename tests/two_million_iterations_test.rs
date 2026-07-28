use poker_engine::crypto::{DeckShuffler, ProvablyFairHand};
use poker_engine::engine::evaluator::{evaluate_hand, Card};
use poker_engine::engine::{
    calculate_loss_deflators, calculate_side_pots, Contribution, PlayerLossStats,
};
use std::time::Instant;

#[test]
fn test_2_point_1_million_fuzz_hands_and_engine_simulation() {
    println!("\n========================================================");
    println!("  INICIANDO SIMULAÇÃO MASSIVA DE 2.100.000 DE MÃOS (2.1M)");
    println!("========================================================\n");

    let total_iterations = 2_100_000;
    let start_time = Instant::now();

    let mut pf_hand = ProvablyFairHand::new("MassiveSeed_2M1", 1);
    let mut total_side_pots = 0u64;
    let mut total_cashbacks = 0u64;
    let mut total_hands = 0u64;

    for i in 1..=total_iterations {
        pf_hand.nonce = i as u64;

        // 1. Shuffler
        let deck = DeckShuffler::shuffle_deck(&pf_hand);
        assert_eq!(deck.len(), 52);

        // 2. Evaluator 7c
        let seven_cards: Vec<Card> = deck[0..7].to_vec();
        let _rank = evaluate_hand(&seven_cards);
        total_hands += 1;

        // 3. Side pots
        if i % 10 == 0 {
            let contributions = vec![
                Contribution {
                    player_id: "P1".into(),
                    total_bet: ((i % 100) * 10) as f64,
                    has_folded: i % 20 == 0,
                },
                Contribution {
                    player_id: "P2".into(),
                    total_bet: ((i % 50) * 20) as f64,
                    has_folded: i % 30 == 0,
                },
                Contribution {
                    player_id: "P3".into(),
                    total_bet: ((i % 80) * 15) as f64,
                    has_folded: false,
                },
            ];
            let pots = calculate_side_pots(&contributions);
            total_side_pots += pots.len() as u64;

            for pot in &pots {
                assert!(pot.amount > 0.0);
                for eligible in &pot.eligible_players {
                    let p = contributions
                        .iter()
                        .find(|c| &c.player_id == eligible)
                        .unwrap();
                    assert!(!p.has_folded);
                }
            }
        }

        // 4. Loss deflator
        if i % 50 == 0 {
            let stats = vec![
                PlayerLossStats {
                    player_id: "P1".into(),
                    total_bet: 500.0,
                    amount_won: if i % 100 == 0 { 0.0 } else { 800.0 },
                    cashback_tier_rate: 0.10,
                },
                PlayerLossStats {
                    player_id: "P2".into(),
                    total_bet: 300.0,
                    amount_won: 100.0,
                    cashback_tier_rate: 0.15,
                },
            ];
            let deflators = calculate_loss_deflators(&stats);
            total_cashbacks += deflators.len() as u64;

            for d in deflators {
                assert!(d.cashback_amount >= 0.0);
            }
        }
    }

    let elapsed = start_time.elapsed();
    let ops_per_sec = (total_iterations as f64) / elapsed.as_secs_f64();

    println!("   ✔ 2.100.000 de iterações concluídas com SUCESSO!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!(
        "   - Taxa de Processamento: {:.2} mãos/segundo",
        ops_per_sec
    );
    println!("   - Mãos Avaliadas: {}", total_hands);
    println!("   - Potes Secundários: {}", total_side_pots);
    println!("   - Cashbacks Calculados: {}", total_cashbacks);
    println!("========================================================\n");

    assert_eq!(total_hands, 2_100_000);
}
