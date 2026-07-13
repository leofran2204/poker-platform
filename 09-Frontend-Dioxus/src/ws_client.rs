//! Cliente WebSocket para comunicação em tempo real com a API Axum.
//!
//! Conecta ao endpoint `/ws/game/:table_id` para receber atualizações
//! do motor de poker e enviar ações do jogador.
//!
//! # URLs
//!
//! - **Dev (debug_assertions):** `ws://localhost:3000`
//! - **Prod:** `wss://api.pokerplatform.com`
//!
//! # Estados
//!
//! O cliente gerencia um `WsState` que reflete o ciclo de vida da conexão:
//! `Disconnected` → `Connecting` → `Connected` → `Disconnected`

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use wasm_bindgen_futures::spawn_local;

// ─── Constantes ───

/// Base URL do WebSocket — WS em dev, WSS em produção.
const WS_BASE: &str = if cfg!(debug_assertions) {
    "ws://localhost:3000"
} else {
    "wss://api.pokerplatform.com"
};

// ─── Tipos de Mensagem ───

/// Ações que o jogador pode enviar para o servidor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Ação do jogador (fold, check, call, raise, all-in).
    #[serde(rename = "action")]
    Action {
        /// Tipo da ação.
        action: String,
        /// Valor do raise (0 se não for raise/all-in).
        #[serde(default)]
        amount: u64,
    },
    /// Ping para manter a conexão ativa.
    #[serde(rename = "ping")]
    Ping,
    /// Solicita informações completas da mesa.
    #[serde(rename = "get_table_info")]
    GetTableInfo,
}

/// Mensagens recebidas do servidor.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Mensagem de boas-vindas ao conectar.
    #[serde(rename = "welcome")]
    Welcome {
        /// ID do jogador na mesa.
        player_id: String,
        /// Número do assento.
        seat: u8,
    },
    /// Estado atualizado da mesa.
    #[serde(rename = "table_state")]
    TableState {
        /// Jogadores na mesa.
        players: Vec<PlayerWsData>,
        /// Cartas comunitárias.
        community_cards: Vec<String>,
        /// Estágio da mão.
        stage: String,
        /// Potes ativos.
        pots: Vec<PotWsData>,
        /// Ações disponíveis para o jogador local.
        available_actions: Vec<String>,
    },
    /// Notificação de que é a vez do jogador.
    #[serde(rename = "your_turn")]
    YourTurn {
        /// Ações disponíveis.
        actions: Vec<String>,
        /// Tempo restante em segundos.
        time_bank: u64,
    },
    /// Resultado de uma ação.
    #[serde(rename = "action_result")]
    ActionResult {
        /// Se a ação foi bem-sucedida.
        success: bool,
        /// Mensagem descritiva.
        message: String,
    },
    /// Resposta a um ping.
    #[serde(rename = "pong")]
    Pong,
    /// Informações completas da mesa (resposta a get_table_info).
    #[serde(rename = "table_info")]
    TableInfo {
        /// Nome da mesa.
        name: String,
        /// Small blind.
        small_blind: u64,
        /// Big blind.
        big_blind: u64,
        /// Tipo de jogo.
        game_type: String,
    },
    /// Mensagem de erro do servidor.
    #[serde(rename = "error")]
    Error {
        /// Mensagem de erro.
        message: String,
    },
}

/// Dados de um jogador na mensagem WebSocket.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PlayerWsData {
    pub id: String,
    pub name: String,
    pub chips: u64,
    pub bet: u64,
    pub cards: Vec<String>,
    pub is_active: bool,
    pub is_dealer: bool,
    pub seat: u8,
}

/// Dados de um pote na mensagem WebSocket.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PotWsData {
    pub name: String,
    pub amount: u64,
    pub eligible_players: Vec<String>,
}

// ─── Estado da Conexão ───

/// Estados possíveis da conexão WebSocket.
#[derive(Debug, Clone, PartialEq)]
pub enum WsConnectionState {
    /// Desconectado (estado inicial ou após desconexão).
    Disconnected,
    /// Conectando (handshake em andamento).
    Connecting,
    /// Conectado e pronto para enviar/receber mensagens.
    Connected,
    /// Erro na conexão.
    Error(String),
}

// ─── Callbacks ───

