// types.rs — Tipos compartilhados entre módulos
//
// Este módulo centraliza tipos que são usados em múltiplos módulos
// do motor de poker, evitando duplicação e garantindo consistência.

use serde::{Deserialize, Serialize};

/// Representa um pote (main pot ou side pot)
/// Usado por rake.rs, side_pots.rs e loss_deflator.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pot {
    pub amount: f64,
    pub eligible_players: Vec<String>,
}

impl Pot {
    /// Cria um novo pote
    pub fn new(amount: f64, eligible_players: Vec<String>) -> Self {
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

/// Configuração da mesa (parâmetros de rake)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub big_blind: f64,
    pub rake_percent: f64,
    pub rake_cap: f64,
}

impl TableConfig {
    /// Cria uma nova configuração de mesa
    pub fn new(big_blind: f64, rake_percent: f64, rake_cap: f64) -> Self {
        Self {
            big_blind,
            rake_percent,
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
    /// Retorna a fase como string legível (lowercase, compatível com serde)
    pub fn as_str(&self) -> &'static str {
        match self {
            GamePhase::Preflop => "preflop",
            GamePhase::Flop => "flop",
            GamePhase::Turn => "turn",
            GamePhase::River => "river",
            GamePhase::Showdown => "showdown",
        }
    }

    /// Avança para a próxima fase do jogo
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
