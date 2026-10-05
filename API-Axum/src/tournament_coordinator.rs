//! Coordenador de torneios — auto-start na data/hora definida pelo admin e aviso FT Short Deck.
//! Q2 B: usa tournament_seats separado. Q3 B: FT só no próximo blind + popup.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::AppState;
use poker_engine::tournament_engine::{self, TournamentStatus};

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Verifica se o torneio pode iniciar: data/hora agendada pelo admin + mínimo configurado.
pub fn should_start(
    store: &crate::tournament_store::TournamentStore,
    scheduled_start_at: Option<i64>,
    auto_min: i32,
    now: i64,
) -> bool {
    if store.state.status != TournamentStatus::Registering {
        return false;
    }
    let scheduled = match scheduled_start_at {
        Some(v) => v,
        None => return false,
    };
    if now < scheduled {
        return false;
    }
    let need = auto_min.max(2) as usize;
    store.state.players.len() >= need
}

/// Tarefa em background: verifica torneios e inicia os que atingiram a agenda e o mínimo.
pub async fn run_coordinator(state: AppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        let now = now_epoch();
        let ids: Vec<String> = {
            let t = state.tournaments.read().await;
            t.keys().cloned().collect()
        };
        for tid in ids {
            let row: Option<(Option<i64>, Option<i32>)> = sqlx::query_as(
                "SELECT scheduled_start_at, auto_start_min_players FROM tournaments WHERE id = $1::uuid",
            )
            .bind(&tid)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);
            let (sched, auto_min_opt) = match row {
                Some(v) => v,
                None => continue,
            };
            let auto_min = auto_min_opt.unwrap_or(5);
            let should = {
                let t = state.tournaments.read().await;
                t.get(&tid)
                    .map(|s| should_start(s, sched, auto_min, now))
                    .unwrap_or(false)
            };
            if !should {
                continue;
            }
            let mut tournaments = state.tournaments.write().await;
            if let Some(store) = tournaments.get_mut(&tid) {
                if tournament_engine::start_tournament(&mut store.state).is_ok() {
                    let _ = sqlx::query(
                        "UPDATE tournaments SET status='running', started_at=$2, current_level=1 WHERE id=$1::uuid",
                    )
                    .bind(&tid)
                    .bind(now)
                    .execute(&state.db)
                    .await;
                    if let Ok(table_ids) = assign_tournament_tables(&state, store, &tid).await {
                        let first = table_ids.first().cloned().unwrap_or_default();
                        store.live_table_id = Some(first.clone());
                        store.live_table_ids = table_ids.clone();
                        let live_uuids: Vec<uuid::Uuid> = table_ids
                            .iter()
                            .filter_map(|id| uuid::Uuid::parse_str(id).ok())
                            .collect();
                        let _ = sqlx::query(
                            "UPDATE tournaments SET live_table_id=$2::uuid, live_table_ids=$3 WHERE id=$1::uuid",
                        )
                        .bind(&tid)
                        .bind(&first)
                        .bind(&live_uuids)
                        .execute(&state.db)
                        .await;
                        // Sobe um ator de torneio por mesa (mãos começam sozinhas).
                        let tname = store.state.config.name.clone();
                        for (idx, table_id) in table_ids.iter().enumerate() {
                            crate::tournament_actor::ensure_tournament_actor(
                                &state,
                                &tid,
                                table_id,
                                idx as u32,
                                format!("{tname} mesa {}", idx + 1),
                            )
                            .await;
                        }
                    }
                    tracing::info!(tournament_id=%tid, "torneio iniciado automaticamente na agenda definida pelo admin");
                }
            }
        }
        advance_expired_blinds(&state).await;
        rebalance_tournament_tables(&state).await;
        consolidate_final_tables(&state).await;
        finish_decided_tournaments(&state).await;
        ensure_live_actors(&state).await;
    }
}

