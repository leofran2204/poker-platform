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

struct MttTable {
    tid: uuid::Uuid,
    table_id: uuid::Uuid,
    u1: uuid::Uuid,
    u2: uuid::Uuid,
    total_chips: i64,
}

/// Sobe mesa de torneio no scratch com 2 jogadores e devolve o ator rodando.
async fn setup_mtt_table(
    db: &sqlx::PgPool,
    tag: &str,
    stack_each: i64,
) -> (AppState, tokio::sync::mpsc::Sender<PlayerCommand>, MttTable) {
    let tid = uuid::Uuid::new_v4();
    let table_id = uuid::Uuid::new_v4();
    let u1 = uuid::Uuid::new_v4();
    let u2 = uuid::Uuid::new_v4();
    let table_name = format!("MTT teste {tag}");
    let (n1, n2) = (format!("tact-{tag}-1"), format!("tact-{tag}-2"));

    sqlx::query("INSERT INTO users (id, username, email, password_hash, role, status) VALUES ($1, $2, $2, 'x', 'player', 'active'), ($3, $4, $4, 'x', 'player', 'active') ON CONFLICT DO NOTHING")
        .bind(u1).bind(&n1).bind(u2).bind(&n2).execute(db).await.unwrap();
    sqlx::query("INSERT INTO tables (id, name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, max_players, visibility, status, poker_variant, money_mode) VALUES ($1, $2, 'tournament', 25, 50, 5000, 5000, 9, 'private', 'OPEN', 'holdem', 'play') ON CONFLICT DO NOTHING")
        .bind(table_id).bind(&table_name).execute(db).await.unwrap();
    sqlx::query("INSERT INTO tournaments (id, name, buy_in, starting_stack, max_players) VALUES ($1, $2, 0, 5000, 27) ON CONFLICT DO NOTHING")
        .bind(tid).bind(&table_name).execute(db).await.unwrap();
    for (uid, name, seat) in [(&u1, &n1, 0i16), (&u2, &n2, 1i16)] {
        sqlx::query("INSERT INTO tournament_seats (tournament_id, table_id, seat, player_id, player_name, stack) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING")
            .bind(tid).bind(table_id).bind(seat).bind(uid.to_string()).bind(name).bind(stack_each)
            .execute(db).await.unwrap();
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
        name: format!("{table_name} mesa 1"),
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
        // Estes testes avançam as mãos pelo tempo esgotado: sem reserva.
        time_banks: poker_api::time_bank::TimeBanks::disabled(),
        db: db.clone(),
        audit_secret: "test-secret-key-for-tests-32-chars".to_string(),
        tournaments: state.tournaments.clone(),
        active_tables: state.active_tables.clone(),
        persistence_halted: false,
        idle_ticks: 0,
        last_winners: Vec::new(),
    };
    // Como em produção: a mesa viva fica no mapa de atores (WS e rotas a acham).
    state.active_tables.write().await.insert(
        table_id.to_string(),
        poker_api::state::TableActorHandle {
            tx_cmd: tx_cmd.clone(),
            tx_broadcast: actor.tx_broadcast.clone(),
        },
    );
    tokio::spawn(actor.run());

    for (uid, name, seat) in [(u1, n1, 0usize), (u2, n2, 1usize)] {
        let (tx_resp, rx_resp) = tokio::sync::oneshot::channel();
        tx_cmd
            .send(PlayerCommand::Sit {
                player_id: uid.to_string(),
                username: name,
                seat: Some(seat),
                chips: stack_each as u64,
                respond_to: tx_resp,
            })
            .await
            .unwrap();
        rx_resp.await.unwrap();
    }

    (
        state,
        tx_cmd,
        MttTable {
            tid,
            table_id,
            u1,
            u2,
            total_chips: 2 * stack_each,
        },
    )
}

