use poker_engine::ledger::{EntryType, LedgerAccount, LedgerError};
use std::sync::Arc;
use std::thread;

#[test]
fn test_stress_ledger_100_concurrent_threads_20k_txs() {
    let initial_balance = 50_000_000i64; // R$ 500.000,00 em centavos
    let account = Arc::new(LedgerAccount::new("StressUser_100T", initial_balance));

    let num_threads = 100;
    let txs_per_thread = 200;

    let mut handles = Vec::new();

    for t_id in 0..num_threads {
        let acc = Arc::clone(&account);
        let handle = thread::spawn(move || {
            for i in 0..txs_per_thread {
                let amount = if (t_id + i) % 2 == 0 { 1000 } else { -600 };
                let entry_type = if amount > 0 { EntryType::Deposit } else { EntryType::TableBuyIn };
                
                let res = acc.record_transaction(amount, entry_type, Some(format!("TX-100T-{}-{}", t_id, i)));
                assert!(res.is_ok(), "Falha em transação no Ledger com 100 threads");
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let expected_delta = (num_threads as i64) * (txs_per_thread as i64 / 2) * (1000 - 600);
    let expected_final = initial_balance + expected_delta;
    assert_eq!(account.get_balance_cents().unwrap(), expected_final);
    assert!(account.verify_integrity().unwrap(), "Integridade de SHA-256 no Ledger violada!");
}

#[test]
fn test_ledger_insufficient_funds_rejection_under_concurrency() {
    let account = Arc::new(LedgerAccount::new("BrokeUser", 1000)); // Saldo inicial: R$ 10,00 (1000 centavos)
    let num_threads = 10;
    let mut handles = Vec::new();

    for _ in 0..num_threads {
        let acc = Arc::clone(&account);
        let handle = thread::spawn(move || {
            // Tentar sacar R$ 5,00 (500 centavos)
            acc.record_transaction(-500, EntryType::Withdrawal, None)
        });
        handles.push(handle);
    }

    let mut successes = 0;
    let mut failures = 0;

    for h in handles {
        match h.join().unwrap() {
            Ok(_) => successes += 1,
            Err(LedgerError::InsufficientFunds) => failures += 1,
            Err(e) => panic!("Erro inesperado: {:?}", e),
        }
    }

    // Com R$ 10,00 de saldo inicial, exatamente 2 saques de R$ 5,00 devem ter sucesso.
    // Os outros 8 saques DEVEM falhar por InsufficientFunds.
    assert_eq!(successes, 2, "Apenas 2 saques poderiam ser permitidos com saldo R$ 10,00");
    assert_eq!(failures, 8, "8 saques deveriam ter sido rejeitados por saldo insuficiente");
    assert_eq!(account.get_balance_cents().unwrap(), 0, "Saldo final DEVE ser exatamente 0");
    assert!(account.verify_integrity().unwrap());
}
