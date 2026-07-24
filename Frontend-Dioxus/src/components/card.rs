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

    /// Cor CSS associada ao naipe (vermelho para copas/ouros, escuro para paus/espadas).
    #[must_use]
    pub fn color_class(self) -> &'static str {
        match self {
            Self::Hearts | Self::Diamonds => "text-red-600",
            Self::Spades | Self::Clubs => "text-gray-900",
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

/// Componente visual de uma carta com estética realista e alta legibilidade.
#[component]
pub fn Card(card: PlayingCard, face_down: Option<bool>) -> Element {
    let face_down = face_down.unwrap_or(false);

    if face_down {
        rsx! {
            div {
                class: "w-16 h-24 bg-gradient-to-br from-indigo-800 via-blue-900 to-indigo-950 \
                        border-2 border-amber-400/40 rounded-xl shadow-xl \
                        flex items-center justify-center transform hover:scale-105 transition-transform duration-200",
                div {
                    class: "text-2xl text-amber-400/70 font-bold",
                    "♠"
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "w-16 h-24 bg-slate-50 border-2 border-slate-300 rounded-xl shadow-xl \
                        flex flex-col items-center justify-between p-2 select-none \
                        transform hover:scale-105 transition-transform duration-200",
                div {
                    class: "text-base font-extrabold leading-none self-start {card.suit.color_class()}",
                    "{card.rank.label()}"
                }
                div {
                    class: "text-3xl leading-none {card.suit.color_class()}",
                    "{card.suit.symbol()}"
                }
                div {
                    class: "text-xs font-bold leading-none self-end rotate-180 {card.suit.color_class()}",
                    "{card.rank.label()}"
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
        assert_eq!(Suit::Hearts.color_class(), "text-red-600");
        assert_eq!(Suit::Diamonds.color_class(), "text-red-600");
        assert_eq!(Suit::Spades.color_class(), "text-gray-900");
        assert_eq!(Suit::Clubs.color_class(), "text-gray-900");
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
