use poker_engine::server::{HumanNotificationPayload, WebSocketServer};

#[test]
fn test_create_player_notification_packet() {
    let packet = WebSocketServer::create_player_notification_packet(
        "Table_7",
        "Connection",
        "Warning",
        "Oscilação de Rede",
        "Sua conexão caiu por 2 segundos. Reconectado automaticamente com sucesso!",
        Some("Sua mão atual permaneceu protegida."),
    );

    assert_eq!(packet.event_type, "PLAYER_NOTIFICATION_TOAST");
    assert_eq!(packet.table_id, "Table_7");

    let payload: HumanNotificationPayload = serde_json::from_str(&packet.payload).unwrap();
    assert_eq!(payload.category, "Connection");
    assert_eq!(payload.level, "Warning");
    assert_eq!(payload.title, "Oscilação de Rede");
    assert!(payload.action_advice.unwrap().contains("protegida"));
}