async fn cleanup_mtt_table(db: &sqlx::PgPool, t: &MttTable) {
    sqlx::query("DELETE FROM hand_participants WHERE user_id IN ($1, $2)")
        .bind(t.u1)
        .bind(t.u2)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM hand_history WHERE table_id = $1")
        .bind(t.table_id)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM table_hand_recovery_guards WHERE table_id = $1")
        .bind(t.table_id)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tournament_seats WHERE tournament_id = $1")
        .bind(t.tid)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tables WHERE id = $1")
        .bind(t.table_id)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tournaments WHERE id = $1")
        .bind(t.tid)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id IN ($1, $2)")
        .bind(t.u1)
        .bind(t.u2)
        .execute(db)
        .await
        .unwrap();
}

/// Os testes deste arquivo dividem o banco e apagam as sobras uns dos outros
/// (`wipe_tact_leftovers`): rodam um de cada vez, mesmo com threads paralelas
/// (a campanha `full-validation` roda o arquivo inteiro em paralelo).
fn db_serial() -> &'static tokio::sync::Mutex<()> {
    static SERIAL: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    SERIAL.get_or_init(|| tokio::sync::Mutex::new(()))
}

async fn wipe_tact_leftovers(db: &sqlx::PgPool) {
    sqlx::query("DELETE FROM hand_participants WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'tact%')")
        .execute(db).await.unwrap();
    sqlx::query("DELETE FROM hand_history WHERE table_id IN (SELECT id FROM tables WHERE name LIKE 'MTT teste%')")
        .execute(db).await.unwrap();
    sqlx::query("DELETE FROM table_hand_recovery_guards WHERE table_id IN (SELECT id FROM tables WHERE name LIKE 'MTT teste%')")
        .execute(db).await.unwrap();
    sqlx::query("DELETE FROM tournament_seats WHERE player_id IN (SELECT id::text FROM users WHERE username LIKE 'tact%')")
        .execute(db).await.unwrap();
    sqlx::query("DELETE FROM tables WHERE name LIKE 'MTT teste%'")
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tournaments WHERE name LIKE 'MTT teste%'")
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE username LIKE 'tact%'")
        .execute(db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "Requires scratch PostgreSQL (migrated) via DATABASE_URL"]
async fn mtt_actor_completes_hand_with_signed_settlement() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .try_init();
    let db = scratch_pool();
    let _serial = db_serial().lock().await;
    wipe_tact_leftovers(&db).await;
    let (_state, _tx, t) = setup_mtt_table(&db, "base", 5000).await;

    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    let (hands, signed, rake): (i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE settlement_signature <> ''), COALESCE(SUM(rake_collected),0)::BIGINT \
         FROM hand_history WHERE table_id = $1 AND game_type = 'tournament'",
    )
    .bind(t.table_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert!(hands >= 1, "ator MTT não completou nenhuma mão");
    assert_eq!(signed, hands, "toda mão MTT precisa de settlement assinado");
    assert_eq!(rake, 0, "mão de torneio não tem rake");

    let parts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM hand_participants WHERE user_id = $1")
            .bind(t.u1)
            .fetch_one(&db)
            .await
            .unwrap();
    assert!(parts >= 1, "participantes do MTT precisam ser registrados");

    let stacks: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(stack),0)::BIGINT FROM tournament_seats WHERE tournament_id = $1",
    )
    .bind(t.tid)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(stacks, t.total_chips, "fichas de torneio se conservam");

    cleanup_mtt_table(&db, &t).await;
}

#[tokio::test]
#[ignore = "Requires scratch PostgreSQL (migrated) via DATABASE_URL"]
async fn mtt_actor_runs_out_allin_stall() {
    let db = scratch_pool();
    let _serial = db_serial().lock().await;
    wipe_tact_leftovers(&db).await;
    // Stacks menores que os blinds: todos all-in na largada — sem run-out,
    // a mão travaria para sempre (fold inválido p/ quem não pode agir).
    let (_state, _tx, t) = setup_mtt_table(&db, "allin", 30).await;

    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    let (hands, signed): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE settlement_signature <> '') \
         FROM hand_history WHERE table_id = $1 AND game_type = 'tournament'",
    )
    .bind(t.table_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert!(hands >= 1, "mão all-in travada não correu até o fim");
    assert_eq!(
        signed, hands,
        "showdown all-in precisa de settlement assinado"
    );

    let stacks: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(stack),0)::BIGINT FROM tournament_seats WHERE tournament_id = $1",
    )
    .bind(t.tid)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(
        stacks, t.total_chips,
        "fichas de torneio se conservam no all-in"
    );

    cleanup_mtt_table(&db, &t).await;
}

