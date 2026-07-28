use crate::engine::evaluator::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub stack: f64,
    pub current_bet: f64,
    pub total_bet: f64,
    pub cards: Vec<Card>,
    pub has_folded: bool,
    pub is_all_in: bool,
    pub has_acted: bool,
}

impl Player {
    pub fn new(id: impl Into<String>, name: impl Into<String>, stack: f64) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            stack,
            current_bet: 0.0,
            total_bet: 0.0,
            cards: Vec::new(),
            has_folded: false,
            is_all_in: false,
            has_acted: false,
        }
    }

    /// Retorna verdadeiro se o jogador pode tomar uma ação ativa (apostar, pagar, aumentar, passar).
    pub fn can_act(&self) -> bool {
        !self.has_folded && !self.is_all_in && self.stack > 0.0
    }

    /// Retorna verdadeiro se o jogador ainda está ativo na disputa do pote (mesmo estando all-in).
    pub fn is_in_hand(&self) -> bool {
        !self.has_folded
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Fold,
    Check,
    Call,
    Bet(f64),
    Raise(f64),
    AllIn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub players: Vec<Player>,
    pub current_street: Street,
    pub current_player_idx: usize,
    pub button_idx: usize,
    pub highest_bet: f64,
    pub min_raise: f64,
    pub pot: f64,
    pub community_cards: Vec<Card>,
}

impl GameState {
    pub fn new(players: Vec<Player>, button_idx: usize, small_blind: f64) -> Self {
        let n = players.len();
        let current_player_idx = if n > 0 { (button_idx + 1) % n } else { 0 };
        Self {
            players,
            current_street: Street::Preflop,
            current_player_idx,
            button_idx,
            highest_bet: small_blind * 2.0,
            min_raise: small_blind * 2.0,
            pot: 0.0,
            community_cards: Vec::new(),
        }
    }

    /// CORREÇÃO CRÍTICA: Encontra o próximo jogador que PODE AGIR na mesa.
    /// Ignora quem já deu fold ou está all-in, prevenindo deadlocks.
    pub fn next_active_player(&self, from: usize) -> Option<usize> {
        let n = self.players.len();
        if n == 0 {
            return None;
        }

        for i in 1..=n {
            let next_idx = (from + i) % n;
            if self.players[next_idx].can_act() {
                return Some(next_idx);
            }
        }

        None
    }

    /// Retorna quantos jogadores ainda podem tomar decisões nesta rodada.
    pub fn count_players_who_can_act(&self) -> usize {
        self.players.iter().filter(|p| p.can_act()).count()
    }

    /// Retorna quantos jogadores ainda disputam o pote (incluindo all-ins).
    pub fn count_players_in_hand(&self) -> usize {
        self.players.iter().filter(|p| p.is_in_hand()).count()
    }
}

pub struct GameLoop {
    pub state: GameState,
}

impl GameLoop {
    pub fn new(state: GameState) -> Self {
        Self { state }
    }

    /// Avança a vez para o próximo jogador válido sem entrar em loop infinito ou deadlock.
    pub fn advance_turn(&mut self) -> bool {
        // Se 1 ou menos jogadores puderem agir, a rodada pode estar encerrada
        if self.state.count_players_who_can_act() <= 1 && self.is_street_complete() {
            return self.next_street();
        }

        if let Some(next_idx) = self.state.next_active_player(self.state.current_player_idx) {
            self.state.current_player_idx = next_idx;
            true
        } else {
            // Nenhum jogador pode agir (todos all-in ou fold)
            self.next_street()
        }
    }

    pub fn is_street_complete(&self) -> bool {
        let active_players = self.state.players.iter().filter(|p| p.can_act());
        active_players
            .into_iter()
            .all(|p| p.has_acted && (p.current_bet == self.state.highest_bet || p.is_all_in))
    }

    pub fn next_street(&mut self) -> bool {
        // Reiniciar apostas atuais da rodada
        for p in &mut self.state.players {
            p.current_bet = 0.0;
            p.has_acted = false;
        }
        self.state.highest_bet = 0.0;

        self.state.current_street = match self.state.current_street {
            Street::Preflop => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Showdown,
            Street::Showdown | Street::Finished => Street::Finished,
        };

        if self.state.current_street == Street::Showdown
            || self.state.current_street == Street::Finished
        {
            return false;
        }

        // Posicionar a vez após o botão
        if let Some(next_idx) = self.state.next_active_player(self.state.button_idx) {
            self.state.current_player_idx = next_idx;
        }

        true
    }
}