/// Cria (ou reaproveita) as 3 mesas físicas do torneio e senta os inscritos
/// em round-robin balanceado. Retorna os UUIDs na ordem dos índices 0, 1, 2.
async fn assign_tournament_tables(
    state: &AppState,
    store: &mut crate::tournament_store::TournamentStore,
    tid: &str,
) -> Result<Vec<String>, sqlx::Error> {
    use poker_engine::tournament_engine as engine;
    let table_max = store.table_max_players as u32;
    let mut table_ids: Vec<uuid::Uuid> = sqlx::query_as(
        "SELECT id FROM tables WHERE club_id IS NULL AND game_type='tournament' \
         AND poker_variant=$1 AND max_players=$2 AND status='OPEN' AND current_players=0 \
         ORDER BY created_at LIMIT 3",
    )
    .bind(&store.poker_variant)
    .bind(table_max as i32)
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(id,)| id)
    .collect();
    while table_ids.len() < engine::TOURNAMENT_TABLE_COUNT as usize {
        let row: (uuid::Uuid,) = sqlx::query_as(
            "INSERT INTO tables (name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, max_players, current_players, visibility, status, poker_variant, money_mode) VALUES ($1,'tournament',$2,$2,$3,$3,$4,0,'private','OPEN',$5,$6) RETURNING id",
        )
        .bind(format!("MTT {} mesa {}", store.state.config.name, table_ids.len() + 1))
        .bind(store.state.config.blind_levels.first().map(|b| b.big_blind as i64).unwrap_or(50))
        .bind(store.state.config.starting_stack as i64)
        .bind(table_max as i32)
        .bind(&store.poker_variant)
        .bind(&store.money_mode)
        .fetch_one(&state.db)
        .await?;
        table_ids.push(row.0);
    }
    // Ordem determinística: registro mais antigo primeiro.
    let mut order: Vec<(String, String, u64)> = store
        .state
        .players
        .values()
        .map(|e| (e.player_id.clone(), e.player_name.clone(), e.stack))
        .collect();
    order.sort_by_key(|(pid, _, _)| {
        store
            .state
            .players
            .get(pid)
            .map(|e| e.registered_at)
            .unwrap_or(u64::MAX)
    });
    let plan = engine::assign_initial_tables(order.len(), table_max);
    for ((pid, pname, stack), (_, table_idx, seat)) in order.iter().zip(plan.iter()) {
        if let Some(entry) = store.state.players.get_mut(pid) {
            entry.table_id = Some(*table_idx);
            entry.seat = Some(*seat);
        }
        let _ = sqlx::query(
            "INSERT INTO tournament_seats (tournament_id, table_id, seat, player_id, player_name, stack) VALUES ($1::uuid,$2::uuid,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
        )
        .bind(tid)
        .bind(table_ids[*table_idx as usize])
        .bind(*seat as i16)
        .bind(pid)
        .bind(pname)
        .bind(*stack as i64)
        .execute(&state.db)
        .await;
    }
    Ok(table_ids.into_iter().map(|id| id.to_string()).collect())
}

/// Balanceamento contínuo: a cada tick, torneios running com 3 mesas vivas
/// movem 1 jogador da mais cheia para a mais vazia quando o desnível > 1.
async fn rebalance_tournament_tables(state: &AppState) {
    use poker_engine::tournament_engine as engine;
    let ids: Vec<String> = { state.tournaments.read().await.keys().cloned().collect() };
    for tid in ids {
        let (tables, running) = {
            let t = state.tournaments.read().await;
            match t.get(&tid) {
                Some(s) => (
                    s.live_table_ids.clone(),
                    s.state.status == engine::TournamentStatus::Running,
                ),
                None => continue,
            }
        };
        if !running || tables.len() != engine::TOURNAMENT_TABLE_COUNT as usize {
            continue;
        }
        let counts: Vec<(String, i64)> = sqlx::query_as(
            "SELECT table_id::text, COUNT(*) FROM tournament_seats \
             WHERE tournament_id=$1::uuid AND status='ACTIVE' GROUP BY table_id",
        )
        .bind(&tid)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        let mut by_idx = vec![0u32; tables.len()];
        for (table_id, count) in counts {
            if let Some(idx) = tables.iter().position(|t| t == &table_id) {
                by_idx[idx] = count.max(0) as u32;
            }
        }
        let Some((from, to)) = engine::rebalance_move(&by_idx) else {
            continue;
        };
        // Move o jogador do maior assento da origem para o próximo assento livre do destino.
        let victim: Option<(String, i16)> = sqlx::query_as(
            "SELECT player_id, seat FROM tournament_seats \
             WHERE tournament_id=$1::uuid AND table_id=$2::uuid AND status='ACTIVE' \
             ORDER BY seat DESC LIMIT 1",
        )
        .bind(&tid)
        .bind(&tables[from])
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
        let Some((pid, _seat)) = victim else { continue };
        let taken: Vec<i16> = sqlx::query_as(
            "SELECT seat FROM tournament_seats \
             WHERE tournament_id=$1::uuid AND table_id=$2::uuid AND status='ACTIVE'",
        )
        .bind(&tid)
        .bind(&tables[to])
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(s,)| s)
        .collect();
        let new_seat = (0..i16::MAX).find(|s| !taken.contains(s)).unwrap_or(0);
        let moved = sqlx::query(
            "UPDATE tournament_seats SET table_id=$3::uuid, seat=$4 \
             WHERE tournament_id=$1::uuid AND table_id=$2::uuid AND player_id=$5 AND status='ACTIVE'",
        )
        .bind(&tid)
        .bind(&tables[from])
        .bind(&tables[to])
        .bind(new_seat)
        .bind(&pid)
        .execute(&state.db)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);
        if moved == 1 {
            let mut tournaments = state.tournaments.write().await;
            if let Some(store) = tournaments.get_mut(&tid) {
                if let Some(entry) = store.state.players.get_mut(&pid) {
                    entry.table_id = Some(to as u32);
                    entry.seat = Some(new_seat as u32);
                }
            }
            tracing::info!(tournament_id=%tid, player=%pid, "jogador rebalanceado entre mesas");
        }
    }
}

