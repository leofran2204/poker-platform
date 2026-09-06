use poker_engine::crypto::{DeckShuffler, ProvablyFairHand};
use poker_engine::engine::evaluator::{evaluate_hand, Card};
use poker_engine::engine::{
    calculate_loss_deflators, calculate_side_pots, Contribution, PlayerLossStats,
};
use std::time::Instant;

#[test]
fn test_1_million_fuzz_hand_evaluations_and_poker_engine_simulations() {
    println!("\n========================================================");
    println!("    INICIANDO SIMULAÇÃO MASSIVA DE 1.000.000 DE MÃOS    ");
    println!("========================================================\n");

    let total_iterations = 1_000_000;
    let start_time = Instant::now();

    let mut pf_hand = ProvablyFairHand::new("MassiveSeed_1M", 1);
    let mut total_side_pots_calculated = 0u64;
    let mut total_cashbacks_calculated = 0u64;
    let mut total_hands_evaluated = 0u64;

    for i in 1..=total_iterations {
        pf_hand.nonce = i as u64;

        // 1. Embaralhar baralho Provably Fair (ChaCha8 RNG)
        let deck = DeckShuffler::shuffle_deck(&pf_hand);
        assert_eq!(deck.len(), 52, "Baralho deve conter exatamente 52 cartas");

        // 2. Extrair 7 cartas para avaliação de mão (2 da mão + 5 comunitárias)
        let seven_cards: Vec<Card> = deck[0..7].to_vec();
        let _hand_rank = evaluate_hand(&seven_cards);
        total_hands_evaluated += 1;

        // 3. Simular apostas e Side Pots a cada 10 iterações
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

            let side_pots = calculate_side_pots(&contributions);
            total_side_pots_calculated += side_pots.len() as u64;

            for pot in &side_pots {
                assert!(pot.amount > 0.0, "Pote com valor <= 0.0 na iteração {}", i);
                for eligible_id in &pot.eligible_players {
                    let p = contributions
                        .iter()
                        .find(|c| &c.player_id == eligible_id)
                        .unwrap();
                    assert!(
                        !p.has_folded,
                        "VIOLAÇÃO CRÍTICA: Jogador folded elegível no pote!"
                    );
                }
            }
        }

        // 4. Simular Loss Deflator a cada 50 iterações
        if i % 50 == 0 {
            let stats = vec![
                PlayerLossStats {
                    player_id: "P1".into(),
                    eligible_loss_after_rake: if i % 100 == 0 { 500.0 } else { 0.0 },
                    loser_equity: 0.60,
                },
                PlayerLossStats {
                    player_id: "P2".into(),
                    eligible_loss_after_rake: 200.0,
                    loser_equity: 0.70,
                },
            ];

            let deflators = calculate_loss_deflators(&stats);
            total_cashbacks_calculated += deflators.len() as u64;

            for d in deflators {
                assert!(
                    d.cashback_amount >= 0.0,
                    "Cashback negativo na iteração {}",
                    i
                );
            }
        }
    }

    let elapsed = start_time.elapsed();
    let ops_per_sec = (total_iterations as f64) / elapsed.as_secs_f64();

    println!("   ✔ 1.000.000 de iterações concluídas com SUCESSO ABSOLUTO!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!(
        "   - Velocidade de Processamento: {:.2} mãos/segundo",
        ops_per_sec
    );
    println!("   - Mãos Avaliadas: {}", total_hands_evaluated);
    println!(
        "   - Potes Secundários Calculados: {}",
        total_side_pots_calculated
    );
    println!(
        "   - Cálculos de Loss Deflator: {}",
        total_cashbacks_calculated
    );
    println!("\n========================================================\n");

    assert_eq!(total_hands_evaluated, 1_000_000);
}
