// mod.rs — Módulo de Antifraude e Integridade do Jogo (Unificado)
// Reúne detecção de bots, chip dumping, colusão e múltiplas contas sob uma fachada unificada.

pub mod bot_detection;
pub mod chip_dumping;
pub mod collusion;
pub mod multi_account;

use serde::{Deserialize, Serialize};
use bot_detection::BotDetector;
use chip_dumping::ChipDumpAnalyzer;
use collusion::CollusionAnalyzer;
use multi_account::MultiAccountDetector;

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

/// Suíte unificada de Antifraude conectada às mesas de Poker em tempo real.
#[derive(Debug, Default)]
pub struct AntiFraudSuite {
    pub bot_detector: BotDetector,
    pub chip_dump_analyzer: ChipDumpAnalyzer,
    pub collusion_analyzer: CollusionAnalyzer,
    pub multi_account_detector: MultiAccountDetector,
}

impl AntiFraudSuite {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra uma ação de jogador e calcula o score de risco atualizado
    pub fn process_action(&mut self, player_id: &str, elapsed_ms: u64) -> RiskScore {
        // Registra o tempo de reação no detector de bots
        self.bot_detector.record_reaction_time(player_id, elapsed_ms);
        let bot_score = self.bot_detector.calculate_bot_score(player_id);
        
        // Retorna o score combinado
        RiskScore::new(bot_score, 0.0)
    }
}
