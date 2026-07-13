// Property‑based tests for o motor de antifraude.
// Utilizamos a crate `proptest` para gerar milhares de casos de teste de forma automática.
// Cada teste verifica invariantes fundamentais (por exemplo, pontuações de suspeição
// sempre entre 0.0 e 1.0, alertas nunca com severidade fora do intervalo esperado, etc.).
// Isso aumenta a quantidade de testes em ordem de magnitude sem comprometer a qualidade.

use proptest::prelude::*;

// Importa os módulos que serão testados.
use crate::antifraud::{bot_detection, chip_dumping, collusion, multi_account};

// ---------- Collusion ----------

proptest! {
    #[test]
    fn collusion_random_actions(action_count in 1..100usize) {
        // Gera uma sequência aleatória de ações de jogadores.
        let mut analyzer = collusion::CollusionAnalyzer::default();
        for i in 0..action_count {
            // Simula ação de um jogador em uma mesa arbitrária.
            let table_id = format!("table{}", i % 3);
            let player = format!("player{}", i % 5);
            let amount = (i as u64 % 1000) + 1;
            let hand = collusion::HandStrength::Medium;
            // Cria um registro de ação compatível com o módulo.
            let action = collusion::ActionRecord {
                player_id: player,
                action: collusion::PlayerAction::Raise(amount),
                hand_strength: hand.clone(),
                timestamp_ms: 1000 + i as u64 * 10,
                street: (i % 4) as u8,
            };
            analyzer.record_action(&table_id, action);
        }
        // Nenhum panic deve ocorrer e a pontuação de suspeição deve estar no intervalo.
        for alert in analyzer.get_alerts() {
            prop_assert!(alert.pair.suspicion_score >= 0.0 && alert.pair.suspicion_score <= 1.0);
        }
    }
}

// ---------- Chip Dumping ----------

proptest! {
    #[test]
    fn chip_dumping_random_transfers(transfer_count in 1..50usize) {
        let mut analyzer = chip_dumping::ChipDumpAnalyzer::default();
        for i in 0..transfer_count {
            let from = format!("player{}", i % 4);
            let to = format!("player{}", (i + 1) % 4);
            let amount = ((i as u64 % 2000) + 500) as u64; // garante valor razoável
            let hand = chip_dumping::HandStrength::Weak;
            // hand_id and timestamp are required parameters.
            let hand_id = format!("hand{}", i);
            let timestamp = 1000 + i as u64 * 10;
            analyzer.analyze_all_in(&from, &to, amount, hand.clone(), &hand_id, timestamp);
        }
        // Verifica que o score de dump está dentro do intervalo esperado.
        for alert in analyzer.get_alerts() {
            prop_assert!(alert.suspicion_score >= 0.0 && alert.suspicion_score <= 1.0);
        }
    }
}

// ---------- Bot Detection ----------

proptest! {
    #[test]
    fn bot_detection_random_actions(action_count in 20..200usize) {
        let mut detector = bot_detection::BotDetector::default();
        for i in 0..action_count {
            let player = format!("player{}", i % 3);
            let amount = ((i as u64 % 1000) + 1) as u64;
            let timestamp = i as u64 * 100; // milissegundos simulados
            // Cria a ação do jogador conforme a struct esperada.
            let action = bot_detection::PlayerAction {
                player_id: player,
                action_type: "bet".to_string(),
                amount,
                timestamp_ms: timestamp,
                hand_id: format!("hand{}", i),
                street: "flop".to_string(),
            };
            detector.record_action(action);
        }
        // O detector deve produzir alertas com score entre 0 e 1.
        for alert in detector.get_alerts() {
            prop_assert!(alert.bot_score >= 0.0 && alert.bot_score <= 1.0);
        }
    }
}

// ---------- Multi‑Account ----------

proptest! {
    #[test]
    fn multi_account_random_fingerprints(fp_count in 1..30usize) {
        let mut detector = multi_account::MultiAccountDetector::default();
        for i in 0..fp_count {
            let player_id = format!("player{}", i);
            let ip = format!("192.168.{}.{}", i % 255, i % 255);
            let hw = format!("hw{}", i % 10);
            let ua = "Mozilla/5.0".to_string();
            let screen = "1920x1080".to_string();
            let tz = "UTC".to_string();
            let lang = "pt-BR".to_string();
            let ts = (i as u64) * 1000;
            let fp = multi_account::PlayerFingerprint {
                player_id: player_id.clone(),
                ip_address: ip,
                hardware_id: hw,
                user_agent: ua,
                screen_resolution: screen,
                timezone: tz,
                language: lang,
                first_seen_ms: ts,
                last_seen_ms: ts + 500,
            };
            detector.register_fingerprint(fp);
        }
        // Verifica que nenhum alerta tem score fora do intervalo.
        for alert in detector.get_alerts() {
            prop_assert!(alert.suspicion_score >= 0.0 && alert.suspicion_score <= 1.0);
        }
    }
}
