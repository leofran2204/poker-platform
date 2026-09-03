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
                    let _ = assign_tournament_tables(&state, store, &tid).await;
                    tracing::info!(tournament_id=%tid, "torneio iniciado auto com 5+ players no horário agendado");
                }
            }
        }
        check_ft_pending(&state).await;
        advance_expired_blinds(&state).await;
    }
}

async fn assign_tournament_tables(
    state: &AppState,
    store: &crate::tournament_store::TournamentStore,
    tid: &str,
) -> Result<(), sqlx::Error> {
    let table_max = store.table_max_players as i32;
    let table_id: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM tables WHERE club_id IS NULL AND game_type='tournament' AND poker_variant=$1 AND max_players=$2 AND status='OPEN' LIMIT 1",
    )
    .bind(&store.poker_variant)
    .bind(table_max)
    .fetch_optional(&state.db)
    .await?;
    let table_uuid = if let Some((id,)) = table_id {
        id
    } else {
        let row: (uuid::Uuid,) = sqlx::query_as(
            "INSERT INTO tables (name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, max_players, current_players, visibility, status, poker_variant, money_mode) VALUES ($1,'tournament',$2,$2,$3,$3,$4,0,'private','OPEN',$5,$6) RETURNING id",
        )
        .bind(format!("MTT {}", store.state.config.name))
        .bind(store.state.config.blind_levels.first().map(|b| b.big_blind as i64).unwrap_or(50))
        .bind(store.state.config.starting_stack as i64)
        .bind(table_max)
        .bind(&store.poker_variant)
        .bind(&store.money_mode)
        .fetch_one(&state.db)
        .await?;
        row.0
    };
    let mut seat: i16 = 0;
    for (pid, entry) in &store.state.players {
        let _ = sqlx::query(
            "INSERT INTO tournament_seats (tournament_id, table_id, seat, player_id, player_name, stack) VALUES ($1::uuid,$2::uuid,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
        )
        .bind(tid)
        .bind(table_uuid)
        .bind(seat)
        .bind(pid)
        .bind(&entry.player_name)
        .bind(entry.stack as i64)
        .execute(&state.db)
        .await;
        seat = (seat + 1) % (table_max as i16);
        if seat < 0 { seat = 0; }
    }
    Ok(())
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
