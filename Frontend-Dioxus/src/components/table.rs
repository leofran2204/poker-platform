//! Componente de Mesa de Poker completa.
//!
//! Integra todos os componentes (Seats, CommunityCards, Pot, ActionButtons)
//! em uma única view oval representando a mesa de poker.

use dioxus::prelude::*;

use super::action_buttons::{ActionButtons, ActionKind};
use super::avatar::{PlayerStatus, Position};
use super::card::{PlayingCard, Rank, Suit};

use super::community_cards::{CommunityCards, CommunityStage};
use super::pot::{Pot, PotEntry};
use super::seat::{Seat, SeatPosition};

/// Dados de um jogador sentado na mesa.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerData {
    pub name: String,
    pub chips: u64,
    pub position: Position,
    pub status: PlayerStatus,
    pub cards: Option<Vec<PlayingCard>>,
    pub seat_pos: SeatPosition,
}

impl PlayerData {
    /// Cria dados de jogador com posição padrão em centavos.
    #[must_use]
    pub fn new(name: String, chips: u64, position: Position, seat_pos: SeatPosition) -> Self {
        Self {
            name,
            chips,
            position,
            status: PlayerStatus::Waiting,
            cards: None,
            seat_pos,
        }
    }
}

/// Componente de mesa de poker completa.
///
/// # Props
///
/// - `table_id`: identificador da mesa
/// - `players`: lista de jogadores sentados
/// - `community_cards`: cartas comunitárias reveladas
/// - `stage`: estágio atual da rodada
/// - `pots`: potes (principal + side pots)
/// - `available_actions`: ações disponíveis para o jogador local
/// - `on_action`: callback quando uma ação é selecionada
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableView(
    table_id: String,
    players: Vec<PlayerData>,
    community_cards: Vec<PlayingCard>,
    stage: CommunityStage,
    pots: Vec<PotEntry>,
    available_actions: Vec<ActionKind>,
    on_action: EventHandler<ActionKind>,
    odd_cent_notice: Option<String>,
) -> Element {
    rsx! {
        main {
            class: "container mx-auto px-6 py-6 max-w-6xl",
            div {
                class: "flex items-center justify-between mb-4",
                h2 {
                    class: "text-2xl font-bold text-yellow-400",
                    "🎲 Mesa: {table_id}"
                }
                div {
                    class: "text-sm text-green-300",
                    "Rodada: {stage.label()}"
                }
            }

            // Área da mesa (formato oval)
            div {
                class: "relative bg-green-800/60 border-4 border-green-700/50 rounded-[50%] \
                        aspect-[2/1] mx-auto max-w-4xl shadow-2xl",

                // Pote central
                div {
                    class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 \
                            flex flex-col items-center gap-4",
                    Pot { pots: pots.clone(), odd_cent_notice: odd_cent_notice }
                    CommunityCards {
                        cards: community_cards.clone(),
                        stage: stage,
                    }
                }

                // Assentos dos jogadores
                for player in players.iter() {
                    Seat {
                        name: player.name.clone(),
                        chips: player.chips,
                        position: player.position,
                        status: player.status,
                        cards: player.cards.clone(),
                        seat_pos: player.seat_pos,
                    }
                }
            }

            // Botões de ação
            ActionButtons {
                available: available_actions.clone(),
                on_action: on_action,
            }
        }
    }
}

/// Helper para gerar dados de exemplo (mock) para desenvolvimento/testes.
#[must_use]
pub fn mock_table_data() -> Vec<PlayerData> {
    vec![
        PlayerData::new(
            "Alice".to_string(),
            1500,
            Position::Dealer,
            SeatPosition::new(20.0, 50.0),
        ),
        PlayerData::new(
            "Bob".to_string(),
            2000,
            Position::SmallBlind,
            SeatPosition::new(50.0, 85.0),
        ),
        PlayerData::new(
            "Carol".to_string(),
            1200,
            Position::BigBlind,
            SeatPosition::new(80.0, 70.0),
        ),
        PlayerData::new(
            "Dave".to_string(),
            1800,
            Position::UTG,
            SeatPosition::new(80.0, 30.0),
        ),
        PlayerData::new(
            "Eve".to_string(),
            2500,
            Position::Button,
            SeatPosition::new(50.0, 15.0),
        ),
    ]
}

/// Helper para gerar cartas comunitárias de exemplo (Flop).
#[must_use]
pub fn mock_flop() -> Vec<PlayingCard> {
    vec![
        PlayingCard::new(Suit::Spades, Rank::Ace),
        PlayingCard::new(Suit::Hearts, Rank::King),
        PlayingCard::new(Suit::Diamonds, Rank::Queen),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_data_new() {
        let p = PlayerData::new(
            "Test".to_string(),
            1000,
            Position::Dealer,
            SeatPosition::new(50.0, 50.0),
        );
        assert_eq!(p.name, "Test");
        assert_eq!(p.chips, 1000);
        assert_eq!(p.position, Position::Dealer);
        assert_eq!(p.status, PlayerStatus::Waiting);
        assert!(p.cards.is_none());
    }

    #[test]
    fn test_mock_table_data_count() {
        let players = mock_table_data();
        assert_eq!(players.len(), 5);
    }

    #[test]
    fn test_mock_flop_count() {
        let flop = mock_flop();
        assert_eq!(flop.len(), 3);
    }
}
