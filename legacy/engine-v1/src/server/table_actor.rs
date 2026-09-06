use crate::engine::{Action, GameLoop, GameState, Player};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum TableMessage {
    PlayerJoin {
        player_id: String,
        name: String,
        stack: f64,
        respond_to: oneshot::Sender<Result<(), String>>,
    },
    PlayerAction {
        player_id: String,
        action: Action,
        respond_to: oneshot::Sender<Result<GameState, String>>,
    },
    GetState {
        respond_to: oneshot::Sender<GameState>,
    },
}

/// Actor de Mesa Stateful para Kubernetes.
/// Cada mesa executa de forma isolada em sua própria task/thread,
/// protegendo o estado contra race conditions sem travar o servidor global.
pub struct TableActor {
    pub table_id: String,
    receiver: mpsc::Receiver<TableMessage>,
    game_loop: Option<GameLoop>,
    players: HashMap<String, Player>,
}

impl TableActor {
    pub fn new(table_id: impl Into<String>, receiver: mpsc::Receiver<TableMessage>) -> Self {
        Self {
            table_id: table_id.into(),
            receiver,
            game_loop: None,
            players: HashMap::new(),
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                TableMessage::PlayerJoin {
                    player_id,
                    name,
                    stack,
                    respond_to,
                } => {
                    let player = Player::new(player_id.clone(), name, stack);
                    self.players.insert(player_id, player);
                    let _ = respond_to.send(Ok(()));
                }
                TableMessage::PlayerAction {
                    player_id: _,
                    action: _,
                    respond_to,
                } => {
                    if let Some(game_loop) = &mut self.game_loop {
                        // Processar ação no game loop
                        game_loop.advance_turn();
                        let _ = respond_to.send(Ok(game_loop.state.clone()));
                    } else {
                        let _ = respond_to.send(Err("Mão ainda não iniciada".into()));
                    }
                }
                TableMessage::GetState { respond_to } => {
                    if let Some(game_loop) = &self.game_loop {
                        let _ = respond_to.send(game_loop.state.clone());
                    } else {
                        let dummy_state =
                            GameState::new(self.players.values().cloned().collect(), 0, 10.0);
                        let _ = respond_to.send(dummy_state);
                    }
                }
            }
        }
    }
}
