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

/// Fase do jogo quando o all-in call aconteceu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamePhase {
    Preflop,
    Flop,
    Turn,
    River,
}

impl GamePhase {
    /// Retorna a fase como string legível
    pub fn as_str(&self) -> &'static str {
        match self {
            GamePhase::Preflop => "Preflop",
            GamePhase::Flop => "Flop",
            GamePhase::Turn => "Turn",
            GamePhase::River => "River",
        }
    }
}