/// Rebuy e add-on pelas rotas do jogador: carteira debitada sem taxa, fichas do
/// add-on entregues no começo da mão seguinte, eliminado volta a sentar (no
/// mesmo número de cadeira de uma linha eliminada) e as fichas se conservam.
#[tokio::test]
#[ignore = "Requires scratch PostgreSQL (migrated) via DATABASE_URL"]
async fn mtt_rebuy_and_addon_deliver_chips_and_charge_wallet() {
    use axum::extract::{Path, State};
    use axum::Json;
    use poker_api::handlers::tournament::{
        addon_player, my_registration, rebuy_player, ChipPurchaseBody,
    };
    use poker_api::middleware::auth::{AuthUser, RequireAuth};
    use poker_engine::tournament_engine::PlayerTournamentEntry;

    let db = scratch_pool();
    let _serial = db_serial().lock().await;
    wipe_tact_leftovers(&db).await;
    let (state, _tx, t) = setup_mtt_table(&db, "rebuy", 5000).await;
    let u3 = uuid::Uuid::new_v4();
    let n3 = "tact-rebuy-3".to_string();
    let tkey = t.tid.to_string();

    sqlx::query("INSERT INTO users (id, username, email, password_hash, role, status) VALUES ($1, $2, $2, 'x', 'player', 'active')")
        .bind(u3).bind(&n3).execute(&db).await.unwrap();
    // Renovação diária do Play Money antes, para o débito ser a única mudança.
    for uid in [t.u1, t.u2, u3] {
        poker_api::wallet::ensure_pm_daily_reset_pool(&db, &uid.to_string())
            .await
            .unwrap();
    }
    sqlx::query("UPDATE users SET balance_pm_mtt = 100000, ledger_reason = 'test_fixture' WHERE id IN ($1, $2, $3)")
        .bind(t.u1)
        .bind(t.u2)
        .bind(u3)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("UPDATE tournaments SET status = 'running', allow_rebuy = TRUE, allow_addon = TRUE WHERE id = $1")
        .bind(t.tid).execute(&db).await.unwrap();
    for (uid, name) in [
        (t.u1, "tact-rebuy-1"),
        (t.u2, "tact-rebuy-2"),
        (u3, "tact-rebuy-3"),
    ] {
        sqlx::query("INSERT INTO tournament_players (tournament_id, player_id, player_name, stack) VALUES ($1, $2, $3, 5000)")
            .bind(t.tid).bind(uid.to_string()).bind(name).execute(&db).await.unwrap();
    }
    // u3 já foi eliminado no assento 2: a linha antiga não pode travar a cadeira.
    sqlx::query("INSERT INTO tournament_seats (tournament_id, table_id, seat, player_id, player_name, stack, status) VALUES ($1, $2, 2, $3, $4, 0, 'ELIMINATED')")
        .bind(t.tid).bind(t.table_id).bind(u3.to_string()).bind(&n3).execute(&db).await.unwrap();
    {
        let mut stores = state.tournaments.write().await;
        let store = stores.get_mut(&tkey).unwrap();
        let cfg = &mut store.state.config;
        cfg.allow_rebuy = true;
        cfg.allow_addon = true;
        cfg.rebuy_max_level = 5;
        cfg.rebuy_cost = 1_000;
        cfg.rebuy_chips = 5_000;
        store.addon = poker_api::tournament_store::AddonTerms {
            cost: 2_000,
            chips: 3_000,
            max_level: 5,
        };
        for (uid, name, eliminated) in [
            (t.u1, "tact-rebuy-1", false),
            (t.u2, "tact-rebuy-2", false),
            (u3, "tact-rebuy-3", true),
        ] {
            store.state.players.insert(
                uid.to_string(),
                PlayerTournamentEntry {
                    player_id: uid.to_string(),
                    player_name: name.to_string(),
                    stack: if eliminated { 0 } else { 5_000 },
                    table_id: (!eliminated).then_some(0),
                    seat: None,
                    rebuys: 0,
                    addon_done: false,
                    final_position: None,
                    prize: None,
                    registered_at: 0,
                    eliminated_at: eliminated.then_some(1),
                },
            );
        }
        store.state.players_remaining = 2;
        store.state.eliminated_order = vec![u3.to_string()];
    }
    let auth = |uid: uuid::Uuid, name: &str| {
        RequireAuth(AuthUser {
            user_id: uid.to_string(),
            username: name.to_string(),
            role: "player".to_string(),
        })
    };
    let body = || {
        Json(ChipPurchaseBody {
            tournament_id: tkey.clone(),
        })
    };

    // Add-on de quem está na mesa: fica pendente e entra na próxima mão.
    let addon = addon_player(State(state.clone()), auth(t.u1, "tact-rebuy-1"), body())
        .await
        .expect("add-on aceito")
        .0;
    assert_eq!(addon.delivery, "next_hand");
    assert_eq!((addon.chips, addon.cost_cents), (3_000, 2_000));
    assert!(
        addon_player(State(state.clone()), auth(t.u1, "tact-rebuy-1"), body())
            .await
            .is_err(),
        "add-on é uma compra por jogador"
    );
    assert!(
        rebuy_player(State(state.clone()), auth(t.u2, "tact-rebuy-2"), body())
            .await
            .is_err(),
        "sem limiar de stack, rebuy só para eliminado"
    );

    // Rebuy do eliminado: volta a sentar já, com stack novo.
    let rebuy = rebuy_player(State(state.clone()), auth(u3, "tact-rebuy-3"), body())
        .await
        .expect("rebuy aceito")
        .0;
    assert_eq!(rebuy.delivery, "seated");
    assert_eq!(
        rebuy.table_id.as_deref(),
        Some(t.table_id.to_string().as_str())
    );
    assert_eq!(
        rebuy.prize_pool, 3_000,
        "rebuy e add-on vão inteiros para o prêmio"
    );

    let reg = my_registration(
        State(state.clone()),
        auth(u3, "tact-rebuy-3"),
        Path(tkey.clone()),
    )
    .await
    .unwrap()
    .0;
    assert!(reg.registered && !reg.eliminated);
    assert_eq!(reg.rebuys, 1);
    assert!(reg.can_addon, "quem voltou pode comprar o add-on");

    // Mãos seguem com 6 s de pausa entre elas: espera a entrega do add-on e
    // uma mão completa com quem voltou, conferindo o banco a cada meio segundo.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let (pending_now, u3_hands): (i64, i64) = sqlx::query_as(
            "SELECT (SELECT COALESCE(SUM(pending_chips),0)::BIGINT FROM tournament_seats WHERE tournament_id = $1), \
                    (SELECT COUNT(*) FROM hand_participants WHERE user_id = $2)",
        )
        .bind(t.tid)
        .bind(u3)
        .fetch_one(&db)
        .await
        .unwrap();
        if (pending_now == 0 && u3_hands >= 1) || tokio::time::Instant::now() >= deadline {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let balances: Vec<(i64,)> = sqlx::query_as(
        "SELECT balance_pm_mtt FROM users WHERE id IN ($1, $2, $3) ORDER BY id = $1 DESC, id = $3 DESC",
    )
    .bind(t.u1).bind(t.u2).bind(u3)
    .fetch_all(&db).await.unwrap();
    assert_eq!(
        balances,
        vec![(98_000,), (99_000,), (100_000,)],
        "sem taxa em rebuy e add-on"
    );

    let (pending, active_u3, chips): (i64, i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(pending_chips),0)::BIGINT, \
                COUNT(*) FILTER (WHERE player_id = $2 AND status = 'ACTIVE'), \
                COALESCE(SUM(stack + pending_chips),0)::BIGINT \
         FROM tournament_seats WHERE tournament_id = $1",
    )
    .bind(t.tid)
    .bind(u3.to_string())
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(pending, 0, "fichas do add-on entregues no começo da mão");
    assert_eq!(active_u3, 1);
    assert_eq!(
        chips,
        t.total_chips + 3_000 + 5_000,
        "fichas se conservam com rebuy e add-on"
    );

    let played: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM hand_participants WHERE user_id = $1")
            .bind(u3)
            .fetch_one(&db)
            .await
            .unwrap();
    assert!(played >= 1, "quem fez rebuy volta a receber cartas");

    let totals: (i64, i64, i64) = sqlx::query_as(
        "SELECT total_rebuys, total_addons, prize_pool FROM tournaments WHERE id = $1",
    )
    .bind(t.tid)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(totals, (1_000, 2_000, 3_000));
    let paid: (i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(rebuy_paid),0)::BIGINT, COALESCE(SUM(addon_paid),0)::BIGINT FROM tournament_players WHERE tournament_id = $1",
    )
    .bind(t.tid).fetch_one(&db).await.unwrap();
    assert_eq!(paid, (1_000, 2_000));

    sqlx::query("DELETE FROM hand_participants WHERE user_id = $1")
        .bind(u3)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM audit_logs WHERE action IN ('MTT_REBUY', 'MTT_ADDON') AND metadata->>'tournament_id' = $1")
        .bind(&tkey).execute(&db).await.unwrap();
    cleanup_mtt_table(&db, &t).await;
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(u3)
        .execute(&db)
        .await
        .unwrap();
}

