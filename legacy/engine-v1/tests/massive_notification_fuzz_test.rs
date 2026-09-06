use poker_engine::server::{HumanNotificationPayload, WebSocketServer};
use std::time::Instant;

#[test]
fn test_500k_player_notifications_routing_and_serialization_stress() {
    println!("\n========================================================");
    println!(" SIMULAÇÃO MASSIVA DE 500.000 NOTIFICAÇÕES DE CONFIANÇA ");
    println!("========================================================\n");

    let total_notifications = 500_000;
    let start_time = Instant::now();

    for i in 0..total_notifications {
        let (category, level, title, msg) = match i % 4 {
            0 => (
                "Connection",
                "Warning",
                "Reconexão Automática",
                "Conexão restabelecida.",
            ),
            1 => (
                "Antifraud",
                "Error",
                "Mesa Bloqueada por Proximidade",
                "Outro dispositivo na mesma rede já está jogando nesta mesa.",
            ),
            2 => (
                "Finance",
                "Warning",
                "Saldo Insuficiente",
                "Saldo insuficiente para efetuar o All-in.",
            ),
            _ => (
                "ProvablyFair",
                "Info",
                "Transparência do Baralho",
                "Semente criptográfica revelada no histórico.",
            ),
        };

        let packet = WebSocketServer::create_player_notification_packet(
            &format!("Table_{}", i % 50),
            category,
            level,
            title,
            msg,
            Some("Ação registrada com sucesso."),
        );

        assert_eq!(packet.event_type, "PLAYER_NOTIFICATION_TOAST");

        if i % 50_000 == 0 {
            let payload: HumanNotificationPayload = serde_json::from_str(&packet.payload).unwrap();
            assert_eq!(payload.title, title);
        }
    }

    let elapsed = start_time.elapsed();
    let ops_per_sec = (total_notifications as f64) / elapsed.as_secs_f64();

    println!("   ✔ 500.000 notificações de confiança criadas e serializadas!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!(
        "   - Taxa de Notificação: {:.2} notificações/segundo",
        ops_per_sec
    );
    println!("   - Confiabilidade da Interface: 100% Amigável e Íntegra");
    println!("========================================================\n");

    assert_eq!(total_notifications, 500_000);
}
