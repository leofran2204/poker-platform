use poker_engine::antifraud::{CollusionDetector, PlayerBehaviorStats};
use poker_engine::security::RateLimiter;
use std::sync::Arc;
use std::thread;

#[test]
fn test_stress_rate_limiter_250k_requests() {
    let limiter = Arc::new(RateLimiter::new(200.0, 100.0));
    let num_threads = 25;
    let checks_per_thread = 10_000; // Total 250.000 requisições

    let mut handles = Vec::new();

    for t_id in 0..num_threads {
        let limiter_clone = Arc::clone(&limiter);
        let handle = thread::spawn(move || {
            let key = format!("10.0.1.{}", t_id % 5);
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

    let mut total_acc = 0;
    let mut total_rej = 0;

    for h in handles {
        let (acc, rej) = h.join().unwrap();
        total_acc += acc;
        total_rej += rej;
    }

    assert_eq!(total_acc + total_rej, 250_000);
}

#[test]
fn test_antifraud_vpip_pfr_extreme_boundary_checks() {
    // Caso 1: Jogador normal (25% VPIP, 15% PFR) -> Ok
    let normal = PlayerBehaviorStats {
        user_id: "NormalPlayer".into(),
        hands_played: 200,
        hands_vpip: 50,
        hands_pfr: 30,
    };
    assert!(CollusionDetector::detect_anomalies(&normal).is_none());

    // Caso 2: PFR > VPIP (30% PFR vs 10% VPIP) -> Anomalia Bot
    let bot_pfr = PlayerBehaviorStats {
        user_id: "BotPfr".into(),
        hands_played: 200,
        hands_vpip: 20,
        hands_pfr: 60,
    };
    assert!(CollusionDetector::detect_anomalies(&bot_pfr).is_some());

    // Caso 3: VPIP Extremo > 98% -> Suspeita de Bot/Chip Dumping
    let bot_maniac = PlayerBehaviorStats {
        user_id: "BotManiac".into(),
        hands_played: 100,
        hands_vpip: 99,
        hands_pfr: 50,
    };
    assert!(CollusionDetector::detect_anomalies(&bot_maniac).is_some());
}
