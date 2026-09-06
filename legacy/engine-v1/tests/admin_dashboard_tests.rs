use poker_engine::admin::AdminDashboard;
use poker_engine::antifraud::PlayerBehaviorStats;
use poker_engine::ledger::{EntryType, LedgerAccount};

#[test]
fn test_ledger_crypto_audit() {
    let admin = AdminDashboard::new();
    let account = LedgerAccount::new("Audit_User_1", 50000);

    let _ = account.record_transaction(10000, EntryType::Deposit, Some("DEP-100".into()));
    let _ = account.record_transaction(-5000, EntryType::TableBuyIn, Some("BUYIN-100".into()));

    let audit = admin.audit_ledger_account(&account);
    assert_eq!(audit.user_id, "Audit_User_1");
    assert_eq!(audit.account_balance_cents, 55000);
    assert_eq!(audit.transaction_count, 2);
    assert!(audit.hash_chain_valid);
}

#[test]
fn test_player_suspension_and_un_suspension() {
    let admin = AdminDashboard::new();
    assert!(!admin.is_player_suspended("Bot_Player_99"));

    let msg = admin.suspend_player("Bot_Player_99", "Uso de RTA detectado");
    assert!(msg.contains("suspenso com sucesso"));
    assert!(admin.is_player_suspended("Bot_Player_99"));

    assert!(admin.unsuspend_player("Bot_Player_99"));
    assert!(!admin.is_player_suspended("Bot_Player_99"));
}

#[test]
fn test_antifraud_behavioral_risk_report() {
    let admin = AdminDashboard::new();
    let mut stats = PlayerBehaviorStats::new("Cheater_1");

    for _ in 0..25 {
        stats.record_hand(true, true); // 100% VPIP e 100% PFR
    }

    let report = admin.analyze_player_risk(&stats);
    assert!(report.is_some());
    let rep = report.unwrap();
    assert_eq!(rep.user_id, "Cheater_1");
    assert!(rep.vpip > 85.0);
    assert!(rep.reason.contains("Anomalia Extrema"));
}

#[test]
fn test_system_metrics_tracking() {
    let admin = AdminDashboard::new();
    admin.update_metrics(150, 20, 500000); // 150 conns, 20 tables, R$ 5.000 volume

    let m = admin.get_metrics();
    assert_eq!(m.active_connections, 150);
    assert_eq!(m.active_tables, 20);
    assert_eq!(m.total_volume_cents, 500000);
    assert_eq!(m.requests_processed, 1);
}
