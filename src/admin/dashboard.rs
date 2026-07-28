use crate::antifraud::PlayerBehaviorStats;
use crate::ledger::LedgerAccount;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub active_connections: usize,
    pub active_tables: usize,
    pub total_volume_cents: i64,
    pub requests_processed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousPlayerReport {
    pub user_id: String,
    pub reason: String,
    pub vpip: f64,
    pub pfr: f64,
    pub is_suspended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerAuditResult {
    pub user_id: String,
    pub account_balance_cents: i64,
    pub transaction_count: usize,
    pub hash_chain_valid: bool,
}

#[derive(Clone)]
pub struct AdminDashboard {
    suspended_players: Arc<Mutex<HashSet<String>>>,
    metrics: Arc<Mutex<SystemMetrics>>,
}

impl AdminDashboard {
    pub fn new() -> Self {
        Self {
            suspended_players: Arc::new(Mutex::new(HashSet::new())),
            metrics: Arc::new(Mutex::new(SystemMetrics {
                active_connections: 0,
                active_tables: 0,
                total_volume_cents: 0,
                requests_processed: 0,
            })),
        }
    }

    /// Executa uma auditoria criptográfica da cadeia de hashes do Ledger imutável
    pub fn audit_ledger_account(&self, account: &LedgerAccount) -> LedgerAuditResult {
        let balance = account.get_balance_cents().unwrap_or(0);
        let history = account.get_history().unwrap_or_default();
        let hash_valid = account.verify_integrity().unwrap_or(false);

        LedgerAuditResult {
            user_id: account.user_id.clone(),
            account_balance_cents: balance,
            transaction_count: history.len(),
            hash_chain_valid: hash_valid,
        }
    }

    /// Suspende/Bane manualmente um jogador sob suspeita de fraude
    pub fn suspend_player(&self, user_id: &str, reason: &str) -> String {
        let mut suspended = self.suspended_players.lock().unwrap();
        suspended.insert(user_id.to_string());
        format!(
            "Jogador {} suspenso com sucesso. Motivo: {}",
            user_id, reason
        )
    }

    /// Reativa um jogador suspenso
    pub fn unsuspend_player(&self, user_id: &str) -> bool {
        let mut suspended = self.suspended_players.lock().unwrap();
        suspended.remove(user_id)
    }

    /// Verifica se um jogador está suspenso
    pub fn is_player_suspended(&self, user_id: &str) -> bool {
        let suspended = self.suspended_players.lock().unwrap();
        suspended.contains(user_id)
    }

    /// Analisa estatísticas comportamentais e gera relatórios de risco
    pub fn analyze_player_risk(
        &self,
        stats: &PlayerBehaviorStats,
    ) -> Option<SuspiciousPlayerReport> {
        let is_suspicious_vpip = stats.vpip_percentage() > 85.0 && stats.hands_played > 20;
        let is_suspicious_pfr = stats.pfr_percentage() > 70.0 && stats.hands_played > 20;

        if is_suspicious_vpip || is_suspicious_pfr {
            let is_suspended = self.is_player_suspended(&stats.user_id);
            let reason = if is_suspicious_vpip && is_suspicious_pfr {
                "Anomalia Extrema: VPIP e PFR anormalmente elevados (Conluio/Bot)".to_string()
            } else if is_suspicious_vpip {
                "VPIP Anômalo (> 85%)".to_string()
            } else {
                "PFR Anômalo (> 70%)".to_string()
            };

            Some(SuspiciousPlayerReport {
                user_id: stats.user_id.clone(),
                reason,
                vpip: stats.vpip_percentage(),
                pfr: stats.pfr_percentage(),
                is_suspended,
            })
        } else {
            None
        }
    }

    /// Atualiza métricas do sistema
    pub fn update_metrics(&self, active_conns: usize, active_tbls: usize, volume_delta_cents: i64) {
        let mut m = self.metrics.lock().unwrap();
        m.active_connections = active_conns;
        m.active_tables = active_tbls;
        m.total_volume_cents += volume_delta_cents;
        m.requests_processed += 1;
    }

    /// Obtém snapshot atual das métricas do cluster
    pub fn get_metrics(&self) -> SystemMetrics {
        let m = self.metrics.lock().unwrap();
        m.clone()
    }
}
