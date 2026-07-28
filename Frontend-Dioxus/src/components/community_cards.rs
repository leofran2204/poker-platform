//! Componente de Cartas Comunitárias.
//!
//! Renderiza as 5 cartas comunitárias no centro da mesa
//! (Flop: 3 cartas, Turn: 4 cartas, River: 5 cartas).

use dioxus::prelude::*;

use super::card::{Card, PlayingCard};

/// Estágio atual da rodada comunitária.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunityStage {
    /// Pré-flop: nenhuma carta comunitária revelada.
    PreFlop,
    /// Flop: 3 cartas reveladas.
    Flop,
    /// Turn: 4 cartas reveladas.
    Turn,
    /// River: 5 cartas reveladas.
    River,
}

impl CommunityStage {
    /// Quantas cartas devem ser exibidas neste estágio.
    #[must_use]
    pub const fn card_count(self) -> usize {
        match self {
            Self::PreFlop => 0,
            Self::Flop => 3,
            Self::Turn => 4,
            Self::River => 5,
        }
    }

    /// Rótulo textual do estágio.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PreFlop => "Pré-Flop",
            Self::Flop => "Flop",
            Self::Turn => "Turn",
            Self::River => "River",
        }
    }
}

/// Componente de cartas comunitárias.
///
/// # Props
///
/// - `cards`: vetor com até 5 cartas (será truncado conforme o estágio)
/// - `stage`: estágio atual da rodada
#[component]
pub fn CommunityCards(cards: Vec<PlayingCard>, stage: CommunityStage) -> Element {
    let max = stage.card_count();
    let visible: Vec<PlayingCard> = cards.into_iter().take(max).collect();

    rsx! {
        div {
            class: "flex flex-col items-center gap-2",
            div {
                class: "text-xs uppercase tracking-wider text-yellow-200/80 font-semibold",
                "{stage.label()}"
            }
            div {
                class: "flex gap-2",
                if visible.is_empty() {
                    div {
                        class: "text-green-200/50 italic text-sm",
                        "Aguardando cartas..."
                    }
                } else {
                    for card in visible.iter() {
                        Card { card: *card, face_down: None }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::card::{Rank, Suit};

    #[test]
    fn test_stage_card_count() {
        assert_eq!(CommunityStage::PreFlop.card_count(), 0);
        assert_eq!(CommunityStage::Flop.card_count(), 3);
        assert_eq!(CommunityStage::Turn.card_count(), 4);
        assert_eq!(CommunityStage::River.card_count(), 5);
    }

    #[test]
    fn test_stage_labels() {
        assert_eq!(CommunityStage::PreFlop.label(), "Pré-Flop");
        assert_eq!(CommunityStage::Flop.label(), "Flop");
        assert_eq!(CommunityStage::Turn.label(), "Turn");
        assert_eq!(CommunityStage::River.label(), "River");
    }

    #[test]
    fn test_cards_truncation() {
        let cards = vec![
            PlayingCard::new(Suit::Spades, Rank::Ace),
            PlayingCard::new(Suit::Hearts, Rank::King),
            PlayingCard::new(Suit::Diamonds, Rank::Queen),
            PlayingCard::new(Suit::Clubs, Rank::Jack),
            PlayingCard::new(Suit::Spades, Rank::Ten),
        ];
        // Flop deve mostrar apenas 3
        let visible: Vec<_> = cards
            .into_iter()
            .take(CommunityStage::Flop.card_count())
            .collect();
        assert_eq!(visible.len(), 3);
    }
}
