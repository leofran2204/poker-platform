use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{calculate_side_pots, Contribution};
use poker_engine::ledger::{EntryType, LedgerAccount};
use std::time::Instant;

#[test]
fn test_sla_latency_sub_millisecond_release() {
    println!("\n========================================================");
    println!(" SLA LATENCY SUITE VALIDATION (SUB-MILLISECOND SLA) ");
    println!("========================================================\n");

    // 1. SLA Check: Hand Evaluation < 50 µs
    let hole_and_board = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
    ];
    let start = Instant::now();
    for _ in 0..1_000 {
        let _ = evaluate_hand(&hole_and_board);
    }
    let micros_eval = start.elapsed().as_micros() as f64 / 1_000.0;
    let max_allowed = if cfg!(debug_assertions) { 150.0 } else { 50.0 };
    println!("   ✔ Hand Evaluation SLA: {:.3} µs (SLA < {:.0} µs)", micros_eval, max_allowed);
    assert!(micros_eval < max_allowed, "Hand eval violou o SLA de {}µs", max_allowed);

    // 2. SLA Check: Side Pots Calculation < 10 µs
    let contributions = vec![
        Contribution { player_id: "P1".into(), total_bet: 100.0, has_folded: false },
        Contribution { player_id: "P2".into(), total_bet: 500.0, has_folded: true },
        Contribution { player_id: "P3".into(), total_bet: 500.0, has_folded: false },
    ];
    let start = Instant::now();
    for _ in 0..1_000 {
        let _ = calculate_side_pots(&contributions);
    }
    let micros_side = start.elapsed().as_micros() as f64 / 1_000.0;
    println!("   ✔ Side Pots SLA: {:.3} µs (SLA < 10 µs)", micros_side);
    assert!(micros_side < 10.0, "Side pots violou o SLA de 10µs");

    // 3. SLA Check: Ledger Transaction < 30 µs
    let account = LedgerAccount::new("SLA_User", 10000);
    let start = Instant::now();
    for _ in 0..100 {
        let _ = account.record_transaction(10, EntryType::Deposit, None);
    }
    let micros_ledger = start.elapsed().as_micros() as f64 / 100.0;
    println!("   ✔ Ledger SHA-256 SLA: {:.3} µs (SLA < 30 µs)", micros_ledger);
    assert!(micros_ledger < 30.0, "Ledger tx violou o SLA de 30µs");

    println!("========================================================\n");
}
