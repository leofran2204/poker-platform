//! Coordenador de torneios — auto-start agendado (5 players, America/Sao_Paulo) e aviso FT Short Deck.
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

/// Verifica se o torneio pode iniciar: horário agendado + 5 players
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

/// Tarefa em background: a cada 30s verifica torneios agendados e inicia os que atingiram horário+5
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
                    }
                    tracing::info!(tournament_id=%tid, "torneio iniciado auto com 5+ players no horário agendado");
                }
            }
        }
        check_ft_pending(&state).await;
        advance_expired_blinds(&state).await;
        rebalance_tournament_tables(&state).await;
        consolidate_final_tables(&state).await;
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
        let mut seat: i16 = 0;
        let actives: Vec<String> = sqlx::query_as(
            "SELECT player_id FROM tournament_seats \
             WHERE tournament_id=$1::uuid AND status='ACTIVE' ORDER BY seat",
        )
        .bind(&tid)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(p,)| p)
        .collect();
        for pid in &actives {
            let _ = sqlx::query(
                "UPDATE tournament_seats SET table_id=$3::uuid, seat=$4 \
                 WHERE tournament_id=$1::uuid AND player_id=$2 AND status='ACTIVE'",
            )
            .bind(&tid)
            .bind(pid)
            .bind(&first)
            .bind(seat)
            .execute(&state.db)
            .await;
            seat += 1;
        }
        {
            let mut tournaments = state.tournaments.write().await;
            if let Some(store) = tournaments.get_mut(&tid) {
                for (idx, pid) in actives.iter().enumerate() {
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

async fn check_ft_pending(state: &AppState) {
    let ids: Vec<String> = { state.tournaments.read().await.keys().cloned().collect() };
    for tid in ids {
        let (remaining, ft_variant, ft_max, status) = {
            let t = state.tournaments.read().await;
            if let Some(s) = t.get(&tid) {
                (
                    s.state.players_remaining,
                    s.final_table_variant.clone(),
                    s.final_table_max_players,
                    s.state.status.clone(),
                )
            } else {
                continue;
            }
        };
        if status != TournamentStatus::Running {
            continue;
        }
        if ft_variant.as_deref() == Some("short_deck") && ft_max == Some(8) && remaining == 8 {
            tracing::info!(tournament_id=%tid, "FT 8-max Short Deck pendente — troca no próximo blind");
            let _ = sqlx::query(
                "INSERT INTO audit_logs (user_id, action, metadata) VALUES ('system','FT_SWITCH_PENDING', $1)",
            )
            .bind(serde_json::json!({"tournament_id":tid,"next_variant":"short_deck","max":8}))
            .execute(&state.db)
            .await;
        }
    }
}

async fn advance_expired_blinds(state: &AppState) {
    let ids: Vec<String> = { state.tournaments.read().await.keys().cloned().collect() };
    for tid in ids {
        let mut tournaments = state.tournaments.write().await;
        if let Some(store) = tournaments.get_mut(&tid) {
            if tournament_engine::is_blind_level_expired(&store.state) {
                if tournament_engine::advance_blinds(&mut store.state).is_ok() {
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
}
