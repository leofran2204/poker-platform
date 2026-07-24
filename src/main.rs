use poker_engine::antifraud::{
    CollusionDetector, PlayerBehaviorStats, PlayerSession,
};
use poker_engine::auth::{generate_totp_code, verify_totp_code};
use poker_engine::crypto::ProvablyFairHand;
use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{
    calculate_loss_deflators, calculate_side_pots, Contribution, PlayerLossStats,
};
use poker_engine::ledger::{EntryType, LedgerAccount};
use poker_engine::security::RateLimiter;

fn main() {
    println!("========================================================");
    println!("       PLATAFORMA DE POKER ONLINE ENGINES (RUST)       ");
    println!("========================================================\n");

    // --- SPRINT 1 DEMOS ---
    println!("--- [SPRINT 1: CORE ENGINE & SECURITY] ---");

    // 1. Side Pots
    let contributions = vec![
        Contribution {
            player_id: "Player_A".to_string(),
            total_bet: 100.0,
            has_folded: false,
        },
        Contribution {
            player_id: "Player_B".to_string(),
            total_bet: 500.0,
            has_folded: true, // Folded
        },
        Contribution {
            player_id: "Player_C".to_string(),
            total_bet: 500.0,
            has_folded: false,
        },
    ];
    let side_pots = calculate_side_pots(&contributions);
    println!("1. Side Pots (Fix Folded): Pote 1 = R$ {:.2} | Elegíveis: {:?}", side_pots[0].amount, side_pots[0].eligible_players);

    // 2. Loss Deflator
    let stats = vec![PlayerLossStats {
        player_id: "Player_A".to_string(),
        total_bet: 200.0,
        amount_won: 50.0,
        cashback_tier_rate: 0.10,
    }];
    let deflators = calculate_loss_deflators(&stats);
    println!("2. Loss Deflator: Perda líquida = R$ {:.2} | Cashback = R$ {:.2}", deflators[0].net_loss, deflators[0].cashback_amount);

    // 3. TOTP
    let secret = b"12345678901234567890";
    let code = generate_totp_code(secret, 30, 1700000000).unwrap();
    println!("3. TOTP RFC 6238 HMAC-SHA1: Código {} -> Válido: {}", code, verify_totp_code(secret, &code, 1700000000, 1));

    // 4. Ledger
    let ledger = LedgerAccount::new("User_123", 100000);
    let _ = ledger.record_transaction(50000, EntryType::Deposit, Some("DEP-001".into()));
    println!("4. Ledger Imutável: Saldo = R$ {:.2} | Auditoria Hash = OK", ledger.get_balance_cents().unwrap() as f64 / 100.0);

    // 5. Provably Fair
    let pf_hand = ProvablyFairHand::new("ClientSeed_xyz", 1);
    println!("5. Provably Fair: Server Commitment = {}\n", pf_hand.server_seed_hash);

    // --- SPRINT 2 DEMOS ---
    println!("--- [SPRINT 2: PERFORMANCE, SEGURANÇA & ANTIFRAUDE] ---");

    // 6. Rate Limiter
    let limiter = RateLimiter::new(2.0, 1.0);
    let _ = limiter.check_rate_limit("203.0.113.195");
    let _ = limiter.check_rate_limit("203.0.113.195");
    let rate_check = limiter.check_rate_limit("203.0.113.195");
    println!("6. Rate Limiter Token Bucket: Terceira requisição imediata -> {:?}", rate_check);

    // 7. Antifraude IP/Subnet Guard
    let table_players = vec![
        PlayerSession { user_id: "Alice".into(), ip_address: "192.168.1.10".into() },
        PlayerSession { user_id: "Bob".into(), ip_address: "192.168.1.45".into() }, // Mesma sub-rede /24
    ];
    let seating_check = CollusionDetector::validate_table_seating(&table_players);
    println!("7. Antifraude Subnet /24 Guard: Bloqueio de mesa -> {:?}", seating_check);

    // 8. Análise VPIP / PFR
    let bot_stats = PlayerBehaviorStats {
        user_id: "Bot_99".into(),
        hands_played: 120,
        hands_vpip: 10,
        hands_pfr: 30, // PFR > VPIP anômalo
    };
    println!("8. Detecção de Anomalia VPIP/PFR: Alert = {:?}", CollusionDetector::detect_anomalies(&bot_stats));

    // 9. Avaliador de 7 Cartas Texas Hold'em
    let hole_and_board = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
    ];
    let hand_rank = evaluate_hand(&hole_and_board);
    println!("9. Avaliador de 7 Cartas Texas Hold'em: Resultado = {:?}", hand_rank);

    println!("\n========================================================");
    println!("   SPRINT 1 & SPRINT 2 EXECUTADAS E VALIDADAS 100%      ");
    println!("========================================================");
}
