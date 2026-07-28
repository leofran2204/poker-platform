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

/// Configuração da mesa (parâmetros de rake e blinds em centavos)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub big_blind: u64,
    /// Commission in basis points. For example, 500 means 5.00%.
    pub rake_basis_points: u16,
    pub rake_cap: u64,
}

impl TableConfig {
    /// Creates a configuration with blinds/cap in cents and rake in basis points.
    pub fn new(big_blind: u64, rake_basis_points: u16, rake_cap: u64) -> Self {
        Self {
            big_blind,
            rake_basis_points,
            rake_cap,
        }
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
