//! Filtros do lobby: tipo de jogo e faixa de blinds.
//!
//! Componente visual que permite ao jogador refinar a lista de mesas.

use dioxus::prelude::*;

use super::table_card::GameType;

/// Filtro por tipo de jogo. `All` exibe todos os tipos.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameTypeFilter {
    All,
    TexasHoldem,
    Omaha,
    Tournament,
    Freeroll,
}

impl GameTypeFilter {
    /// Rótulo de exibição do filtro.
    #[must_use]
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "Todos",
            Self::TexasHoldem => "Texas Hold'em",
            Self::Omaha => "Omaha",
            Self::Tournament => "Torneios",
            Self::Freeroll => "Freerolls",
        }
    }

    /// Converte o filtro para `Option<GameType>` (`None` = sem filtro).
    #[must_use]
    #[allow(dead_code)]
    pub fn to_game_type(self) -> Option<GameType> {
        match self {
            Self::All => None,
            Self::TexasHoldem => Some(GameType::TexasHoldem),
            Self::Omaha => Some(GameType::Omaha),
            Self::Tournament => Some(GameType::Tournament),
            Self::Freeroll => Some(GameType::Freeroll),
        }
    }
}

/// Faixa de blinds para filtragem.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlindsFilter {
    All,
    Micro,   // big blind < R$ 1
    Low,     // R$ 1–5
    Medium,  // R$ 6–25
    High,    // big blind > R$ 25
}

impl BlindsFilter {
    /// Rótulo de exibição da faixa de blinds.
    #[must_use]
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "Todos os blinds",
            Self::Micro => "Micro (< R$1)",
            Self::Low => "Baixo (R$1-5)",
            Self::Medium => "Médio (R$5-25)",
            Self::High => "Alto (> R$25)",
        }
    }

    /// Verifica se um big blind pertence a esta faixa.
    #[must_use]
    #[allow(dead_code)]
    pub fn matches(self, big_blind: u32) -> bool {
        match self {
            Self::All => true,
            Self::Micro => big_blind < 1,
            Self::Low => (1..=5).contains(&big_blind),
            Self::Medium => (6..=25).contains(&big_blind),
            Self::High => big_blind > 25,
        }
    }
}

/// Painel de filtros exibido acima da lista de mesas.
///
/// Estilo Full Tilt Poker: felt verde escuro sólido, borda grossa dourada,
/// cantos sutis, sem transparência.
#[allow(dead_code)]
#[component]
pub fn LobbyFilters(
    game_type: GameTypeFilter,
    blinds: BlindsFilter,
    only_available: bool,
) -> Element {
    rsx! {
        div {
            class: "lobby-filters",
            div {
                class: "lobby-filters-row",
                div {
                    class: "lobby-filters-field",
                    label {
                        class: "lobby-filters-label",
                        "Tipo de jogo"
                    }
                    select {
                        class: "lobby-filters-select",
                        value: "{game_type:?}",
                        option { value: "All", "Todos" }
                        option { value: "TexasHoldem", "Texas Hold'em" }
                        option { value: "Omaha", "Omaha" }
                        option { value: "Tournament", "Torneios" }
                        option { value: "Freeroll", "Freerolls" }
                    }
                }
                div {
                    class: "lobby-filters-field",
                    label {
                        class: "lobby-filters-label",
                        "Faixa de blinds"
                    }
                    select {
                        class: "lobby-filters-select",
                        value: "{blinds:?}",
                        option { value: "All", "Todos os blinds" }
                        option { value: "Micro", "Micro (< R$1)" }
                        option { value: "Low", "Baixo (R$1-5)" }
                        option { value: "Medium", "Médio (R$5-25)" }
                        option { value: "High", "Alto (> R$25)" }
                    }
                }
                div {
                    class: "lobby-filters-checkbox",
                    input {
                        r#type: "checkbox",
                        checked: only_available,
                        class: "lobby-filters-checkbox-input",
                    }
                    label {
                        class: "lobby-filters-checkbox-label",
                        "Apenas mesas com vagas"
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
    fn game_type_filter_label_retorna_strings_nao_vazias() {
        assert_eq!(GameTypeFilter::All.label(), "Todos");
        assert_eq!(GameTypeFilter::TexasHoldem.label(), "Texas Hold'em");
        assert_eq!(GameTypeFilter::Tournament.label(), "Torneios");
        assert!(!GameTypeFilter::Freeroll.label().is_empty());
    }

    #[test]
    fn game_type_filter_to_game_type_all_retorna_none() {
        assert_eq!(GameTypeFilter::All.to_game_type(), None);
        assert_eq!(
            GameTypeFilter::TexasHoldem.to_game_type(),
            Some(GameType::TexasHoldem)
        );
        assert_eq!(
            GameTypeFilter::Tournament.to_game_type(),
            Some(GameType::Tournament)
        );
    }

    #[test]
    fn blinds_filter_label_retorna_strings_nao_vazias() {
        assert!(!BlindsFilter::All.label().is_empty());
        assert!(!BlindsFilter::Micro.label().is_empty());
        assert!(!BlindsFilter::Low.label().is_empty());
        assert!(!BlindsFilter::Medium.label().is_empty());
        assert!(!BlindsFilter::High.label().is_empty());
    }

    #[test]
    fn blinds_filter_matches_all_aceita_qualquer_valor() {
        assert!(BlindsFilter::All.matches(0));
        assert!(BlindsFilter::All.matches(100));
        assert!(BlindsFilter::All.matches(1_000));
    }

    #[test]
    fn blinds_filter_matches_micro_apenas_abaixo_de_1() {
        assert!(BlindsFilter::Micro.matches(0));
        assert!(!BlindsFilter::Micro.matches(1));
        assert!(!BlindsFilter::Micro.matches(5));
    }

    #[test]
    fn blinds_filter_matches_low_entre_1_e_5() {
        assert!(!BlindsFilter::Low.matches(0));
        assert!(BlindsFilter::Low.matches(1));
        assert!(BlindsFilter::Low.matches(3));
        assert!(BlindsFilter::Low.matches(5));
        assert!(!BlindsFilter::Low.matches(6));
    }

    #[test]
    fn blinds_filter_matches_medium_entre_6_e_25() {
        assert!(!BlindsFilter::Medium.matches(5));
        assert!(BlindsFilter::Medium.matches(6));
        assert!(BlindsFilter::Medium.matches(25));
        assert!(!BlindsFilter::Medium.matches(26));
    }

    #[test]
    fn blinds_filter_matches_high_acima_de_25() {
        assert!(!BlindsFilter::High.matches(25));
        assert!(BlindsFilter::High.matches(26));
        assert!(BlindsFilter::High.matches(100));
        assert!(BlindsFilter::High.matches(1_000));
    }
}
