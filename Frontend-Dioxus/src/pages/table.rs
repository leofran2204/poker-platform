//! Página de Mesa — tela principal de jogo.
//!
//! Renderiza a mesa de poker com cartas, jogadores e ações.
//! O `id` da mesa vem como parâmetro de path (`/table/:id`).
//!
//! Conecta via WebSocket à API Axum para receber atualizações
//! em tempo real do motor de poker.

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::api_client;
use crate::components::action_buttons::ActionKind;
use crate::components::avatar::{PlayerStatus, Position};
use crate::components::card::{PlayingCard, Rank, Suit};
use crate::components::community_cards::CommunityStage;
use crate::components::deflator_notification::{DeflatorNotification, DeflatorPayload};
use crate::components::pot::PotEntry;
use crate::components::seat::SeatPosition;
use crate::components::table::{PlayerData, TableView};
use crate::router::Route;
use crate::ws_client::{self, WsCallbacks, WsClient, WsConnectionState};

/// Estado interno da página de mesa.
#[derive(Clone)]
struct TableState {
    /// Jogadores na mesa.
    players: Vec<PlayerData>,
    /// Cartas comunitárias.
    community_cards: Vec<PlayingCard>,
    /// Estágio atual.
    stage: CommunityStage,
    /// Potes ativos.
    pots: Vec<PotEntry>,
    /// Ações disponíveis.
    available_actions: Vec<ActionKind>,
    ws_state: WsConnectionState,
    /// ID do jogador local.
    local_player_id: Option<String>,
    /// Payload do evento Loss Deflator para exibir o overlay.
    deflator_payload: Option<DeflatorPayload>,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            players: Vec::new(),
            community_cards: Vec::new(),
            stage: CommunityStage::PreFlop,
            pots: Vec::new(),
            available_actions: Vec::new(),
            ws_state: WsConnectionState::Disconnected,
            local_player_id: None,
            deflator_payload: None,
        }
    }
}

// ─── Helpers de Conversão ───

/// Converte string de carta (ex: "As") para PlayingCard.
fn parse_card(s: &str) -> Option<PlayingCard> {
    if s.len() < 2 {
        return None;
    }
    let (rank_str, suit_char) = s.split_at(s.len() - 1);
    let suit = match suit_char {
        "s" => Some(Suit::Spades),
        "h" => Some(Suit::Hearts),
        "d" => Some(Suit::Diamonds),
        "c" => Some(Suit::Clubs),
        _ => None,
    }?;
    let rank = match rank_str {
        "A" => Some(Rank::Ace),
        "2" => Some(Rank::Two),
        "3" => Some(Rank::Three),
        "4" => Some(Rank::Four),
        "5" => Some(Rank::Five),
        "6" => Some(Rank::Six),
        "7" => Some(Rank::Seven),
        "8" => Some(Rank::Eight),
        "9" => Some(Rank::Nine),
        "10" => Some(Rank::Ten),
        "J" => Some(Rank::Jack),
        "Q" => Some(Rank::Queen),
        "K" => Some(Rank::King),
        _ => None,
    }?;
    Some(PlayingCard::new(suit, rank))
}

/// Converte string de ação do WS para ActionKind.
fn parse_action(s: &str) -> Option<ActionKind> {
    match s.to_lowercase().as_str() {
        "fold" => Some(ActionKind::Fold),
        "check" => Some(ActionKind::Check),
        "call" => Some(ActionKind::Call),
        "raise" => Some(ActionKind::Raise),
        "allin" | "all_in" | "all-in" => Some(ActionKind::AllIn),
        _ => None,
    }
}

/// Converte string de estágio para CommunityStage.
fn parse_stage(s: &str) -> CommunityStage {
    match s.to_lowercase().as_str() {
        "preflop" | "pre-flop" => CommunityStage::PreFlop,
        "flop" => CommunityStage::Flop,
        "turn" => CommunityStage::Turn,
        "river" => CommunityStage::River,
        _ => CommunityStage::PreFlop,
    }
}

