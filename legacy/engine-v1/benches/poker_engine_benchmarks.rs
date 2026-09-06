use poker_engine::crypto::{DeckShuffler, ProvablyFairHand};
use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{calculate_side_pots, Contribution};
use poker_engine::ledger::{EntryType, LedgerAccount};
use poker_engine::tournament::{TableBalancer, TableStateSummary};
use std::time::Instant;

fn main() {
    println!("\n========================================================");
    println!("  MICRO-BENCHMARKS DE LATÊNCIA EXTREMA (RELEASE MODE) ");
    println!("========================================================\n");

    // 1. Benchmark: Avaliador de 7 Cartas Texas Hold'em
    let hole_and_board = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
    ];

    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = evaluate_hand(&hole_and_board);
    }
    let elapsed = start.elapsed();
    let nanos_per_eval = elapsed.as_nanos() as f64 / iterations as f64;
    println!("1. Avaliador de 7 Cartas (`evaluate_hand`):");
    println!(
        "   - Latência Médias: {:.2} ns ({:.3} µs) por avaliação",
        nanos_per_eval,
        nanos_per_eval / 1000.0
    );
    println!(
        "   - Throughput: {:.2} avaliações/segundo\n",
        (iterations as f64) / elapsed.as_secs_f64()
    );

    // 2. Benchmark: Cálculo de Potes Secundários (Side Pots)
    let contributions = vec![
        Contribution {
            player_id: "P1".into(),
            total_bet: 100.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P2".into(),
            total_bet: 500.0,
            has_folded: true,
        },
        Contribution {
            player_id: "P3".into(),
            total_bet: 500.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P4".into(),
            total_bet: 1000.0,
            has_folded: false,
        },
    ];

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = calculate_side_pots(&contributions);
    }
    let elapsed = start.elapsed();
    let nanos_per_side_pot = elapsed.as_nanos() as f64 / iterations as f64;
    println!("2. Cálculo de Side Pots (`calculate_side_pots`):");
    println!(
        "   - Latência Média: {:.2} ns ({:.3} µs) por cálculo",
        nanos_per_side_pot,
        nanos_per_side_pot / 1000.0
    );
    println!(
        "   - Throughput: {:.2} cálculos/segundo\n",
        (iterations as f64) / elapsed.as_secs_f64()
    );

    // 3. Benchmark: Transação Criptográfica de Ledger SHA-256
    let account = LedgerAccount::new("Bench_User", 1_000_000);
    let start = Instant::now();
    for i in 0..10_000 {
        let _ = account.record_transaction(100, EntryType::PotWin, Some(format!("TX-{}", i)));
    }
    let elapsed = start.elapsed();
    let nanos_per_tx = elapsed.as_nanos() as f64 / 10_000.0;
    println!("3. Transação Criptográfica do Ledger (`record_transaction`):");
    println!(
        "   - Latência Média: {:.2} ns ({:.3} µs) por transação",
        nanos_per_tx,
        nanos_per_tx / 1000.0
    );
    println!(
        "   - Throughput: {:.2} transações/segundo\n",
        10_000.0 / elapsed.as_secs_f64()
    );

    // 4. Benchmark: Reconstrução Provably Fair ChaCha8
    let pf_hand = ProvablyFairHand::new("Bench_Client_Seed", 100);
    let start = Instant::now();
    for _ in 0..10_000 {
        let _ = DeckShuffler::shuffle_deck(&pf_hand);
    }
    let elapsed = start.elapsed();
    let nanos_per_shuffle = elapsed.as_nanos() as f64 / 10_000.0;
    println!("4. Reconstrução Provably Fair ChaCha8 (`shuffle_deck`):");
    println!(
        "   - Latência Média: {:.2} ns ({:.3} µs) por embaralhamento",
        nanos_per_shuffle,
        nanos_per_shuffle / 1000.0
    );
    println!(
        "   - Throughput: {:.2} embaralhamentos/segundo\n",
        10_000.0 / elapsed.as_secs_f64()
    );

    // 5. Benchmark: Balanceador Dinâmico de Mesas de Torneio
    let tables = vec![
        TableStateSummary {
            table_id: "T1".into(),
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
            table_id: "T2".into(),
            active_player_ids: vec!["P7".into(), "P8".into(), "P9".into()],
        },
    ];
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = TableBalancer::balance_tables(&tables);
    }
    let elapsed = start.elapsed();
    let nanos_per_balance = elapsed.as_nanos() as f64 / iterations as f64;
    println!("5. Balanceamento Dinâmico de Mesas (`balance_tables`):");
    println!(
        "   - Latência Média: {:.2} ns ({:.3} µs) por análise",
        nanos_per_balance,
        nanos_per_balance / 1000.0
    );
    println!(
        "   - Throughput: {:.2} análises/segundo",
        (iterations as f64) / elapsed.as_secs_f64()
    );

    println!("========================================================\n");
}
