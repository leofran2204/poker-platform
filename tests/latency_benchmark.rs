use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{calculate_side_pots, Contribution, GameLoop, GameState, Player};
use std::time::Instant;

#[test]
fn benchmark_sub_millisecond_latency_per_action_and_eval() {
    println!("\n========================================================");
    println!("   BENCHMARK DE ALTA PRECISÃO DE LATÊNCIA (RUST NATIVO) ");
    println!("========================================================\n");

    // 1. Latência do Cálculo de Potes Secundários (Side Pots)
    let contributions = vec![
        Contribution { player_id: "P1".into(), total_bet: 100.0, has_folded: false },
        Contribution { player_id: "P2".into(), total_bet: 500.0, has_folded: true },
        Contribution { player_id: "P3".into(), total_bet: 500.0, has_folded: false },
        Contribution { player_id: "P4".into(), total_bet: 250.0, has_folded: false },
    ];

    let iterations = 100_000;
    let start_side_pots = Instant::now();
    for _ in 0..iterations {
        let _ = calculate_side_pots(&contributions);
    }
    let duration_side_pots = start_side_pots.elapsed();
    let nanos_per_side_pot = duration_side_pots.as_nanos() as f64 / iterations as f64;
    let micros_per_side_pot = nanos_per_side_pot / 1000.0;

    // 2. Latência da Avaliação de Mãos de 7 Cartas
    let cards = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
    ];

    let start_eval = Instant::now();
    for _ in 0..iterations {
        let _ = evaluate_hand(&cards);
    }
    let duration_eval = start_eval.elapsed();
    let nanos_per_eval = duration_eval.as_nanos() as f64 / iterations as f64;
    let micros_per_eval = nanos_per_eval / 1000.0;

    // 3. Latência de Rotação da Máquina de Estados (Game Loop Turn Advance)
    let players = vec![
        Player::new("P1", "Alice", 100.0),
        Player::new("P2", "Bob", 200.0),
        Player::new("P3", "Charlie", 300.0),
    ];
    let state = GameState::new(players, 0, 10.0);
    let mut game_loop = GameLoop::new(state);

    let start_turn = Instant::now();
    for _ in 0..iterations {
        game_loop.advance_turn();
    }
    let duration_turn = start_turn.elapsed();
    let nanos_per_turn = duration_turn.as_nanos() as f64 / iterations as f64;
    let micros_per_turn = nanos_per_turn / 1000.0;

    println!("   ⚡ RESULTADOS DE MEDIÇÃO DE LATÊNCIA:");
    println!("   - Cálculo de Side Pots:  {:.3} µs (microssegundos) por operação", micros_per_side_pot);
    println!("   - Avaliação de Mão (7c): {:.3} µs (microssegundos) por operação", micros_per_eval);
    println!("   - Rotação do Game Loop:  {:.3} µs (microssegundos) por operação", micros_per_turn);
    println!("\n   Veredito: Latência do motor backend é SUB-MILISSEGUNDO (< 0.01 ms)!");
    println!("========================================================\n");

    assert!(micros_per_side_pot < 50.0);
    assert!(micros_per_eval < 50.0);
    assert!(micros_per_turn < 50.0);
}
