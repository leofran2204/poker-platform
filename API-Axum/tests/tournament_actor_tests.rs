// tournament_actor_tests.rs — Ator MTT fim a fim contra Postgres scratch.
//
// Run with (schema migrado 001..048 no scratch):
//   DATABASE_URL=postgres://user:password@localhost:5544/poker_db \
//     cargo test --test tournament_actor_tests -- --ignored --nocapture
//
// Nunca aponte para o banco da demo/VPS: o teste cria e apaga suas linhas.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, RwLock};

use poker_api::game_actor::PlayerCommand;
use poker_api::state::AppState;
use poker_api::tournament_actor::TournamentActor;
use poker_api::tournament_store::TournamentStore;

fn scratch_pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL do scratch ausente");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&url)
        .expect("pool scratch inválida")
}

fn test_state(db: sqlx::PgPool) -> AppState {
    AppState {
        db,
        auth: Arc::new(RwLock::new(poker_engine::auth::AuthManager::new(
            "test-secret-key-for-tests-32-chars",
        ))),
        tournaments: Arc::new(RwLock::new(HashMap::new())),
        active_tables: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "test-secret-key-for-tests-32-chars".to_string(),
        rate_limiter: poker_api::middleware::rate_limit::RateLimiter::default(),
        redis: None,
        ws_tickets: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        require_email_verification: false,
        require_invite: false,
        presence: poker_api::presence::PresenceTracker::new(),
        bots: poker_api::bots::BotFleet::for_test(),
    }
}