/// Consolidação para a mesa final: restantes cabem no limite da FT — junta
/// todos na mesa 0 e reduz as mesas vivas a ela. A troca de variante da FT
/// segue `active_poker_variant` (Short Deck 8-max no blind seguinte + popup).
async fn consolidate_final_tables(state: &AppState) {
    use poker_engine::tournament_engine as engine;
    let ids: Vec<String> = { state.tournaments.read().await.keys().cloned().collect() };
    for tid in ids {
        let (tables, remaining, ft_limit, running) = {
            let t = state.tournaments.read().await;
            match t.get(&tid) {
                Some(s) => (
                    s.live_table_ids.clone(),
                    s.state.players_remaining,
                    s.final_table_max_players
                        .map(u32::from)
                        .unwrap_or(u32::from(s.table_max_players)),
                    s.state.status == engine::TournamentStatus::Running,
                ),
                None => continue,
            }
        };
        if !running || tables.len() <= 1 {
            continue;
        }
        if !engine::should_consolidate(remaining, ft_limit) {
            continue;
        }
        let first = tables[0].clone();
        // DELETE + re-INSERT evita colisão de PK (tournament, table, seat)
        // ao trazer assentos ocupados para a mesa 0.
        let actives: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT player_id, player_name, stack FROM tournament_seats \
             WHERE tournament_id=$1::uuid AND status='ACTIVE' ORDER BY seat",
        )
        .bind(&tid)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        let _ = sqlx::query(
            "DELETE FROM tournament_seats WHERE tournament_id=$1::uuid AND status='ACTIVE'",
        )
        .bind(&tid)
        .execute(&state.db)
        .await;
        for (idx, (pid, pname, stack)) in actives.iter().enumerate() {
            let _ = sqlx::query(
                "INSERT INTO tournament_seats (tournament_id, table_id, seat, player_id, player_name, stack) \
                 VALUES ($1::uuid,$2::uuid,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
            )
            .bind(&tid)
            .bind(&first)
            .bind(idx as i16)
            .bind(pid)
            .bind(pname)
            .bind(stack)
            .execute(&state.db)
            .await;
        }
        {
            let mut tournaments = state.tournaments.write().await;
            if let Some(store) = tournaments.get_mut(&tid) {
                for (idx, (pid, _, _)) in actives.iter().enumerate() {
                    if let Some(entry) = store.state.players.get_mut(pid) {
                        entry.table_id = Some(0);
                        entry.seat = Some(idx as u32);
                    }
                }
                store.live_table_ids = vec![first.clone()];
                store.live_table_id = Some(first.clone());
            }
        }
        let first_uuid = uuid::Uuid::parse_str(&first).ok();
        let Some(first_uuid) = first_uuid else {
            continue;
        };
        let _ = sqlx::query(
            "UPDATE tournaments SET live_table_id=$2::uuid, live_table_ids=$3 WHERE id=$1::uuid",
        )
        .bind(&tid)
        .bind(&first)
        .bind(vec![first_uuid])
        .execute(&state.db)
        .await;
        let _ = sqlx::query(
            "INSERT INTO audit_logs (user_id, action, metadata) VALUES ('system','FT_CONSOLIDATED', $1)",
        )
        .bind(serde_json::json!({"tournament_id":tid,"tables":tables.len(),"remaining":remaining}))
        .execute(&state.db)
        .await;
        tracing::info!(tournament_id=%tid, remaining, "mesa final consolidada");
    }
}

