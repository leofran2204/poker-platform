//! Componente de Carta individual.
//!
//! Renderiza uma carta de poker com naipe e valor, ou o verso.
//! Suporta estado "virada" (face down) para cartas não reveladas.

use dioxus::prelude::*;

/// Naipe da carta.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    /// Símbolo unicode do naipe.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Spades => "♠",
            Self::Hearts => "♥",
            Self::Diamonds => "♦",
            Self::Clubs => "♣",
        }
    }

    /// Cor CSS associada ao naipe (vermelho para copas/ouros, preto para paus/espadas).
    #[must_use]
    pub fn color_class(self) -> &'static str {
        match self {
            Self::Hearts | Self::Diamonds => "text-red-500",
            Self::Spades | Self::Clubs => "text-white",
        }
    }
}

/// Valor da carta (Ás a Rei).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    /// Representação textual do valor (A, 2-10, J, Q, K).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Ace => "A",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
        }
    }
}

/// Carta de poker completa (naipe + valor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayingCard {
    pub suit: Suit,
    pub rank: Rank,
}

impl PlayingCard {
    /// Cria uma nova carta.
    #[must_use]
    pub const fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }
}

/// Componente visual de uma carta.
///
/// # Props
///
/// - `card`: dados da carta (naipe + valor)
/// - `face_down`: se `true`, mostra o verso em vez da face
#[component]
pub fn Card(card: PlayingCard, face_down: Option<bool>) -> Element {
    let face_down = face_down.unwrap_or(false);

    if face_down {
        rsx! {
            div {
                class: "w-16 h-24 bg-gradient-to-br from-blue-700 to-blue-900 \
                        border-2 border-white/30 rounded-lg shadow-lg \
                        flex items-center justify-center",
                div {
                    class: "text-2xl text-white/50",
                    "🂠"
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "w-16 h-24 bg-white border border-gray-300 rounded-lg shadow-lg \
                        flex flex-col items-center justify-center p-1",
                div {
                    class: "text-lg font-bold leading-none {card.suit.color_class()}",
                    "{card.rank.label()}"
                }
                div {
                    class: "text-2xl leading-none {card.suit.color_class()}",
                    "{card.suit.symbol()}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suit_symbols() {
        assert_eq!(Suit::Spades.symbol(), "♠");
        assert_eq!(Suit::Hearts.symbol(), "♥");
        assert_eq!(Suit::Diamonds.symbol(), "♦");
        assert_eq!(Suit::Clubs.symbol(), "♣");
    }

    #[test]
    fn test_suit_colors() {
        assert_eq!(Suit::Hearts.color_class(), "text-red-500");
        assert_eq!(Suit::Diamonds.color_class(), "text-red-500");
        assert_eq!(Suit::Spades.color_class(), "text-white");
        assert_eq!(Suit::Clubs.color_class(), "text-white");
    }

    #[test]
    fn test_rank_labels() {
        assert_eq!(Rank::Ace.label(), "A");
        assert_eq!(Rank::Ten.label(), "10");
        assert_eq!(Rank::Jack.label(), "J");
        assert_eq!(Rank::Queen.label(), "Q");
        assert_eq!(Rank::King.label(), "K");
    }

    #[test]
    fn test_playing_card_new() {
        let card = PlayingCard::new(Suit::Hearts, Rank::Ace);
        assert_eq!(card.suit, Suit::Hearts);
        assert_eq!(card.rank, Rank::Ace);
    }

    #[test]
    fn test_playing_card_equality() {
        let a = PlayingCard::new(Suit::Spades, Rank::King);
        let b = PlayingCard::new(Suit::Spades, Rank::King);
        let c = PlayingCard::new(Suit::Hearts, Rank::King);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
