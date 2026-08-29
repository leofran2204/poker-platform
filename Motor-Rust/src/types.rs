// types.rs — Tipos compartilhados entre módulos
//
// Este módulo centraliza tipos que são usados em múltiplos módulos
// do motor de poker, evitando duplicação e garantindo consistência.
//
// Arquitetura Monetária:
// - Todos os valores monetários (pot, rake_cap, blinds) utilizam `u64` centavos inteiros.
// - Percentuais monetários usam pontos-base inteiros (`u16`): 500 = 5,00%.

use serde::{Deserialize, Serialize};

/// Representa um pote em centavos inteiros (main pot ou side pot)
/// Usado por rake.rs, side_pots.rs e loss_deflator.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pot {
    pub amount: u64,
    pub eligible_players: Vec<String>,
}

impl Pot {
    /// Cria um novo pote com valor em centavos
    pub fn new(amount: u64, eligible_players: Vec<String>) -> Self {
        Self {
            amount,
            eligible_players,
        }
    }

    /// Verifica se um jogador é elegível para este pote
    pub fn is_eligible(&self, player_id: &str) -> bool {
        self.eligible_players.iter().any(|p| p == player_id)
    }
}

/// Caps opcionais conforme a quantidade de jogadores que receberam cartas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RakeCapSchedule {
    pub heads_up: u64,
    pub three_to_four: u64,
    pub five_plus: u64,
}

impl RakeCapSchedule {
    pub fn cap_for_players(self, players_dealt: usize) -> u64 {
        match players_dealt {
            0..=2 => self.heads_up,
            3..=4 => self.three_to_four,
            _ => self.five_plus,
        }
    }
}

/// Variante de poker da mesa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PokerVariant {
    #[default]
    Holdem,
    ShortDeck,
    /// Short Deck Omaha (PLO-4 no baralho 36; usa exatamente 2 hole + 3 board).
    ShortDeckOmaha,
}

impl PokerVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Holdem => "holdem",
            Self::ShortDeck => "short_deck",
            Self::ShortDeckOmaha => "short_deck_omaha",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "short_deck_omaha" | "sd_omaha" | "omaha_sd" | "shortdeck_omaha" | "plo_sd" => {
                Self::ShortDeckOmaha
            }
            "short_deck" | "shortdeck" | "sd" | "six_plus" => Self::ShortDeck,
            _ => Self::Holdem,
        }
    }

    pub fn hole_card_count(self) -> usize {
        match self {
            Self::ShortDeckOmaha => 4,
            Self::Holdem | Self::ShortDeck => 2,
        }
    }

    pub fn uses_short_deck(self) -> bool {
        matches!(self, Self::ShortDeck | Self::ShortDeckOmaha)
    }
}

/// Configuração da mesa (parâmetros de rake e blinds em centavos)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub big_blind: u64,
    /// Small blind em centavos. Se omitido/0 no JSON legado, usa `big_blind / 2`.
    /// Permite stakes iguais (ex.: 0,25/0,25 → sb=25, bb=25).
    #[serde(default)]
    pub small_blind: u64,
    /// Commission in basis points. For example, 500 means 5.00%.
    pub rake_basis_points: u16,
    /// Cap legado aplicado quando não há agenda por número de jogadores.
    pub rake_cap: u64,
    /// Caps opcionais para heads-up, 3–4 jogadores e 5+ jogadores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rake_cap_schedule: Option<RakeCapSchedule>,
    #[serde(default)]
    pub poker_variant: PokerVariant,
}

impl TableConfig {
    /// Creates a configuration with blinds/cap in cents and rake in basis points.
    /// Small blind padrão = big_blind / 2 (Hold'em clássico).
    pub fn new(big_blind: u64, rake_basis_points: u16, rake_cap: u64) -> Self {
        Self {
            big_blind,
            small_blind: big_blind / 2,
            rake_basis_points,
            rake_cap,
            rake_cap_schedule: None,
            poker_variant: PokerVariant::Holdem,
        }
    }

    pub fn with_small_blind(mut self, small_blind: u64) -> Self {
        self.small_blind = small_blind;
        self
    }

    /// Small blind efetivo (fallback BB/2 se small_blind não foi setado).
    pub fn effective_small_blind(&self) -> u64 {
        if self.small_blind > 0 {
            self.small_blind
        } else {
            self.big_blind / 2
        }
    }

    pub fn with_poker_variant(mut self, variant: PokerVariant) -> Self {
        self.poker_variant = variant;
        self
    }

    /// Configura caps distintos para heads-up, 3–4 jogadores e 5+ jogadores.
    pub fn with_rake_cap_schedule(mut self, schedule: RakeCapSchedule) -> Self {
        self.rake_cap_schedule = Some(schedule);
        self
    }

    /// Retorna o cap correspondente à quantidade de jogadores que receberam cartas.
    pub fn rake_cap_for_players(&self, players_dealt: usize) -> u64 {
        self.rake_cap_schedule
            .map(|schedule| schedule.cap_for_players(players_dealt))
            .unwrap_or(self.rake_cap)
    }
}

/// Fase do jogo (Texas Hold'em)
/// Usada por loss_deflator.rs, hand_history.rs e game_loop.rs
/// Serializada em lowercase para compatibilidade com hand_history JSON
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GamePhase {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

impl GamePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            GamePhase::Preflop => "preflop",
            GamePhase::Flop => "flop",
            GamePhase::Turn => "turn",
            GamePhase::River => "river",
            GamePhase::Showdown => "showdown",
        }
    }

    pub fn next(&self) -> Option<GamePhase> {
        match self {
            GamePhase::Preflop => Some(GamePhase::Flop),
            GamePhase::Flop => Some(GamePhase::Turn),
            GamePhase::Turn => Some(GamePhase::River),
            GamePhase::River => Some(GamePhase::Showdown),
            GamePhase::Showdown => None,
        }
    }
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
