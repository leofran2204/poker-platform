use poker_engine::server::{TableActor, TableMessage};
use tokio::sync::{mpsc, oneshot};

#[tokio::test]
async fn test_massive_concurrent_k8s_table_actors_50_tables_10k_messages() {
    let num_tables = 50;
    let msgs_per_table = 200; // Total 10.000 mensagens assíncronas concorrentes

    let mut table_senders = Vec::new();

    for t_idx in 0..num_tables {
        let (tx, rx) = mpsc::channel(500);
        let mut actor = TableActor::new(format!("table_{}", t_idx), rx);

        tokio::spawn(async move {
            actor.run().await;
        });

        table_senders.push(tx);
    }

    let mut tasks = Vec::new();

    for (t_idx, tx) in table_senders.into_iter().enumerate() {
        let task = tokio::spawn(async move {
            for m_idx in 0..msgs_per_table {
                if m_idx % 2 == 0 {
                    let (reply_tx, reply_rx) = oneshot::channel();
                    let join_msg = TableMessage::PlayerJoin {
                        player_id: format!("P_{}_{}", t_idx, m_idx),
                        name: format!("Player {}", m_idx),
                        stack: 1000.0,
                        respond_to: reply_tx,
                    };
                    tx.send(join_msg).await.unwrap();
                    let res = reply_rx.await.unwrap();
                    assert!(res.is_ok());
                } else {
                    let (state_tx, state_rx) = oneshot::channel();
                    let state_msg = TableMessage::GetState {
                        respond_to: state_tx,
                    };
                    tx.send(state_msg).await.unwrap();
                    let state = state_rx.await.unwrap();
                    assert!(state.players.len() >= 1);
                }
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.unwrap();
    }
}
