use crate::engine::Action;
use crate::security::RateLimiter;
use crate::server::table_actor::TableMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsActionType {
    JoinTable {
        table_id: String,
        ip_address: String,
    },
    PostBet {
        amount: f64,
    },
    Fold,
    Check,
    Call,
    LeaveTable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsIncomingPacket {
    pub player_id: String,
    pub action: WsActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsOutgoingPacket {
    pub event_type: String,
    pub table_id: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanNotificationPayload {
    pub category: String, // "Connection", "Antifraud", "Finance", "Tournament", "ProvablyFair"
    pub level: String,    // "Info", "Success", "Warning", "Error"
    pub title: String,
    pub message: String,
    pub action_advice: Option<String>,
}

pub struct ClientSession {
    pub player_id: String,
    pub table_id: Option<String>,
    pub sender: mpsc::Sender<WsOutgoingPacket>,
}

#[derive(Clone)]
pub struct WebSocketServer {
    clients: Arc<Mutex<HashMap<String, mpsc::Sender<WsOutgoingPacket>>>>,
    table_broadcasters: Arc<Mutex<HashMap<String, broadcast::Sender<WsOutgoingPacket>>>>,
    rate_limiter: Arc<RateLimiter>,
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            table_broadcasters: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: Arc::new(RateLimiter::new(100.0, 50.0)),
        }
    }

    /// Gera uma notificação de alta confiança traduzida para linguagem humana legível.
    pub fn create_player_notification_packet(
        table_id: &str,
        category: &str,
        level: &str,
        title: &str,
        message: &str,
        action_advice: Option<&str>,
    ) -> WsOutgoingPacket {
        let payload_struct = HumanNotificationPayload {
            category: category.to_string(),
            level: level.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            action_advice: action_advice.map(|s| s.to_string()),
        };

        WsOutgoingPacket {
            event_type: "PLAYER_NOTIFICATION_TOAST".into(),
            table_id: table_id.to_string(),
            payload: serde_json::to_string(&payload_struct).unwrap_or_else(|_| message.to_string()),
        }
    }

    /// Registra uma nova conexão de cliente WebSocket
    pub fn register_client(
        &self,
        player_id: &str,
    ) -> (
        mpsc::Sender<WsOutgoingPacket>,
        mpsc::Receiver<WsOutgoingPacket>,
    ) {
        let (tx, rx) = mpsc::channel(100);
        let mut clients = self.clients.lock().unwrap();
        clients.insert(player_id.to_string(), tx.clone());
        (tx, rx)
    }

    /// Desconecta um cliente WebSocket
    pub fn disconnect_client(&self, player_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(player_id);
    }

    /// Obtém a contagem de clientes conectados
    pub fn active_clients_count(&self) -> usize {
        let clients = self.clients.lock().unwrap();
        clients.len()
    }

    /// Cria ou recupera o canal de Broadcast de uma mesa
    pub fn get_or_create_table_broadcaster(
        &self,
        table_id: &str,
    ) -> broadcast::Sender<WsOutgoingPacket> {
        let mut broadcasters = self.table_broadcasters.lock().unwrap();
        broadcasters
            .entry(table_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1000);
                tx
            })
            .clone()
    }

    /// Transmite uma atualização de estado para todos os clientes inscritos na mesa
    pub fn broadcast_to_table(&self, table_id: &str, payload: &str) -> Result<usize, String> {
        let broadcaster = self.get_or_create_table_broadcaster(table_id);
        let packet = WsOutgoingPacket {
            event_type: "TABLE_STATE_UPDATE".into(),
            table_id: table_id.into(),
            payload: payload.into(),
        };

        broadcaster
            .send(packet)
            .map_err(|e| format!("Erro no broadcast: {}", e))
    }

    /// Processa pacotes WebSocket recebidos e os encaminha para o TableActor correspondente
    pub async fn process_incoming_packet(
        &self,
        packet: WsIncomingPacket,
        table_sender: &mpsc::Sender<TableMessage>,
        ip_address: &str,
    ) -> Result<WsOutgoingPacket, String> {
        // 1. Rate Limiting Check com Notificação Amigável
        if self.rate_limiter.check_rate_limit(ip_address).is_err() {
            let notif = Self::create_player_notification_packet(
                "Lobby",
                "Connection",
                "Warning",
                "Muitas requisições em curto tempo",
                "Para a sua segurança, desaceleramos temporariamente o envio de mensagens.",
                Some("Aguarde alguns segundos para enviar novas ações."),
            );
            return Ok(notif);
        }

        // 2. Encaminhamento para o Actor de Mesa via canais Tokio sem bloqueio de I/O
        match packet.action {
            WsActionType::JoinTable { ref table_id, .. } => {
                let (resp_tx, resp_rx) = oneshot::channel();
                if table_sender
                    .send(TableMessage::PlayerJoin {
                        player_id: packet.player_id.clone(),
                        name: packet.player_id.clone(),
                        stack: 1000.0,
                        respond_to: resp_tx,
                    })
                    .await
                    .is_err()
                {
                    return Ok(Self::create_player_notification_packet(
                        table_id,
                        "Connection",
                        "Error",
                        "Servidor de Mesa Ocupado",
                        "Não foi possível conectar a esta mesa no momento.",
                        Some("Tente novamente em alguns segundos."),
                    ));
                }

                let resp = resp_rx
                    .await
                    .map_err(|_| "Resposta do ator cancelada".to_string())?;
                match resp {
                    Ok(()) => Ok(WsOutgoingPacket {
                        event_type: "JOIN_SUCCESS".into(),
                        table_id: table_id.clone(),
                        payload: format!("Jogador {} entrou na mesa", packet.player_id),
                    }),
                    Err(err) => Ok(Self::create_player_notification_packet(
                        table_id,
                        "Antifraud",
                        "Warning",
                        "Entrada Não Permitida",
                        &format!("Entrada recusada: {}", err),
                        Some("Escolha outra mesa para jogar com total segurança."),
                    )),
                }
            }
            WsActionType::PostBet { amount } => {
                let (resp_tx, resp_rx) = oneshot::channel();
                if table_sender
                    .send(TableMessage::PlayerAction {
                        player_id: packet.player_id.clone(),
                        action: Action::Bet(amount),
                        respond_to: resp_tx,
                    })
                    .await
                    .is_err()
                {
                    return Ok(Self::create_player_notification_packet(
                        "Table_1",
                        "Connection",
                        "Error",
                        "Falha no envio de aposta",
                        "Sua aposta não pôde ser enviada ao servidor.",
                        Some("Verifique sua conexão e tente novamente."),
                    ));
                }

                let resp = resp_rx
                    .await
                    .map_err(|_| "Timeout de resposta".to_string())?;
                match resp {
                    Ok(_state) => {
                        let broadcast_msg =
                            format!("Jogador {} apostou {:.2} Fichas", packet.player_id, amount);
                        let _ = self.broadcast_to_table("Table_1", &broadcast_msg);

                        Ok(WsOutgoingPacket {
                            event_type: "BET_SUCCESS".into(),
                            table_id: "Table_1".into(),
                            payload: broadcast_msg,
                        })
                    }
                    Err(err) => Ok(Self::create_player_notification_packet(
                        "Table_1",
                        "Finance",
                        "Warning",
                        "Aposta Não Efetuada",
                        &format!("Ação recusada: {}", err),
                        Some("Ajuste o valor da aposta conforme seu saldo disponível."),
                    )),
                }
            }
            WsActionType::Fold => Ok(WsOutgoingPacket {
                event_type: "ACTION_FOLD".into(),
                table_id: "Table_1".into(),
                payload: format!("Jogador {} desistiu (Fold)", packet.player_id),
            }),
            WsActionType::Check => Ok(WsOutgoingPacket {
                event_type: "ACTION_CHECK".into(),
                table_id: "Table_1".into(),
                payload: format!("Jogador {} passou a vez (Check)", packet.player_id),
            }),
            WsActionType::Call => Ok(WsOutgoingPacket {
                event_type: "ACTION_CALL".into(),
                table_id: "Table_1".into(),
                payload: format!("Jogador {} pagou a aposta (Call)", packet.player_id),
            }),
            WsActionType::LeaveTable => Ok(WsOutgoingPacket {
                event_type: "LEFT_TABLE".into(),
                table_id: "Table_1".into(),
                payload: format!("Jogador {} saiu da mesa", packet.player_id),
            }),
        }
    }
}
