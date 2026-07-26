//! Componente de card de mesa para exibição no lobby.
//!
//! Mostra nome da mesa, tipo de jogo, blinds e contagem de jogadores.

use dioxus::prelude::*;

/// Tipos de jogo suportados pela plataforma.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    TexasHoldem,
    Omaha,
    OmahaHiLo,
    SevenCardStud,
    Tournament,
    Freeroll,
}

impl GameType {
    /// Nome de exibição do tipo de jogo.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::TexasHoldem => "Texas Hold'em",
            Self::Omaha => "Omaha",
            Self::OmahaHiLo => "Omaha Hi/Lo",
            Self::SevenCardStud => "Seven Card Stud",
            Self::Tournament => "Torneio",
            Self::Freeroll => "Freeroll",
        }
    }

    /// Símbolo de naipe associado ao tipo de jogo.
    #[must_use]
    pub fn icon(self) -> &'static str {
        match self {
            Self::TexasHoldem => "♠",
            Self::Omaha => "♥",
            Self::OmahaHiLo => "♦",
            Self::SevenCardStud => "♣",
            Self::Tournament => "♣",
            Self::Freeroll => "♣",
        }
    }
}

/// Dados de uma mesa provenientes do backend.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableData {
    pub id: String,
    pub name: String,
    pub game_type: GameType,
    pub small_blind: u64,
    pub big_blind: u64,
    pub players: u32,
    pub max_players: u32,
}

impl TableData {
    /// Formata os blinds no padrão "R$ X / R$ Y".
    #[must_use]
    pub fn blinds_display(&self) -> String {
        format!("R$ {} / R$ {}", self.small_blind, self.big_blind)
    }

    /// Verifica se a mesa está completamente ocupada.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.players >= self.max_players
    }

    /// Verifica se a mesa está vazia (zero jogadores).
    #[must_use]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.players == 0
    }

    /// Percentual de ocupação da mesa (0–100).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
    pub fn occupancy_percent(&self) -> u32 {
        if self.max_players == 0 {
            return 0;
        }
        (self.players * 100) / self.max_players
    }
}

/// Card de mesa exibido no lobby.
///
/// Estilo visual Full Tilt Poker: felt verde escuro, bordas grossas douradas,
/// cores sólidas sem transparência, cantos sutis, sem animações.
#[allow(dead_code)]
#[component]
pub fn TableCard(table: TableData) -> Element {
    let is_full = table.is_full();
    let occupancy = table.occupancy_percent();
    let economy_badge = match table.game_type {
        GameType::Tournament | GameType::Freeroll => "Fee 7% (Rebuy 0%)",
        _ => "Rake 3.5% (Cap R$ 5,00)",
    };

    rsx! {
        div {
            class: "table-card",
            div {
                class: "table-card-inner",
                div {
                    class: "table-card-left",
                    span {
                        class: "table-card-icon",
                        "{table.game_type.icon()}"
                    }
                    div {
                        h3 {
                            class: "table-card-name",
                            "{table.name}"
                        }
                        p {
                            class: "table-card-info",
                            "{table.game_type.display_name()}  •  {table.blinds_display()}  •  "
                            span {
                                class: "economy-badge",
                                style: "color: #ffca28; font-size: 0.9em; font-weight: bold;",
                                "{economy_badge}"
                            }
                        }
                    }
                }
                div {
                    class: "table-card-right",
                    div {
                        class: "table-card-count",
                        "{table.players}/{table.max_players}"
                    }
                    div {
                        class: "table-card-occupancy",
                        "jogadores ({occupancy}%)"
                    }
                }
            }
            if is_full {
                div {
                    class: "table-card-full",
                    "Mesa cheia"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(players: u32, max_players: u32) -> TableData {
        TableData {
            id: "test-001".to_string(),
            name: "Mesa Teste".to_string(),
            game_type: GameType::TexasHoldem,
            small_blind: 1,
            big_blind: 2,
            players,
            max_players,
        }
    }

    #[test]
    fn game_type_display_name_retorna_nomes_corretos() {
        assert_eq!(GameType::TexasHoldem.display_name(), "Texas Hold'em");
        assert_eq!(GameType::Omaha.display_name(), "Omaha");
        assert_eq!(GameType::Tournament.display_name(), "Torneio");
        assert_eq!(GameType::Freeroll.display_name(), "Freeroll");
    }

    #[test]
    fn game_type_icon_retorna_simbolos_nao_vazios() {
        assert!(!GameType::TexasHoldem.icon().is_empty());
        assert!(!GameType::Tournament.icon().is_empty());
        assert!(!GameType::Freeroll.icon().is_empty());
    }

    #[test]
    fn table_data_blinds_display_formata_corretamente() {
        let table = make_table(4, 8);
        assert_eq!(table.blinds_display(), "R$ 1 / R$ 2");
    }

    #[test]
    fn table_data_is_full_quando_atinge_max() {
        let full = make_table(8, 8);
        let not_full = make_table(7, 8);
        assert!(full.is_full());
        assert!(!not_full.is_full());
    }

    #[test]
    fn table_data_is_empty_quando_zero_jogadores() {
        let empty = make_table(0, 8);
        let not_empty = make_table(1, 8);
        assert!(empty.is_empty());
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn table_data_occupancy_percent_calcula_corretamente() {
        assert_eq!(make_table(0, 8).occupancy_percent(), 0);
        assert_eq!(make_table(4, 8).occupancy_percent(), 50);
        assert_eq!(make_table(8, 8).occupancy_percent(), 100);
        assert_eq!(make_table(2, 8).occupancy_percent(), 25);
    }

    #[test]
    fn table_data_occupancy_percent_zero_max_players_retorna_zero() {
        let table = make_table(0, 0);
        assert_eq!(table.occupancy_percent(), 0);
    }
}
