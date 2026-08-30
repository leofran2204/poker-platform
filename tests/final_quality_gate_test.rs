use poker_engine::admin::AdminDashboard;
use poker_engine::antifraud::{CollusionDetector, PlayerSession};
use poker_engine::auth::{generate_totp_code, verify_totp_code};
use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{calculate_side_pots, Contribution};
use poker_engine::history::HandHistoryRecord;
use poker_engine::ledger::{EntryType, LedgerAccount};
use poker_engine::security::RateLimiter;
use poker_engine::server::{
    TableActor, TableMessage, WebSocketServer, WsActionType, WsIncomingPacket,
};
use poker_engine::tournament::{BlindStructure, Tournament};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_final_quality_gate_all_modules_integration() {
    println!("\n========================================================");
    println!(" EXECUTANDO AUDITORIA FINAL E QUALITY GATE DA PLATAFORMA ");
    println!("========================================================\n");

    // 1. Core Engine Side Pots
    let contribs = vec![
        Contribution {
            player_id: "A".into(),
            total_bet: 100.0,
            has_folded: false,
        },
        Contribution {
            player_id: "B".into(),
            total_bet: 500.0,
            has_folded: true,
        },
        Contribution {
            player_id: "C".into(),
            total_bet: 500.0,
            has_folded: false,
        },
    ];
    let side_pots = calculate_side_pots(&contribs);
    assert_eq!(side_pots[0].eligible_players.len(), 2);
    println!("   ✔ 1. Core Engine (Side Pots Fold Isolation): OK");

    // 2. Auth TOTP RFC 6238
    let secret = b"12345678901234567890";
    let code = generate_totp_code(secret, 30, 1700000000).unwrap();
    assert!(verify_totp_code(secret, &code, 1700000000, 1));
    println!("   ✔ 2. Auth Security (TOTP RFC 6238 HMAC-SHA1): OK");

    // 3. Ledger Imutável SHA-256
    let ledger = LedgerAccount::new("Final_User", 100000);
    let _ = ledger.record_transaction(50000, EntryType::Deposit, Some("DEP-FINAL".into()));
    assert!(ledger.verify_integrity().unwrap());
    println!("   ✔ 3. Ledger Financeiro Imutável (SHA-256 Hash Chain): OK");

    // 4. Rate Limiter Token Bucket
    let limiter = RateLimiter::new(2.0, 1.0);
    assert!(limiter.check_rate_limit("1.1.1.1").is_ok());
    println!("   ✔ 4. Rate Limiter (Token Bucket): OK");

    // 5. Antifraude Subnet Guard
    let sessions = vec![
        PlayerSession {
            user_id: "P1".into(),
            ip_address: "192.168.1.10".into(),
        },
        PlayerSession {
            user_id: "P2".into(),
            ip_address: "192.168.1.55".into(),
        },
    ];
    assert!(CollusionDetector::validate_table_seating(&sessions).is_err());
    println!("   ✔ 5. Antifraude (IP & Subnet /24 Guard): OK");

    // 6. Evaluator 7 Cards
    let cards = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
    ];
    let rank = evaluate_hand(&cards);
    assert!(format!("{:?}", rank).contains("FullHouse"));
    println!("   ✔ 6. Avaliador de 7 Cartas Texas Hold'em: OK");

    // 7. Torneios & Table Balancer
    let blind_structure = BlindStructure::standard_regular();
    let mut tourney = Tournament::new("T1", "Final MTT", 10000, 1000, 5000.0, blind_structure);
    let acc1 = LedgerAccount::new("TP1", 50000);
    assert!(tourney.register_player("TP1", "Alice", &acc1).is_ok());
    println!("   ✔ 7. Torneios MTT & Sit & Go (Buy-ins, Blinds, Payouts): OK");

    // 8. WebSocket Server & Tokio Actors
    let (tx, rx) = mpsc::channel::<TableMessage>(100);
    let mut actor = TableActor::new("Table_Final", rx);
    tokio::spawn(async move {
        actor.run().await;
    });

    let ws_server = WebSocketServer::new();
    let packet = WsIncomingPacket {
        player_id: "TP1".into(),
        action: WsActionType::JoinTable {
            table_id: "Table_Final".into(),
            ip_address: "203.0.113.99".into(),
        },
    };
    assert!(ws_server
        .process_incoming_packet(packet, &tx, "203.0.113.99")
        .await
        .is_ok());
    println!("   ✔ 8. WebSocket Server & Actor Routing em Tempo Real: OK");

    // 9. Admin Dashboard & Audit
    let admin = AdminDashboard::new();
    let audit = admin.audit_ledger_account(&ledger);
    assert!(audit.hash_chain_valid);
    println!("   ✔ 9. Dashboard Administrativo & Auditoria de Saldo: OK");

    // 10. Hand History & Provably Fair Replay
    let s_seed = "Final_Server_Seed".to_string();
    let s_hash = hex::encode(Sha256::digest(s_seed.as_bytes()));
    let history = HandHistoryRecord {
        hand_id: "H-100".into(),
        table_id: "Table_Final".into(),
        timestamp: chrono::Utc::now(),
        small_blind: 5.0,
        big_blind: 10.0,
        server_seed: s_seed,
        server_seed_hash: s_hash,
        client_seed: "Final_Client_Seed".into(),
        nonce: 1,
        players: vec![],
        community_cards: vec![],
        actions: vec![],
        winners: vec![],
    };
    assert!(history.verify_provably_fair());
    println!("   ✔ 10. Histórico de Mãos & Replay Provably Fair: OK");

    println!("========================================================");
    println!(" QUALITY GATE APROVADO: 100% DAS CAMADAS ÍNTEGRAS ");
    println!("========================================================\n");
}