/// A entrada pelo WebSocket numa mesa de torneio lê o assento ativo no banco.
/// A consulta devolvia BIGINT onde a linha espera SMALLINT e falhava sempre
/// (05/09–23/11/2026): o jogador recebia "Não foi possível validar o assento do
/// torneio" e o servidor desistia por ele a cada vez.
#[tokio::test]
#[ignore = "Requires scratch PostgreSQL (migrated) via DATABASE_URL"]
async fn mtt_websocket_admission_reads_the_tournament_seat() {
    use poker_api::handlers::websocket::tournament_seat_admission;

    let db = scratch_pool();
    let _serial = db_serial().lock().await;
    wipe_tact_leftovers(&db).await;
    let (state, tx, t) = setup_mtt_table(&db, "wsadmit", 5000).await;
    let table_id = t.table_id.to_string();

    let seat = tournament_seat_admission(&db, &table_id, &t.u1.to_string())
        .await
        .expect("a consulta do assento do torneio precisa ler a linha")
        .expect("assento ativo do jogador");
    assert_eq!(seat.2, 50, "big blind da mesa");
    assert_eq!((seat.3, seat.4), (0, 0), "torneio sem rake por mão");
    assert_eq!(seat.8, 5000, "fichas do assento");
    assert_eq!(seat.10, "holdem");

    let stranger = tournament_seat_admission(&db, &table_id, &uuid::Uuid::new_v4().to_string())
        .await
        .expect("consulta sem assento");
    assert!(stranger.is_none(), "quem não está sentado não entra");

    // Para o ator antes de apagar a mesa: sem isso ele pode abrir uma mão (e gravar
    // a guarda de recuperação) no meio da limpeza.
    state.active_tables.write().await.remove(&table_id);
    drop(tx);
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    cleanup_mtt_table(&db, &t).await;
}

