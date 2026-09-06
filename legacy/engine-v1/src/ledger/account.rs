use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Saldo insuficiente para a transação")]
    InsufficientFunds,
    #[error("Valor de transação inválido (deve ser positivo)")]
    InvalidAmount,
    #[error("Erro de concorrência ou trava no Ledger")]
    LockError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    Deposit,
    Withdrawal,
    TableBuyIn,
    TableCashOut,
    PotWin,
    LossDeflatorCashback,
    RakeDeduction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub user_id: String,
    pub amount_cents: i64, // Valores em centavos para evitar ponto flutuante
    pub balance_after_cents: i64,
    pub entry_type: EntryType,
    pub reference_id: Option<String>, // Ex: hand_id ou tx_hash
    pub timestamp: DateTime<Utc>,
    pub prev_hash: String,
    pub hash: String,
}

impl LedgerEntry {
    pub fn calculate_hash(
        id: &str,
        user_id: &str,
        amount_cents: i64,
        balance_after_cents: i64,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(amount_cents.to_le_bytes());
        hasher.update(balance_after_cents.to_le_bytes());
        hasher.update(prev_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone)]
pub struct LedgerAccount {
    pub user_id: String,
    inner: Arc<Mutex<AccountState>>,
}

#[derive(Debug)]
struct AccountState {
    balance_cents: i64,
    history: Vec<LedgerEntry>,
    last_hash: String,
}

impl LedgerAccount {
    pub fn new(user_id: impl Into<String>, initial_balance_cents: i64) -> Self {
        let user_id = user_id.into();
        let initial_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        let state = AccountState {
            balance_cents: initial_balance_cents,
            history: Vec::new(),
            last_hash: initial_hash,
        };

        Self {
            user_id,
            inner: Arc::new(Mutex::new(state)),
        }
    }

    pub fn get_balance_cents(&self) -> Result<i64, LedgerError> {
        let state = self.inner.lock().map_err(|_| LedgerError::LockError)?;
        Ok(state.balance_cents)
    }

    /// Executa uma transação de crédito ou débito com GARANTIA ATÔMICA e registro encadeado no Ledger.
    pub fn record_transaction(
        &self,
        amount_cents: i64,
        entry_type: EntryType,
        reference_id: Option<String>,
    ) -> Result<LedgerEntry, LedgerError> {
        if amount_cents == 0 {
            return Err(LedgerError::InvalidAmount);
        }

        let mut state = self.inner.lock().map_err(|_| LedgerError::LockError)?;

        // Se for um débito, verificar se o saldo é suficiente
        if amount_cents < 0 && state.balance_cents + amount_cents < 0 {
            return Err(LedgerError::InsufficientFunds);
        }

        let new_balance = state.balance_cents + amount_cents;
        let entry_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        let prev_hash = state.last_hash.clone();

        let hash = LedgerEntry::calculate_hash(
            &entry_id,
            &self.user_id,
            amount_cents,
            new_balance,
            &prev_hash,
        );

        let entry = LedgerEntry {
            id: entry_id,
            user_id: self.user_id.clone(),
            amount_cents,
            balance_after_cents: new_balance,
            entry_type,
            reference_id,
            timestamp,
            prev_hash,
            hash: hash.clone(),
        };

        state.balance_cents = new_balance;
        state.last_hash = hash;
        state.history.push(entry.clone());

        Ok(entry)
    }

    /// Valida a integridade matemática da cadeia de auditoria do ledger.
    pub fn verify_integrity(&self) -> Result<bool, LedgerError> {
        let state = self.inner.lock().map_err(|_| LedgerError::LockError)?;
        let mut expected_prev_hash =
            "0000000000000000000000000000000000000000000000000000000000000000";

        for entry in &state.history {
            if entry.prev_hash != expected_prev_hash {
                return Ok(false);
            }

            let calculated_hash = LedgerEntry::calculate_hash(
                &entry.id,
                &entry.user_id,
                entry.amount_cents,
                entry.balance_after_cents,
                &entry.prev_hash,
            );

            if calculated_hash != entry.hash {
                return Ok(false);
            }

            expected_prev_hash = &entry.hash;
        }

        Ok(true)
    }

    pub fn get_history(&self) -> Result<Vec<LedgerEntry>, LedgerError> {
        let state = self.inner.lock().map_err(|_| LedgerError::LockError)?;
        Ok(state.history.clone())
    }
}