#[tokio::test]
#[ignore = "Requires scratch PostgreSQL (migrated) via DATABASE_URL"]
async fn mtt_actor_completes_hand_with_signed_settlement() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .try_init();
    let db = scratch_pool();
    // Limpa restos de rodadas anteriores (asserts podem pular o cleanup final).
    sqlx::query("DELETE FROM hand_participants WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'tact%')")
        .execute(&db).await.unwrap();
    sqlx::query("DELETE FROM hand_history WHERE table_id IN (SELECT id FROM tables WHERE name = 'MTT teste')")
        .execute(&db).await.unwrap();
    sqlx::query("DELETE FROM table_hand_recovery_guards WHERE table_id IN (SELECT id FROM tables WHERE name = 'MTT teste')")
        .execute(&db).await.unwrap();
    sqlx::query("DELETE FROM tournament_seats WHERE player_id IN (SELECT id::text FROM users WHERE username LIKE 'tact%')")
        .execute(&db).await.unwrap();
    sqlx::query("DELETE FROM tables WHERE name = 'MTT teste'")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tournaments WHERE name = 'MTT teste'")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE username LIKE 'tact%'")
        .execute(&db)
        .await
        .unwrap();
    let tid = uuid::Uuid::new_v4();
    let table_id = uuid::Uuid::new_v4();
    let u1 = uuid::Uuid::new_v4();
    let u2 = uuid::Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, username, email, password_hash, role, status) VALUES ($1, 'tact1', 'tact1@t.local', 'x', 'player', 'active'), ($2, 'tact2', 'tact2@t.local', 'x', 'player', 'active') ON CONFLICT DO NOTHING")
        .bind(u1).bind(u2).execute(&db).await.unwrap();
    sqlx::query("INSERT INTO tables (id, name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, max_players, visibility, status, poker_variant, money_mode) VALUES ($1, 'MTT teste', 'tournament', 25, 50, 5000, 5000, 9, 'private', 'OPEN', 'holdem', 'play') ON CONFLICT DO NOTHING")
        .bind(table_id).execute(&db).await.unwrap();
    sqlx::query("INSERT INTO tournaments (id, name, buy_in, starting_stack, max_players) VALUES ($1, 'MTT teste', 0, 5000, 27) ON CONFLICT DO NOTHING")
        .bind(tid).execute(&db).await.unwrap();
    for (uid, name, seat) in [(u1, "tact1", 0i16), (u2, "tact2", 1i16)] {
        sqlx::query("INSERT INTO tournament_seats (tournament_id, table_id, seat, player_id, player_name, stack) VALUES ($1, $2, $3, $4, $5, 5000) ON CONFLICT DO NOTHING")
            .bind(tid).bind(table_id).bind(seat).bind(uid.to_string()).bind(name)
            .execute(&db).await.unwrap();
    }

    let state = test_state(db.clone());
    let tkey = tid.to_string();
    {
        let mut stores = state.tournaments.write().await;
        let mut store = TournamentStore::with_mode_and_variant(
            tkey.clone(),
            poker_engine::tournament_engine::TournamentConfig {
                blind_levels: vec![poker_engine::tournament_engine::BlindLevel {
                    level: 1,
                    small_blind: 25,
                    big_blind: 50,
                    ante: 50,
                    duration_minutes: 60,
                }],
                ..Default::default()
            },
            "play".into(),
            "holdem".into(),
        );
        store.state.status = poker_engine::tournament_engine::TournamentStatus::Running;
        store.state.current_level = 1;
        store.state.players_remaining = 2;
        store.live_table_ids = vec![table_id.to_string()];
        stores.insert(tkey.clone(), store);
    }

    let (tx_cmd, rx_cmd) = mpsc::channel(100);
    let (tx_broadcast, _) = broadcast::channel(100);
    let actor = TournamentActor {
        table_id: table_id.to_string(),
        tournament_id: tkey.clone(),
        table_index: 0,
        name: "MTT teste mesa 1".to_string(),
        players: Vec::new(),
        game_loop: None,
        rx: rx_cmd,
        tx_broadcast,
        next_hand_at: None,
        dealer_index: 0,
        dealer_seat: None,
        antifraud: poker_engine::antifraud::AntiFraudSuite::new(),
        last_turn_start: Some(tokio::time::Instant::now()),
        turn_timeout: std::time::Duration::from_millis(200),
        db: db.clone(),
        audit_secret: "test-secret-key-for-tests-32-chars".to_string(),
        tournaments: state.tournaments.clone(),
        active_tables: state.active_tables.clone(),
        persistence_halted: false,
    };
    tokio::spawn(actor.run());

    for (uid, name, seat) in [(u1, "tact1", 0usize), (u2, "tact2", 1usize)] {
        let (tx_resp, rx_resp) = tokio::sync::oneshot::channel();
        tx_cmd
            .send(PlayerCommand::Sit {
                player_id: uid.to_string(),
                username: name.to_string(),
                seat: Some(seat),
                chips: 5000,
                respond_to: tx_resp,
            })
            .await
            .unwrap();
        rx_resp.await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    let (hands, signed, rake): (i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE settlement_signature <> ''), COALESCE(SUM(rake_collected),0)::BIGINT \
         FROM hand_history WHERE table_id = $1 AND game_type = 'tournament'",
    )
    .bind(table_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert!(hands >= 1, "ator MTT não completou nenhuma mão");
    assert_eq!(signed, hands, "toda mão MTT precisa de settlement assinado");
    assert_eq!(rake, 0, "mão de torneio não tem rake");

    let parts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM hand_participants WHERE user_id = $1")
            .bind(u1)
            .fetch_one(&db)
            .await
            .unwrap();
    assert!(parts >= 1, "participantes do MTT precisam ser registrados");

    let stacks: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(stack),0)::BIGINT FROM tournament_seats WHERE tournament_id = $1",
    )
    .bind(tid)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(stacks, 10000, "fichas de torneio se conservam");

    // Limpeza do scratch.
    sqlx::query("DELETE FROM hand_participants WHERE user_id IN ($1, $2)")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM hand_history WHERE table_id = $1")
        .bind(table_id)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tournament_seats WHERE tournament_id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tables WHERE id = $1")
        .bind(table_id)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tournaments WHERE id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id IN ($1, $2)")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .unwrap();
}