/// Converte PlayerWsData para PlayerData com posição no layout.
fn ws_player_to_player_data(ws: &ws_client::PlayerWsData, index: usize) -> PlayerData {
    // Posições no layout oval (distribuição circular)
    let positions = [
        SeatPosition::new(20.0, 50.0), // topo
        SeatPosition::new(50.0, 85.0), // direita
        SeatPosition::new(80.0, 70.0), // baixo-direita
        SeatPosition::new(80.0, 30.0), // baixo-esquerda
        SeatPosition::new(50.0, 15.0), // esquerda
        SeatPosition::new(35.0, 30.0), // meio-esquerda
        SeatPosition::new(35.0, 70.0), // meio-direita
        SeatPosition::new(65.0, 50.0), // centro-baixo
    ];
    let seat_pos = positions[index % positions.len()];

    let position = if ws.is_dealer {
        Position::Dealer
    } else {
        match index {
            0 => Position::Button,
            1 => Position::SmallBlind,
            2 => Position::BigBlind,
            3 => Position::UTG,
            4 => Position::Middle,
            5 => Position::Cutoff,
            _ => Position::Middle,
        }
    };

    let status = if !ws.is_active {
        PlayerStatus::Folded
    } else {
        PlayerStatus::Waiting
    };

    let cards = if ws.cards.is_empty() {
        None
    } else {
        Some(ws.cards.iter().filter_map(|c| parse_card(c)).collect())
    };

    PlayerData {
        name: ws.name.clone(),
        chips: ws.chips,
        position,
        status,
        cards,
        seat_pos,
    }
}

/// Converte PotWsData para PotEntry.
fn ws_pot_to_pot_entry(ws: &ws_client::PotWsData) -> PotEntry {
    PotEntry::new(ws.name.clone(), ws.amount)
}

/// Converte string de ação do frontend para string do WebSocket.
fn action_kind_to_ws(action: ActionKind) -> &'static str {
    match action {
        ActionKind::Fold => "fold",
        ActionKind::Check => "check",
        ActionKind::Call => "call",
        ActionKind::Raise => "raise",
        ActionKind::AllIn => "all-in",
    }
}

