// ─── Módulo Antifraude: Detecção de Chip Dumping ───
// Detecta transferência intencional de fichas entre jogadores.
// Regras de negócio: BUSINESS_RULES.md §14.2

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Força da Mão (compatível com collusion.rs) ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandStrength {
    VeryWeak,
    Weak,
    Medium,
    Strong,
    VeryStrong,
    Monster,
}

// ─── Registro de Chip Dump ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChipDumpRecord {
    /// Jogador que perdeu fichas (dumper)
    pub from_player: String,
    /// Jogador que recebeu fichas (dumpee)
    pub to_player: String,
    /// Valor transferido
    pub amount: u64,
    /// Força da mão do dumper no momento do all-in
    pub hand_strength: HandStrength,
    /// ID da mão
    pub hand_id: String,
    /// Timestamp
    pub timestamp_ms: u64,
}

// ─── Alerta de Chip Dumping ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChipDumpAlert {
    /// Jogador suspeito de dumping
    pub dumper: String,
    /// Jogador beneficiado
    pub dumpee: String,
    /// Total de fichas transferidas
    pub total_dumped: u64,
    /// Número de ocorrências
    pub occurrences: u32,
    /// Score de suspeita (0.0 a 1.0)
    pub suspicion_score: f64,
    /// Severidade
    pub severity: String,
    /// Timestamp
    pub timestamp_ms: u64,
}

// ─── Analisador de Chip Dumping ───

#[derive(Debug, Clone, Default)]
pub struct ChipDumpAnalyzer {
    /// Histórico de transfers por par (dumper|dumpee)
    transfers: HashMap<String, Vec<ChipDumpRecord>>,
    /// Alertas gerados
    alerts: Vec<ChipDumpAlert>,
    /// Thresholds
    thresholds: ChipDumpThresholds,
}

#[derive(Debug, Clone)]
pub struct ChipDumpThresholds {
    /// Força máxima da mão para considerar dump (VeryWeak ou Weak)
    pub max_hand_strength: HandStrength,
    /// Valor mínimo para considerar suspeito (em big blinds ou fichas)
    pub min_amount: u64,
    /// Número mínimo de ocorrências para alerta
    pub min_occurrences: u32,
    /// Total mínimo transferido para alerta
    pub min_total_dumped: u64,
    /// Score mínimo para alerta
    pub alert_threshold: f64,
    /// Thresholds de severidade
    pub critical_threshold: f64,
    pub high_threshold: f64,
    pub medium_threshold: f64,
}

impl Default for ChipDumpThresholds {
    fn default() -> Self {
        Self {
            max_hand_strength: HandStrength::Weak,
            min_amount: 500, // 5 BB em NL100
            min_occurrences: 2,
            min_total_dumped: 1000,
            alert_threshold: 0.3,
            critical_threshold: 0.8,
            high_threshold: 0.6,
            medium_threshold: 0.4,
        }
    }
}