/// Callbacks que o usuário do WebSocket pode registrar.
///
/// Os callbacks são `FnMut` (não `Fn`) e envolvidos em `Rc<RefCell<>>`
/// para permitir mutação de estado capturado (ex: `Signal::write()`).
type OnConnectionStateCb = std::rc::Rc<RefCell<dyn FnMut(WsConnectionState)>>;
type OnMessageCb = std::rc::Rc<RefCell<dyn FnMut(ServerMessage)>>;
type OnErrorCb = std::rc::Rc<RefCell<dyn FnMut(String)>>;

#[derive(Clone, Default)]
pub struct WsCallbacks {
    /// Chamado quando o estado da conexão muda.
    pub on_connection_state: Option<OnConnectionStateCb>,
    /// Chamado ao receber uma mensagem do servidor.
    pub on_message: Option<OnMessageCb>,
    /// Chamado em caso de erro.
    pub on_error: Option<OnErrorCb>,
}

// ─── Cliente WebSocket ───

/// Cliente WebSocket para comunicação com o servidor de jogo.
///
/// # Exemplo
///
/// ```ignore
/// let mut client = WsClient::new("mesa-01", callbacks);
/// client.connect().await;
/// client.send_action("raise", Some(200)).await;
/// ```
pub struct WsClient {
    /// ID da mesa conectada.
    table_id: String,
    /// Token JWT para autenticação.
    token: String,
    /// Callbacks registrados.
    callbacks: WsCallbacks,
    /// Sender para enviar mensagens para o loop WebSocket.
    tx: Option<UnboundedSender<String>>,
    /// Estado atual da conexão.
    state: WsConnectionState,
}

impl WsClient {
    /// Cria um novo cliente WebSocket para a mesa especificada.
    ///
    /// Requer o `table_id` e o token JWT para autenticação.
    pub fn new(table_id: String, token: String, callbacks: WsCallbacks) -> Self {
        Self {
            table_id,
            token,
            callbacks,
            tx: None,
            state: WsConnectionState::Disconnected,
        }
    }

    /// Retorna o estado atual da conexão.
    pub fn state(&self) -> &WsConnectionState {
        &self.state
    }

    /// Retorna o ID da mesa.
    pub fn table_id(&self) -> &str {
        &self.table_id
    }