/// Finalização: resta 1 jogador — encerra no motor, credita prêmios nas
/// carteiras MTT (play/real conforme o torneio) e marca finished.
async fn finish_decided_tournaments(state: &AppState) {
    use poker_engine::tournament_engine as engine;
    let ids: Vec<String> = { state.tournaments.read().await.keys().cloned().collect() };
    for tid in ids {
        let (decided, money_mode) = {
            let t = state.tournaments.read().await;
            match t.get(&tid) {
                Some(s) => (
                    s.state.status == engine::TournamentStatus::Running
                        && s.state.players_remaining <= 1,
                    s.money_mode.clone(),
                ),
                None => continue,
            }
        };
        if !decided {
            continue;
        }
        let mut tournaments = state.tournaments.write().await;
        let Some(store) = tournaments.get_mut(&tid) else {
            continue;
        };
        // A rebuy can occur between the read lock above and this write lock.
        if store.state.status != engine::TournamentStatus::Running
            || store.state.players_remaining > 1
        {
            continue;
        }
        let mut finished = store.state.clone();
        let result = match engine::finish_tournament(&mut finished) {
            Ok(r) => r,
            Err(error) => {
                tracing::error!(tournament_id=%tid, %error, "classificação MTT inválida; prêmios não creditados");
                continue;
            }
        };
        match persist_tournament_prizes(&state.db, &result, &money_mode).await {
            Ok(_) => store.state = finished,
            Err(error) => {
                tracing::error!(tournament_id=%tid, %error, "premiação MTT revertida; próxima rodada tentará novamente");
                continue;
            }
        }
        tracing::info!(tournament_id=%tid, winners=result.winners.len(), "torneio finalizado com prêmios");
    }
}

