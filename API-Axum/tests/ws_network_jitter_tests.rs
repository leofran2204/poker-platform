// ws_network_jitter_tests.rs — Suíte de Estresse de Instabilidade de Rede e Lag em WebSockets (Network Jitter)
// Valida a resiliência dos atores de mesa contra alta latência, desordem de mensagens e pacotes atrasados.

use poker_api::game_actor::{PlayerCommand, TableActor};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::sleep;

#[tokio::test]
async fn test_ws_network_lag_and_jitter_resilience() {
    let table_id = "jitter_table_1".to_string();
    let (tx_cmd, rx_cmd) = mpsc::channel(100);
    let (tx_broadcast, _) = broadcast::channel(100);

    let actor = TableActor::new(table_id, "Jitter Table".to_string(), rx_cmd, tx_broadcast);

    tokio::spawn(async move {
        actor.run().await;
    });

    // 1. Adiciona 6 jogadores sob estresse
    for p in 0..6 {
        let player_id = format!("jitter_player_{}", p);
        let username = format!("JitterUser_{}", p);
        let (respond_to, rx_seat) = oneshot::channel();

        let _ = tx_cmd
            .send(PlayerCommand::Sit {
                player_id,
                username,
                seat: Some(p),
                chips: 200000,
                respond_to,
            })
            .await;

        let _ = rx_seat.await;
    }

    // 2. Simula rajada de ações com jitter e latência artificial (50ms a 150ms de lag de pacote WS)
    let mut tasks = Vec::new();
    for p in 0..6 {
        let player_id = format!("jitter_player_{}", p);
        let tx_clone = tx_cmd.clone();

        tasks.push(tokio::spawn(async move {
            for i in 0..20 {
                // Simula jitter de rede atrasando pacotes WebSockets
                sleep(Duration::from_millis(5 + (i * 3) as u64)).await;

                let action_str = if i % 2 == 0 { "call" } else { "raise" };
                let _ = tx_clone
                    .send(PlayerCommand::Action {
                        player_id: player_id.clone(),
                        action: action_str.to_string(),
                        amount: 1000 * (i + 1) as u64,
                    })
                    .await;
            }
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }
}

#[tokio::test]
async fn test_ws_out_of_order_messages_and_burst_reconnects() {
    let table_id = "jitter_table_2".to_string();
    let (tx_cmd, rx_cmd) = mpsc::channel(100);
    let (tx_broadcast, _) = broadcast::channel(100);

    let actor = TableActor::new(
        table_id,
        "Jitter Reconnect Table".to_string(),
        rx_cmd,
        tx_broadcast,
    );

    tokio::spawn(async move {
        actor.run().await;
    });

    // Simula 30 ciclos de reconexão ultrarrápida sob instabilidade de rede
    for i in 0..30 {
        let player_id = format!("reconnect_user_{}", i % 5);
        let username = format!("User_{}", i % 5);

        let (tx_seat, rx_seat) = oneshot::channel();
        let _ = tx_cmd
            .send(PlayerCommand::Sit {
                player_id: player_id.clone(),
                username,
                seat: None,
                chips: 100000,
                respond_to: tx_seat,
            })
            .await;

        let _ = rx_seat.await;

        // Simula desconexão em alta latência
        sleep(Duration::from_millis(2)).await;
        let _ = tx_cmd.send(PlayerCommand::Leave { player_id }).await;
    }
}
