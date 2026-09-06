use poker_engine::server::{TableActor, TableMessage};
use tokio::sync::{mpsc, oneshot};

#[tokio::test]
async fn test_stress_k8s_actor_200_tables_50k_messages() {
    let num_tables = 200;
    let msgs_per_table = 250; // Total 50.000 mensagens assíncronas

    let mut senders = Vec::new();

    for t_id in 0..num_tables {
        let (tx, rx) = mpsc::channel(500);
        let mut actor = TableActor::new(format!("table_k8s_{}", t_id), rx);

        tokio::spawn(async move {
            actor.run().await;
        });

        senders.push(tx);
    }

    let mut tasks = Vec::new();

    for (t_id, tx) in senders.into_iter().enumerate() {
        let task = tokio::spawn(async move {
            for m_id in 0..msgs_per_table {
                if m_id % 2 == 0 {
                    let (reply_tx, reply_rx) = oneshot::channel();
                    let join_msg = TableMessage::PlayerJoin {
                        player_id: format!("P_{}_{}", t_id, m_id),
                        name: format!("Player {}", m_id),
                        stack: 2000.0,
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
                    assert!(!state.players.is_empty());
                }
            }
        });
        tasks.push(task);
    }

    for t in tasks {
        t.await.unwrap();
    }
}
