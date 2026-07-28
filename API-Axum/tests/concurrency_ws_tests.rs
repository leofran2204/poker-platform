// concurrency_ws_tests.rs — Testes de Concorrência, Isolamento e Resiliência dos Atores de Mesa

use poker_api::game_actor::{PlayerCommand, TableActor};
use tokio::sync::{broadcast, mpsc, oneshot};

#[tokio::test]
async fn test_multi_table_isolation_and_concurrency() {
    let num_tables = 5;
    let mut table_txs = Vec::new();
    let mut broadcast_rxs = Vec::new();

    // 1. Inicializa 5 atores de mesa simultâneos em background
    for i in 0..num_tables {
        let table_id = format!("table_{}", i);
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (bcast_tx, bcast_rx) = broadcast::channel(100);

        let actor = TableActor::new(table_id.clone(), format!("Mesa {}", i), cmd_rx, bcast_tx);
        tokio::spawn(async move {
            actor.run().await;
        });

        table_txs.push(cmd_tx);
        broadcast_rxs.push(bcast_rx);
    }

    // 2. Concorrentemente envia comandos Sit em cada uma das 5 mesas
    let mut handles = Vec::new();

    for (table_idx, tx) in table_txs.iter().enumerate() {
        let tx_clone = tx.clone();
        let handle = tokio::spawn(async move {
            for p_idx in 0..3 {
                let (resp_tx, resp_rx) = oneshot::channel();
                let player_id = format!("t{}_player{}", table_idx, p_idx);
                let username = format!("Player_{}_{}", table_idx, p_idx);

                tx_clone
                    .send(PlayerCommand::Sit {
                        player_id,
                        username,
                        seat: Some(p_idx),
                        chips: 100000,
                        respond_to: resp_tx,
                    })
                    .await
                    .expect("Falha ao enviar comando Sit");

                let seat = resp_rx
                    .await
                    .expect("Falha ao receber confirmação de assento");
                assert_eq!(seat, p_idx, "Assento retornado difere do solicitado");
            }
        });
        handles.push(handle);
    }

    // Aguarda todos os comandos concorrentes finalizarem
    for handle in handles {
        handle.await.expect("Task de sit falhou");
    }

    // 3. Valida isolamento consultando o estado de cada mesa
    for (table_idx, tx) in table_txs.iter().enumerate() {
        let (info_tx, mut info_rx) = mpsc::channel(1);
        tx.send(PlayerCommand::GetTableInfo {
            respond_to: info_tx,
        })
        .await
        .expect("Falha ao solicitar info da mesa");

        let info = info_rx.recv().await.expect("Falha ao receber info da mesa");
        assert_eq!(
            info["table_id"],
            format!("table_{}", table_idx),
            "ID da mesa no evento não confere"
        );
        let players = info["players"]
            .as_array()
            .expect("Array de players inválido");
        assert_eq!(
            players.len(),
            3,
            "Quantidade de jogadores na mesa incorreta"
        );
    }
}

#[tokio::test]
async fn test_player_disconnect_resilience() {
    let (cmd_tx, cmd_rx) = mpsc::channel(100);
    let (bcast_tx, _bcast_rx) = broadcast::channel(100);

    let actor = TableActor::new(
        "table_disconnect".into(),
        "Mesa Disconnect".into(),
        cmd_rx,
        bcast_tx,
    );
    tokio::spawn(async move {
        actor.run().await;
    });

    // 1. Jogador entra na mesa
    let (resp_tx, resp_rx) = oneshot::channel();
    cmd_tx
        .send(PlayerCommand::Sit {
            player_id: "p_drop".into(),
            username: "PlayerDrop".into(),
            seat: Some(0),
            chips: 50000,
            respond_to: resp_tx,
        })
        .await
        .unwrap();

    let seat = resp_rx.await.unwrap();
    assert_eq!(seat, 0);

    // 2. Simula desconexão abrupta enviando comando Leave
    cmd_tx
        .send(PlayerCommand::Leave {
            player_id: "p_drop".into(),
        })
        .await
        .unwrap();

    // 3. Valida que a mesa continua operacional após a desconexão
    let (info_tx, mut info_rx) = mpsc::channel(1);
    cmd_tx
        .send(PlayerCommand::GetTableInfo {
            respond_to: info_tx,
        })
        .await
        .unwrap();

    let info = info_rx.recv().await.unwrap();
    let players = info["players"].as_array().unwrap();
    assert_eq!(
        players.len(),
        0,
        "Jogador deveria ter sido removido após desconexão"
    );
}
