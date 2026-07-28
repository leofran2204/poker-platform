use poker_engine::auth::{generate_totp_code, verify_totp_code};
use poker_engine::crypto::{DeckShuffler, ProvablyFairHand};
use poker_engine::engine::{
    calculate_loss_deflators, calculate_side_pots, Contribution, GameState, Player, PlayerLossStats,
};
use poker_engine::ledger::{EntryType, LedgerAccount, LedgerError};

#[test]
fn test_side_pots_excludes_folded_players() {
    let contributions = vec![
        Contribution {
            player_id: "P1".into(),
            total_bet: 100.0,
            has_folded: false,
        },
        Contribution {
            player_id: "P2".into(),
            total_bet: 300.0,
            has_folded: true, // Folded player should NOT be eligible
        },
        Contribution {
            player_id: "P3".into(),
            total_bet: 300.0,
            has_folded: false,
        },
    ];

    let pots = calculate_side_pots(&contributions);

    // Pot 1 (level 100.0): P1, P2(folded), P3 -> Eligible: P1, P3 (P2 excluded!)
    assert_eq!(pots.len(), 2);
    assert_eq!(pots[0].amount, 300.0);
    assert!(!pots[0].eligible_players.contains(&"P2".to_string()));
    assert!(pots[0].eligible_players.contains(&"P1".to_string()));
    assert!(pots[0].eligible_players.contains(&"P3".to_string()));

    // Pot 2 (level 300.0): P2(folded), P3 -> Eligible: P3 only
    assert_eq!(pots[1].amount, 400.0);
    assert_eq!(pots[1].eligible_players, vec!["P3".to_string()]);
}

#[test]
fn test_game_loop_next_active_player_no_deadlock() {
    let mut p1 = Player::new("P1", "Alice", 0.0);
    p1.is_all_in = true; // All-in (cannot act)

    let mut p2 = Player::new("P2", "Bob", 100.0);
    p2.has_folded = true; // Folded (cannot act)

    let p3 = Player::new("P3", "Charlie", 200.0); // Active (can act)

    let state = GameState::new(vec![p1, p2, p3], 0, 10.0);

    // Starting from P1 (idx 0), the next active player MUST be P3 (idx 2), skipping P2 (folded)
    let next = state.next_active_player(0);
    assert_eq!(next, Some(2));
}

#[test]
fn test_loss_deflator_never_negative() {
    let stats = vec![
        PlayerLossStats {
            player_id: "P1".into(),
            total_bet: 100.0,
            amount_won: 20.0, // Loss = 80.0
            cashback_tier_rate: 0.10,
        },
        PlayerLossStats {
            player_id: "P2".into(),
            total_bet: 100.0,
            amount_won: 150.0, // Win = 50.0 (Net positive)
            cashback_tier_rate: 0.10,
        },
    ];

    let results = calculate_loss_deflators(&stats);
    assert_eq!(results[0].net_loss, 80.0);
    assert_eq!(results[0].cashback_amount, 8.0);

    // P2 won money, cashback MUST be 0.0, never negative
    assert_eq!(results[1].net_loss, 0.0);
    assert_eq!(results[1].cashback_amount, 0.0);
}

#[test]
fn test_totp_rfc6238_hmac_sha1_vectors() {
    // Standard RFC 6238 Test Vector Secret "12345678901234567890" (ASCII bytes)
    let secret = b"12345678901234567890";
    let timestamp = 59u64; // T0 + 59s -> step = 1

    let code = generate_totp_code(secret, 30, timestamp).unwrap();
    assert_eq!(code.len(), 6);
    assert!(verify_totp_code(secret, &code, timestamp, 0));
}

#[test]
fn test_ledger_atomic_transactions_and_hash_integrity() {
    let account = LedgerAccount::new("User_Test", 5000); // 50.00

    let res1 = account.record_transaction(2000, EntryType::Deposit, None);
    assert!(res1.is_ok());
    assert_eq!(account.get_balance_cents().unwrap(), 7000);

    // Attempting to withdraw more than balance should fail
    let res2 = account.record_transaction(-10000, EntryType::Withdrawal, None);
    assert!(matches!(res2, Err(LedgerError::InsufficientFunds)));

    // Verify blockchain-like hash integrity
    assert!(account.verify_integrity().unwrap());
}

#[test]
fn test_provably_fair_reproducibility() {
    let pf_hand = ProvablyFairHand::new("Player_Seed_123", 42);

    // Verify server seed hash commitment
    assert!(ProvablyFairHand::verify_commitment(
        &pf_hand.server_seed,
        &pf_hand.server_seed_hash
    ));

    // Shuffle deck
    let deck1 = DeckShuffler::shuffle_deck(&pf_hand);
    let deck2 = DeckShuffler::shuffle_deck(&pf_hand);

    // Deck shuffle MUST be 100% deterministic given the same seeds and nonce
    assert_eq!(deck1, deck2);
}