/// The tournament row lock makes retries/concurrent finalizers idempotent.
/// Wallets, paid positions, status and audit commit together, or all roll back.
async fn persist_tournament_prizes(
    pool: &sqlx::PgPool,
    result: &poker_engine::tournament_engine::TournamentResult,
    money_mode: &str,
) -> Result<bool, sqlx::Error> {
    let invalid = |message: &str| sqlx::Error::Protocol(message.to_string());
    if result
        .winners
        .iter()
        .map(|w| u128::from(w.prize))
        .sum::<u128>()
        != u128::from(result.total_prize_pool)
    {
        return Err(invalid("Premiação não conserva o prize pool"));
    }
    let prize_pool =
        i64::try_from(result.total_prize_pool).map_err(|_| invalid("Premiação excede BIGINT"))?;
    let mut tx = pool.begin().await?;
    let (status, mode): (String, String) =
        sqlx::query_as("SELECT status, money_mode FROM tournaments WHERE id=$1::uuid FOR UPDATE")
            .bind(&result.tournament_id)
            .fetch_one(&mut *tx)
            .await?;
    if mode != money_mode || !matches!(mode.as_str(), "play" | "real") {
        return Err(invalid("Carteira de torneio divergente"));
    }
    if status == "finished" {
        tx.rollback().await?;
        return Ok(false);
    }
    if status != "running" {
        return Err(invalid("Torneio não está running no banco"));
    }
    let credit = if mode == "real" {
        "UPDATE users SET balance_real=balance_real+$1 WHERE id=$2::uuid"
    } else {
        "UPDATE users SET balance_pm_mtt=balance_pm_mtt+$1 WHERE id=$2::uuid"
    };
    let mut ids = std::collections::HashSet::new();
    for winner in &result.winners {
        if !ids.insert(&winner.player_id) {
            return Err(invalid("Vencedor duplicado"));
        }
        let prize = i64::try_from(winner.prize).map_err(|_| invalid("Prêmio excede BIGINT"))?;
        let paid = sqlx::query(credit)
            .bind(prize)
            .bind(&winner.player_id)
            .execute(&mut *tx)
            .await?;
        if paid.rows_affected() != 1 {
            return Err(invalid("Carteira premiada ausente"));
        }
        let recorded = sqlx::query("UPDATE tournament_players SET final_position=$3, prize=$4 WHERE tournament_id=$1::uuid AND player_id=$2")
            .bind(&result.tournament_id).bind(&winner.player_id).bind(winner.position as i32).bind(prize)
            .execute(&mut *tx).await?;
        if recorded.rows_affected() != 1 {
            return Err(invalid("Inscrição premiada ausente"));
        }
    }
    sqlx::query(
        "UPDATE tournaments SET status='finished', finished_at=$2, prize_pool=$3 WHERE id=$1::uuid",
    )
    .bind(&result.tournament_id)
    .bind(result.finished_at as i64)
    .bind(prize_pool)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs (user_id, action, metadata) VALUES ('system','MTT_FINISHED', $1)",
    )
    .bind(
        serde_json::json!({"tournament_id":result.tournament_id,"money_mode":mode,"result":result}),
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

/// Cura pós-restart: garante um ator vivo para cada mesa viva de torneio
/// running. `ensure_tournament_actor` é idempotente (retorna o existente).
async fn ensure_live_actors(state: &AppState) {
    use poker_engine::tournament_engine as engine;
    let ids: Vec<String> = { state.tournaments.read().await.keys().cloned().collect() };
    for tid in ids {
        let (tables, running, tname) = {
            let t = state.tournaments.read().await;
            match t.get(&tid) {
                Some(s) => (
                    s.live_table_ids.clone(),
                    s.state.status == engine::TournamentStatus::Running,
                    s.state.config.name.clone(),
                ),
                None => continue,
            }
        };
        if !running {
            continue;
        }
        for (idx, table_id) in tables.iter().enumerate() {
            crate::tournament_actor::ensure_tournament_actor(
                state,
                &tid,
                table_id,
                idx as u32,
                format!("{tname} mesa {}", idx + 1),
            )
            .await;
        }
    }
}

async fn advance_expired_blinds(state: &AppState) {
    let ids: Vec<String> = { state.tournaments.read().await.keys().cloned().collect() };
    for tid in ids {
        let mut tournaments = state.tournaments.write().await;
        if let Some(store) = tournaments.get_mut(&tid) {
            if tournament_engine::is_blind_level_expired(&store.state)
                && tournament_engine::advance_blinds(&mut store.state).is_ok()
            {
                let _ = sqlx::query("UPDATE tournaments SET current_level=$2 WHERE id=$1::uuid")
                    .bind(&tid)
                    .bind(store.state.current_level as i32)
                    .execute(&state.db)
                    .await;
                tracing::info!(tournament_id=%tid, level=%store.state.current_level, "blind avançado");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tournament_store::TournamentStore;
    use poker_engine::tournament_engine::{register_player, TournamentConfig};

    #[tokio::test]
    #[ignore = "requires isolated migrated PostgreSQL"]
    async fn tournament_prizes_are_atomic_idempotent_and_wallet_isolated() {
        use poker_engine::tournament_engine::{TournamentResult, WinnerEntry};
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect(&std::env::var("DATABASE_URL").expect("isolated DATABASE_URL"))
            .await
            .unwrap();
        for mode in ["play", "real"] {
            let tid = uuid::Uuid::new_v4();
            let ids: Vec<_> = (0..3).map(|_| uuid::Uuid::new_v4()).collect();
            sqlx::query("INSERT INTO tournaments (id,name,buy_in,starting_stack,max_players,status,money_mode,prize_pool) VALUES ($1,'payout regression',0,1000,5,'running',$2,10003)")
                .bind(tid).bind(mode).execute(&pool).await.unwrap();
            for id in &ids {
                let name = format!("prize_{}", &id.simple().to_string()[..20]);
                sqlx::query("INSERT INTO users (id,username,email,password_hash,balance_pm_cash,balance_pm_mtt,balance_real) VALUES ($1,$2,$2,'synthetic',100,200,300)")
                    .bind(id).bind(&name).execute(&pool).await.unwrap();
                sqlx::query("INSERT INTO tournament_players (tournament_id,player_id,player_name,stack) VALUES ($1,$2,$3,0)")
                    .bind(tid).bind(id.to_string()).bind(name).execute(&pool).await.unwrap();
            }
            let result = TournamentResult {
                tournament_id: tid.to_string(),
                tournament_name: "payout regression".into(),
                total_players: 3,
                total_prize_pool: 10003,
                started_at: 1,
                finished_at: 2,
                duration_seconds: 1,
                winners: ids
                    .iter()
                    .zip([5001, 3001, 2001])
                    .enumerate()
                    .map(|(i, (id, prize))| WinnerEntry {
                        position: (i + 1) as u32,
                        player_id: id.to_string(),
                        player_name: format!("P{i}"),
                        prize,
                    })
                    .collect(),
            };
            let mut broken = result.clone();
            broken.winners[2].player_id = uuid::Uuid::new_v4().to_string();
            assert!(persist_tournament_prizes(&pool, &broken, mode)
                .await
                .is_err());
            let balances: Vec<(i64, i64, i64)> = sqlx::query_as(
                "SELECT balance_pm_cash,balance_pm_mtt,balance_real FROM users WHERE id=ANY($1)",
            )
            .bind(&ids)
            .fetch_all(&pool)
            .await
            .unwrap();
            assert!(
                balances.iter().all(|b| *b == (100, 200, 300)),
                "partial credits must roll back"
            );
            let status: String = sqlx::query_scalar("SELECT status FROM tournaments WHERE id=$1")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(status, "running");
            let (a, b) = tokio::join!(
                persist_tournament_prizes(&pool, &result, mode),
                persist_tournament_prizes(&pool, &result, mode)
            );
            assert_ne!(a.unwrap(), b.unwrap(), "exactly one finalizer pays");
            assert!(!persist_tournament_prizes(&pool, &result, mode)
                .await
                .unwrap());
            for (id, prize) in ids.iter().zip([5001, 3001, 2001]) {
                let balance: (i64, i64, i64) = sqlx::query_as(
                    "SELECT balance_pm_cash,balance_pm_mtt,balance_real FROM users WHERE id=$1",
                )
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
                assert_eq!(
                    balance,
                    if mode == "play" {
                        (100, 200 + prize, 300)
                    } else {
                        (100, 200, 300 + prize)
                    }
                );
            }
            let paid: i64 = sqlx::query_scalar(
                "SELECT sum(prize)::bigint FROM tournament_players WHERE tournament_id=$1",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(paid, 10003);
            let audits: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_logs WHERE action='MTT_FINISHED' AND metadata->>'tournament_id'=$1")
                .bind(tid.to_string()).fetch_one(&pool).await.unwrap();
            assert_eq!(audits, 1);
            sqlx::query("DELETE FROM audit_logs WHERE action='MTT_FINISHED' AND metadata->>'tournament_id'=$1").bind(tid.to_string()).execute(&pool).await.unwrap();
            sqlx::query("DELETE FROM tournaments WHERE id=$1")
                .bind(tid)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM users WHERE id=ANY($1)")
                .bind(&ids)
                .execute(&pool)
                .await
                .unwrap();
        }
    }

    fn store_with_players(count: usize) -> TournamentStore {
        let mut store = TournamentStore::new("scheduled".into(), TournamentConfig::default());
        for index in 0..count {
            register_player(
                &mut store.state,
                &format!("player-{index}"),
                &format!("Player {index}"),
            )
            .expect("test registration must succeed");
        }
        store
    }

    #[test]
    fn should_start_requires_schedule_status_and_minimum_players() {
        let scheduled = 1_000;
        assert!(!should_start(
            &store_with_players(5),
            Some(scheduled),
            5,
            999
        ));
        assert!(!should_start(
            &store_with_players(4),
            Some(scheduled),
            5,
            1_000
        ));
        assert!(should_start(
            &store_with_players(5),
            Some(scheduled),
            5,
            1_000
        ));

        let mut running = store_with_players(5);
        running.state.status = TournamentStatus::Running;
        assert!(!should_start(&running, Some(scheduled), 5, 1_000));
    }
}
