use chrono::Utc;
use poker_engine::history::{
    HandHistoryRecord, HandPlayerInfo, HandWinnerInfo,
};
use sha2::{Digest, Sha256};
use std::time::Instant;

#[test]
fn test_100k_hand_history_export_and_provably_fair_audits_stress() {
    println!("\n========================================================");
    println!(" INICIANDO SIMULAÇÃO MASSIVA DE 100.000 REGISTROS DE HISTÓRICO ");
    println!("========================================================\n");

    let total_records = 100_000;
    let start_time = Instant::now();

    for i in 0..total_records {
        let server_seed = format!("Server_Seed_{}", i);
        let server_seed_hash = hex::encode(Sha256::digest(server_seed.as_bytes()));

        let record = HandHistoryRecord {
            hand_id: format!("HAND-{}", i),
            table_id: "Table_Main".into(),
            timestamp: Utc::now(),
            small_blind: 10.0,
            big_blind: 20.0,
            server_seed,
            server_seed_hash,
            client_seed: format!("Client_Seed_{}", i),
            nonce: i as u64,
            players: vec![HandPlayerInfo {
                player_id: "Player_1".into(),
                name: "Alice".into(),
                starting_stack: 2000.0,
                hole_cards: None,
            }],
            community_cards: vec![],
            actions: vec![],
            winners: vec![HandWinnerInfo {
                player_id: "Player_1".into(),
                amount_won: 30.0,
                hand_description: "Uncontested Pot".into(),
            }],
        };

        assert!(record.verify_provably_fair());

        if i % 25_000 == 0 {
            let exported = record.export_pokerstars_format();
            assert!(exported.contains("Poker Hand #HAND-"));
        }
    }

    let elapsed = start_time.elapsed();
    let records_per_sec = (total_records as f64) / elapsed.as_secs_f64();

    println!("   ✔ 100.000 históricos e verificações Provably Fair concluídos!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!("   - Taxa de Gravação/Auditoria: {:.2} registros/segundo", records_per_sec);
    println!("   - Transparência Criptográfica: 100% Provably Fair Válida");
    println!("========================================================\n");

    assert_eq!(total_records, 100_000);
}
