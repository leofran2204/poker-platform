use poker_engine::admin::AdminDashboard;
use poker_engine::ledger::{EntryType, LedgerAccount};
use std::time::Instant;

#[test]
fn test_100k_ledger_crypto_audits_stress() {
    println!("\n========================================================");
    println!(" INICIANDO SIMULAÇÃO MASSIVA DE 100.000 AUDITORIAS LEDGER ");
    println!("========================================================\n");

    let admin = AdminDashboard::new();
    let account = LedgerAccount::new("Audit_Target", 100000);

    for i in 0..10 {
        let _ = account.record_transaction(1000 * (i + 1), EntryType::Deposit, Some(format!("TX-{}", i)));
    }

    let total_audits = 100_000;
    let start_time = Instant::now();

    for _ in 0..total_audits {
        let audit = admin.audit_ledger_account(&account);
        assert!(audit.hash_chain_valid);
        assert_eq!(audit.transaction_count, 10);
    }

    let elapsed = start_time.elapsed();
    let audits_per_sec = (total_audits as f64) / elapsed.as_secs_f64();

    println!("   ✔ 100.000 auditorias criptográficas de Ledger concluídas!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!("   - Throughput Auditoria: {:.2} auditorias/segundo", audits_per_sec);
    println!("   - Cadeia de Hashes SHA-256: 100% Integra e Válida");
    println!("========================================================\n");

    assert_eq!(total_audits, 100_000);
}
