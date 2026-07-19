// ─── Módulo Antifraude: Detecção de Multi-Conta ───
// Detecta jogadores operando múltiplas contas (fingerprinting, IP/Hardware duplicados).
// Regras de negócio: BUSINESS_RULES.md §14.4

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Fingerprint do Jogador ───

/// Identificadores únicos que podem revelar multi-contas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerFingerprint {
    /// ID do jogador
    pub player_id: String,
    /// Endereço IP
    pub ip_address: String,
    /// ID de hardware (hash do dispositivo)
    pub hardware_id: String,
    /// User-Agent do navegador
    pub user_agent: String,
    /// Resolução de tela (ex: "1920x1080")
    pub screen_resolution: String,
    /// Fuso horário do navegador
    pub timezone: String,
    /// Idioma do navegador
    pub language: String,
    /// Timestamp do primeiro registro
    pub first_seen_ms: u64,
    /// Timestamp do último registro
    pub last_seen_ms: u64,
}

// ─── Alerta de Multi-Conta ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MultiAccountAlert {
    /// Conta primária (mais antiga)
    pub primary_account: String,
    /// Conta suspeita (mais recente)
    pub secondary_account: String,
    /// Pontuação de suspeição (0.0 a 1.0)
    pub suspicion_score: f64,
    /// Severidade do alerta
    pub severity: String,
    /// Fatores que contribuíram (IP, hardware, etc.)
    pub matching_factors: Vec<String>,
    /// Timestamp da detecção
    pub timestamp_ms: u64,
}

// ─── Limiares de Detecção ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MultiAccountThresholds {
    /// Pontuação mínima para gerar alerta
    pub alert_threshold: f64,
    /// Limiar para severidade crítica
    pub critical_threshold: f64,
    /// Limiar para severidade alta
    pub high_threshold: f64,
    /// Limiar para severidade média
    pub medium_threshold: f64,
    /// Peso do IP correspondente
    pub ip_weight: f64,
    /// Peso do hardware ID correspondente
    pub hardware_weight: f64,
    /// Peso do user-agent correspondente
    pub user_agent_weight: f64,
    /// Peso da resolução de tela correspondente
    pub screen_weight: f64,
    /// Peso do fuso horário correspondente
    pub timezone_weight: f64,
    /// Peso do idioma correspondente
    pub language_weight: f64,
}

impl Default for MultiAccountThresholds {
    fn default() -> Self {
        Self {
            alert_threshold: 0.3,
            critical_threshold: 0.8,
            high_threshold: 0.6,
            medium_threshold: 0.4,
            ip_weight: 0.30,
            hardware_weight: 0.30,
            user_agent_weight: 0.15,
            screen_weight: 0.10,
            timezone_weight: 0.10,
            language_weight: 0.05,
        }
    }
}

// ─── Detector de Multi-Conta ───

pub struct MultiAccountDetector {
    /// Fingerprints registrados por player_id
    fingerprints: HashMap<String, PlayerFingerprint>,
    /// Alertas gerados
    alerts: Vec<MultiAccountAlert>,
    /// Limiares de detecção
    thresholds: MultiAccountThresholds,
}

impl MultiAccountDetector {
    pub fn new() -> Self {
        Self {
            fingerprints: HashMap::new(),
            alerts: Vec::new(),
            thresholds: MultiAccountThresholds::default(),
        }
    }

    pub fn with_thresholds(thresholds: MultiAccountThresholds) -> Self {
        Self {
            fingerprints: HashMap::new(),
            alerts: Vec::new(),
            thresholds,
        }
    }

    /// Registra ou atualiza o fingerprint de um jogador.
    /// Retorna `true` se um novo alerta foi gerado.
    pub fn register_fingerprint(&mut self, fingerprint: PlayerFingerprint) -> bool {
        let player_id = fingerprint.player_id.clone();
        let new_alert = self.check_against_existing(&fingerprint);

        self.fingerprints.insert(player_id, fingerprint);

        if let Some(alert) = new_alert {
            self.alerts.push(alert);
            true
        } else {
            false
        }
    }

