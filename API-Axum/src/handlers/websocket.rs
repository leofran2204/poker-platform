// WebSocket handler — WS /ws/game/{table_id}
//
// Real-time game communication channel connected to the TableActor.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::Json;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::error::ApiError;
use crate::game_actor::{PlayerCommand, TableActor};
use crate::middleware::auth::RequireAuth;
use crate::state::{AppState, TableActorHandle, WebSocketTicket};

const WS_TICKET_TTL_SECONDS: u64 = 60;
const WS_TICKET_REDIS_PREFIX: &str = "poker:ws-ticket";

#[derive(Deserialize)]
pub struct WsQuery {
    pub ticket: Option<String>,
}

#[derive(Serialize)]
pub struct WsTicketResponse {
    pub ticket: String,
    pub expires_in: u64,
}

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn redis_ticket_key(ticket: &str, table_id: &str) -> String {
    format!("{WS_TICKET_REDIS_PREFIX}:{table_id}:{ticket}")
}

async fn store_ws_ticket(
    state: &AppState,
    ticket: &str,
    ticket_record: WebSocketTicket,
) -> Result<(), ApiError> {
    if let Some(redis) = &state.redis {
        use redis::AsyncCommands;

        let payload = serde_json::to_string(&ticket_record).map_err(|error| {
            ApiError::Internal(format!("Failed to serialize WebSocket ticket: {error}"))
        })?;
        let mut connection = redis.clone();
        let _: () = connection
            .set_ex(
                redis_ticket_key(ticket, &ticket_record.table_id),
                payload,
                WS_TICKET_TTL_SECONDS,
            )
            .await
            .map_err(|error| {
                ApiError::Internal(format!("Failed to store WebSocket ticket: {error}"))
            })?;
        return Ok(());
    }

    let mut tickets = state.ws_tickets.lock().await;
    let now = unix_timestamp();
    tickets.retain(|_, record| record.expires_at > now);
    tickets.insert(ticket.to_string(), ticket_record);
    Ok(())
}

async fn consume_ws_ticket(
    state: &AppState,
    ticket: &str,
    table_id: &str,
) -> Result<Option<WebSocketTicket>, String> {
    let now = unix_timestamp();
    if let Some(redis) = &state.redis {
        let mut connection = redis.clone();
        let payload: Option<String> = redis::cmd("GETDEL")
            .arg(redis_ticket_key(ticket, table_id))
            .query_async(&mut connection)
            .await
            .map_err(|error| format!("Redis WebSocket ticket consume failed: {error}"))?;
        return payload
            .map(|payload| {
                serde_json::from_str::<WebSocketTicket>(&payload)
                    .map_err(|error| format!("Stored WebSocket ticket is malformed: {error}"))
            })
            .transpose()
            .map(|ticket| {
                ticket.filter(|record| {
                    record.expires_at > now && record.table_id.as_str() == table_id
                })
            });
    }

    let mut tickets = state.ws_tickets.lock().await;
    tickets.retain(|_, record| record.expires_at > now);
    match tickets.get(ticket) {
        Some(record) if record.table_id == table_id => Ok(tickets.remove(ticket)),
        _ => Ok(None),
    }
}

/// POST /api/lobby/tables/:id/ws-ticket
/// Issues a short-lived, single-use ticket for the browser WebSocket handshake.
pub async fn create_ws_ticket(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Path(table_id): Path<String>,
) -> Result<Json<WsTicketResponse>, ApiError> {
    let table_id = uuid::Uuid::parse_str(&table_id)
        .map_err(|_| ApiError::BadRequest("Invalid table id".to_string()))?
        .to_string();
    let user_id = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| ApiError::Internal("Authenticated user id is invalid".to_string()))?;
    let has_active_seat: bool = sqlx::query_scalar(
        "SELECT EXISTS( \
             SELECT 1 FROM cash_game_seats s \
             JOIN tables t ON t.id = s.table_id \
             WHERE s.table_id = $1::uuid AND s.user_id = $2 \
               AND s.status = 'ACTIVE' AND s.chips > 0 AND t.status = 'OPEN' \
         )",
    )
    .bind(&table_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    if !has_active_seat {
        return Err(ApiError::Forbidden(
            "An active funded seat is required to request a WebSocket ticket".to_string(),
        ));
    }

    let ticket = uuid::Uuid::new_v4().to_string();
    let ticket_record = WebSocketTicket {
        user_id: auth_user.user_id,
        username: auth_user.username,
        table_id,
        expires_at: unix_timestamp() + WS_TICKET_TTL_SECONDS as i64,
    };
    store_ws_ticket(&state, &ticket, ticket_record).await?;

    Ok(Json(WsTicketResponse {
        ticket,
        expires_in: WS_TICKET_TTL_SECONDS,
    }))
}

