//! Componente de Avatar/Jogador.
//!
//! Mostra informações de um jogador sentado na mesa:
//! nome, fichas, posição e indicador de turno.

use dioxus::prelude::*;

/// Posição do jogador na mesa.
#[allow(dead_code, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Dealer,
    SmallBlind,
    BigBlind,
    UTG,
    Middle,
    Cutoff,
    Button,
}

impl Position {
    /// Rótulo curto da posição.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dealer => "D",
            Self::SmallBlind => "SB",
            Self::BigBlind => "BB",
            Self::UTG => "UTG",
            Self::Middle => "MP",
            Self::Cutoff => "CO",
            Self::Button => "BTN",
        }
    }
}

/// Estado do jogador na rodada atual.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    /// Aguardando ação.
    Waiting,
    /// É a vez dele de agir.
    Acting,
    /// Já foldou nesta rodada.
    Folded,
    /// All-in (sem mais fichas).
    AllIn,
    /// Venceu o pot.
    Winner,
}

impl PlayerStatus {
    /// Cor CSS associada ao status.
    #[must_use]
    pub const fn color_class(self) -> &'static str {
        match self {
            Self::Waiting => "border-green-700/50",
            Self::Acting => "border-yellow-400 ring-2 ring-yellow-400",
            Self::Folded => "border-gray-600 opacity-50",
            Self::AllIn => "border-red-500",
            Self::Winner => "border-yellow-300 ring-2 ring-yellow-300",
        }
    }

    /// Ícone representativo do status.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Waiting => "",
            Self::Acting => "👉",
            Self::Folded => "✗",
            Self::AllIn => "🔥",
            Self::Winner => "🏆",
        }
    }
}

/// Componente visual de avatar/jogador.
///
/// # Props
///
/// - `name`: nome do jogador
/// - `chips`: quantidade de fichas
/// - `position`: posição na mesa
/// - `status`: estado atual do jogador
/// - `cards`: cartas na mão (None = não mostrar)
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Avatar(
    name: String,
    chips: u32,
    position: Position,
    status: PlayerStatus,
    cards: Option<Vec<super::card::PlayingCard>>,
) -> Element {
    let border_class = status.color_class();
    let icon = status.icon();

    rsx! {
        div {
            class: "bg-green-900/80 border-2 {border_class} rounded-lg p-3 min-w-[140px] \
                    flex flex-col items-center gap-1 shadow-md",
            div {
                class: "flex items-center justify-between w-full",
                span {
                    class: "text-xs font-bold text-yellow-300 bg-green-950 \
                            border border-yellow-300/30 rounded px-1.5",
                    "{position.label()}"
                }
                span {
                    class: "text-sm",
                    "{icon}"
                }
            }
            div {
                class: "w-12 h-12 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 \
                        flex items-center justify-center text-white font-bold text-lg",
                "{name.chars().next().unwrap_or('?')}"
            }
            div {
                class: "text-sm font-semibold text-white truncate max-w-[120px]",
                "{name}"
            }
            div {
                class: "text-xs text-yellow-200 font-mono",
                "💰 {chips}"
            }
            if let Some(hand) = cards {
                if !hand.is_empty() {
                    div {
                        class: "flex gap-1 mt-1",
                        for c in hand.iter() {
                            super::card::Card { card: *c, face_down: Some(false) }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_labels() {
        assert_eq!(Position::Dealer.label(), "D");
        assert_eq!(Position::SmallBlind.label(), "SB");
        assert_eq!(Position::BigBlind.label(), "BB");
        assert_eq!(Position::UTG.label(), "UTG");
        assert_eq!(Position::Button.label(), "BTN");
    }

    #[test]
    fn test_status_colors() {
        assert!(PlayerStatus::Acting.color_class().contains("yellow"));
        assert!(PlayerStatus::Folded.color_class().contains("opacity"));
        assert!(PlayerStatus::AllIn.color_class().contains("red"));
        assert!(PlayerStatus::Winner.color_class().contains("yellow"));
    }

    #[test]
    fn test_status_icons() {
        assert_eq!(PlayerStatus::Acting.icon(), "👉");
        assert_eq!(PlayerStatus::Folded.icon(), "✗");
        assert_eq!(PlayerStatus::AllIn.icon(), "🔥");
        assert_eq!(PlayerStatus::Winner.icon(), "🏆");
    }
}
