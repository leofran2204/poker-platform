//! Indicador visual de jogadores (X/Y) com barra de progresso.
//!
//! Muda de cor conforme o nível de ocupação da mesa.

use dioxus::prelude::*;

/// Nível de ocupação da mesa.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OccupancyLevel {
    Empty,
    Low,
    Medium,
    High,
    Full,
}

impl OccupancyLevel {
    /// Determina o nível a partir das contagens de jogadores.
    #[must_use]
    pub fn from_counts(players: u32, max_players: u32) -> Self {
        match (max_players, players) {
            (0, _) | (_, 0) => Self::Empty,
            (max, p) if p >= max => Self::Full,
            (max, p) => {
                let pct = (p * 100) / max;
                match pct {
                    0..=33 => Self::Low,
                    34..=66 => Self::Medium,
                    _ => Self::High,
                }
            }
        }
    }

    /// Cor do texto conforme o nível.
    #[must_use]
    pub fn text_color(self) -> &'static str {
        match self {
            Self::Empty => "player-count-text-empty",
            Self::Low => "player-count-text-low",
            Self::Medium => "player-count-text-medium",
            Self::High => "player-count-text-high",
            Self::Full => "player-count-text-full",
        }
    }

    /// Cor da barra de progresso conforme o nível.
    #[must_use]
    pub fn bar_color(self) -> &'static str {
        match self {
            Self::Empty => "player-count-bar-empty",
            Self::Low => "player-count-bar-low",
            Self::Medium => "player-count-bar-medium",
            Self::High => "player-count-bar-high",
            Self::Full => "player-count-bar-full",
        }
    }

    /// Descrição textual curta do nível.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "vazia",
            Self::Low => "poucos jogadores",
            Self::Medium => "moderada",
            Self::High => "quase cheia",
            Self::Full => "cheia",
        }
    }
}

/// Indicador de contagem de jogadores exibido no card da mesa.
///
/// Estilo Full Tilt Poker: barra de progresso mais espessa,
/// cores sólidas, sem animações de transição.
#[allow(dead_code)]
#[component]
pub fn PlayerCount(players: u32, max_players: u32) -> Element {
    let level = OccupancyLevel::from_counts(players, max_players);
    #[allow(clippy::manual_checked_ops)]
    let percent = if max_players == 0 {
        0
    } else {
        (players * 100) / max_players
    };

    rsx! {
        div {
            class: "player-count",
            div {
                class: "player-count-numbers",
                span {
                    class: "player-count-current {level.text_color()}",
                    "{players}"
                }
                span {
                    class: "player-count-separator",
                    "/"
                }
                span {
                    class: "player-count-max",
                    "{max_players}"
                }
            }
            div {
                class: "player-count-bar-track",
                div {
                    class: "player-count-bar-fill {level.bar_color()}",
                    style: "width: {percent}%;",
                }
            }
            span {
                class: "player-count-label",
                "{level.label()}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occupancy_level_empty_quando_zero_jogadores() {
        assert_eq!(OccupancyLevel::from_counts(0, 8), OccupancyLevel::Empty);
    }

    #[test]
    fn occupancy_level_empty_quando_max_zero() {
        assert_eq!(OccupancyLevel::from_counts(5, 0), OccupancyLevel::Empty);
    }

    #[test]
    fn occupancy_level_full_quando_atinge_max() {
        assert_eq!(OccupancyLevel::from_counts(8, 8), OccupancyLevel::Full);
        assert_eq!(OccupancyLevel::from_counts(10, 10), OccupancyLevel::Full);
    }

    #[test]
    fn occupancy_level_low_ate_33_porcento() {
        assert_eq!(OccupancyLevel::from_counts(1, 8), OccupancyLevel::Low);
        assert_eq!(OccupancyLevel::from_counts(2, 8), OccupancyLevel::Low);
    }

    #[test]
    fn occupancy_level_medium_entre_34_e_66_porcento() {
        assert_eq!(OccupancyLevel::from_counts(3, 8), OccupancyLevel::Medium);
        assert_eq!(OccupancyLevel::from_counts(5, 8), OccupancyLevel::Medium);
    }

    #[test]
    fn occupancy_level_high_acima_de_66_porcento() {
        assert_eq!(OccupancyLevel::from_counts(6, 8), OccupancyLevel::High);
        assert_eq!(OccupancyLevel::from_counts(7, 8), OccupancyLevel::High);
    }

    #[test]
    fn occupancy_level_text_color_diferentes_por_nivel() {
        assert_ne!(
            OccupancyLevel::Empty.text_color(),
            OccupancyLevel::Full.text_color()
        );
        assert_ne!(
            OccupancyLevel::Low.text_color(),
            OccupancyLevel::High.text_color()
        );
    }

    #[test]
    fn occupancy_level_label_retorna_strings_nao_vazias() {
        assert!(!OccupancyLevel::Empty.label().is_empty());
        assert!(!OccupancyLevel::Full.label().is_empty());
        assert!(!OccupancyLevel::Medium.label().is_empty());
    }
}
