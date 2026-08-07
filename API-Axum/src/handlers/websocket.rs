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

use crate::binary_codec::{BinaryOpcode, BinaryPacket};
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

async fn forward_player_action(
    tx_cmd: &mpsc::Sender<PlayerCommand>,
    player_id: &str,
    action: String,
    amount: u64,
) {
    let _ = tx_cmd
        .send(PlayerCommand::Action {
            player_id: player_id.to_string(),
            action,
            amount,
        })
        .await;
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
    type SeatAdmissionRow = (
        String,
        i64,
        i16,
        i64,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        i64,
        i16,
    );
    let seat: Option<SeatAdmissionRow> = match sqlx::query_as(
        "SELECT t.name, t.big_blind, t.rake_basis_points, t.rake_cap, t.rake_cap_heads_up, t.rake_cap_three_to_four, t.rake_cap_five_plus, s.chips, s.seat \
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
    let (
        table_name,
        big_blind,
        rake_basis_points,
        rake_cap,
        rake_cap_heads_up,
        rake_cap_three_to_four,
        rake_cap_five_plus,
        chips,
        seat,
    ) = match seat {
        Some((
            table_name,
            big_blind,
            rake_basis_points,
            rake_cap,
            rake_cap_heads_up,
            rake_cap_three_to_four,
            rake_cap_five_plus,
            chips,
            seat,
        )) if big_blind > 0
            && rake_basis_points >= 0
            && rake_cap >= 0
            && chips > 0
            && seat >= 0
            && matches!(
                (
                    rake_cap_heads_up,
                    rake_cap_three_to_four,
                    rake_cap_five_plus
                ),
                (None, None, None) | (Some(0..), Some(0..), Some(0..))
            ) =>
        {
            (
                table_name,
                big_blind as u64,
                rake_basis_points as u16,
                rake_cap as u64,
                rake_cap_heads_up.map(|value| value as u64),
                rake_cap_three_to_four.map(|value| value as u64),
                rake_cap_five_plus.map(|value| value as u64),
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
    let _connection_guard = crate::track_websocket_connection();

    // 2. Get or spawn the TableActor
    let handle = {
        let mut active_tables = state.active_tables.write().await;
        if let Some(h) = active_tables.get(&table_id) {
            h.clone()
        } else {
            let (tx_cmd, rx_cmd) = mpsc::channel(100);
            let (tx_broadcast, _) = tokio::sync::broadcast::channel(100);

            let mut table_config =
                poker_engine::types::TableConfig::new(big_blind, rake_basis_points, rake_cap);
            if let (Some(heads_up), Some(three_to_four), Some(five_plus)) = (
                rake_cap_heads_up,
                rake_cap_three_to_four,
                rake_cap_five_plus,
            ) {
                table_config =
                    table_config.with_rake_cap_schedule(poker_engine::types::RakeCapSchedule {
                        heads_up,
                        three_to_four,
                        five_plus,
                    });
            }
            let mut actor =
                TableActor::new(table_id.clone(), table_name, rx_cmd, tx_broadcast.clone())
                    .with_db(state.db.clone())
                    .with_audit_secret(state.jwt_secret.clone())
                    .with_config(table_config);
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

    // 4. Send all outbound frames through one task. This lets heartbeat,
    // command replies, and broadcasts share the socket without bypassing the
    // redaction boundary below.
    let (tx_outbound, mut rx_outbound) = mpsc::channel::<Message>(64);
    let mut rx_broadcast = handle.tx_broadcast.subscribe();
    let user_id_for_broadcast = user_id.clone();
    let ws_sender_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                broadcast_message = rx_broadcast.recv() => {
                    match broadcast_message {
                        Ok(message) => {
                            let safe_message = filter_table_state(message, &user_id_for_broadcast);
                            if ws_sender.send(Message::Text(safe_message.to_string())).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!(skipped, "WebSocket broadcast receiver lagged; waiting for the next authoritative state");
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                outbound_message = rx_outbound.recv() => {
                    match outbound_message {
                        Some(message) => {
                            if ws_sender.send(message).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // 5. Send Welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "player_id": &user_id,
        "seat": seat
    });

    if tx_outbound
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        warn!("Failed to send welcome message for user {}", username);
        return;
    }

    // 6. Main receive loop to process messages from WebSocket and forward to actor.
    // The outbound channel above is the only writer to the socket.
    let username_clone = username.clone();
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
                                let _ = tx_outbound
                                    .send(Message::Text(
                                        serde_json::json!({"type": "pong"}).to_string(),
                                    ))
                                    .await;
                            }
                            "action" => {
                                let action =
                                    parsed.get("action").and_then(|a| a.as_str()).unwrap_or("");
                                let amount =
                                    parsed.get("amount").and_then(|a| a.as_u64()).unwrap_or(0);
                                forward_player_action(
                                    &tx_cmd,
                                    &user_id_for_recv,
                                    action.to_string(),
                                    amount,
                                )
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
                                    if let Some(info_payload) = rx_info.recv().await {
                                        let safe_info =
                                            filter_table_state(info_payload, &user_id_for_recv);
                                        let _ = tx_outbound
                                            .send(Message::Text(safe_info.to_string()))
                                            .await;
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
                Message::Ping(payload) => {
                    let _ = tx_outbound.send(Message::Pong(payload)).await;
                }
                Message::Binary(bytes) => {
                    match BinaryPacket::decode(&bytes).and_then(|packet| {
                        BinaryOpcode::try_from(packet.opcode).map(|opcode| (opcode, packet))
                    }) {
                        Ok((BinaryOpcode::Ping, _)) => {
                            let pong = BinaryPacket::new(BinaryOpcode::Pong, Vec::new()).encode();
                            let _ = tx_outbound.send(Message::Binary(pong)).await;
                        }
                        Ok((BinaryOpcode::PlayerAction, packet)) => {
                            match serde_json::from_slice::<serde_json::Value>(&packet.payload) {
                                Ok(action) => {
                                    let name = action
                                        .get("action")
                                        .and_then(|value| value.as_str())
                                        .unwrap_or("");
                                    let amount = action
                                        .get("amount")
                                        .and_then(|value| value.as_u64())
                                        .unwrap_or(0);
                                    forward_player_action(
                                        &tx_cmd,
                                        &user_id_for_recv,
                                        name.to_string(),
                                        amount,
                                    )
                                    .await;
                                }
                                Err(_) => {
                                    let _ = tx_outbound
                                        .send(Message::Text(
                                            serde_json::json!({
                                                "type": "error",
                                                "message": "Ação binária inválida"
                                            })
                                            .to_string(),
                                        ))
                                        .await;
                                }
                            }
                        }
                        Ok((_opcode, _)) => {
                            let _ = tx_outbound
                                .send(Message::Text(
                                    serde_json::json!({
                                        "type": "error",
                                        "message": "Opcode binário não suportado"
                                    })
                                    .to_string(),
                                ))
                                .await;
                        }
                        Err(_) => {
                            let _ = tx_outbound
                                .send(Message::Text(
                                    serde_json::json!({
                                        "type": "error",
                                        "message": "Pacote binário inválido"
                                    })
                                    .to_string(),
                                ))
                                .await;
                        }
                    }
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

/// Redaction boundary for every server-originated WebSocket payload.
///
/// A recipient may see only their own hole cards. Credentials and server seeds
/// are never valid shared-socket fields, so they are removed recursively from
/// all event types as a defence against future JSON events.
fn filter_table_state(mut state_json: serde_json::Value, for_player_id: &str) -> serde_json::Value {
    redact_global_sensitive_fields(&mut state_json);
    let is_table_state =
        state_json.get("type").and_then(|value| value.as_str()) == Some("table_state");
    let current_bet_to_match = state_json
        .get("current_bet_to_match")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let min_raise = state_json
        .get("min_raise")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let mut available_actions = Vec::new();
    let mut call_amount = 0;
    let mut minimum_wager = 0;
    let mut maximum_wager = 0;

    if let Some(players) = state_json.get_mut("players").and_then(|v| v.as_array_mut()) {
        for player in players {
            let pid = player.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if pid == for_player_id && is_table_state {
                let is_active = player
                    .get("is_active")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                let chips = player
                    .get("chips")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0);
                let player_bet = player
                    .get("bet")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0);

                if is_active && chips > 0 {
                    let to_call = current_bet_to_match.saturating_sub(player_bet);
                    available_actions.push("fold");
                    if to_call == 0 {
                        available_actions.push("check");
                        if min_raise > 0 && chips >= min_raise {
                            available_actions.push("bet");
                            minimum_wager = min_raise;
                            maximum_wager = chips;
                        }
                    } else {
                        available_actions.push("call");
                        call_amount = to_call.min(chips);
                        let all_in_total = player_bet.saturating_add(chips);
                        let minimum_raise_total = current_bet_to_match.saturating_add(min_raise);
                        if all_in_total >= minimum_raise_total {
                            available_actions.push("raise");
                            minimum_wager = minimum_raise_total;
                            maximum_wager = all_in_total;
                        }
                    }
                    available_actions.push("allin");
                }
            } else if pid != for_player_id {
                // A broadcast state is shared by every socket; never leak an
                // opponent's private cards even when a future state adds an
                // alternative field name.
                for private_cards_key in ["cards", "hole_cards", "private_cards"] {
                    if let Some(cards) = player
                        .get_mut(private_cards_key)
                        .and_then(|value| value.as_array_mut())
                    {
                        cards.clear();
                    }
                }
            }
        }
    }
    if is_table_state {
        state_json["available_actions"] = serde_json::json!(available_actions);
        state_json["call_amount"] = serde_json::json!(call_amount);
        state_json["minimum_wager"] = serde_json::json!(minimum_wager);
        state_json["maximum_wager"] = serde_json::json!(maximum_wager);
    }
    state_json
}

fn redact_global_sensitive_fields(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                redact_global_sensitive_fields(value);
            }
        }
        serde_json::Value::Object(object) => {
            for key in [
                "server_seed",
                "server_seed_hex",
                "mfa_secret",
                "password_hash",
                "refresh_token",
            ] {
                object.remove(key);
            }
            for value in object.values_mut() {
                redact_global_sensitive_fields(value);
            }
        }
        _ => {}
    }
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

    #[test]
    fn removes_sensitive_fields_from_any_outbound_event() {
        let event = json!({
            "type": "future_event",
            "server_seed": "must-never-leave-the-server",
            "nested": {"mfa_secret": "private", "refresh_token": "private"}
        });

        let filtered = filter_table_state(event, "me");
        assert!(filtered.get("server_seed").is_none());
        assert!(filtered["nested"].get("mfa_secret").is_none());
        assert!(filtered["nested"].get("refresh_token").is_none());
    }

    #[test]
    fn adds_recipient_specific_actions_without_exposing_them_to_opponents() {
        let state = json!({
            "type": "table_state",
            "current_bet_to_match": 200,
            "min_raise": 200,
            "players": [
                {"id": "me", "chips": 1800, "bet": 100, "is_active": true, "cards": ["Ah", "Kd"]},
                {"id": "opponent", "chips": 1800, "bet": 200, "is_active": false, "cards": ["Qs", "Qc"]}
            ]
        });

        let filtered = filter_table_state(state, "me");

        assert_eq!(
            filtered["available_actions"],
            json!(["fold", "call", "raise", "allin"])
        );
        assert_eq!(filtered["call_amount"], json!(100));
        assert_eq!(filtered["minimum_wager"], json!(400));
        assert_eq!(filtered["maximum_wager"], json!(1900));
        assert_eq!(filtered["players"][1]["cards"], json!([]));
    }

    #[test]
    fn exposes_check_and_bet_when_there_is_nothing_to_call() {
        let state = json!({
            "type": "table_state",
            "current_bet_to_match": 0,
            "min_raise": 200,
            "players": [
                {"id": "me", "chips": 1500, "bet": 0, "is_active": true, "cards": ["Ah", "Kd"]}
            ]
        });

        let filtered = filter_table_state(state, "me");

        assert_eq!(
            filtered["available_actions"],
            json!(["fold", "check", "bet", "allin"])
        );
        assert_eq!(filtered["minimum_wager"], json!(200));
        assert_eq!(filtered["maximum_wager"], json!(1500));
    }
}
