//! Componente de Assento (Seat).
//!
//! Combina um Avatar com sua posição absoluta na mesa
//! (coordenadas em %) para layout em mesa oval.

use dioxus::prelude::*;

use super::avatar::{Avatar, PlayerStatus, Position};
use super::card::PlayingCard;

/// Posição absoluta na mesa (em % do container).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeatPosition {
    pub top_percent: f32,
    pub left_percent: f32,
}

impl SeatPosition {
    /// Cria uma nova posição absoluta.
    #[must_use]
    pub const fn new(top_percent: f32, left_percent: f32) -> Self {
        Self {
            top_percent,
            left_percent,
        }
    }
}

/// Componente de assento de jogador.
///
/// # Props
///
/// - `name`: nome do jogador
/// - `chips`: quantidade de fichas
/// - `position`: posição relativa na mesa
/// - `status`: estado atual
/// - `cards`: cartas na mão
/// - `seat_pos`: posição absoluta (top/left em %)
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Seat(
    name: String,
    chips: u64,
    position: Position,
    status: PlayerStatus,
    cards: Option<Vec<PlayingCard>>,
    seat_pos: SeatPosition,
) -> Element {
    let style = format!(
        "position: absolute; top: {}%; left: {}%; transform: translate(-50%, -50%);",
        seat_pos.top_percent, seat_pos.left_percent
    );

    rsx! {
        div {
            style: "{style}",
            Avatar {
                name: name,
                chips: chips,
                position: position,
                status: status,
                cards: cards,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seat_position_new() {
        let pos = SeatPosition::new(50.0, 50.0);
        assert_eq!(pos.top_percent, 50.0);
        assert_eq!(pos.left_percent, 50.0);
    }

    #[test]
    fn test_seat_position_equality() {
        let a = SeatPosition::new(10.0, 20.0);
        let b = SeatPosition::new(10.0, 20.0);
        let c = SeatPosition::new(30.0, 40.0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
