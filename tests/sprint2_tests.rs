use poker_engine::antifraud::{
    CollusionDetector, CollusionViolation, PlayerBehaviorStats, PlayerSession,
};
use poker_engine::engine::evaluator::{evaluate_hand, Card, HandRank, Rank, Suit};
use poker_engine::security::RateLimiter;
use poker_engine::server::{TableActor, TableMessage};
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

#[test]
fn test_rate_limiter_token_bucket() {
    let limiter = RateLimiter::new(3.0, 10.0); // Capacidade 3 tokens, recarga de 10/sec

    // Consumir 3 tokens válidos
    assert!(limiter.check_rate_limit("192.168.1.100").is_ok());
    assert!(limiter.check_rate_limit("192.168.1.100").is_ok());
    assert!(limiter.check_rate_limit("192.168.1.100").is_ok());

    // 4ª requisição imediata deve estourar o limite
    assert!(limiter.check_rate_limit("192.168.1.100").is_err());

    // Esperar recarga de tokens
    sleep(Duration::from_millis(150));
    assert!(limiter.check_rate_limit("192.168.1.100").is_ok());
}

#[test]
fn test_antifraud_ip_and_subnet_collusion_detection() {
    // Caso 1: IPs Idênticos
    let players_same_ip = vec![
        PlayerSession {
            user_id: "User_1".into(),
            ip_address: "200.150.10.5".into(),
        },
        PlayerSession {
            user_id: "User_2".into(),
            ip_address: "200.150.10.5".into(), // Mesmo IP!
        },
    ];

    let result_ip = CollusionDetector::validate_table_seating(&players_same_ip);
    assert!(matches!(
        result_ip,
        Err(CollusionViolation::SameIpAddress(_, _, _))
    ));

    // Caso 2: Mesma Sub-rede /24
    let players_same_subnet = vec![
        PlayerSession {
            user_id: "User_A".into(),
            ip_address: "189.40.50.12".into(),
        },
        PlayerSession {
            user_id: "User_B".into(),
            ip_address: "189.40.50.99".into(), // Mesma sub-rede /24 (189.40.50)
        },
    ];

    let result_subnet = CollusionDetector::validate_table_seating(&players_same_subnet);
    assert!(matches!(
        result_subnet,
        Err(CollusionViolation::SameSubnet(_, _, _))
    ));
}

#[test]
fn test_antifraud_vpip_pfr_anomaly_detection() {
    let normal_stats = PlayerBehaviorStats {
        user_id: "GoodPlayer".into(),
        hands_played: 100,
        hands_vpip: 25, // 25% VPIP
        hands_pfr: 18,  // 18% PFR
    };
    assert!(CollusionDetector::detect_anomalies(&normal_stats).is_none());

    let suspicious_bot = PlayerBehaviorStats {
        user_id: "Bot123".into(),
        hands_played: 100,
        hands_vpip: 10,
        hands_pfr: 20, // PFR > VPIP (Impossível em jogo humano legítimo)
    };
    assert!(CollusionDetector::detect_anomalies(&suspicious_bot).is_some());
}

#[test]
fn test_evaluator_7_card_texas_holdem() {
    let cards = vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Two, Suit::Hearts),
        Card::new(Rank::Three, Suit::Clubs),
    ];

    let rank = evaluate_hand(&cards);
    assert_eq!(rank, HandRank::FullHouse(14, 13));
}

#[tokio::test]
async fn test_k8s_stateful_table_actor() {
    let (tx, rx) = mpsc::channel(32);
    let mut actor = TableActor::new("table_42", rx);

    // Rodar o actor em background task
    tokio::spawn(async move {
        actor.run().await;
    });

    // Enviar mensagem de PlayerJoin
    let (reply_tx, reply_rx) = oneshot::channel();
    tx.send(TableMessage::PlayerJoin {
        player_id: "Player_1".into(),
        name: "Alice".into(),
        stack: 500.0,
        respond_to: reply_tx,
    })
    .await
    .unwrap();

    let join_res = reply_rx.await.unwrap();
    assert!(join_res.is_ok());

    // Obter estado da mesa via Actor
    let (state_tx, state_rx) = oneshot::channel();
    tx.send(TableMessage::GetState {
        respond_to: state_tx,
    })
    .await
    .unwrap();

    let state = state_rx.await.unwrap();
    assert_eq!(state.players.len(), 1);
    assert_eq!(state.players[0].name, "Alice");
}
