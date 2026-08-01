// ─── Módulo Antifraude: Detecção de Bots ───
// Detecta comportamento automatizado via análise temporal e padrões matemáticos.
// Regras de negócio: BUSINESS_RULES.md §14.3

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Ação do Jogador (para análise temporal) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerAction {
    /// ID do jogador
    pub player_id: String,
    /// Tipo de ação
    pub action_type: String,
    /// Valor (bet/raise amount, 0 para fold/check/call)
    pub amount: u64,
    /// Timestamp da ação em ms
    pub timestamp_ms: u64,
    /// ID da mão
    pub hand_id: String,
    /// Street (preflop, flop, turn, river)
    pub street: String,
}

// ─── Métricas de Bot ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BotMetrics {
    /// Desvio padrão dos tempos de resposta (ms)
    pub response_time_stddev: f64,
    /// Média dos tempos de resposta (ms)
    pub response_time_mean: f64,
    /// Coeficiente de variação (stddev/mean)
    pub coefficient_of_variation: f64,
    /// Número de ações analisadas
    pub total_actions: u32,
    /// Score de precisão matemática (0-1, quão "perfeito" é o sizing)
    pub mathematical_precision: f64,
    /// Score de consistência temporal (0-1, quão constante é o tempo)
    pub temporal_consistency: f64,
    /// Score combinado de bot (0-1)
    pub bot_score: f64,
}

// ─── Alerta de Bot ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BotAlert {
    /// ID do jogador suspeito
    pub player_id: String,
    /// Score de bot (0-1)
    pub bot_score: f64,
    /// Métricas detalhadas
    pub metrics: BotMetrics,
    /// Severidade
    pub severity: String,
    /// Timestamp
    pub timestamp_ms: u64,
}

// ─── Analisador de Bots ───

#[derive(Debug, Clone, Default)]
pub struct BotDetector {
    /// Histórico de ações por jogador
    player_actions: HashMap<String, Vec<PlayerAction>>,
    /// Alertas gerados
    alerts: Vec<BotAlert>,
    /// Thresholds
    thresholds: BotThresholds,
}

#[derive(Debug, Clone)]
pub struct BotThresholds {
    /// Número mínimo de ações para análise
    pub min_actions: u32,
    /// Coeficiente de variação máximo (abaixo disso = suspeito de bot)
    /// Humanos têm variação natural; bots são muito consistentes
    pub max_coefficient_of_variation: f64,
    /// Precisão matemática máxima (acima disso = suspeito)
    /// Bots usam sizing GTO-perfeito (ex: 66.7% pot, 33.3% pot)
    pub max_mathematical_precision: f64,
    /// Score mínimo para alerta
    pub alert_threshold: f64,
    /// Thresholds de severidade
    pub critical_threshold: f64,
    pub high_threshold: f64,
    pub medium_threshold: f64,
}

impl Default for BotThresholds {
    fn default() -> Self {
        Self {
            min_actions: 20,
            max_coefficient_of_variation: 0.15, // < 15% variação = suspeito
            max_mathematical_precision: 0.85,   // > 85% precisão = suspeito
            alert_threshold: 0.4,
            critical_threshold: 0.8,
            high_threshold: 0.6,
            medium_threshold: 0.4,
        }
    }
}

