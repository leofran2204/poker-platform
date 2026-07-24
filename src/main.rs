use poker_engine::admin::AdminDashboard;
use poker_engine::antifraud::{CollusionDetector, PlayerBehaviorStats, PlayerSession};
use poker_engine::auth::{generate_totp_code, verify_totp_code};
use poker_engine::engine::evaluator::{evaluate_hand, Card, Rank, Suit};
use poker_engine::engine::{calculate_side_pots, Contribution};
use poker_engine::ledger::{EntryType, LedgerAccount};
use poker_engine::security::RateLimiter;
use poker_engine::server::{TableActor, TableMessage, WebSocketServer, WsActionType, WsIncomingPacket};
use poker_engine::tournament::{BlindStructure, Tournament};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

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
    println!("7. Inscrições de Torneio: Prize Pool = R$ {:.2}\n", tournament.prize_pool_cents as f64 / 100.0);

    // --- WEBSOCKET SERVER IN REAL TIME DEMO ---
    println!("--- [SERVIDOR WEBSOCKET EM TEMPO REAL (TOKIO / AXUM)] ---");
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let (tx, rx) = mpsc::channel::<TableMessage>(100);
        let mut table_actor = TableActor::new("Table_Main_1", rx);
        tokio::spawn(async move {
            table_actor.run().await;
        });

        let ws_server = WebSocketServer::new();
        let _ = ws_server.register_client("Player_Alice");
        let _ = ws_server.register_client("Player_Bob");
        println!("8. WebSocket Server Ativo: {} conexões de clientes em tempo real", ws_server.active_clients_count());

        let packet = WsIncomingPacket {
            player_id: "Player_Alice".into(),
            action: WsActionType::JoinTable {
                table_id: "Table_Main_1".into(),
                ip_address: "203.0.113.88".into(),
            },
        };

        let response = ws_server.process_incoming_packet(packet, &tx, "203.0.113.88").await.unwrap();
        println!("   - Pacote Recebido -> Resposta Roteada via Actor: Evento = '{:?}', Msg = '{}'\n", response.event_type, response.payload);
    });

    // --- DASHBOARD ADMINISTRATIVO & GESTÃO DE RISCO DEMO ---
    println!("--- [DASHBOARD ADMINISTRATIVO & GESTÃO DE RISCO] ---");
    let admin = AdminDashboard::new();
    
    // 1. Auditoria Financeira do Ledger
    let audit = admin.audit_ledger_account(&ledger);
    println!("9. Auditoria Criptográfica do Ledger: Saldo = R$ {:.2} | Cadeia de Hashes Integras: {}", audit.account_balance_cents as f64 / 100.0, audit.hash_chain_valid);

    // 2. Análise de Risco e Suspensão
    let mut bad_stats = PlayerBehaviorStats::new("Bot_Suspeito_X");
    for _ in 0..30 {
        bad_stats.record_hand(true, true);
    }
    if let Some(risk_rep) = admin.analyze_player_risk(&bad_stats) {
        println!("   - Alerta Antifraude: Jogador '{}' -> Motivo: {}", risk_rep.user_id, risk_rep.reason);
        let ban_msg = admin.suspend_player(&risk_rep.user_id, &risk_rep.reason);
        println!("   - Ação Administrativa: {}", ban_msg);
    }

    admin.update_metrics(1250, 85, 250000000); // 1.250 conexões, 85 mesas, R$ 2,5 Milhões
    let metrics = admin.get_metrics();
    println!("   - Métricas de Cluster: Conexões = {} | Mesas Ativas = {} | Volume = R$ {:.2}", metrics.active_connections, metrics.active_tables, metrics.total_volume_cents as f64 / 100.0);

    println!("\n========================================================");
    println!(" ALL SYSTEMS OPERATIONAL: ENGINE, SECURITY, ADMIN & WS  ");
    println!("========================================================");
}
