use poker_engine::server::{
    TableActor, TableMessage, WebSocketServer, WsActionType, WsIncomingPacket,
};
use tokio::sync::mpsc;

#[test]
fn test_ws_client_registration_and_disconnect() {
    let ws_server = WebSocketServer::new();
    assert_eq!(ws_server.active_clients_count(), 0);

    let (_tx1, _rx1) = ws_server.register_client("Player_1");
    let (_tx2, _rx2) = ws_server.register_client("Player_2");
    assert_eq!(ws_server.active_clients_count(), 2);

    ws_server.disconnect_client("Player_1");
    assert_eq!(ws_server.active_clients_count(), 1);
}

#[tokio::test]
async fn test_ws_incoming_packet_processing_and_actor_routing() {
    let (tx, rx) = mpsc::channel::<TableMessage>(100);
    let mut table_actor = TableActor::new("Table_WS_1", rx);
    tokio::spawn(async move {
        table_actor.run().await;
    });

    let ws_server = WebSocketServer::new();

    // 1. Processar Pacote de Entrada JoinTable via WebSocket
    let packet_join = WsIncomingPacket {
        player_id: "Player_Alice".into(),
        action: WsActionType::JoinTable {
            table_id: "Table_WS_1".into(),
            ip_address: "203.0.113.10".into(),
        },
    };

    let response = ws_server
        .process_incoming_packet(packet_join, &tx, "203.0.113.10")
        .await;
    assert!(response.is_ok());
    let res_packet = response.unwrap();
    assert_eq!(res_packet.event_type, "JOIN_SUCCESS");
}

#[test]
fn test_ws_table_broadcasting_to_multiple_subscribers() {
    let ws_server = WebSocketServer::new();
    let broadcaster = ws_server.get_or_create_table_broadcaster("Table_100");

    let mut rx1 = broadcaster.subscribe();
    let mut rx2 = broadcaster.subscribe();

    let receivers_notified = ws_server
        .broadcast_to_table("Table_100", "Flop virou: Ace-Spades, King-Hearts, Ten-Clubs")
        .unwrap();

    assert_eq!(receivers_notified, 2);

    let msg1 = rx1.try_recv().unwrap();
    let msg2 = rx2.try_recv().unwrap();

    assert_eq!(msg1.event_type, "TABLE_STATE_UPDATE");
    assert_eq!(msg2.payload, "Flop virou: Ace-Spades, King-Hearts, Ten-Clubs");
}