impl ChipDumpAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thresholds(thresholds: ChipDumpThresholds) -> Self {
        Self {
            thresholds,
            ..Default::default()
        }
    }

    /// Analisa uma ação de all-in para detectar possível chip dumping.
    /// Retorna true se foi registrado como suspeito.
    pub fn analyze_all_in(
        &mut self,
        from_player: &str,
        to_player: &str,
        amount: u64,
        hand_strength: HandStrength,
        hand_id: &str,
        timestamp_ms: u64,
    ) -> bool {
        // Só considera dump se mão for fraca e valor significativo
        if hand_strength > self.thresholds.max_hand_strength {
            return false;
        }
        if amount < self.thresholds.min_amount {
            return false;
        }

        let record = ChipDumpRecord {
            from_player: from_player.to_string(),
            to_player: to_player.to_string(),
            amount,
            hand_strength,
            hand_id: hand_id.to_string(),
            timestamp_ms,
        };

        let pair_key = make_pair_key(from_player, to_player);
        self.transfers
            .entry(pair_key.clone())
            .or_default()
            .push(record);

        // Verifica se deve gerar alerta
        let transfers = &self.transfers[&pair_key];
        let total: u64 = transfers.iter().map(|t| t.amount).sum();
        let occurrences = transfers.len() as u32;

        if occurrences >= self.thresholds.min_occurrences
            && total >= self.thresholds.min_total_dumped
        {
            let score = calculate_dump_score(transfers, total);
            if score >= self.thresholds.alert_threshold {
                let severity = if score >= self.thresholds.critical_threshold {
                    "critical"
                } else if score >= self.thresholds.high_threshold {
                    "high"
                } else if score >= self.thresholds.medium_threshold {
                    "medium"
                } else {
                    "low"
                };

                let alert = ChipDumpAlert {
                    dumper: from_player.to_string(),
                    dumpee: to_player.to_string(),
                    total_dumped: total,
                    occurrences,
                    suspicion_score: score,
                    severity: severity.to_string(),
                    timestamp_ms,
                };
                self.alerts.push(alert);
                return true;
            }
        }

        false
    }

    /// Retorna todos os alertas
    pub fn get_alerts(&self) -> Vec<ChipDumpAlert> {
        self.alerts.clone()
    }

    /// Retorna alertas por severidade
    pub fn get_alerts_by_severity(&self, severity: &str) -> Vec<ChipDumpAlert> {
        self.alerts
            .iter()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    /// Retorna histórico de transfers para um par
    pub fn get_transfers(&self, player_a: &str, player_b: &str) -> Vec<ChipDumpRecord> {
        let key = make_pair_key(player_a, player_b);
        self.transfers.get(&key).cloned().unwrap_or_default()
    }

    /// Retorna total transferido entre dois jogadores
    pub fn get_total_dumped(&self, player_a: &str, player_b: &str) -> u64 {
        self.get_transfers(player_a, player_b)
            .iter()
            .map(|t| t.amount)
            .sum()
    }

    /// Retorna todos os pares com transfers registrados
    pub fn get_all_pairs(&self) -> Vec<(String, u64, u32)> {
        self.transfers
            .iter()
            .map(|(key, records)| {
                let total: u64 = records.iter().map(|r| r.amount).sum();
                let count = records.len() as u32;
                (key.clone(), total, count)
            })
            .collect()
    }

    /// Reseta o analisador
    pub fn reset(&mut self) {
        self.transfers.clear();
        self.alerts.clear();
    }
}

// ─── Funções Auxiliares ───

fn make_pair_key(a: &str, b: &str) -> String {
    if a < b {
        format!("{}|{}", a, b)
    } else {
        format!("{}|{}", b, a)
    }
}