    /// Compara um novo fingerprint contra todos os existentes.
    fn check_against_existing(&self, new_fp: &PlayerFingerprint) -> Option<MultiAccountAlert> {
        let mut best_match: Option<(String, f64, Vec<String>)> = None;

        for (existing_id, existing_fp) in &self.fingerprints {
            if *existing_id == new_fp.player_id {
                continue;
            }

            let (score, factors) = self.calculate_match_score(existing_fp, new_fp);

            if score >= self.thresholds.alert_threshold {
                match &best_match {
                    None => best_match = Some((existing_id.clone(), score, factors)),
                    Some((_, best_score, _)) if score > *best_score => {
                        best_match = Some((existing_id.clone(), score, factors))
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(existing_id, score, factors)| {
            // A conta mais antiga é a primária
            let existing_fp = self.fingerprints.get(&existing_id).unwrap();
            let (primary, secondary) = if existing_fp.first_seen_ms <= new_fp.first_seen_ms {
                (existing_id, new_fp.player_id.clone())
            } else {
                (new_fp.player_id.clone(), existing_id)
            };

            let severity = self.classify_severity(score);

            MultiAccountAlert {
                primary_account: primary,
                secondary_account: secondary,
                suspicion_score: score,
                severity,
                matching_factors: factors,
                timestamp_ms: new_fp.last_seen_ms,
            }
        })
    }

    /// Calcula a pontuação de correspondência entre dois fingerprints.
    /// Retorna (score, fatores correspondentes).
    fn calculate_match_score(
        &self,
        a: &PlayerFingerprint,
        b: &PlayerFingerprint,
    ) -> (f64, Vec<String>) {
        let mut score = 0.0;
        let mut factors = Vec::new();

        if a.ip_address == b.ip_address && !a.ip_address.is_empty() {
            score += self.thresholds.ip_weight;
            factors.push("ip_address".to_string());
        }

        if a.hardware_id == b.hardware_id && !a.hardware_id.is_empty() {
            score += self.thresholds.hardware_weight;
            factors.push("hardware_id".to_string());
        }

        if a.user_agent == b.user_agent && !a.user_agent.is_empty() {
            score += self.thresholds.user_agent_weight;
            factors.push("user_agent".to_string());
        }

        if a.screen_resolution == b.screen_resolution && !a.screen_resolution.is_empty() {
            score += self.thresholds.screen_weight;
            factors.push("screen_resolution".to_string());
        }

        if a.timezone == b.timezone && !a.timezone.is_empty() {
            score += self.thresholds.timezone_weight;
            factors.push("timezone".to_string());
        }

        if a.language == b.language && !a.language.is_empty() {
            score += self.thresholds.language_weight;
            factors.push("language".to_string());
        }

        (score, factors)
    }

    /// Classifica a severidade com base na pontuação.
    fn classify_severity(&self, score: f64) -> String {
        if score >= self.thresholds.critical_threshold {
            "critical".to_string()
        } else if score >= self.thresholds.high_threshold {
            "high".to_string()
        } else if score >= self.thresholds.medium_threshold {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    /// Analisa todos os fingerprints e gera alertas para todas as combinações suspeitas.
    /// Útil para re-análise após mudança de limiares.
    pub fn analyze_all(&mut self) -> Vec<MultiAccountAlert> {
        let fps: Vec<PlayerFingerprint> = self.fingerprints.values().cloned().collect();
        let mut new_alerts = Vec::new();

        for i in 0..fps.len() {
            for j in (i + 1)..fps.len() {
                let (score, factors) = self.calculate_match_score(&fps[i], &fps[j]);
                if score >= self.thresholds.alert_threshold {
                    let (primary, secondary) = if fps[i].first_seen_ms <= fps[j].first_seen_ms {
                        (fps[i].player_id.clone(), fps[j].player_id.clone())
                    } else {
                        (fps[j].player_id.clone(), fps[i].player_id.clone())
                    };
                    let severity = self.classify_severity(score);
                    new_alerts.push(MultiAccountAlert {
                        primary_account: primary,
                        secondary_account: secondary,
                        suspicion_score: score,
                        severity,
                        matching_factors: factors,
                        timestamp_ms: fps[j].last_seen_ms,
                    });
                }
            }
        }

        // Evita duplicatas: apenas adiciona alertas para pares ainda não alertados
        for alert in &new_alerts {
            let already = self.alerts.iter().any(|existing| {
                (existing.primary_account == alert.primary_account
                    && existing.secondary_account == alert.secondary_account)
                    || (existing.primary_account == alert.secondary_account
                        && existing.secondary_account == alert.primary_account)
            });
            if !already {
                self.alerts.push(alert.clone());
            }
        }

        new_alerts
    }

    /// Retorna todos os alertas.
    pub fn get_alerts(&self) -> &[MultiAccountAlert] {
        &self.alerts
    }

    /// Retorna alertas filtrados por severidade.
    pub fn get_alerts_by_severity(&self, severity: &str) -> Vec<MultiAccountAlert> {
        self.alerts
            .iter()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    /// Retorna alertas para um jogador específico (como primária ou secundária).
    pub fn get_alerts_for_player(&self, player_id: &str) -> Vec<MultiAccountAlert> {
        self.alerts
            .iter()
            .filter(|a| a.primary_account == player_id || a.secondary_account == player_id)
            .cloned()
            .collect()
    }

    /// Retorna o fingerprint de um jogador.
    pub fn get_fingerprint(&self, player_id: &str) -> Option<&PlayerFingerprint> {
        self.fingerprints.get(player_id)
    }

    /// Retorna o número de fingerprints registrados.
    pub fn get_player_count(&self) -> usize {
        self.fingerprints.len()
    }

    /// Verifica se dois jogadores compartilham o mesmo IP.
    pub fn share_ip(&self, player_a: &str, player_b: &str) -> bool {
        match (
            self.fingerprints.get(player_a),
            self.fingerprints.get(player_b),
        ) {
            (Some(a), Some(b)) => !a.ip_address.is_empty() && a.ip_address == b.ip_address,
            _ => false,
        }
    }

    /// Verifica se dois jogadores compartilham o mesmo hardware ID.
    pub fn share_hardware(&self, player_a: &str, player_b: &str) -> bool {
        match (
            self.fingerprints.get(player_a),
            self.fingerprints.get(player_b),
        ) {
            (Some(a), Some(b)) => !a.hardware_id.is_empty() && a.hardware_id == b.hardware_id,
            _ => false,
        }
    }

    /// Limpa todos os alertas (mantém fingerprints).
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    /// Reseta completamente o detector.
    pub fn reset(&mut self) {
        self.fingerprints.clear();
        self.alerts.clear();
    }
}

impl Default for MultiAccountDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn make_fingerprint(
        player_id: &str,
        ip: &str,
        hw: &str,
        ua: &str,
        screen: &str,
        tz: &str,
        lang: &str,
        ts: u64,
    ) -> PlayerFingerprint {
        PlayerFingerprint {
            player_id: player_id.to_string(),
            ip_address: ip.to_string(),
            hardware_id: hw.to_string(),
            user_agent: ua.to_string(),
            screen_resolution: screen.to_string(),
            timezone: tz.to_string(),
            language: lang.to_string(),
            first_seen_ms: ts,
            last_seen_ms: ts,
        }
    }

    #[test]
    fn test_register_single_player_no_alert() {
        let mut detector = MultiAccountDetector::new();
        let fp = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let alert = detector.register_fingerprint(fp);
        assert!(!alert);
        assert!(detector.get_alerts().is_empty());
        assert_eq!(detector.get_player_count(), 1);
    }

    #[test]
    fn test_different_players_no_alert() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.2",
            "hw2",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        let alert = detector.register_fingerprint(fp2);
        assert!(!alert);
        assert!(detector.get_alerts().is_empty());
    }

    #[test]
    fn test_same_ip_generates_alert() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw2",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        let alert = detector.register_fingerprint(fp2);
        assert!(alert);
        assert_eq!(detector.get_alerts().len(), 1);
    }

    #[test]
    fn test_same_hardware_generates_alert() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.2",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        let alert = detector.register_fingerprint(fp2);
        assert!(alert);
    }

    #[test]
    fn test_same_ip_and_hardware_critical() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        let alerts = detector.get_alerts();
        assert_eq!(alerts.len(), 1);
        // IP (0.30) + hardware (0.30) = 0.60 → high
        assert_eq!(alerts[0].severity, "high");
        assert!(alerts[0].suspicion_score >= 0.6);
    }

    #[test]
    fn test_full_match_critical_severity() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        let alerts = detector.get_alerts();
        assert_eq!(alerts.len(), 1);
        // Todos os fatores: 0.30+0.30+0.15+0.10+0.10+0.05 = 1.0 → critical
        assert_eq!(alerts[0].severity, "critical");
        assert!((alerts[0].suspicion_score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_primary_is_older_account() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        let alert = &detector.get_alerts()[0];
        // alice (first_seen=1000) é mais antiga → primária
        assert_eq!(alert.primary_account, "alice");
        assert_eq!(alert.secondary_account, "bob");
    }

    #[test]
    fn test_matching_factors_listed() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        let factors = &detector.get_alerts()[0].matching_factors;
        assert!(factors.contains(&"ip_address".to_string()));
        assert!(factors.contains(&"hardware_id".to_string()));
        assert!(!factors.contains(&"user_agent".to_string()));
    }

    #[test]
    fn test_get_alerts_by_severity() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            2000,
        );
        let fp3 = make_fingerprint(
            "carol",
            "10.0.0.1",
            "hw3",
            "Safari",
            "2560x1440",
            "UTC+1",
            "fr-FR",
            3000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        detector.register_fingerprint(fp3);

        let critical = detector.get_alerts_by_severity("critical");
        assert!(!critical.is_empty());
        let low = detector.get_alerts_by_severity("low");
        assert!(low.is_empty());
    }

