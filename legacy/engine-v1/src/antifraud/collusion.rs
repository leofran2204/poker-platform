use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum CollusionViolation {
    #[error("Jogadores {0} e {1} compartilham o mesmo IP ({2}) na mesma mesa!")]
    SameIpAddress(String, String, String),
    #[error("Jogadores {0} e {1} pertencem à mesma sub-rede /24 ({2})!")]
    SameSubnet(String, String, String),
    #[error("Jogadores {0} e {1} compartilham o mesmo dispositivo físico/hardware (Hash: {2})!")]
    SameDeviceFingerprint(String, String, String),
    #[error("Jogadores {0} e {1} estão em extrema proximidade física ({2:.1} metros < 50.0m de distância)!")]
    PhysicalProximityViolation(String, String, f64),
    #[error("Anomalia comportamental detectada para o jogador {0}: VPIP={1:.1}%, PFR={2:.1}% (Suspeita de Bot/Chip Dumping)")]
    SuspiciousBehaviorPattern(String, f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSession {
    pub user_id: String,
    pub ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBehaviorStats {
    pub user_id: String,
    pub hands_played: u64,
    pub hands_vpip: u64, // Voluntarily Put Money in Pot
    pub hands_pfr: u64,  // Pre-Flop Raise
}

impl PlayerBehaviorStats {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            hands_played: 0,
            hands_vpip: 0,
            hands_pfr: 0,
        }
    }

    pub fn record_hand(&mut self, vpip: bool, pfr: bool) {
        self.hands_played += 1;
        if vpip {
            self.hands_vpip += 1;
        }
        if pfr {
            self.hands_pfr += 1;
        }
    }

    pub fn vpip_percentage(&self) -> f64 {
        if self.hands_played == 0 {
            0.0
        } else {
            (self.hands_vpip as f64 / self.hands_played as f64) * 100.0
        }
    }

    pub fn pfr_percentage(&self) -> f64 {
        if self.hands_played == 0 {
            0.0
        } else {
            (self.hands_pfr as f64 / self.hands_played as f64) * 100.0
        }
    }
}

pub struct CollusionDetector;

impl CollusionDetector {
    /// Extrai o prefixo de sub-rede IPv4 /24 de um endereço IP.
    fn extract_subnet_v4(ip: &str) -> String {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.{}", parts[0], parts[1], parts[2])
        } else {
            ip.to_string()
        }
    }

    /// Valida se uma lista de sessões de jogadores pode se sentar na mesma mesa.
    /// Rejeita instantaneamente se houver IPs idênticos ou mesma sub-rede /24.
    pub fn validate_table_seating(players: &[PlayerSession]) -> Result<(), CollusionViolation> {
        let mut seen_ips: HashMap<String, String> = HashMap::new();
        let mut seen_subnets: HashMap<String, String> = HashMap::new();

        for player in players {
            // Check 1: IP estrito
            if let Some(existing_user) = seen_ips.get(&player.ip_address) {
                return Err(CollusionViolation::SameIpAddress(
                    existing_user.clone(),
                    player.user_id.clone(),
                    player.ip_address.clone(),
                ));
            }
            seen_ips.insert(player.ip_address.clone(), player.user_id.clone());

            // Check 2: Sub-rede /24
            let subnet = Self::extract_subnet_v4(&player.ip_address);
            if let Some(existing_user) = seen_subnets.get(&subnet) {
                return Err(CollusionViolation::SameSubnet(
                    existing_user.clone(),
                    player.user_id.clone(),
                    subnet,
                ));
            }
            seen_subnets.insert(subnet, player.user_id.clone());
        }

        Ok(())
    }

    /// Monitora estatísticas comportamentais de aposta (VPIP/PFR) após amostragem mínima de 50 mãos.
    pub fn detect_anomalies(stats: &PlayerBehaviorStats) -> Option<CollusionViolation> {
        if stats.hands_played < 50 {
            return None;
        }

        let vpip = stats.vpip_percentage();
        let pfr = stats.pfr_percentage();

        // Alerta de bot ou chip-dumping: PFR > VPIP (impossível matematicamente) ou VPIP < 2% / > 98%
        if pfr > vpip || !(2.0..=98.0).contains(&vpip) {
            Some(CollusionViolation::SuspiciousBehaviorPattern(
                stats.user_id.clone(),
                vpip,
                pfr,
            ))
        } else {
            None
        }
    }
}
