//! Módulo Antifraude & Detecção de Bots e Colusão no Motor Rust.
//!
//! Analisa padrões comportamentais de tempo de reação (detectando bots/autoclickers)
//! e cooperação anômala entre duplas de jogadores na mesma mesa (chip dumping/colusão).

use serde::{Deserialize, Serialize};

/// Recomendação emitida pela análise de risco antifraude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskRecommendation {
    /// Jogador limpo — ação permitida normalmente.
    Allow,
    /// Comportamento suspeito — sinalizado para revisão da equipe de integridade.
    FlagForReview,
    /// Alto risco de bot ou colusão — sessão deve ser bloqueada/encerrada.
    BlockSession,
}

/// Score de risco detalhado de um jogador.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskScore {
    /// Score numérico de 0.0 (sem risco) a 100.0 (risco máximo).
    pub total_score: f64,
    /// Score específico de bot/autoclicker (0.0 a 100.0).
    pub bot_score: f64,
    /// Score específico de colusão/chip-dumping (0.0 a 100.0).
    pub collusion_score: f64,
    /// Recomendação automatizada baseada no total_score.
    pub recommendation: RiskRecommendation,
}

impl RiskScore {
    /// Cria um novo RiskScore calculando a recomendação automática.
    #[must_use]
    pub fn new(bot_score: f64, collusion_score: f64) -> Self {
        let total_score = (bot_score * 0.6 + collusion_score * 0.4).clamp(0.0, 100.0);
        let recommendation = if total_score >= 75.0 {
            RiskRecommendation::BlockSession
        } else if total_score >= 40.0 {
            RiskRecommendation::FlagForReview
        } else {
            RiskRecommendation::Allow
        };

        Self {
            total_score,
            bot_score,
            collusion_score,
            recommendation,
        }
    }
}

/// Detector de Bots e Autoclickers baseado em tempo de tomada de decisão.
#[derive(Debug, Clone, Default)]
pub struct BotDetector {
    /// Histórico de tempos de reação (em milissegundos) das últimas jogadas.
    reaction_times_ms: Vec<u64>,
}

impl BotDetector {
    /// Cria um novo detector de bots.
    #[must_use]
    pub fn new() -> Self {
        Self {
            reaction_times_ms: Vec::new(),
        }
    }

    /// Registra o tempo de reação (em ms) de uma ação do jogador.
    pub fn record_action(&mut self, elapsed_ms: u64) {
        self.reaction_times_ms.push(elapsed_ms);
        // Mantém apenas as últimas 50 amostras de tempo
        if self.reaction_times_ms.len() > 50 {
            self.reaction_times_ms.remove(0);
        }
    }

    /// Calcula o score de risco de bot (0.0 a 100.0).
    ///
    /// Bots exibem:
    /// 1. Ações ultrarápidas (< 50ms) de forma consistente.
    /// 2. Variância estatística quase nula nos tempos de reação (reações robóticas idênticas).
    #[must_use]
    pub fn calculate_score(&self) -> f64 {
        if self.reaction_times_ms.len() < 5 {
            return 0.0;
        }

        let len = self.reaction_times_ms.len() as f64;
        let sum: u64 = self.reaction_times_ms.iter().sum();
        let mean = (sum as f64) / len;

        // Desvio padrão dos tempos de reação
        let variance: f64 = self
            .reaction_times_ms
            .iter()
            .map(|&t| {
                let diff = (t as f64) - mean;
                diff * diff
            })
            .sum::<f64>()
            / len;
        let std_dev = variance.sqrt();

        let mut score = 0.0f64;

        // 1. Penalidade por tempo médio ultrarrápido (< 80ms)
        if mean < 50.0 {
            score += 60.0;
        } else if mean < 150.0 {
            score += 30.0;
        }

        // 2. Penalidade por variância extremamente baixa (sinal claro de automação robótica)
        if std_dev < 10.0 {
            score += 40.0;
        } else if std_dev < 25.0 {
            score += 20.0;
        }

        score.clamp(0.0, 100.0)
    }
}

/// Detector de Colusão e Transferência de Fichas (Chip Dumping).
#[derive(Debug, Clone, Default)]
pub struct CollusionDetector {
    /// Registro de pares de jogadores e histórico de potes disputados.
    headsup_hands: u32,
    soft_plays: u32,
}

impl CollusionDetector {
    /// Cria um novo detector de colusão.
    #[must_use]
    pub fn new() -> Self {
        Self {
            headsup_hands: 0,
            soft_plays: 0,
        }
    }

    /// Registra um confronto entre dois jogadores específicos.
    pub fn record_headsup_hand(&mut self, was_soft_play: bool) {
        self.headsup_hands += 1;
        if was_soft_play {
            self.soft_plays += 1;
        }
    }

    /// Calcula o score de colusão (0.0 a 100.0).
    #[must_use]
    pub fn calculate_score(&self) -> f64 {
        if self.headsup_hands < 3 {
            return 0.0;
        }

        let soft_ratio = (self.soft_plays as f64) / (self.headsup_hands as f64);
        if soft_ratio > 0.6 {
            100.0 * soft_ratio
        } else if soft_ratio > 0.3 {
            50.0 * soft_ratio
        } else {
            0.0
        }
        .clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_detector_human_behavior() {
        let mut detector = BotDetector::new();
        // Simula tempos humanos (variados entre 800ms e 2500ms)
        let human_times = vec![1200, 850, 2100, 1400, 950, 1800, 3100, 1150, 1600, 2050];
        for t in human_times {
            detector.record_action(t);
        }

        let score = detector.calculate_score();
        assert!(score < 30.0, "Humano não deve ter score alto de bot: {}", score);
    }

    #[test]
    fn test_bot_detector_bot_behavior() {
        let mut detector = BotDetector::new();
        // Simula bot robótico (tempo cravado em ~30ms com desvio quase zero)
        for _ in 0..10 {
            detector.record_action(30);
        }

        let score = detector.calculate_score();
        assert!(score >= 80.0, "Bot robótico deve ter score alto: {}", score);
    }

    #[test]
    fn test_collusion_detector_normal_vs_suspicious() {
        let mut detector = CollusionDetector::new();

        // 1. Jogo normal (poucas jogadas passivas suspeitas)
        for _ in 0..10 {
            detector.record_headsup_hand(false);
        }
        assert_eq!(detector.calculate_score(), 0.0);

        // 2. Colusão detectada (muitas desistências intencionais quando parceiro aposta)
        for _ in 0..10 {
            detector.record_headsup_hand(true);
        }
        assert!(detector.calculate_score() >= 50.0);
    }

    #[test]
    fn test_risk_score_recommendations() {
        let low_risk = RiskScore::new(10.0, 0.0);
        assert_eq!(low_risk.recommendation, RiskRecommendation::Allow);

        let mid_risk = RiskScore::new(50.0, 30.0);
        assert_eq!(mid_risk.recommendation, RiskRecommendation::FlagForReview);

        let high_risk = RiskScore::new(90.0, 80.0);
        assert_eq!(high_risk.recommendation, RiskRecommendation::BlockSession);
    }
}
