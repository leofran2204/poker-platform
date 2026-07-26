// actor_disconnect_stress_tests.rs — Testes de estresse para desconexão e remoção de jogadores no TableActor
use tokio::sync::{broadcast, mpsc, oneshot};
use poker_api::game_actor::{PlayerCommand, TableActor};

#[tokio::test]
async fn test_disconnect_inactive_turn_player_stress() {
    let (cmd_tx, cmd_rx) = mpsc::channel(100);
    let (bcast_tx, _bcast_rx) = broadcast::channel(100);

    let actor = TableActor::new("table_disconnect_stress".into(), "Stress Disconnect Table".into(), cmd_rx, bcast_tx);
    tokio::spawn(async move {
        actor.run().await;
    });

    // 1. Senta 3 jogadores na mesa para iniciar uma mão
    for i in 0..3 {
        let (resp_tx, resp_rx) = oneshot::channel();
        cmd_tx
            .send(PlayerCommand::Sit {
                player_id: format!("player_{}", i),
                username: format!("User {}", i),
                seat: Some(i),
                chips: 100000,
                respond_to: resp_tx,
            })
            .await
            .unwrap();
        let _ = resp_rx.await.unwrap();
    }

    // Dá um tempo curto para o tick do ator iniciar a mão
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // 2. Envia comando de saída (Leave) para o jogador player_2 (que pode NÃO ser a vez dele agora)
    cmd_tx
        .send(PlayerCommand::Leave {
            player_id: "player_2".to_string(),
        })
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 3. Valida se a mesa continua operacional consultando as informações da mesa
    let (info_tx, mut info_rx) = mpsc::channel(1);
    cmd_tx
        .send(PlayerCommand::GetTableInfo { respond_to: info_tx })
        .await
        .unwrap();

    let info = info_rx.recv().await.unwrap();
    let players = info["players"].as_array().unwrap();
    
    // player_2 deve ter sido removido da lista de atores da mesa
    assert!(
        !players.iter().any(|p| p["id"] == "player_2"),
        "player_2 deveria ter sido removido dos jogadores da mesa"
    );
}

#[tokio::test]
async fn test_disconnect_all_players_successive_stress() {
    // Teste de estresse: desconecta sucessivamente 5 jogadores um por um enquanto a mão inicia
    for seed in 0..5 {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (bcast_tx, _bcast_rx) = broadcast::channel(100);

        let actor = TableActor::new(format!("table_seq_{}", seed), "Seq Table".into(), cmd_rx, bcast_tx);
        tokio::spawn(async move {
            actor.run().await;
        });

        // Senta 4 jogadores
        for i in 0..4 {
            let (resp_tx, resp_rx) = oneshot::channel();
            cmd_tx
                .send(PlayerCommand::Sit {
                    player_id: format!("p_{}", i),
                    username: format!("User {}", i),
                    seat: Some(i),
                    chips: 50000,
                    respond_to: resp_tx,
                })
                .await
                .unwrap();
            let _ = resp_rx.await.unwrap();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Abandona os jogadores um a um em ordem inversa
        for i in (0..4).rev() {
            cmd_tx
                .send(PlayerCommand::Leave {
                    player_id: format!("p_{}", i),
                })
                .await
                .unwrap();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let (info_tx, mut info_rx) = mpsc::channel(1);
        cmd_tx
            .send(PlayerCommand::GetTableInfo { respond_to: info_tx })
            .await
            .unwrap();

        let info = info_rx.recv().await.unwrap();
        let players = info["players"].as_array().unwrap();
        assert_eq!(players.len(), 0, "Todos os jogadores deveriam ter sido removidos com sucesso");
    }
}
