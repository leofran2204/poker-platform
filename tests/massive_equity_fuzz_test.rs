use poker_engine::analytics::EquityCalculator;
use poker_engine::engine::evaluator::{Card, Rank, Suit};
use std::time::Instant;

#[test]
fn test_100k_monte_carlo_equity_simulations_stress() {
    println!("\n========================================================");
    println!(" INICIANDO SIMULAÇÃO MASSIVA DE EQUIDADE MONTE CARLO (100K) ");
    println!("========================================================\n");

    let player_cards = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Spades), // AK Suited
    ];
    let board = vec![
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Jack, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs), // Flush Draw + Nut Straight Draw (Royal Flush Draw)
    ];

    let start_time = Instant::now();
    let result = EquityCalculator::calculate_equity(&player_cards, &board, 2, 100_000);
    let elapsed = start_time.elapsed();

    println!("   ✔ 100.000 simulações de Monte Carlo concluídas!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!(
        "   - Taxa: {:.2} simulações/segundo",
        100_000.0 / elapsed.as_secs_f64()
    );
    println!("   - AKs no Flop com Royal Draw vs 2 Oponentes:");
    println!("     - Win Rate:  {:.2}%", result.win_percentage);
    println!("     - Tie Rate:  {:.2}%", result.tie_percentage);
    println!("     - Loss Rate: {:.2}%", result.loss_percentage);
    println!("========================================================\n");

    // AKs com Royal Flush Draw no Flop deve ter equidade dominante (> 45% contra 2 oponentes aleatórios)
    assert!(result.win_percentage > 40.0);
    assert_eq!(result.total_simulations, 100_000);
}