impl BotDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thresholds(thresholds: BotThresholds) -> Self {
        Self {
            thresholds,
            ..Default::default()
        }
    }

    /// Registra uma ação do jogador para análise
    pub fn record_action(&mut self, action: PlayerAction) {
        self.player_actions
            .entry(action.player_id.clone())
            .or_default()
            .push(action);
    }

    /// Registra o tempo de reação de uma ação do jogador (em ms)
    pub fn record_reaction_time(&mut self, player_id: &str, elapsed_ms: u64) {
        self.record_action(PlayerAction {
            player_id: player_id.to_string(),
            action_type: "action".to_string(),
            amount: 0,
            timestamp_ms: elapsed_ms,
            hand_id: "hand_auto".to_string(),
            street: "preflop".to_string(),
        });
    }

    /// Retorna o score de bot de um jogador (0.0 a 100.0)
    pub fn calculate_bot_score(&mut self, player_id: &str) -> f64 {
        let now = 1700000000000u64;
        
        // FASE 1: ML Antifraude via Tract ONNX (Mock / Documentação de Interface)
        // Em produção, o modelo treinado `antifraud_model.onnx` seria carregado globalmente.
        // Aqui simulamos a inferência chamando a lógica do Tensor caso tivéssemos o arquivo carregado.
        if let Some(actions) = self.player_actions.get(player_id) {
            if actions.len() >= 5 {
                // Prepara o tensor 1D com os tempos de resposta para a IA
                let times: Vec<f32> = actions.iter().map(|a| a.timestamp_ms as f32).collect();
                
                /* Lógica Tract-ONNX comentada até a entrega do modelo pela ciência de dados:
                use tract_onnx::prelude::*;
                if let Ok(model) = tract_onnx::onnx().model_for_path("antifraud_model.onnx") {
                    if let Ok(runnable) = model.into_optimized().unwrap().into_runnable() {
                        let tensor = tract_ndarray::Array1::from_vec(times.clone()).into_tensor();
                        if let Ok(result) = runnable.run(tvec!(tensor.into())) {
                            let score = result[0].to_array_view::<f32>().unwrap()[0];
                            return (score * 100.0).clamp(0.0, 100.0) as f64;
                        }
                    }
                }
                */

                // Fallback Heurístico (Até que o arquivo .onnx esteja no disco)
                let mean: f64 = times.iter().sum::<f32>() as f64 / times.len() as f64;
                if mean < 50.0 {
                    return 85.0; // Bots têm reação instantânea
                } else if mean < 150.0 {
                    return 45.0;
                }
            }
        }

        if let Some(alert) = self.analyze_player(player_id, now) {
            alert.bot_score * 100.0
        } else {
            0.0
        }
    }

    /// Analisa um jogador específico e retorna métricas + alerta se suspeito
    pub fn analyze_player(&mut self, player_id: &str, current_time_ms: u64) -> Option<BotAlert> {
        let actions = match self.player_actions.get(player_id) {
            Some(actions) if actions.len() >= self.thresholds.min_actions as usize => actions,
            _ => return None,
        };

        let metrics = compute_bot_metrics(actions);

        if metrics.bot_score >= self.thresholds.alert_threshold {
            let severity = if metrics.bot_score >= self.thresholds.critical_threshold {
                "critical"
            } else if metrics.bot_score >= self.thresholds.high_threshold {
                "high"
            } else if metrics.bot_score >= self.thresholds.medium_threshold {
                "medium"
            } else {
                "low"
            };

            let alert = BotAlert {
                player_id: player_id.to_string(),
                bot_score: metrics.bot_score,
                metrics: metrics.clone(),
                severity: severity.to_string(),
                timestamp_ms: current_time_ms,
            };
            self.alerts.push(alert.clone());
            Some(alert)
        } else {
            None
        }
    }

    /// Analisa todos os jogadores com ações suficientes
    pub fn analyze_all(&mut self, current_time_ms: u64) -> Vec<BotAlert> {
        let player_ids: Vec<String> = self.player_actions.keys().cloned().collect();
        let mut new_alerts = Vec::new();

        for pid in player_ids {
            if let Some(alert) = self.analyze_player(&pid, current_time_ms) {
                new_alerts.push(alert);
            }
        }

        new_alerts
    }

    /// Retorna métricas de um jogador sem gerar alerta
    pub fn get_metrics(&self, player_id: &str) -> Option<BotMetrics> {
        let actions = self.player_actions.get(player_id)?;
        if actions.len() < self.thresholds.min_actions as usize {
            return None;
        }
        Some(compute_bot_metrics(actions))
    }

    /// Retorna todos os alertas
    pub fn get_alerts(&self) -> Vec<BotAlert> {
        self.alerts.clone()
    }

    /// Retorna alertas por severidade
    pub fn get_alerts_by_severity(&self, severity: &str) -> Vec<BotAlert> {
        self.alerts
            .iter()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    /// Retorna alertas para um jogador específico
    pub fn get_alerts_for_player(&self, player_id: &str) -> Vec<BotAlert> {
        self.alerts
            .iter()
            .filter(|a| a.player_id == player_id)
            .cloned()
            .collect()
    }

    /// Retorna número de ações registradas para um jogador
    pub fn get_action_count(&self, player_id: &str) -> usize {
        self.player_actions
            .get(player_id)
            .map(|a| a.len())
            .unwrap_or(0)
    }

    /// Reseta o detector
    pub fn reset(&mut self) {
        self.player_actions.clear();
        self.alerts.clear();
    }
}

// ─── Cálculo de Métricas ───