/// Componente da página de mesa.
///
/// Recebe o `id` da mesa via parâmetro de rota do `dioxus-router`.
#[component]
pub fn Table(id: String) -> Element {
    // Estado compartilhado via Rc<RefCell<>> dentro de Signal para permitir
    // mutação em closures Fn (callbacks do WebSocket).
    let state = use_signal(|| Rc::new(RefCell::new(TableState::default())));
    let ws_client_ref = use_signal(|| Option::<WsClient>::None);

    // Clone de id para uso no render (use_effect captura id por move)
    let id_display = id.clone();

    // Conecta ao WebSocket ao montar o componente
    use_effect(move || {
        let table_id = id.clone();
        let mut state_for_effect = state;
        let mut ws_ref = ws_client_ref;

        spawn(async move {
            let token = match api_client::get_token() {
                Some(t) => t,
                None => {
                    state_for_effect.write().borrow_mut().ws_state =
                        WsConnectionState::Error("Usuário não autenticado".to_string());
                    return;
                }
            };

            // Clones para cada closure (cada closure precisa de sua própria Rc).
            let mut state_for_conn = state_for_effect;
            let mut state_for_msg = state_for_effect;
            let mut state_for_err = state_for_effect;

            let callbacks = WsCallbacks {
                on_connection_state: Some(std::rc::Rc::new(std::cell::RefCell::new(
                    move |conn_state: WsConnectionState| {
                        state_for_conn.write().borrow_mut().ws_state = conn_state;
                    },
                ))),
                on_message: Some(std::rc::Rc::new(std::cell::RefCell::new(
                    move |msg: ws_client::ServerMessage| {
                        let binding = state_for_msg.write();
                        let mut s = binding.borrow_mut();
                        match msg {
                            ws_client::ServerMessage::Welcome { player_id, .. } => {
                                s.local_player_id = Some(player_id);
                            }
                            ws_client::ServerMessage::TableState {
                                players,
                                community_cards,
                                stage,
                                pots,
                                available_actions,
                            } => {
                                s.players = players
                                    .iter()
                                    .enumerate()
                                    .map(|(i, p)| ws_player_to_player_data(p, i))
                                    .collect();
                                s.community_cards = community_cards
                                    .iter()
                                    .filter_map(|c| parse_card(c))
                                    .collect();
                                s.stage = parse_stage(&stage);
                                s.pots = pots.iter().map(ws_pot_to_pot_entry).collect();
                                s.available_actions = available_actions
                                    .iter()
                                    .filter_map(|a| parse_action(a))
                                    .collect();

                                // Se começou uma nova mão (PreFlop sem cartas comunitárias), limpar o overlay do Deflator
                                if s.stage == CommunityStage::PreFlop
                                    && s.community_cards.is_empty()
                                {
                                    s.deflator_payload = None;
                                }
                            }
                            ws_client::ServerMessage::DeflatorTriggered {
                                loser_name,
                                winner_name,
                                cashback_amount,
                                deflator_percent,
                                loser_equity_percent,
                                odds_broken: _, // winner upset % (compat); not cashback tier
                                prevented_elimination,
                                is_tournament,
                            } => {
                                let applied_percent = deflator_percent.unwrap_or_default();
                                s.deflator_payload = Some(DeflatorPayload {
                                    loser_name,
                                    winner_name,
                                    cashback_amount,
                                    deflator_percent: applied_percent,
                                    loser_equity_percent,
                                    prevented_elimination,
                                    is_tournament,
                                });
                            }
                            ws_client::ServerMessage::YourTurn { actions, .. } => {
                                s.available_actions =
                                    actions.iter().filter_map(|a| parse_action(a)).collect();
                            }
                            ws_client::ServerMessage::ActionResult { success, message } => {
                                if !success {
                                    log::error!("Ação rejeitada: {message}");
                                }
                            }
                            ws_client::ServerMessage::Error { message } => {
                                log::error!("Erro do servidor: {message}");
                            }
                            _ => {}
                        }
                    },
                ))),
                on_error: Some(std::rc::Rc::new(std::cell::RefCell::new(
                    move |err: String| {
                        log::error!("WebSocket error: {err}");
                        let _ = state_for_err.write().borrow_mut();
                    },
                ))),
            };

            let mut client = WsClient::new(table_id, token, callbacks);
            client.connect().await;
            ws_ref.set(Some(client));
        });
    });

    // Callback de ação — envia via WebSocket
    let on_action = move |action: ActionKind| {
        let ws_ref = ws_client_ref;
        let action_str = action_kind_to_ws(action);
        let snapshot_ref = state.read().borrow().clone();
        if let Some(ref client) = *ws_ref.read() {
            let amount: u64 = match action {
                ActionKind::AllIn => {
                    let local_id = snapshot_ref.local_player_id.as_ref();
                    snapshot_ref
                        .players
                        .iter()
                        .find(|p| Some(&p.name) == local_id)
                        .map(|p| p.chips)
                        .unwrap_or(0)
                }
                ActionKind::Raise => 3000, // 3000 centavos (R$ 30,00)
                _ => 0,
            };
            client.send_action(action_str, Some(amount));
        }
    };

    // Snapshot do estado atual para renderização
    let snapshot = state.read().borrow().clone();
    let ws_state = snapshot.ws_state.clone();

    // Determina mensagem de status da conexão
    let status_text = match ws_state {
        WsConnectionState::Connected => {
            format!("Conectado · Mesa {}", id_display)
        }
        WsConnectionState::Connecting => "Conectando...".to_string(),
        WsConnectionState::Disconnected => "Desconectado".to_string(),
        WsConnectionState::Error(ref e) => format!("Erro: {e}"),
    };

    let status_class = match ws_state {
        WsConnectionState::Connected => "text-green-300/70",
        WsConnectionState::Connecting => "text-yellow-300/70",
        WsConnectionState::Disconnected => "text-gray-500",
        WsConnectionState::Error(_) => "text-red-400",
    };

    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-green-950 to-green-900 text-white",
            // Header da mesa
            div {
                class: "container mx-auto px-6 py-4 max-w-6xl flex items-center justify-between",
                Link {
                    to: Route::Lobby {},
                    class: "text-sm text-green-300 hover:text-white underline",
                    "← Voltar ao Lobby"
                }
                div {
                    class: "text-xs {status_class}",
                    "{status_text}"
                }
            }

            // Notificação do Loss Deflator
            DeflatorNotification {
                payload: snapshot.deflator_payload.clone(),
            }

            // Mesa completa
            TableView {
                table_id: id_display,
                players: snapshot.players.clone(),
                community_cards: snapshot.community_cards.clone(),
                stage: snapshot.stage,
                pots: snapshot.pots.clone(),
                available_actions: snapshot.available_actions.clone(),
                on_action: on_action,
                odd_cent_notice: None,
            }

            // Indicador de status do jogador local
            div {
                class: "container mx-auto px-6 py-2 max-w-6xl text-center",
                p {
                    class: "text-xs text-green-300/70 italic",
                    if ws_state == WsConnectionState::Connected {
                        "Você está na mesa. Aguardando sua vez..."
                    } else {
                        "Aguardando conexão com o servidor..."
                    }
                }
            }
        }
    }
}
