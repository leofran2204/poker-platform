use poker_engine::ledger::{EntryType, LedgerAccount};
use std::sync::Arc;
use std::thread;

#[test]
fn test_massive_concurrent_ledger_stress_10k_transactions() {
    let initial_balance = 10_000_000i64; // R$ 100.000,00 em centavos
    let account = Arc::new(LedgerAccount::new("StressUser_001", initial_balance));

    let num_threads = 50;
    let txs_per_thread = 200; // Total 10.000 transações concorrentes

    let mut handles = Vec::new();

    for t_id in 0..num_threads {
        let acc_clone = Arc::clone(&account);

        let handle = thread::spawn(move || {
            for i in 0..txs_per_thread {
                let amount = if (t_id + i) % 2 == 0 { 500 } else { -300 }; // Alternar depósito e débito
                let entry_type = if amount > 0 {
                    EntryType::PotWin
                } else {
                    EntryType::TableBuyIn
                };

                let res = acc_clone.record_transaction(
                    amount,
                    entry_type,
                    Some(format!("TX-T{}-{}", t_id, i)),
                );

                assert!(
                    res.is_ok(),
                    "Falha ao gravar transação no ledger concorrente"
                );
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Thread de estresse do Ledger falhou");
    }

    // Cálculo do saldo esperado:
    // 50 threads * (100 depósitos de 500 + 100 débitos de -300) = 50 * (50000 - 30000) = 50 * 20000 = 1.000.000
    let expected_delta = (num_threads as i64) * (txs_per_thread as i64 / 2) * (500 - 300);
    let expected_final_balance = initial_balance + expected_delta;

    let final_balance = account.get_balance_cents().unwrap();
    assert_eq!(
        final_balance,
        expected_final_balance,
        "VIOLAÇÃO CRÍTICA: Saldo final do Ledger (R$ {}) não bate com o esperado (R$ {})",
        final_balance as f64 / 100.0,
        expected_final_balance as f64 / 100.0
    );

    // Validação estrita da cadeia de auditoria SHA-256 de todas as 10.000 transações
    let integrity = account.verify_integrity().unwrap();
    assert!(
        integrity,
        "VIOLAÇÃO CRÍTICA: A cadeia de hashes do Ledger foi corrompida sob concorrência!"
    );
}