    #[test]
    fn test_get_alerts_for_player() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );
        let fp3 = make_fingerprint(
            "carol",
            "10.0.0.1",
            "hw3",
            "Safari",
            "2560x1440",
            "UTC+1",
            "fr-FR",
            3000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        detector.register_fingerprint(fp3);

        let alice_alerts = detector.get_alerts_for_player("alice");
        assert_eq!(alice_alerts.len(), 1);
        let carol_alerts = detector.get_alerts_for_player("carol");
        assert!(carol_alerts.is_empty());
    }

    #[test]
    fn test_share_ip() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw2",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );
        let fp3 = make_fingerprint(
            "carol",
            "10.0.0.1",
            "hw3",
            "Safari",
            "2560x1440",
            "UTC+1",
            "fr-FR",
            3000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        detector.register_fingerprint(fp3);

        assert!(detector.share_ip("alice", "bob"));
        assert!(!detector.share_ip("alice", "carol"));
    }

    #[test]
    fn test_share_hardware() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.2",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);

        assert!(detector.share_hardware("alice", "bob"));
        assert!(!detector.share_ip("alice", "bob"));
    }

    #[test]
    fn test_get_fingerprint() {
        let mut detector = MultiAccountDetector::new();
        let fp = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        detector.register_fingerprint(fp);

        let retrieved = detector.get_fingerprint("alice").unwrap();
        assert_eq!(retrieved.ip_address, "192.168.1.1");
        assert_eq!(retrieved.hardware_id, "hw1");
        assert!(detector.get_fingerprint("unknown").is_none());
    }

    #[test]
    fn test_analyze_all() {
        let mut detector = MultiAccountDetector::new();
        // Registra 3 jogadores sem gerar alertas no registro (threshold alto)
        detector.thresholds = MultiAccountThresholds {
            alert_threshold: 0.5,
            ..Default::default()
        };

        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw2",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );
        let fp3 = make_fingerprint(
            "carol",
            "10.0.0.1",
            "hw3",
            "Safari",
            "2560x1440",
            "UTC+1",
            "fr-FR",
            3000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        detector.register_fingerprint(fp3);

        // IP sozinho = 0.30 < 0.5 → sem alerta no registro
        assert!(detector.get_alerts().is_empty());

        // Baixa o limiar e re-analisa
        detector.thresholds.alert_threshold = 0.25;
        let new_alerts = detector.analyze_all();
        assert!(!new_alerts.is_empty());
        assert!(!detector.get_alerts().is_empty());
    }

    #[test]
    fn test_clear_alerts() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        assert!(!detector.get_alerts().is_empty());

        detector.clear_alerts();
        assert!(detector.get_alerts().is_empty());
        // Fingerprints ainda presentes
        assert_eq!(detector.get_player_count(), 2);
    }

    #[test]
    fn test_reset() {
        let mut detector = MultiAccountDetector::new();
        let fp1 = make_fingerprint(
            "alice",
            "192.168.1.1",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob",
            "192.168.1.1",
            "hw1",
            "Firefox",
            "1366x768",
            "UTC-5",
            "en-US",
            2000,
        );

        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        detector.reset();
        assert!(detector.get_alerts().is_empty());
        assert_eq!(detector.get_player_count(), 0);
    }

    #[test]
    fn test_empty_fingerprint_fields_ignored() {
        let mut detector = MultiAccountDetector::new();
        // IPs vazios não devem gerar alerta
        let fp1 = make_fingerprint(
            "alice",
            "",
            "hw1",
            "Chrome",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "bob", "", "hw1", "Firefox", "1366x768", "UTC-5", "en-US", 2000,
        );

        detector.register_fingerprint(fp1);
        let alert = detector.register_fingerprint(fp2);
        // IP vazio não conta, mas hardware_id igual conta (0.30)
        assert!(alert);
        let alerts = detector.get_alerts();
        assert_eq!(alerts[0].matching_factors.len(), 1);
        assert_eq!(alerts[0].matching_factors[0], "hardware_id");
    }

    #[test]
    fn test_severity_classification() {
        let mut detector = MultiAccountDetector::with_thresholds(MultiAccountThresholds {
            alert_threshold: 0.1,
            critical_threshold: 0.8,
            high_threshold: 0.6,
            medium_threshold: 0.4,
            ip_weight: 0.30,
            hardware_weight: 0.30,
            user_agent_weight: 0.15,
            screen_weight: 0.10,
            timezone_weight: 0.10,
            language_weight: 0.05,
        });

        // Apenas IP = 0.30 → low (abaixo de medium=0.40)
        let fp1 = make_fingerprint(
            "a",
            "1.1.1.1",
            "hw1",
            "UA1",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp2 = make_fingerprint(
            "b", "1.1.1.1", "hw2", "UA2", "1366x768", "UTC-5", "en-US", 2000,
        );
        detector.register_fingerprint(fp1);
        detector.register_fingerprint(fp2);
        assert_eq!(detector.get_alerts()[0].severity, "low");

        detector.reset();

        // IP + hardware = 0.60 → high
        let fp3 = make_fingerprint(
            "c",
            "2.2.2.2",
            "hw3",
            "UA3",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp4 = make_fingerprint(
            "d", "2.2.2.2", "hw3", "UA4", "1366x768", "UTC-5", "en-US", 2000,
        );
        detector.register_fingerprint(fp3);
        detector.register_fingerprint(fp4);
        assert_eq!(detector.get_alerts()[0].severity, "high");

        detector.reset();

        // IP + hardware + user_agent = 0.75 → medium (abaixo de high=0.60? não, 0.75 >= 0.60 → high)
        // Vamos testar IP + user_agent = 0.45 → medium
        let fp5 = make_fingerprint(
            "e",
            "3.3.3.3",
            "hw5",
            "UA5",
            "1920x1080",
            "UTC-3",
            "pt-BR",
            1000,
        );
        let fp6 = make_fingerprint(
            "f", "3.3.3.3", "hw6", "UA5", "1366x768", "UTC-5", "en-US", 2000,
        );
        detector.register_fingerprint(fp5);
        detector.register_fingerprint(fp6);
        // IP (0.30) + user_agent (0.15) = 0.45 → medium
        assert_eq!(detector.get_alerts()[0].severity, "medium");
    }
}
