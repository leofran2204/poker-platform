use chrono::Utc;
use poker_engine::engine::evaluator::{Card, Rank, Suit};
use poker_engine::engine::Action;
use poker_engine::history::{
    HandHistoryRecord, HandPlayerInfo, HandWinnerInfo, RecordedAction,
};
use sha2::{Digest, Sha256};

#[test]
fn test_hand_history_export_pokerstars_format() {
    let server_seed = "SecretServerSeed_123456789".to_string();
    let server_seed_hash = hex::encode(Sha256::digest(server_seed.as_bytes()));

    let record = HandHistoryRecord {
        hand_id: "HAND-999".into(),
        table_id: "Table_High_Rollers".into(),
        timestamp: Utc::now(),
        small_blind: 5.0,
        big_blind: 10.0,
        server_seed,
        server_seed_hash,
        client_seed: "ClientSeed_ABC".into(),
        nonce: 42,
        players: vec![
            HandPlayerInfo {
                player_id: "P1".into(),
                name: "Alice".into(),
                starting_stack: 1000.0,
                hole_cards: Some(vec![
                    Card::new(Rank::Ace, Suit::Spades),
                    Card::new(Rank::Ace, Suit::Hearts),
                ]),
            },
            HandPlayerInfo {
                player_id: "P2".into(),
                name: "Bob".into(),
                starting_stack: 800.0,
                hole_cards: None,
            },
        ],
        community_cards: vec![
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Queen, Suit::Diamonds),
            Card::new(Rank::Jack, Suit::Spades),
        ],
        actions: vec![RecordedAction {
            player_id: "P1".into(),
            action: Action::Bet(50.0),
            stage: "Flop".into(),
        }],
        winners: vec![HandWinnerInfo {
            player_id: "P1".into(),
            amount_won: 115.0,
            hand_description: "Par de Ases".into(),
        }],
    };

    let exported_text = record.export_pokerstars_format();
    assert!(exported_text.contains("Poker Hand #HAND-999"));
    assert!(exported_text.contains("Alice (R$1000.00 in chips)"));
    assert!(exported_text.contains("*** HOLE CARDS ***"));
    assert!(exported_text.contains("Provably Fair Server Seed: SecretServerSeed_123456789"));

    assert!(record.verify_provably_fair());
}
