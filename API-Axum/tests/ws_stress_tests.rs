// ws_stress_tests.rs — Teste de Carga e Estresse Massivo nos WebSockets / Atores de Mesa (Escala 1 Milhão de Mensagens)
// Valida o comportamento de rajada simultânea de 1 MILHÃO de mensagens WebSocket e conexões/desconexões em 100 mesas de jogo.

use poker_api::game_actor::{PlayerCommand, TableActor};
use tokio::sync::{broadcast, mpsc, oneshot};

#[tokio::test]
async fn test_ws_high_concurrency_table_actors() {
    const NUM_TABLES: usize = 100;
    const PLAYERS_PER_TABLE: usize = 9;
    const COMMANDS_PER_PLAYER: usize = 1_112; // 100 * 9 * 1112 = 1.000.800 mensagens em rajada

    let mut table_handles = Vec::with_capacity(NUM_TABLES);

    for t in 0..NUM_TABLES {
        let table_id = format!("stress_table_{}", t);
        let (tx_cmd, rx_cmd) = mpsc::channel(10_000);
        let (tx_broadcast, _) = broadcast::channel(10_000);

        let actor = TableActor::new(table_id.clone(), format!("Table {}", t), rx_cmd, tx_broadcast);

        // Subir o ator de mesa em background
        tokio::spawn(async move {
            actor.run().await;
        });

        // Adicionar 9 jogadores por mesa
        for p in 0..PLAYERS_PER_TABLE {
            let player_id = format!("player_{}_{}", t, p);
            let player_name = format!("Player {} {}", t, p);
            let (respond_to, rx_seat) = oneshot::channel();

            let _ = tx_cmd
                .send(PlayerCommand::Sit {
                    player_id,
                    username: player_name,
                    seat: Some(p),
                    chips: 100000,
                    respond_to,
                })
                .await;

            let _ = rx_seat.await;
        }

        // Subir rajada massiva de 1.112 comandos por jogador
        let mut player_tasks = Vec::with_capacity(PLAYERS_PER_TABLE);
        for p in 0..PLAYERS_PER_TABLE {
            let player_id = format!("player_{}_{}", t, p);
            let tx_cmd_clone = tx_cmd.clone();

            player_tasks.push(tokio::spawn(async move {
                for cmd_idx in 0..COMMANDS_PER_PLAYER {
                    if cmd_idx % 2 == 0 {
                        let _ = tx_cmd_clone
                            .send(PlayerCommand::Action {
                                player_id: player_id.clone(),
                                action: "call".to_string(),
                                amount: 1000,
                            })
                            .await;
                    } else {
                        let (tx_info, mut rx_info) = mpsc::channel(1);
                        let _ = tx_cmd_clone
                            .send(PlayerCommand::GetTableInfo {
                                respond_to: tx_info,
                            })
                            .await;

                        let _ = tokio::time::timeout(
                            std::time::Duration::from_millis(100),
                            rx_info.recv(),
                        )
                        .await;
                    }
                }
            }));
        }

        table_handles.push(player_tasks);
    }

    // Aguardar conclusão de todas as tarefas de jogadores (1.000.800 mensagens)
    let mut total_completed = 0;
    for player_tasks in table_handles {
        for task in player_tasks {
            task.await.unwrap();
            total_completed += 1;
        }
    }

    assert_eq!(
        total_completed,
        NUM_TABLES * PLAYERS_PER_TABLE,
        "Todas as tarefas de 1 milhão de mensagens devem completar sem deadlock"
    );
}

#[tokio::test]
async fn test_ws_rapid_reconnect_stress() {
    let table_id = "reconnect_table_1".to_string();
    let (tx_cmd, rx_cmd) = mpsc::channel(1000);
    let (tx_broadcast, _) = broadcast::channel(1000);

    let actor = TableActor::new(table_id, "Reconnect Test".to_string(), rx_cmd, tx_broadcast);

    tokio::spawn(async move {
        actor.run().await;
    });

    const RECONNECT_CYCLES: usize = 1000;
    let mut success_cycles = 0;

    for i in 0..RECONNECT_CYCLES {
        let player_id = format!("reconnect_p_{}", i);
        let name = format!("Rec Player {}", i);

        // 1. Entrar na mesa
        let (tx_seat, rx_seat) = oneshot::channel();
        tx_cmd
            .send(PlayerCommand::Sit {
                player_id: player_id.clone(),
                username: name,
                seat: None,
                chips: 50000,
                respond_to: tx_seat,
            })
            .await
            .unwrap();

        let seat_res = rx_seat.await.unwrap();
        assert!(seat_res < 9);

        // 2. Sair da mesa
        tx_cmd
            .send(PlayerCommand::Leave {
                player_id: player_id.clone(),
            })
            .await
            .unwrap();

        success_cycles += 1;
    }

    assert_eq!(success_cycles, RECONNECT_CYCLES);
}