fn compute_bot_metrics(actions: &[PlayerAction]) -> BotMetrics {
    let n = actions.len();

    // ─── 1. Análise Temporal: tempos entre ações consecutivas ───
    let mut response_times: Vec<f64> = Vec::new();
    for i in 1..n {
        let dt = actions[i].timestamp_ms as f64 - actions[i - 1].timestamp_ms as f64;
        // Ignora gaps muito grandes (> 30s = entre mãos diferentes)
        if dt < 30000.0 {
            response_times.push(dt);
        }
    }

    let (mean, stddev, cv) = if response_times.is_empty() {
        (0.0, 0.0, 0.0)
    } else {
        let m = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let variance = response_times.iter().map(|t| (t - m).powi(2)).sum::<f64>()
            / response_times.len() as f64;
        let s = variance.sqrt();
        let cv = if m > 0.0 { s / m } else { 0.0 };
        (m, s, cv)
    };

    // ─── 2. Precisão Matemática: quão "perfeitos" são os sizing ───
    // Bots tendem a usar frações exatas do pote (1/3, 2/3, 1/2, 3/4, etc.)
    let precision_scores: Vec<f64> = actions
        .iter()
        .filter(|a| a.amount > 0)
        .map(|a| {
            let amount = a.amount as f64;
            // Verifica se o valor é múltiplo exato de big blinds (BB=100)
            // Bots usam sizing em BB exato: 300, 500, 700, 1000, etc.
            let bb = 100.0;
            let remainder = amount % bb;
            let is_exact_bb = remainder < 0.01;

            if is_exact_bb {
                1.0 // muito preciso = suspeito
            } else {
                0.0
            }
        })
        .collect();

    let mathematical_precision = if precision_scores.is_empty() {
        0.0
    } else {
        precision_scores.iter().sum::<f64>() / precision_scores.len() as f64
    };

    // ─── 3. Consistência Temporal ───
    // Quanto menor o CV, mais consistente (suspeito)
    // CV=0 → 1.0 (timing perfeitamente constante = muito suspeito)
    // CV=0.5+ → 0.0 (variação natural humana = normal)
    let temporal_consistency = (1.0 - (cv / 0.5).min(1.0)).max(0.0);

    // ─── 4. Score Combinado de Bot ───
    // temporal 50% + precisão matemática 50%
    let bot_score = temporal_consistency * 0.50 + mathematical_precision * 0.50;

    BotMetrics {
        response_time_stddev: stddev,
        response_time_mean: mean,
        coefficient_of_variation: cv,
        total_actions: n as u32,
        mathematical_precision,
        temporal_consistency,
        bot_score,
    }
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action(
        player_id: &str,
        action_type: &str,
        amount: u64,
        timestamp_ms: u64,
        hand_id: &str,
        street: &str,
    ) -> PlayerAction {
        PlayerAction {
            player_id: player_id.to_string(),
            action_type: action_type.to_string(),
            amount,
            timestamp_ms,
            hand_id: hand_id.to_string(),
            street: street.to_string(),
        }
    }

    #[test]
    fn test_insufficient_actions_no_alert() {
        let mut detector = BotDetector::new();
        // Apenas 5 ações (min = 20)
        for i in 0..5 {
            detector.record_action(make_action(
                "alice",
                "bet",
                300,
                1000 + i as u64 * 500,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("alice", 5000);
        assert!(alert.is_none());
    }

    #[test]
    fn test_human_like_variation_no_alert() {
        let mut detector = BotDetector::new();
        // 30 ações com tempos variados (humano) e sizing não-exato
        let mut ts = 1000u64;
        for i in 0..30u64 {
            // Tempos variam entre 2s e 15s
            ts += 2000 + (i * 300) % 13000;
            // Valores não-exatos: 327, 483, 551, 679, etc.
            let amounts: [u64; 6] = [327, 483, 551, 679, 812, 934];
            detector.record_action(make_action(
                "alice",
                "bet",
                amounts[(i as usize) % amounts.len()],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("alice", ts + 1000);
        // Humano com variação → sem alerta
        assert!(alert.is_none());
    }

    #[test]
    fn test_bot_like_constant_timing() {
        let mut detector = BotDetector::new();
        // 25 ações com timing quase constante (bot-like)
        let mut ts = 1000u64;
        for i in 0..25 {
            // Tempo entre ações: sempre ~3s ± 50ms
            ts += 3000 + (i as u64 % 2) * 50;
            detector.record_action(make_action(
                "bob",
                "bet",
                300 + (i as u64 * 7) % 150,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("bob", ts + 1000);
        // Timing muito consistente → deve gerar alerta
        assert!(alert.is_some());
        let a = alert.unwrap();
        assert!(a.metrics.temporal_consistency > 0.5);
    }

    #[test]
    fn test_bot_like_precise_sizing() {
        let mut detector = BotDetector::new();
        // 25 ações com sizing exato (múltiplos de BB)
        let mut ts = 1000u64;
        for i in 0..25 {
            ts += 2000 + (i as u64 * 500) % 8000; // variação humana no timing
                                                  // Sizing sempre exato: 300, 500, 700, 1000 (múltiplos de 100 = 1 BB)
            let exact_amounts = [300, 500, 700, 1000, 1500, 2000];
            let amount = exact_amounts[i % exact_amounts.len()];
            detector.record_action(make_action(
                "charlie",
                "bet",
                amount,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("charlie", ts + 1000);
        // Sizing muito preciso → deve gerar alerta
        assert!(alert.is_some());
        let a = alert.unwrap();
        assert!(a.metrics.mathematical_precision > 0.5);
    }

    #[test]
    fn test_combined_bot_score_high() {
        let mut detector = BotDetector::new();
        let mut ts = 1000u64;
        for i in 0..25 {
            // Timing constante + sizing exato = bot clássico
            ts += 3000 + (i as u64 % 3) * 10; // quase constante
            let exact_amounts = [500, 1000, 1500, 2000];
            let amount = exact_amounts[i % exact_amounts.len()];
            detector.record_action(make_action(
                "dave",
                "bet",
                amount,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("dave", ts + 1000);
        assert!(alert.is_some());
        let a = alert.unwrap();
        // Score combinado deve ser alto (> 0.7)
        assert!(a.bot_score > 0.6);
    }

    #[test]
    fn test_get_metrics() {
        let mut detector = BotDetector::new();
        let mut ts = 1000u64;
        for i in 0..25 {
            ts += 3000 + (i as u64 * 10) % 100;
            detector.record_action(make_action(
                "eve",
                "bet",
                500,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let metrics = detector.get_metrics("eve");
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.total_actions, 25);
        assert!(m.response_time_mean > 0.0);
    }

    #[test]
    fn test_get_metrics_insufficient_actions() {
        let detector = BotDetector::new();
        let metrics = detector.get_metrics("unknown");
        assert!(metrics.is_none());
    }

    #[test]
    fn test_analyze_all() {
        let mut detector = BotDetector::new();
        let mut ts = 1000u64;

        // Alice: humana (variação normal)
        for i in 0..25 {
            ts += 2000 + (i as u64 * 700) % 15000;
            detector.record_action(make_action(
                "alice",
                "bet",
                350 + (i as u64 * 15) % 300,
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }

        // Bob: bot (constante + preciso)
        let mut ts2 = 1000u64;
        for i in 0..25 {
            ts2 += 3000 + (i as u64 % 2) * 20;
            detector.record_action(make_action(
                "bob",
                "bet",
                [500, 1000, 1500][i % 3],
                ts2,
                &format!("h{}", i),
                "flop",
            ));
        }

        let alerts = detector.analyze_all(ts.max(ts2) + 1000);
        // Bob deve ter alerta, Alice não
        let bob_alerts: Vec<_> = alerts.iter().filter(|a| a.player_id == "bob").collect();
        assert!(!bob_alerts.is_empty());
    }

    #[test]
    fn test_severity_levels() {
        let mut detector = BotDetector::with_thresholds(BotThresholds {
            min_actions: 10,
            max_coefficient_of_variation: 0.3,
            max_mathematical_precision: 0.7,
            alert_threshold: 0.2,
            critical_threshold: 0.7,
            high_threshold: 0.5,
            medium_threshold: 0.3,
        });

        let mut ts = 1000u64;
        for i in 0..15 {
            ts += 3000 + (i as u64 % 2) * 5; // extremamente constante
            detector.record_action(make_action(
                "frank",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let alert = detector.analyze_player("frank", ts + 1000);
        assert!(alert.is_some());
        // Com timing ultra-constante + sizing exato → critical ou high
        let severity = &alert.unwrap().severity;
        assert!(severity == "critical" || severity == "high");
    }

    #[test]
    fn test_get_alerts_by_severity() {
        let mut detector = BotDetector::with_thresholds(BotThresholds {
            min_actions: 10,
            max_coefficient_of_variation: 0.3,
            max_mathematical_precision: 0.7,
            alert_threshold: 0.2,
            critical_threshold: 0.9,
            high_threshold: 0.6,
            medium_threshold: 0.3,
        });

        let mut ts = 1000u64;
        for i in 0..15u64 {
            ts += 3000 + (i % 2) * 5;
            let amounts: [u64; 2] = [500, 1000];
            detector.record_action(make_action(
                "grace",
                "bet",
                amounts[(i as usize) % amounts.len()],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        detector.analyze_player("grace", ts + 1000);

        // Deve ter alertas em alguma severidade
        let all_alerts = detector.get_alerts();
        assert!(!all_alerts.is_empty());
        // Verifica que get_alerts_by_severity funciona para a severidade correta
        let severity = &all_alerts[0].severity;
        let filtered = detector.get_alerts_by_severity(severity);
        assert!(!filtered.is_empty());
    }

    #[test]
    fn test_get_alerts_for_player() {
        let mut detector = BotDetector::with_thresholds(BotThresholds {
            min_actions: 10,
            max_coefficient_of_variation: 0.3,
            max_mathematical_precision: 0.7,
            alert_threshold: 0.2,
            critical_threshold: 0.9,
            high_threshold: 0.6,
            medium_threshold: 0.3,
        });

        let mut ts = 1000u64;
        for i in 0..15 {
            ts += 3000 + (i as u64 % 2) * 5;
            detector.record_action(make_action(
                "henry",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        detector.analyze_player("henry", ts + 1000);

        let henry_alerts = detector.get_alerts_for_player("henry");
        assert!(!henry_alerts.is_empty());
        assert_eq!(henry_alerts[0].player_id, "henry");

        let unknown_alerts = detector.get_alerts_for_player("unknown");
        assert!(unknown_alerts.is_empty());
    }

    #[test]
    fn test_get_action_count() {
        let mut detector = BotDetector::new();
        for i in 0..7 {
            detector.record_action(make_action(
                "ivan",
                "check",
                0,
                1000 + i as u64 * 1000,
                &format!("h{}", i),
                "preflop",
            ));
        }
        assert_eq!(detector.get_action_count("ivan"), 7);
        assert_eq!(detector.get_action_count("unknown"), 0);
    }

    #[test]
    fn test_reset() {
        let mut detector = BotDetector::with_thresholds(BotThresholds {
            min_actions: 5,
            max_coefficient_of_variation: 0.3,
            max_mathematical_precision: 0.7,
            alert_threshold: 0.2,
            critical_threshold: 0.9,
            high_threshold: 0.6,
            medium_threshold: 0.3,
        });

        let mut ts = 1000u64;
        for i in 0..10 {
            ts += 3000 + (i as u64 % 2) * 5;
            detector.record_action(make_action(
                "jack",
                "bet",
                [500, 1000][i % 2],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        detector.analyze_player("jack", ts + 1000);
        assert!(!detector.get_alerts().is_empty());

        detector.reset();
        assert!(detector.get_alerts().is_empty());
        assert_eq!(detector.get_action_count("jack"), 0);
    }

    #[test]
    fn test_metrics_for_normal_player() {
        let mut detector = BotDetector::new();
        let mut ts = 1000u64;
        for i in 0..25u64 {
            // Variação humana: 2s a 18s entre ações
            ts += 2000 + (i * 700) % 16000;
            // Valores não-exatos: 325, 478, 551, 673, 821, 947 (não múltiplos de 100)
            let amounts: [u64; 6] = [325, 478, 551, 673, 821, 947];
            detector.record_action(make_action(
                "kate",
                "bet",
                amounts[(i as usize) % amounts.len()],
                ts,
                &format!("h{}", i),
                "flop",
            ));
        }
        let metrics = detector.get_metrics("kate").unwrap();
        // Jogador humano: CV alto, precisão baixa
        assert!(metrics.coefficient_of_variation > 0.1);
        assert!(metrics.mathematical_precision < 0.5);
        assert!(metrics.bot_score < 0.5);
    }

    #[test]
    fn test_empty_actions_no_panic() {
        let mut detector = BotDetector::new();
        let alert = detector.analyze_player("noone", 1000);
        assert!(alert.is_none());
        let metrics = detector.get_metrics("noone");
        assert!(metrics.is_none());
    }
}
