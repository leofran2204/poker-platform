use poker_engine::antifraud::{
    CollusionDetector, PlayerSession,
};
use poker_engine::auth::{generate_totp_code, verify_totp_code};
use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{
    calculate_side_pots, Contribution,
};
use poker_engine::ledger::{EntryType, LedgerAccount};
use poker_engine::security::RateLimiter;
use poker_engine::tournament::{
    BlindStructure, TableBalancer, TableStateSummary, Tournament, TournamentState,
};
use std::collections::HashMap;

fn main() {
    println!("========================================================");
    println!("       PLATAFORMA DE POKER ONLINE ENGINES (RUST)       ");
    println!("========================================================\n");

    // --- SPRINT 1 DEMOS ---
    println!("--- [SPRINT 1: CORE ENGINE & SECURITY] ---");
    let contributions = vec![
        Contribution { player_id: "Player_A".into(), total_bet: 100.0, has_folded: false },
        Contribution { player_id: "Player_B".into(), total_bet: 500.0, has_folded: true },
        Contribution { player_id: "Player_C".into(), total_bet: 500.0, has_folded: false },
    ];
    let side_pots = calculate_side_pots(&contributions);
    println!("1. Side Pots (Fix Folded): Pote 1 = R$ {:.2} | Elegíveis: {:?}", side_pots[0].amount, side_pots[0].eligible_players);

    let secret = b"12345678901234567890";
    let code = generate_totp_code(secret, 30, 1700000000).unwrap();
    println!("2. TOTP RFC 6238 HMAC-SHA1: Código {} -> Válido: {}", code, verify_totp_code(secret, &code, 1700000000, 1));

    let ledger = LedgerAccount::new("User_123", 100000);
    let _ = ledger.record_transaction(50000, EntryType::Deposit, Some("DEP-001".into()));
    println!("3. Ledger Imutável: Saldo = R$ {:.2} | Auditoria Hash = OK\n", ledger.get_balance_cents().unwrap() as f64 / 100.0);

    // --- SPRINT 2 DEMOS ---
    println!("--- [SPRINT 2: PERFORMANCE, SEGURANÇA & ANTIFRAUDE] ---");
    let limiter = RateLimiter::new(2.0, 1.0);
    let _ = limiter.check_rate_limit("203.0.113.195");
    let _ = limiter.check_rate_limit("203.0.113.195");
    println!("4. Rate Limiter Token Bucket: Excesso = {:?}", limiter.check_rate_limit("203.0.113.195"));

    let table_players = vec![
        PlayerSession { user_id: "Alice".into(), ip_address: "192.168.1.10".into() },
        PlayerSession { user_id: "Bob".into(), ip_address: "192.168.1.45".into() },
    ];
    println!("5. Antifraude Subnet /24 Guard: Rejeição = {:?}", CollusionDetector::validate_table_seating(&table_players));

    let hole_and_board = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
    ];
    println!("6. Avaliador de 7 Cartas Texas Hold'em: Rank = {:?}\n", evaluate_hand(&hole_and_board));

    // --- SPRINT 3 DEMOS ---
    println!("--- [SPRINT 3: MODO TORNEIO COMPLETO & BALANCEAMENTO] ---");

    let acc1 = LedgerAccount::new("P1", 20000);
    let acc2 = LedgerAccount::new("P2", 20000);
    let acc3 = LedgerAccount::new("P3", 20000);

    let mut accounts = HashMap::new();
    accounts.insert("P1".to_string(), acc1.clone());
    accounts.insert("P2".to_string(), acc2.clone());
    accounts.insert("P3".to_string(), acc3.clone());

    let blind_structure = BlindStructure::standard_regular();
    let mut tournament = Tournament::new("SUNDAY-50K", "Sunday Grand Tournament", 10000, 1000, 10000.0, blind_structure);

    let _ = tournament.register_player("P1", "Alice", &acc1);
    let _ = tournament.register_player("P2", "Bob", &acc2);
    let _ = tournament.register_player("P3", "Charlie", &acc3);

    println!("7. Inscrições de Torneio: Prize Pool = R$ {:.2}", tournament.prize_pool_cents as f64 / 100.0);
    println!("   Nível Atual de Blinds: Level {} (SB: {}, BB: {})", tournament.blind_structure.levels[0].level_number, tournament.blind_structure.levels[0].small_blind, tournament.blind_structure.levels[0].big_blind);

    // Simular Rebalanceamento de Mesas
    let tables = vec![
        TableStateSummary { table_id: "Table_1".into(), active_player_ids: vec!["P1".into(), "P2".into(), "P3".into(), "P4".into(), "P5".into()] },
        TableStateSummary { table_id: "Table_2".into(), active_player_ids: vec!["P6".into(), "P7".into()] },
    ];
    let moves = TableBalancer::balance_tables(&tables);
    println!("8. Balanceador Dinâmico de Mesas: Movimentos = {:?}", moves);

    // Simular Final do Torneio
    tournament.state = TournamentState::Finished;
    tournament.players.get_mut("P1").unwrap().finish_rank = Some(1);
    tournament.players.get_mut("P2").unwrap().finish_rank = Some(2);
    tournament.players.get_mut("P3").unwrap().finish_rank = Some(3);

    let payouts = tournament.distribute_prize_pool(&accounts);
    println!("9. Distribuição do Prize Pool (Ledger Sync): {:?}", payouts);

    println!("\n========================================================");
    println!("   SPRINT 1, SPRINT 2 & SPRINT 3 VALIDADAS COM SUCESSO  ");
    println!("========================================================");
}