    /// Conecta ao servidor WebSocket.
    ///
    /// Inicia o loop de mensagens em background. As mensagens recebidas
    /// são entregues via `callbacks.on_message`.
    pub async fn connect(&mut self) {
        let url = format!(
            "{WS_BASE}/ws/game/{}?token={}",
            self.table_id, self.token
        );

        self.state = WsConnectionState::Connecting;
        self.notify_connection_state();

        match ws_stream_wasm::WsMeta::connect(url, None).await {
            Ok((_ws_meta, ws_stream)) => {
                self.state = WsConnectionState::Connected;
                self.notify_connection_state();

                let (mut write, read) = ws_stream.split();
                let (tx, mut rx) = unbounded::<String>();

                self.tx = Some(tx);

                // Canal de saída: envia mensagens do sender para o WebSocket
                spawn_local(async move {
                    while let Some(msg) = rx.next().await {
                        if let Err(e) = write.send(msg.into()).await {
                            log::error!("Erro ao enviar mensagem WS: {e}");
                            break;
                        }
                    }
                });

                // Canal de entrada: recebe mensagens do WebSocket e chama callback
                let on_msg = self.callbacks.on_message.clone();
                let on_err = self.callbacks.on_error.clone();
                let on_conn = self.callbacks.on_connection_state.clone();

                spawn_local(async move {
                    let mut read = read;
                    while let Some(msg) = read.next().await {
                        // WsMessage::Text contém a string; outros tipos são ignorados
                        let text = match msg {
                            ws_stream_wasm::WsMessage::Text(t) => t,
                            _ => continue,
                        };
                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                            if let Some(ref cb) = on_msg {
                                (cb.borrow_mut())(server_msg);
                            }
                        } else {
                            log::warn!("Mensagem WS não reconhecida: {text}");
                        }
                    }

                    // Conexão fechou
                    if let Some(ref cb) = on_conn {
                        (cb.borrow_mut())(WsConnectionState::Disconnected);
                    }
                    if let Some(ref cb) = on_err {
                        (cb.borrow_mut())("Conexão WebSocket fechada".to_string());
                    }
                });
            }
            Err(e) => {
                let err_msg = format!("Erro ao conectar WebSocket: {e:?}");
                self.state = WsConnectionState::Error(err_msg.clone());
                self.notify_connection_state();
                if let Some(ref cb) = self.callbacks.on_error {
                    (cb.borrow_mut())(err_msg);
                }
            }
        }
    }

    /// Envia uma ação do jogador para o servidor.
    pub fn send_action(&self, action: &str, amount: Option<u64>) {
        let msg = ClientMessage::Action {
            action: action.to_string(),
            amount: amount.unwrap_or(0),
        };
        self.send_json(&msg);
    }

    /// Envia um ping para manter a conexão ativa.
    pub fn send_ping(&self) {
        self.send_json(&ClientMessage::Ping);
    }

    /// Solicita informações completas da mesa.
    pub fn request_table_info(&self) {
        self.send_json(&ClientMessage::GetTableInfo);
    }

    /// Desconecta do servidor WebSocket.
    pub fn disconnect(&mut self) {
        self.tx = None;
        self.state = WsConnectionState::Disconnected;
        self.notify_connection_state();
    }

    // ─── Métodos Internos ───

    /// Serializa e envia uma mensagem JSON pelo canal.
    fn send_json(&self, msg: &ClientMessage) {
        if let Some(ref tx) = self.tx
            && let Ok(json) = serde_json::to_string(msg)
        {
            let _ = tx.unbounded_send(json);
        }
    }

    /// Notifica o callback de mudança de estado.
    fn notify_connection_state(&self) {
        if let Some(ref cb) = self.callbacks.on_connection_state {
            (cb.borrow_mut())(self.state.clone());
        }
    }
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_serialize_action() {
        let msg = ClientMessage::Action {
            action: "raise".to_string(),
            amount: 200,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"action\""));
        assert!(json.contains("\"action\":\"raise\""));
        assert!(json.contains("\"amount\":200"));
    }

    #[test]
    fn test_client_message_serialize_ping() {
        let msg = ClientMessage::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, "{\"type\":\"ping\"}");
    }

    #[test]
    fn test_server_message_deserialize_welcome() {
        let json = r#"{"type":"welcome","player_id":"abc123","seat":3}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert_eq!(
            msg,
            ServerMessage::Welcome {
                player_id: "abc123".to_string(),
                seat: 3,
            }
        );
    }

    #[test]
    fn test_server_message_deserialize_pong() {
        let json = r#"{"type":"pong"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg, ServerMessage::Pong);
    }

    #[test]
    fn test_server_message_deserialize_error() {
        let json = r#"{"type":"error","message":"Mesa não encontrada"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert_eq!(
            msg,
            ServerMessage::Error {
                message: "Mesa não encontrada".to_string()
            }
        );
    }

    #[test]
    fn test_server_message_deserialize_table_state() {
        let json = r#"{
            "type":"table_state",
            "players":[
                {"id":"p1","name":"Alice","chips":1000,"bet":50,"cards":[],"is_active":true,"is_dealer":true,"seat":0}
            ],
            "community_cards":["As","Kh"],
            "stage":"flop",
            "pots":[{"name":"Main","amount":150,"eligible_players":["p1"]}],
            "available_actions":["fold","check","call"]
        }"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::TableState {
                players,
                community_cards,
                stage,
                pots,
                available_actions,
            } => {
                assert_eq!(players.len(), 1);
                assert_eq!(players[0].name, "Alice");
                assert_eq!(community_cards, vec!["As", "Kh"]);
                assert_eq!(stage, "flop");
                assert_eq!(pots.len(), 1);
                assert_eq!(pots[0].name, "Main");
                assert_eq!(available_actions, vec!["fold", "check", "call"]);
            }
            _ => panic!("Esperava TableState"),
        }
    }

    #[test]
    fn test_ws_connection_state_partial_eq() {
        assert_eq!(
            WsConnectionState::Disconnected,
            WsConnectionState::Disconnected
        );
        assert_eq!(
            WsConnectionState::Connected,
            WsConnectionState::Connected
        );
        assert_ne!(
            WsConnectionState::Disconnected,
            WsConnectionState::Connected
        );
        assert_ne!(
            WsConnectionState::Error("x".into()),
            WsConnectionState::Error("y".into())
        );
    }

    #[test]
    fn test_client_message_action_default_amount() {
        let msg = ClientMessage::Action {
            action: "fold".to_string(),
            amount: 0,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"amount\":0"));
    }
}