/// Calcula score de suspeita baseado no padrão de transfers
fn calculate_dump_score(transfers: &[ChipDumpRecord], total: u64) -> f64 {
    if transfers.is_empty() {
        return 0.0;
    }

    let n = transfers.len() as f64;

    // Fator 1: Consistência (todas as transfers são do mesmo dumper → dumpee?)
    // Se for sempre unidirecional, é mais suspeito
    let from_counts: HashMap<String, u32> = transfers.iter().fold(HashMap::new(), |mut acc, t| {
        *acc.entry(t.from_player.clone()).or_default() += 1;
        acc
    });
    let max_from = *from_counts.values().max().unwrap_or(&0) as f64;
    let consistency = max_from / n; // 1.0 = sempre o mesmo dumper

    // Fator 2: Fraqueza das mãos (mãos mais fracas = mais suspeito)
    let weakness_score = transfers
        .iter()
        .map(|t| match t.hand_strength {
            HandStrength::VeryWeak => 1.0,
            HandStrength::Weak => 0.7,
            HandStrength::Medium => 0.3,
            _ => 0.0,
        })
        .sum::<f64>()
        / n;

    // Fator 3: Volume (mais fichas = mais suspeito, normalizado)
    let volume_score = (total as f64 / 10000.0).min(1.0);

    // Fator 4: Frequência (mais ocorrências = mais suspeito)
    let frequency_score = (n / 10.0).min(1.0);

    // Score combinado: consistência 30%, fraqueza 30%, volume 25%, frequência 15%
    consistency * 0.30 + weakness_score * 0.30 + volume_score * 0.25 + frequency_score * 0.15
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_strong_hand() {
        let mut analyzer = ChipDumpAnalyzer::new();
        // All-in com Monster hand → não é dump
        let result =
            analyzer.analyze_all_in("alice", "bob", 5000, HandStrength::Monster, "hand1", 1000);
        assert!(!result);
        assert!(analyzer.get_alerts().is_empty());
    }

    #[test]
    fn test_ignore_small_amount() {
        let mut analyzer = ChipDumpAnalyzer::new();
        // All-in com VeryWeak mas valor pequeno → não atinge threshold
        let result =
            analyzer.analyze_all_in("alice", "bob", 100, HandStrength::VeryWeak, "hand1", 1000);
        assert!(!result);
    }

    #[test]
    fn test_single_dump_no_alert() {
        let mut analyzer = ChipDumpAnalyzer::new();
        // Uma única ocorrência não gera alerta (min_occurrences = 2)
        let result =
            analyzer.analyze_all_in("alice", "bob", 2000, HandStrength::VeryWeak, "hand1", 1000);
        // Registrado mas sem alerta
        assert!(!result);
        assert!(analyzer.get_alerts().is_empty());

        // Mas a transfer foi registrada
        let transfers = analyzer.get_transfers("alice", "bob");
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].amount, 2000);
    }

    #[test]
    fn test_multiple_dumps_generate_alert() {
        let mut analyzer = ChipDumpAnalyzer::new();

        // 3 all-ins com VeryWeak, valores altos
        analyzer.analyze_all_in("alice", "bob", 3000, HandStrength::VeryWeak, "hand1", 1000);
        analyzer.analyze_all_in("alice", "bob", 4000, HandStrength::VeryWeak, "hand2", 2000);
        let alerted =
            analyzer.analyze_all_in("alice", "bob", 3000, HandStrength::VeryWeak, "hand3", 3000);

        assert!(alerted);
        let alerts = analyzer.get_alerts();
        // Gera alerta na 2ª e na 3ª ocorrência (cada vez que thresholds são atingidos)
        assert!(!alerts.is_empty());
        // Último alerta tem os dados acumulados
        let last = alerts.last().unwrap();
        assert_eq!(last.dumper, "alice");
        assert_eq!(last.dumpee, "bob");
        assert_eq!(last.total_dumped, 10000);
        assert_eq!(last.occurrences, 3);
    }

    #[test]
    fn test_bidirectional_not_confused() {
        let mut analyzer = ChipDumpAnalyzer::new();

        // Alice → Bob (dump)
        analyzer.analyze_all_in("alice", "bob", 2000, HandStrength::VeryWeak, "hand1", 1000);
        analyzer.analyze_all_in("alice", "bob", 3000, HandStrength::VeryWeak, "hand2", 2000);

        // Bob → Charlie (outro par, não deve interferir)
        analyzer.analyze_all_in(
            "bob",
            "charlie",
            2000,
            HandStrength::VeryWeak,
            "hand3",
            3000,
        );
        analyzer.analyze_all_in(
            "bob",
            "charlie",
            3000,
            HandStrength::VeryWeak,
            "hand4",
            4000,
        );

        let alerts = analyzer.get_alerts();
        // Dois pares diferentes, cada um com 2 ocorrências → 2 alertas
        assert_eq!(alerts.len(), 2);
    }

    #[test]
    fn test_get_total_dumped() {
        let mut analyzer = ChipDumpAnalyzer::new();
        analyzer.analyze_all_in("alice", "bob", 1000, HandStrength::VeryWeak, "hand1", 1000);
        analyzer.analyze_all_in("alice", "bob", 2000, HandStrength::Weak, "hand2", 2000);

        let total = analyzer.get_total_dumped("alice", "bob");
        assert_eq!(total, 3000);

        // Ordem inversa deve funcionar
        let total_rev = analyzer.get_total_dumped("bob", "alice");
        assert_eq!(total_rev, 3000);
    }

    #[test]
    fn test_get_all_pairs() {
        let mut analyzer = ChipDumpAnalyzer::new();
        analyzer.analyze_all_in("alice", "bob", 1000, HandStrength::VeryWeak, "hand1", 1000);
        analyzer.analyze_all_in("charlie", "dave", 2000, HandStrength::Weak, "hand2", 2000);

        let pairs = analyzer.get_all_pairs();
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_reset() {
        let mut analyzer = ChipDumpAnalyzer::new();
        analyzer.analyze_all_in("alice", "bob", 2000, HandStrength::VeryWeak, "hand1", 1000);
        analyzer.analyze_all_in("alice", "bob", 3000, HandStrength::VeryWeak, "hand2", 2000);

        analyzer.reset();
        assert!(analyzer.get_alerts().is_empty());
        assert!(analyzer.get_transfers("alice", "bob").is_empty());
        assert_eq!(analyzer.get_total_dumped("alice", "bob"), 0);
    }

    #[test]
    fn test_dump_score_calculation() {
        let records = vec![
            ChipDumpRecord {
                from_player: "alice".to_string(),
                to_player: "bob".to_string(),
                amount: 5000,
                hand_strength: HandStrength::VeryWeak,
                hand_id: "h1".to_string(),
                timestamp_ms: 1000,
            },
            ChipDumpRecord {
                from_player: "alice".to_string(),
                to_player: "bob".to_string(),
                amount: 5000,
                hand_strength: HandStrength::VeryWeak,
                hand_id: "h2".to_string(),
                timestamp_ms: 2000,
            },
        ];

        let score = calculate_dump_score(&records, 10000);
        // consistency = 1.0 (sempre alice→bob)
        // weakness = 1.0 (todas VeryWeak)
        // volume = min(10000/10000, 1.0) = 1.0
        // frequency = min(2/10, 1.0) = 0.2
        // total = 0.30 + 0.30 + 0.25 + 0.03 = 0.88
        assert!((score - 0.88).abs() < 0.01);
    }

    #[test]
    fn test_severity_classification() {
        let mut analyzer = ChipDumpAnalyzer::with_thresholds(ChipDumpThresholds {
            max_hand_strength: HandStrength::Weak,
            min_amount: 100,
            min_occurrences: 2,
            min_total_dumped: 200,
            alert_threshold: 0.2,
            critical_threshold: 0.8,
            high_threshold: 0.5,
            medium_threshold: 0.3,
        });

        // 2 dumps VeryWeak → score ≈ 0.88 → critical
        analyzer.analyze_all_in("alice", "bob", 5000, HandStrength::VeryWeak, "hand1", 1000);
        analyzer.analyze_all_in("alice", "bob", 5000, HandStrength::VeryWeak, "hand2", 2000);

        let alerts = analyzer.get_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, "critical");
    }

    #[test]
    fn test_get_alerts_by_severity() {
        let mut analyzer = ChipDumpAnalyzer::with_thresholds(ChipDumpThresholds {
            max_hand_strength: HandStrength::Weak,
            min_amount: 100,
            min_occurrences: 1,
            min_total_dumped: 100,
            alert_threshold: 0.1,
            critical_threshold: 0.9,
            high_threshold: 0.6,
            medium_threshold: 0.3,
        });

        // Alice→Bob: score alto → high
        analyzer.analyze_all_in("alice", "bob", 5000, HandStrength::VeryWeak, "hand1", 1000);
        analyzer.analyze_all_in("alice", "bob", 5000, HandStrength::VeryWeak, "hand2", 2000);

        let high_alerts = analyzer.get_alerts_by_severity("high");
        assert!(!high_alerts.is_empty());
    }

    #[test]
    fn test_weak_hand_accepted() {
        let mut analyzer = ChipDumpAnalyzer::new();
        // Weak hand (threshold max = Weak) → aceito
        let result =
            analyzer.analyze_all_in("alice", "bob", 2000, HandStrength::Weak, "hand1", 1000);
        // Registrado (não gera alerta com 1 ocorrência)
        assert!(!result);
        let transfers = analyzer.get_transfers("alice", "bob");
        assert_eq!(transfers.len(), 1);
    }

    #[test]
    fn test_medium_hand_rejected() {
        let mut analyzer = ChipDumpAnalyzer::new();
        // Medium hand > max_hand_strength (Weak) → rejeitado
        let result =
            analyzer.analyze_all_in("alice", "bob", 5000, HandStrength::Medium, "hand1", 1000);
        assert!(!result);
        assert!(analyzer.get_transfers("alice", "bob").is_empty());
    }
}
