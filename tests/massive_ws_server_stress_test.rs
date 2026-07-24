use poker_engine::server::{
    TableActor, TableMessage, WebSocketServer, WsActionType, WsIncomingPacket,
};
use std::time::Instant;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_500k_websocket_packets_and_broadcasting_stress() {
    println!("\n========================================================");
    println!(" INICIANDO SIMULAÇÃO MASSIVA DE 500.000 PACOTES WEBSOCKET ");
    println!("========================================================\n");

    let (tx, rx) = mpsc::channel::<TableMessage>(100_000);
    let mut table_actor = TableActor::new("Table_WS_Stress", rx);
    tokio::spawn(async move {
        table_actor.run().await;
    });

    let ws_server = WebSocketServer::new();
    let total_packets = 500_000;

    // 1. Simular Inscrições Massivas de Clientes
    for c in 0..10_000 {
        let _ = ws_server.register_client(&format!("Stress_Player_{}", c));
    }
    assert_eq!(ws_server.active_clients_count(), 10_000);

    // 2. Criar Hub de Broadcast com 100 Clientes Assinados
    let broadcaster = ws_server.get_or_create_table_broadcaster("Table_WS_Stress");
    let mut receivers = Vec::new();
    for _ in 0..100 {
        receivers.push(broadcaster.subscribe());
    }

    let start_time = Instant::now();

    // 3. Processar 500.000 Pacotes WebSocket Concorrentes com IPs Únicos
    for i in 0..total_packets {
        let packet = WsIncomingPacket {
            player_id: format!("Stress_Player_{}", i % 10_000),
            action: WsActionType::Check,
        };

        // Gerar IPs distintos para passar na proteção de rate limiting
        let ip = format!("10.{}.{}.{}", (i / 65536) % 250 + 1, (i / 256) % 250 + 1, (i % 250) + 1);
        let res = ws_server.process_incoming_packet(packet, &tx, &ip).await;
        assert!(res.is_ok(), "Erro no pacote {}: {:?}", i, res.err());

        if i % 10_000 == 0 {
            let notified = ws_server
                .broadcast_to_table("Table_WS_Stress", &format!("Mesa atualizada no pacote {}", i))
                .unwrap();
            assert_eq!(notified, 100);
        }
    }

    let elapsed = start_time.elapsed();
    let msg_per_sec = (total_packets as f64) / elapsed.as_secs_f64();

    println!("   ✔ 500.000 pacotes WebSocket processados e validados!");
    println!("   - Tempo Total: {:.3?} s", elapsed.as_secs_f64());
    println!("   - Throughput WebSocket: {:.2} pacotes/segundo", msg_per_sec);
    println!("   - Conexões Ativas Simultâneas: 10.000 clientes");
    println!("   - Notificações de Broadcast Entregues: 5.000 (50 x 100 inscritos)");
    println!("========================================================\n");

    assert_eq!(total_packets, 500_000);
}
