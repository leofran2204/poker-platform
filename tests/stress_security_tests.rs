use poker_engine::antifraud::{CollusionDetector, PlayerSession};
use poker_engine::security::RateLimiter;
use std::sync::Arc;
use std::thread;

#[test]
fn test_massive_rate_limiter_stress_100k_checks() {
    let limiter = Arc::new(RateLimiter::new(100.0, 50.0)); // 100 max, 50/sec refill
    let num_threads = 20;
    let checks_per_thread = 5_000; // 100.000 requisições

    let mut handles = Vec::new();

    for t_id in 0..num_threads {
        let limiter_clone = Arc::clone(&limiter);
        let handle = thread::spawn(move || {
            let key = format!("10.0.0.{}", t_id % 5); // 5 IPs concorrentes disputando limite
            let mut accepted = 0;
            let mut rejected = 0;

            for _ in 0..checks_per_thread {
                if limiter_clone.check_rate_limit(&key).is_ok() {
                    accepted += 1;
                } else {
                    rejected += 1;
                }
            }

            (accepted, rejected)
        });
        handles.push(handle);
    }

    let mut total_accepted = 0;
    let mut total_rejected = 0;

    for h in handles {
        let (acc, rej) = h.join().unwrap();
        total_accepted += acc;
        total_rejected += rej;
    }

    assert!(total_accepted > 0);
    assert!(total_rejected > 0);
    assert_eq!(total_accepted + total_rejected, 100_000);
}

#[test]
fn test_massive_antifraud_subnet_stress_10k_sessions() {
    let mut sessions = Vec::new();

    // Gerar 10.000 sessões de jogadores sintéticos em sub-redes distintas
    for i in 0..10_000 {
        let ip = format!("{}.{}.{}.{}", (i / 65536) % 250 + 1, (i / 256) % 250 + 1, i % 250 + 1, (i * 7) % 250 + 1);
        sessions.push(PlayerSession {
            user_id: format!("Player_{}", i),
            ip_address: ip,
        });
    }

    // Processar validações de mesa em blocos de 6 jogadores
    for chunk in sessions.chunks(6) {
        let res = CollusionDetector::validate_table_seating(chunk);
        assert!(res.is_ok(), "Falha inesperada de colusão para IPs distintos");
    }
}
