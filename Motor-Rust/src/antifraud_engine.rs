// antifraud_engine.rs — Módulo de compatibilidade e transição para a suíte antifraud unificada.
// Re-exporta os tipos de `poker_engine::antifraud` para compatibilidade.

pub use crate::antifraud::{RiskRecommendation, RiskScore, AntiFraudSuite};
pub use crate::antifraud::bot_detection::BotDetector;