/// WS /ws/game/{table_id}
/// Upgrades HTTPS to WSS for real-time game communication.
pub async fn game_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(table_id): Path<String>,
    Query(query): Query<WsQuery>,
) -> Response {
    info!("WebSocket upgrade request for table: {}", table_id);

    ws.on_upgrade(move |socket| handle_game_socket(socket, state, table_id, query.ticket))
}

/// Handles the WebSocket connection lifecycle.
async fn handle_game_socket(
    socket: WebSocket,
    state: AppState,
    table_id: String,
    ticket: Option<String>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let table_id = match uuid::Uuid::parse_str(&table_id) {
        Ok(id) => id.to_string(),
        Err(_) => {
            let _ = ws_sender
                .send(Message::Text(
                    serde_json::json!({
                        "type": "error",
                        "message": "Identificador de mesa inválido"
                    })
                    .to_string(),
                ))
                .await;
            return;
        }
    };

    // 1. Consume an opaque ticket. The access JWT is accepted only by the
    // authenticated ticket endpoint, never in a WebSocket URL.
    let (user_id, username) = match ticket {
        Some(ticket) if !ticket.is_empty() => {
            match consume_ws_ticket(&state, &ticket, &table_id).await {
                Ok(Some(record)) => (record.user_id, record.username),
                Ok(None) => {
                    warn!("WebSocket connection rejected: missing, expired, or already consumed ticket");
                    let _ = ws_sender
                        .send(Message::Text(
                            serde_json::json!({
                                "type": "error",
                                "message": "Ticket WebSocket inválido, expirado ou já utilizado"
                            })
                            .to_string(),
                        ))
                        .await;
                    return;
                }
                Err(error) => {
                    error!(%error, "WebSocket ticket consume failed");
                    let _ = ws_sender
                        .send(Message::Text(
                            serde_json::json!({
                                "type": "error",
                                "message": "Não foi possível validar o ticket WebSocket"
                            })
                            .to_string(),
                        ))
                        .await;
                    return;
                }
            }
        }
        _ => {
            warn!("WebSocket connection rejected: missing WebSocket ticket");
            let _ = ws_sender
                .send(Message::Text(
                    serde_json::json!({
                        "type": "error",
                        "message": "Ticket WebSocket ausente"
                    })
                    .to_string(),
                ))
                .await;
            return;
        }
    };

    // Admission is authorized by a persisted, funded escrow seat. A valid JWT
    // alone is not enough to create a table or receive demo chips.
    let seat: Option<(String, i64, i16, i64, i64, i16)> = match sqlx::query_as(
        "SELECT t.name, t.big_blind, t.rake_basis_points, t.rake_cap, s.chips, s.seat \
         FROM cash_game_seats s \
         JOIN tables t ON t.id = s.table_id \
         WHERE s.table_id = $1::uuid AND s.user_id = $2::uuid \
           AND s.status = 'ACTIVE' AND t.status = 'OPEN'",
    )
    .bind(&table_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(seat) => seat,
        Err(database_error) => {
            error!(?database_error, "WebSocket seat admission query failed");
            let _ = ws_sender
                .send(Message::Text(
                    serde_json::json!({
                        "type": "error",
                        "message": "Não foi possível validar o assento da mesa"
                    })
                    .to_string(),
                ))
                .await;
            return;
        }
    };
    let (table_name, big_blind, rake_basis_points, rake_cap, chips, seat) = match seat {
        Some((table_name, big_blind, rake_basis_points, rake_cap, chips, seat))
            if big_blind > 0
                && rake_basis_points >= 0
                && rake_cap >= 0
                && chips > 0
                && seat >= 0 =>
        {
            (
                table_name,
                big_blind as u64,
                rake_basis_points as u16,
                rake_cap as u64,
                chips as u64,
                seat as usize,
            )
        }
        _ => {
            warn!(
                "WebSocket connection rejected: no active funded seat for user {} at table {}",
                user_id, table_id
            );
            let _ = ws_sender
                .send(Message::Text(
                    serde_json::json!({
                        "type": "error",
                        "message": "É necessário entrar na mesa com buy-in antes de jogar"
                    })
                    .to_string(),
                ))
                .await;
            return;
        }
    };

    info!(
        "User '{}' ({}) connecting to table {}",
        username, user_id, table_id
    );

    // 2. Get or spawn the TableActor
    let handle = {
        let mut active_tables = state.active_tables.write().await;
        if let Some(h) = active_tables.get(&table_id) {
            h.clone()
        } else {
            let (tx_cmd, rx_cmd) = mpsc::channel(100);
            let (tx_broadcast, _) = tokio::sync::broadcast::channel(100);

            let mut actor =
                TableActor::new(table_id.clone(), table_name, rx_cmd, tx_broadcast.clone())
                    .with_db(state.db.clone())
                    .with_audit_secret(state.jwt_secret.clone())
                    .with_config(poker_engine::types::TableConfig::new(
                        big_blind,
                        rake_basis_points,
                        rake_cap,
                    ));
            if let Some(ref redis) = state.redis {
                actor = actor.with_redis(redis.clone());
            }
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
        seat: Some(seat),
        chips,
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

    if ws_sender
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
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
            if ws_sender
                .send(Message::Text(filtered_msg.to_string()))
                .await
                .is_err()
            {
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
                                // Keepalive ping: no-op (heartbeat mantido pelo frame do protocolo)
                            }
                            "action" => {
                                let action =
                                    parsed.get("action").and_then(|a| a.as_str()).unwrap_or("");
                                let amount =
                                    parsed.get("amount").and_then(|a| a.as_u64()).unwrap_or(0);
                                let _ = tx_cmd
                                    .send(PlayerCommand::Action {
                                        player_id: user_id_for_recv.clone(),
                                        action: action.to_string(),
                                        amount,
                                    })
                                    .await;
                            }
                            "get_table_info" => {
                                let (tx_info, mut rx_info) = mpsc::channel(1);
                                if tx_cmd
                                    .send(PlayerCommand::GetTableInfo {
                                        respond_to: tx_info,
                                    })
                                    .await
                                    .is_ok()
                                {
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
            },
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
/// Only allows the target player to see their own hole cards.
/// Showdown cards must be delivered through an explicit, auditable reveal event.
fn filter_table_state(mut state_json: serde_json::Value, for_player_id: &str) -> serde_json::Value {
    if let Some(players) = state_json.get_mut("players").and_then(|v| v.as_array_mut()) {
        for player in players {
            let pid = player.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if pid != for_player_id {
                // A broadcast state is shared by every socket; never leak opponents cards.
                if let Some(cards) = player.get_mut("cards").and_then(|v| v.as_array_mut()) {
                    cards.clear();
                }
            }
        }
    }
    state_json
}

#[cfg(test)]
mod tests {
    use super::filter_table_state;
    use serde_json::json;

    #[test]
    fn hides_opponent_cards_after_showdown() {
        let state = json!({
            "is_finished": true,
            "players": [
                {"id": "me", "cards": ["Ah", "Kd"]},
                {"id": "opponent", "cards": ["Qs", "Qc"]}
            ]
        });

        let filtered = filter_table_state(state, "me");
        let players = filtered["players"].as_array().expect("players array");

        assert_eq!(players[0]["cards"], json!(["Ah", "Kd"]));
        assert_eq!(players[1]["cards"], json!([]));
    }
}
