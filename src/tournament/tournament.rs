use crate::ledger::{EntryType, LedgerAccount, LedgerError};
use crate::tournament::blind_structure::{BlindLevel, BlindStructure};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TournamentError {
    #[error("Inscrições encerradas para este torneio")]
    RegistrationClosed,
    #[error("Re-buy não permitido ou limite de nível excedido")]
    RebuyNotAllowed,
    #[error("Jogador não encontrado no torneio")]
    PlayerNotFound,
    #[error("Erro financeiro no Ledger: {0}")]
    Ledger(#[from] LedgerError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TournamentState {
    Registration,
    LateRegistration,
    Running,
    HandForHand,
    FinalTable,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentPlayer {
    pub user_id: String,
    pub name: String,
    pub chip_stack: f64,
    pub rebuys_count: u32,
    pub addons_count: u32,
    pub is_eliminated: bool,
    pub finish_rank: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentPayout {
    pub rank: usize,
    pub percentage: f64,
}

pub struct Tournament {
    pub id: String,
    pub name: String,
    pub buy_in_cents: i64,
    pub rake_cents: i64,
    pub starting_stack: f64,
    pub blind_structure: BlindStructure,
    pub current_level_idx: usize,
    pub state: TournamentState,
    pub players: HashMap<String, TournamentPlayer>,
    pub prize_pool_cents: i64,
    pub payouts: Vec<TournamentPayout>,
    pub max_rebuys_level: usize,
}

impl Tournament {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        buy_in_cents: i64,
        rake_cents: i64,
        starting_stack: f64,
        blind_structure: BlindStructure,
    ) -> Self {
        let payouts = vec![
            TournamentPayout { rank: 1, percentage: 50.0 },
            TournamentPayout { rank: 2, percentage: 30.0 },
            TournamentPayout { rank: 3, percentage: 20.0 },
        ];

        Self {
            id: id.into(),
            name: name.into(),
            buy_in_cents,
            rake_cents,
            starting_stack,
            blind_structure,
            current_level_idx: 0,
            state: TournamentState::Registration,
            players: HashMap::new(),
            prize_pool_cents: 0,
            payouts,
            max_rebuys_level: 4,
        }
    }

    /// Inscreve um jogador no torneio debitando buy_in + rake do Ledger imutável.
    pub fn register_player(
        &mut self,
        user_id: &str,
        name: &str,
        account: &LedgerAccount,
    ) -> Result<(), TournamentError> {
        if self.state != TournamentState::Registration && self.state != TournamentState::LateRegistration {
            return Err(TournamentError::RegistrationClosed);
        }

        let total_cost = self.buy_in_cents + self.rake_cents;
        account.record_transaction(-total_cost, EntryType::TableBuyIn, Some(format!("BUYIN-{}", self.id)))?;

        self.prize_pool_cents += self.buy_in_cents;
        self.players.insert(
            user_id.to_string(),
            TournamentPlayer {
                user_id: user_id.to_string(),
                name: name.to_string(),
                chip_stack: self.starting_stack,
                rebuys_count: 0,
                addons_count: 0,
                is_eliminated: false,
                finish_rank: None,
            },
        );

        Ok(())
    }

    /// Realiza o Re-buy de fichas se o torneio estiver no nível permitido.
    pub fn rebuy_player(
        &mut self,
        user_id: &str,
        account: &LedgerAccount,
    ) -> Result<(), TournamentError> {
        if self.current_level_idx > self.max_rebuys_level {
            return Err(TournamentError::RebuyNotAllowed);
        }

        let player = self.players.get_mut(user_id).ok_or(TournamentError::PlayerNotFound)?;
        if player.chip_stack > self.starting_stack * 0.5 {
            return Err(TournamentError::RebuyNotAllowed);
        }

        account.record_transaction(-self.buy_in_cents, EntryType::TableBuyIn, Some(format!("REBUY-{}", self.id)))?;
        self.prize_pool_cents += self.buy_in_cents;
        player.chip_stack += self.starting_stack;
        player.rebuys_count += 1;
        player.is_eliminated = false;

        Ok(())
    }

    /// Avança o nível de blinds do torneio.
    pub fn advance_blind_level(&mut self) -> Option<&BlindLevel> {
        if self.current_level_idx + 1 < self.blind_structure.levels.len() {
            self.current_level_idx += 1;
        }
        self.blind_structure.get_level(self.current_level_idx)
    }

    /// Registra a eliminação de um jogador e calcula sua colocação.
    pub fn eliminate_player(&mut self, user_id: &str) -> Option<usize> {
        let active_count = self.players.values().filter(|p| !p.is_eliminated).count();
        if let Some(player) = self.players.get_mut(user_id) {
            if !player.is_eliminated {
                player.is_eliminated = true;
                player.finish_rank = Some(active_count);

                // Checar se restam 9 ou menos jogadores para a Mesa Final
                if active_count <= 9 && self.state == TournamentState::Running {
                    self.state = TournamentState::FinalTable;
                }
                if active_count == 1 {
                    self.state = TournamentState::Finished;
                }
                return Some(active_count);
            }
        }
        None
    }

    /// Distribui os prêmios do Prize Pool no Ledger dos vencedores ao finalizar o torneio.
    pub fn distribute_prize_pool(&self, accounts: &HashMap<String, LedgerAccount>) -> Vec<(String, usize, i64)> {
        let mut payouts_done = Vec::new();
        if self.prize_pool_cents == 0 {
            return payouts_done;
        }

        for payout in &self.payouts {
            let amount = ((self.prize_pool_cents as f64) * (payout.percentage / 100.0)) as i64;
            
            // Encontrar jogador colocado nesta posição
            if let Some(player) = self.players.values().find(|p| p.finish_rank == Some(payout.rank)) {
                if let Some(account) = accounts.get(&player.user_id) {
                    let _ = account.record_transaction(
                        amount,
                        EntryType::PotWin,
                        Some(format!("PRIZE-{}-RANK{}", self.id, payout.rank)),
                    );
                    payouts_done.push((player.user_id.clone(), payout.rank, amount));
                }
            }
        }

        payouts_done
    }
}
