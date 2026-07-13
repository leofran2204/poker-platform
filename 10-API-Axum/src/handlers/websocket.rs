// WebSocket handler — WS /ws/game/{table_id}
//
// Real-time game communication channel. For now, this is a minimal
// echo/broadcast implementation. Full game state sync will be added
// when the game engine is wired in.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;

use crate::state::AppState;

/// WS /ws/game/{table_id}
/// Upgrades HTTP to WebSocket for real-time game communication.
pub async fn game_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(table_id): Path<String>,
) -> Response {
    tracing::info!("WebSocket upgrade for table: {}", table_id);

    ws.on_upgrade(move |socket| handle_game_socket(socket, state, table_id))
}

/// Handles the WebSocket connection lifecycle.
async fn handle_game_socket(mut socket: WebSocket, state: AppState, table_id: String) {
    // Send initial connection acknowledgment
    let welcome = serde_json::json!({
        "type": "connected",
        "table_id": table_id,
        "message": "Connected to game WebSocket"
    });

    if socket
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        tracing::warn!("Failed to send welcome message for table {}", table_id);
        return;
    }

    // Main message loop
    while let Some(msg_result) = socket.recv().await {
        match msg_result {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    tracing::debug!("WS recv from table {}: {}", table_id, text);

                    // Parse the message to determine action
                    let response = handle_game_message(&text, &state, &table_id).await;

                    if socket.send(Message::Text(response)).await.is_err() {
                        break;
                    }
                }
                Message::Binary(data) => {
                    tracing::debug!("WS binary from table {}: {} bytes", table_id, data.len());
                }
                Message::Ping(ping) => {
                    if socket.send(Message::Pong(ping)).await.is_err() {
                        break;
                    }
                }
                Message::Pong(_) => {}
                Message::Close(_) => {
                    tracing::info!("WS closed for table {}", table_id);
                    break;
                }
            },
            Err(e) => {
                tracing::warn!("WS error for table {}: {}", table_id, e);
                break;
            }
        }
    }

    tracing::info!("WebSocket disconnected for table {}", table_id);
}

/// Processes an incoming game message and returns a JSON response.
async fn handle_game_message(text: &str, state: &AppState, table_id: &str) -> String {
    // Try to parse as JSON
    let parsed: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "type": "error",
                "message": format!("Invalid JSON: {}", e)
            })
            .to_string();
        }
    };

    let msg_type = parsed
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    match msg_type {
        "ping" => serde_json::json!({ "type": "pong" }).to_string(),
        "get_table_info" => {
            let lobby = state.lobby.lock().await;
            match lobby.find_table(table_id) {
                Some(table) => serde_json::json!({
                    "type": "table_info",
                    "table": {
                        "id": table.id,
                        "name": table.name,
                        "players": table.current_players,
                        "max_players": table.max_players,
                        "small_blind": table.small_blind,
                        "big_blind": table.big_blind,
                    }
                })
                .to_string(),
                None => serde_json::json!({
                    "type": "error",
                    "message": "Table not found"
                })
                .to_string(),
            }
        }
        _ => serde_json::json!({
            "type": "error",
            "message": format!("Unknown message type: {}", msg_type)
        })
        .to_string(),
    }
}
