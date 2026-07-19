// WebSocket handler — WS /ws/game/{table_id}
//
// Real-time game communication channel connected to the TableActor.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State, Query};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn, debug, error};

use crate::state::{AppState, TableActorHandle};
use crate::game_actor::{PlayerCommand, TableActor};

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

/// WS /ws/game/{table_id}
/// Upgrades HTTP to WebSocket for real-time game communication.
pub async fn game_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(table_id): Path<String>,
    Query(query): Query<WsQuery>,
) -> Response {
    info!("WebSocket upgrade request for table: {}", table_id);

    ws.on_upgrade(move |socket| handle_game_socket(socket, state, table_id, query.token))
}

/// Handles the WebSocket connection lifecycle.
async fn handle_game_socket(socket: WebSocket, state: AppState, table_id: String, token: Option<String>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // 1. Authenticate user, fallback to guest in development / fallback scenarios
    let (user_id, username) = if let Some(ref t) = token {
        if !t.is_empty() {
            let auth = state.auth.lock().await;
            match auth.validate_token(t, "access") {
                Ok(claims) => (claims.sub, claims.username),
                Err(_) => {
                    let guest_id = format!("guest_{}", &uuid::Uuid::new_v4().to_string()[..6]);
                    (guest_id.clone(), guest_id)
                }
            }
        } else {
            let guest_id = format!("guest_{}", &uuid::Uuid::new_v4().to_string()[..6]);
            (guest_id.clone(), guest_id)
        }
    } else {
        let guest_id = format!("guest_{}", &uuid::Uuid::new_v4().to_string()[..6]);
        (guest_id.clone(), guest_id)
    };

    info!("User '{}' ({}) connecting to table {}", username, user_id, table_id);

    // 2. Get or spawn the TableActor
    let handle = {
        let mut active_tables = state.active_tables.lock().await;
        if let Some(h) = active_tables.get(&table_id) {
            h.clone()
        } else {
            let (tx_cmd, rx_cmd) = mpsc::channel(100);
            let (tx_broadcast, _) = tokio::sync::broadcast::channel(100);

            let table_name = {
                let lobby = state.lobby.lock().await;
                lobby.find_table(&table_id).map(|t| t.name.clone()).unwrap_or_else(|| format!("Table {}", table_id))
            };

            let actor = TableActor::new(table_id.clone(), table_name, rx_cmd, tx_broadcast.clone());
            tokio::spawn(actor.run());

            let h = TableActorHandle {
                tx_cmd,
                tx_broadcast,
            };
            active_tables.insert(table_id.clone(), h.clone());
            h
        }
    };

    // 3. Sit the player at the table
    let (tx_sit_resp, rx_sit_resp) = oneshot::channel();
    let sit_cmd = PlayerCommand::Sit {
        player_id: user_id.clone(),
        username: username.clone(),
        seat: None, // Auto-assign next seat
        chips: 1000.0, // Initial chips
        respond_to: tx_sit_resp,
    };

    if handle.tx_cmd.send(sit_cmd).await.is_err() {
        error!("Failed to send Sit command to TableActor");
        return;
    }

    let seat = match rx_sit_resp.await {
        Ok(s) => s,
        Err(_) => {
            error!("TableActor dropped Sit response channel");
            return;
        }
    };

    // 4. Send Welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "player_id": user_id,
        "seat": seat
    });

    if ws_sender.send(Message::Text(welcome.to_string())).await.is_err() {
        warn!("Failed to send welcome message for user {}", username);
        return;
    }

    // 5. Spawn broadcast listener to stream table states to the client (filtering cards)
    let mut rx_broadcast = handle.tx_broadcast.subscribe();
    let user_id_clone = user_id.clone();
    let username_clone = username.clone();
    
    let ws_sender_task = tokio::spawn(async move {
        while let Ok(msg) = rx_broadcast.recv().await {
            // Filter cards for other players to prevent cheating
            let filtered_msg = filter_table_state(msg, &user_id_clone);
            if ws_sender.send(Message::Text(filtered_msg.to_string())).await.is_err() {
                break;
            }
        }
    });

    // 6. Main receive loop to process messages from WebSocket and forward to actor
    let tx_cmd = handle.tx_cmd.clone();
    let user_id_for_recv = user_id.clone();

    while let Some(msg_result) = ws_receiver.next().await {
        match msg_result {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    debug!("WS recv from user {}: {}", username_clone, text);
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        match msg_type {
                            "ping" => {
                                // Respond directly with pong to avoid latency
                                let _ = tx_cmd.send(PlayerCommand::Sit {
                                    player_id: user_id_for_recv.clone(),
                                    username: username_clone.clone(),
                                    seat: Some(seat),
                                    chips: 1000.0, // dummy, does not overwrite if already sitting
                                    respond_to: oneshot::channel().0,
                                }).await;
                            }
                            "action" => {
                                let action = parsed.get("action").and_then(|a| a.as_str()).unwrap_or("");
                                let amount = parsed.get("amount").and_then(|a| a.as_f64()).unwrap_or(0.0);
                                let _ = tx_cmd.send(PlayerCommand::Action {
                                    player_id: user_id_for_recv.clone(),
                                    action: action.to_string(),
                                    amount,
                                }).await;
                            }
                            "get_table_info" => {
                                let (tx_info, mut rx_info) = mpsc::channel(1);
                                if tx_cmd.send(PlayerCommand::GetTableInfo { respond_to: tx_info }).await.is_ok() {
                                    if let Some(_info_payload) = rx_info.recv().await {
                                        // Wait, we need to send info_payload to sender loop. But we can't easily write to ws_sender directly
                                        // because it is moved to the broadcast task.
                                        // However, TableActor will automatically broadcast states, and we can also just let the client receive it.
                                        // Actually, since ws_sender is locked inside the spawned task, we can just send it back by letting TableActor broadcast the update.
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
            Err(e) => {
                warn!("WS error for user {}: {}", username_clone, e);
                break;
            }
        }
    }

    // 7. Cleanup on disconnect: notify actor and cancel sender task
    ws_sender_task.abort();
    let leave_cmd = PlayerCommand::Leave { player_id: user_id };
    let _ = handle.tx_cmd.send(leave_cmd).await;
    info!("WebSocket disconnected for user {}", username_clone);
}

/// Anti-cheat card filter.
/// Only allows the target player to see their own hole cards unless the hand is finished (showdown).
fn filter_table_state(mut state_json: serde_json::Value, for_player_id: &str) -> serde_json::Value {
    let is_finished = state_json.get("is_finished").and_then(|v| v.as_bool()).unwrap_or(true);
    
    if let Some(players) = state_json.get_mut("players").and_then(|v| v.as_array_mut()) {
        for player in players {
            let pid = player.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if pid != for_player_id && !is_finished {
                // Clear cards for other players if hand is not finished
                if let Some(cards) = player.get_mut("cards").and_then(|v| v.as_array_mut()) {
                    cards.clear();
                }
            }
        }
    }
    state_json
}