/// A inscrição grava no torneio a contagem de entradas e o fee de 15% cobrado,
/// como já faziam o cancelamento e os bots da casa. Antes o fee das inscrições
/// pelo site ficava fora de `tournaments.total_fees` e o painel do admin mostrava
/// zero (achado na etapa 2 de 23/11/2026: R$ 101,25 cobrados, R$ 0 registrados).
#[tokio::test]
#[ignore = "Requires scratch PostgreSQL (migrated) via DATABASE_URL"]
async fn mtt_registration_records_entry_fee() {
    use axum::extract::State;
    use axum::Json;
    use poker_api::handlers::tournament::{register_player, RegisterBody};
    use poker_api::middleware::auth::{AuthUser, RequireAuth};

    let db = scratch_pool();
    let _serial = db_serial().lock().await;
    wipe_tact_leftovers(&db).await;
    let tid = uuid::Uuid::new_v4();
    let uid = uuid::Uuid::new_v4();
    let name = "tact-fee-1".to_string();
    sqlx::query("INSERT INTO users (id, username, email, password_hash, role, status) VALUES ($1, $2, $2, 'x', 'player', 'active')")
        .bind(uid).bind(&name).execute(&db).await.unwrap();
    poker_api::wallet::ensure_pm_daily_reset_pool(&db, &uid.to_string())
        .await
        .unwrap();
    sqlx::query("INSERT INTO tournaments (id, name, buy_in, starting_stack, max_players) VALUES ($1, 'MTT teste fee', 2500, 5000, 27)")
        .bind(tid).execute(&db).await.unwrap();

    let state = test_state(db.clone());
    let tkey = tid.to_string();
    {
        let mut stores = state.tournaments.write().await;
        let store = TournamentStore::with_mode_and_variant(
            tkey.clone(),
            poker_engine::tournament_engine::TournamentConfig {
                buy_in: 2500,
                max_players: 27,
                ..Default::default()
            },
            "play".into(),
            "holdem".into(),
        );
        stores.insert(tkey.clone(), store);
    }

    let response = register_player(
        State(state.clone()),
        RequireAuth(AuthUser {
            user_id: uid.to_string(),
            username: name.clone(),
            role: "player".to_string(),
        }),
        poker_api::middleware::rate_limit::ClientIp("127.0.0.1".to_string()),
        Json(RegisterBody {
            tournament_id: tkey.clone(),
            player_id: None,
            player_name: None,
            wallet_mode: Some("play".into()),
        }),
    )
    .await
    .expect("inscrição em Play Money");
    let fee = poker_engine::tournament_engine::entry_fee_cents(2500) as i64;
    assert_eq!(response.0.fee_cents, fee);

    let (entries, fees): (i32, i64) =
        sqlx::query_as("SELECT total_buyins, total_fees FROM tournaments WHERE id = $1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(entries, 1, "total_buyins conta as entradas");
    assert_eq!(
        fees, fee,
        "o fee cobrado na inscrição fica registrado no torneio"
    );

    sqlx::query("DELETE FROM tournament_players WHERE tournament_id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tournaments WHERE id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .unwrap();
}

/// Inscrição tardia (25/11/2026): com o torneio em andamento, quem se inscreve
/// paga e senta na hora numa mesa viva, e o ator o coloca na próxima mão. Antes
/// ninguém dava assento a quem entrava depois do início. A inscrição que falha
/// (aqui, por saldo) devolve o motor ao estado anterior: prêmio, contagem e
/// totais não ficam inflados para a próxima gravação.
#[tokio::test]
#[ignore = "Requires scratch PostgreSQL (migrated) via DATABASE_URL"]
async fn mtt_late_registration_seats_player_and_failed_one_keeps_prize() {
    use axum::extract::State;
    use axum::Json;
    use poker_api::handlers::tournament::{register_player, RegisterBody};
    use poker_api::middleware::auth::{AuthUser, RequireAuth};
    use poker_api::middleware::rate_limit::ClientIp;

    let db = scratch_pool();
    let _serial = db_serial().lock().await;
    wipe_tact_leftovers(&db).await;
    let (state, _tx, t) = setup_mtt_table(&db, "late", 5000).await;
    let tkey = t.tid.to_string();
    let (rich, poor) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
    for (uid, name) in [(rich, "tact-late-3"), (poor, "tact-late-4")] {
        sqlx::query("INSERT INTO users (id, username, email, password_hash, role, status) VALUES ($1, $2, $2, 'x', 'player', 'active')")
            .bind(uid).bind(name).execute(&db).await.unwrap();
        poker_api::wallet::ensure_pm_daily_reset_pool(&db, &uid.to_string())
            .await
            .unwrap();
    }
    sqlx::query("UPDATE users SET balance_pm_mtt = CASE WHEN id = $1 THEN 100000 ELSE 0 END, ledger_reason = 'test_fixture' WHERE id IN ($1, $2)")
        .bind(rich).bind(poor).execute(&db).await.unwrap();
    sqlx::query("UPDATE tournaments SET status = 'running' WHERE id = $1")
        .bind(t.tid)
        .execute(&db)
        .await
        .unwrap();
    let snapshot = |s: &poker_engine::tournament_engine::TournamentState| {
        (
            s.prize_pool,
            s.players_remaining,
            s.players.len(),
            s.total_buyins,
            s.total_fees,
        )
    };
    let before = {
        let mut stores = state.tournaments.write().await;
        let store = stores.get_mut(&tkey).unwrap();
        store.state.config.buy_in = 2_500;
        store.state.config.starting_stack = 5_000;
        snapshot(&store.state)
    };
    let register = |uid: uuid::Uuid, name: &str| {
        register_player(
            State(state.clone()),
            RequireAuth(AuthUser {
                user_id: uid.to_string(),
                username: name.to_string(),
                role: "player".to_string(),
            }),
            ClientIp("127.0.0.1".to_string()),
            Json(RegisterBody {
                tournament_id: tkey.clone(),
                player_id: None,
                player_name: None,
                wallet_mode: Some("play".into()),
            }),
        )
    };

    // Sem saldo: recusada, e o motor volta exatamente ao que era.
    assert!(register(poor, "tact-late-4").await.is_err());
    {
        let stores = state.tournaments.read().await;
        assert_eq!(snapshot(&stores[&tkey].state), before);
    }

    // Com saldo, já em andamento: assento na mesa viva, com o stack inicial.
    let response = register(rich, "tact-late-3")
        .await
        .expect("inscrição tardia");
    assert!(response.0.registered);
    let (table_id, stack): (uuid::Uuid, i64) = sqlx::query_as(
        "SELECT table_id, stack FROM tournament_seats \
         WHERE tournament_id = $1 AND player_id = $2 AND status = 'ACTIVE'",
    )
    .bind(t.tid)
    .bind(rich.to_string())
    .fetch_one(&db)
    .await
    .expect("assento do inscrito tardio");
    assert_eq!((table_id, stack), (t.table_id, 5_000));
    {
        let stores = state.tournaments.read().await;
        let s = &stores[&tkey].state;
        assert_eq!(s.players[&rich.to_string()].table_id, Some(0));
        assert_eq!(s.players_remaining, before.1 + 1);
        assert!(s.prize_pool > before.0, "o buy-in tardio entra no prêmio");
    }

    // O ator recebe o jogador: ele entra numa mão completa.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut hands = 0i64;
    while tokio::time::Instant::now() < deadline {
        hands = sqlx::query_scalar("SELECT COUNT(*) FROM hand_participants WHERE user_id = $1")
            .bind(rich)
            .fetch_one(&db)
            .await
            .unwrap();
        if hands >= 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    assert!(hands >= 1, "o inscrito tardio joga a próxima mão");

    sqlx::query("DELETE FROM hand_participants WHERE user_id = $1")
        .bind(rich)
        .execute(&db)
        .await
        .unwrap();
    cleanup_mtt_table(&db, &t).await;
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(poor)
        .execute(&db)
        .await
        .unwrap();
}
