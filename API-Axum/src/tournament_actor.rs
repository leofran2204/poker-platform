//! Ator de mesa de torneio — joga as mãos MTT com o mesmo protocolo do cash.
//!
//! Reusa `GameLoop` (`GameType::Tournament`, BBA automático), `PlayerCommand`
//! e o formato de broadcast do `game_actor`, então WebSocket, bots e o
//! frontend da mesa funcionam sem mudança. Diferenças para o cash:
//! rake zero, sem loss deflator, sem cash-out, stacks de torneio,
//! eliminações no motor + `tournament_seats`, persistência `game_type='tournament'`.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tracing::{error, info};

use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::types::{PokerVariant, TableConfig};

use crate::game_actor::{sign_settlement, PlayerCommand, TablePlayer};
use crate::state::{AppState, TableActorHandle};
use crate::tournament_store::TournamentStore;

const TURN_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(30);

pub struct TournamentActor {
    pub table_id: String,
    pub tournament_id: String,
    pub table_index: u32,
    pub name: String,
    pub players: Vec<TablePlayer>,
    pub game_loop: Option<GameLoop>,
    pub rx: mpsc::Receiver<PlayerCommand>,
    pub tx_broadcast: broadcast::Sender<serde_json::Value>,
    pub next_hand_at: Option<tokio::time::Instant>,
    pub dealer_index: usize,
    pub dealer_seat: Option<usize>,
    pub antifraud: poker_engine::antifraud::AntiFraudSuite,
    pub last_turn_start: Option<tokio::time::Instant>,
    pub turn_timeout: tokio::time::Duration,
    pub db: sqlx::PgPool,
    pub audit_secret: String,
    pub tournaments: Arc<RwLock<HashMap<String, TournamentStore>>>,
    pub active_tables: Arc<RwLock<HashMap<String, TableActorHandle>>>,
    pub persistence_halted: bool,
}

struct BlindSpec {
    small_blind: u64,
    big_blind: u64,
    variant: String,
}

impl TournamentActor {
    async fn blind_spec(&self) -> Option<BlindSpec> {
        let t = self.tournaments.read().await;
        let store = t.get(&self.tournament_id)?;
        let level = poker_engine::tournament_engine::get_current_blinds(&store.state)?;
        Some(BlindSpec {
            small_blind: level.small_blind,
            big_blind: level.big_blind,
            variant: store.active_poker_variant().to_string(),
        })
    }

    async fn table_still_live(&self) -> bool {
        let t = self.tournaments.read().await;
        t.get(&self.tournament_id)
            .is_some_and(|s| s.live_table_ids.iter().any(|id| id == &self.table_id))
    }

    pub async fn run(mut self) {
        info!(
            "Tournament actor started for table {} (torneio {})",
            self.table_id, self.tournament_id
        );
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(250));
        loop {
            tokio::select! {
                cmd = self.rx.recv() => {
                    match cmd {
                        Some(command) => self.handle_command(command).await,
                        None => {
                            info!("Command channel closed for tournament table {}", self.table_id);
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    if !self.tick().await {
                        break;
                    }
                }
            }
        }
        // Sai do mapa de atores para o coordenador/WSS recriarem se preciso.
        self.active_tables.write().await.remove(&self.table_id);
        info!("Tournament actor stopped for table {}", self.table_id);
    }

    async fn handle_command(&mut self, cmd: PlayerCommand) {
        match cmd {
            PlayerCommand::Sit {
                player_id,
                username,
                seat,
                chips,
                respond_to,
            } => {
                let assigned = self.handle_sit(player_id, username, seat, chips);
                let _ = respond_to.send(assigned);
            }
            PlayerCommand::Leave { player_id } => {
                // Em torneio não há cash-out: sair = ficar sit-out até blindar.
                self.handle_set_sitting(player_id, false);
            }
            PlayerCommand::CashOut {
                player_id: _,
                respond_to,
            } => {
                let _ = respond_to.send(Err("torneio não tem cash-out".to_string()));
            }
            PlayerCommand::Action {
                player_id,
                action,
                amount,
            } => {
                self.handle_action(player_id, action, amount).await;
            }
            PlayerCommand::GetTableInfo { respond_to } => {
                let _ = respond_to.send(self.table_info_json().await).await;
            }
            PlayerCommand::SetSitting { player_id, sitting } => {
                self.handle_set_sitting(player_id, sitting);
            }
        }
    }

    /// Retorna false quando o ator deve encerrar.
    async fn tick(&mut self) -> bool {
        if self.persistence_halted {
            return true;
        }
        if !self.table_still_live().await {
            info!(
                "Mesa {} saiu das mesas vivas; ator de torneio encerrando",
                self.table_id
            );
            return false;
        }
        // Fold automático p/ jogador da vez fora do assento.
        let away_active = self.game_loop.as_ref().and_then(|gl| {
            (!gl.state.is_finished)
                .then(|| gl.state.active_player().map(|p| p.id.clone()))
                .flatten()
                .filter(|pid| {
                    self.players
                        .iter()
                        .find(|p| &p.id == pid)
                        .is_none_or(|p| !p.is_sitting)
                })
        });
        if let Some(pid) = away_active {
            self.handle_action(pid, "fold".to_string(), 0).await;
            return true;
        }
        // Timeout de turno = fold.
        let timed_out = self.game_loop.as_ref().and_then(|gl| {
            (!gl.state.is_finished)
                .then_some(())
                .and(self.last_turn_start)
                .filter(|t| t.elapsed() >= self.turn_timeout)
                .and_then(|_| gl.state.active_player().map(|p| p.id.clone()))
        });
        if let Some(pid) = timed_out {
            self.handle_action(pid, "fold".to_string(), 0).await;
            return true;
        }
        if let Some(next_at) = self.next_hand_at {
            if tokio::time::Instant::now() >= next_at {
                self.next_hand_at = None;
                self.game_loop = None;
                self.start_new_hand().await;
            }
        } else if self.game_loop.is_none()
            && self
                .players
                .iter()
                .filter(|p| p.is_sitting && p.chips > 0)
                .count()
                >= 2
        {
            self.start_new_hand().await;
        }
        // Mesa sem jogo possível: encerra para o coordenador fundir/eliminar.
        if self.game_loop.is_none()
            && self.next_hand_at.is_none()
            && self
                .players
                .iter()
                .filter(|p| p.is_sitting && p.chips > 0)
                .count()
                < 2
        {
            // Espera um ciclo antes de sair (rebalance pode trazer gente).
            static mut IDLE_TICKS: u32 = 0;
            unsafe {
                IDLE_TICKS += 1;
                if IDLE_TICKS > 240 {
                    // ~60s sem jogo
                    IDLE_TICKS = 0;
                    info!("Mesa {} sem jogo; ator encerrando", self.table_id);
                    return false;
                }
            }
        }
        true
    }

    fn handle_sit(
        &mut self,
        player_id: String,
        username: String,
        seat: Option<usize>,
        chips: u64,
    ) -> usize {
        if let Some(existing) = self.players.iter_mut().find(|p| p.id == player_id) {
            existing.name = username;
            existing.chips = chips;
            if let Some(seat) = seat {
                existing.seat = seat;
            }
            existing.is_sitting = true;
            existing.disconnected_since = None;
            let s = existing.seat;
            self.broadcast_state();
            return s;
        }
        let assigned = match seat {
            Some(s) => s,
            None => {
                let mut found = 0;
                for s in 0..32 {
                    if !self.players.iter().any(|p| p.seat == s) {
                        found = s;
                        break;
                    }
                }
                found
            }
        };
        self.players.push(TablePlayer {
            id: player_id,
            name: username,
            chips,
            seat: assigned,
            is_sitting: true,
            disconnected_since: None,
        });
        self.broadcast_state();
        assigned
    }

    fn handle_set_sitting(&mut self, player_id: String, sitting: bool) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.is_sitting = sitting;
            if sitting {
                p.disconnected_since = None;
            }
        }
        self.broadcast_state();
    }

    async fn handle_action(&mut self, player_id: String, action: String, amount: u64) {
        let elapsed_ms = self
            .last_turn_start
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(500);
        let risk = self.antifraud.process_action(&player_id, elapsed_ms);
        if risk.recommendation == poker_engine::antifraud::RiskRecommendation::BlockSession {
            error!(
                "Ação bloqueada pelo antifraude p/ {player_id} na mesa {}",
                self.table_id
            );
            return;
        }
        let m = action.to_lowercase();
        let mv = match m.as_str() {
            "fold" => PlayerMove::Fold,
            "check" => PlayerMove::Check,
            "call" => PlayerMove::Call,
            "bet" => PlayerMove::Bet(amount),
            "raise" => PlayerMove::Raise(amount),
            "all-in" | "allin" => PlayerMove::AllIn,
            _ => {
                error!("Ação inválida '{action}' na mesa {}", self.table_id);
                return;
            }
        };
        let gl = match &mut self.game_loop {
            Some(gl) => gl,
            None => return,
        };
        if gl.player_action(&player_id, mv).is_err() {
            return;
        }
        self.last_turn_start = (!gl.state.is_finished).then_some(tokio::time::Instant::now());
        if gl.state.is_finished {
            if let Ok(res) = gl.resolve_hand() {
                gl.finalize_history(&res);
                self.settle_hand(res).await;
            }
            self.broadcast_state();
            self.next_hand_at =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(6));
        } else {
            self.broadcast_state();
        }
    }

    /// Pausa auditável: log + trilha em audit_logs + mesa PAUSED + flag.
    /// Toda parada do ator MTT passa por aqui — nunca silenciosa.
    async fn halt(&mut self, reason: &str) {
        error!(table_id = %self.table_id, reason, "mesa MTT pausada");
        let _ = sqlx::query(
            "INSERT INTO audit_logs (user_id, action, metadata) VALUES ('system','MTT_TABLE_HALTED', $1)",
        )
        .bind(serde_json::json!({
            "table_id": self.table_id,
            "tournament_id": self.tournament_id,
            "reason": reason,
        }))
        .execute(&self.db)
        .await;
        if let Ok(table_uuid) = uuid::Uuid::parse_str(&self.table_id) {
            let _ = sqlx::query("UPDATE tables SET status='PAUSED' WHERE id=$1")
                .bind(table_uuid)
                .execute(&self.db)
                .await;
        }
        self.persistence_halted = true;
    }

    async fn start_new_hand(&mut self) {
        if self.persistence_halted {
            return;
        }
        let mut active: Vec<(String, u64, usize)> = self
            .players
            .iter()
            .filter(|p| p.is_sitting && p.chips > 0)
            .map(|p| (p.id.clone(), p.chips, p.seat))
            .collect();
        if active.len() < 2 {
            return;
        }
        active.sort_by_key(|(_, _, seat)| *seat);
        let spec = match self.blind_spec().await {
            Some(s) => s,
            None => return,
        };
        let hand_id = uuid::Uuid::new_v4();
        if let Err(error) = sqlx::query(
            "INSERT INTO table_hand_recovery_guards (table_id, hand_id) \
             VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(uuid::Uuid::parse_str(&self.table_id).unwrap_or_else(|_| uuid::Uuid::nil()))
        .bind(hand_id)
        .execute(&self.db)
        .await
        {
            error!(
                ?error,
                "Falha no guard da mão MTT; pausando mesa {}", self.table_id
            );
            self.halt("falha irrecuperável (ver error! anterior)").await;
            return;
        }
        let config = TableConfig::new(spec.big_blind, 0, 0)
            .with_small_blind(spec.small_blind)
            .with_poker_variant(PokerVariant::parse(&spec.variant));
        let mut gl = GameLoop::new(
            config,
            hand_id.to_string(),
            self.name.clone(),
            GameType::Tournament,
        )
        .with_skip_loss_deflator(true);
        for (pid, chips, _) in &active {
            gl.add_player(pid.clone(), *chips);
        }
        self.dealer_index = match self.dealer_seat {
            Some(prev) => active
                .iter()
                .position(|(_, _, seat)| *seat > prev)
                .unwrap_or(0),
            None => 0,
        };
        self.dealer_seat = Some(active[self.dealer_index].2);
        gl.set_dealer(self.dealer_index);
        if gl.start_hand().is_err() {
            self.halt("start_hand do motor falhou").await;
            return;
        }
        self.game_loop = Some(gl);
        self.last_turn_start = Some(tokio::time::Instant::now());
        info!(table_id = %self.table_id, hand_id = %hand_id, "Mão MTT iniciada");
        self.broadcast_state();
    }

    async fn settle_hand(&mut self, res: poker_engine::game_loop::HandResolution) {
        // UUIDs.parseados ANTES da transação: qualquer id inválido trava a
        // mesa com erro visível em vez de envenenar o tx em silêncio.
        let parse_uuid = |pid: &str| {
            uuid::Uuid::parse_str(pid)
                .map_err(|_| format!("player id inválido no settle MTT: {pid}"))
        };
        let participant_uuids: Vec<uuid::Uuid> = match self
            .game_loop
            .as_ref()
            .map(|gl| {
                gl.state
                    .players
                    .iter()
                    .map(|p| parse_uuid(&p.id))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
        {
            Ok(Some(uuids)) => uuids,
            _ => {
                error!(table_id = %self.table_id, "settle MTT sem jogadores válidos; pausando");
                self.halt("falha irrecuperável (ver error! anterior)").await;
                return;
            }
        };
        // Conservação de fichas com rake zero.
        let memory_stacks: Vec<(String, u64)> = self
            .game_loop
            .as_ref()
            .map(|gl| {
                gl.state
                    .players
                    .iter()
                    .map(|p| {
                        let payout = res.payouts.get(&p.id).copied().unwrap_or(0);
                        (p.id.clone(), p.stack + payout)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let starting_total: u128 = self
            .game_loop
            .as_ref()
            .and_then(|gl| gl.history.as_ref())
            .map(|h| h.starting_stacks.values().map(|c| u128::from(*c)).sum())
            .unwrap_or(0);
        let final_total: u128 = memory_stacks.iter().map(|(_, c)| u128::from(*c)).sum();
        let payout_total: u128 = res.payouts.values().map(|c| u128::from(*c)).sum();
        let pot_total: u128 = self
            .game_loop
            .as_ref()
            .map(|gl| u128::from(gl.state.total_pot()))
            .unwrap_or(0);
        if payout_total != pot_total || final_total != starting_total || res.rake != 0 {
            error!(table_id = %self.table_id, "Conservação de fichas MTT violada; pausando");
            self.halt("falha irrecuperável (ver error! anterior)").await;
            return;
        }
        let gl = match self.game_loop.as_mut() {
            Some(gl) => gl,
            None => return,
        };
        let history = match gl.history.as_mut() {
            Some(h) => h,
            None => {
                self.halt("falha irrecuperável (ver error! anterior)").await;
                return;
            }
        };
        poker_engine::hand_history::sign_hand(history, self.audit_secret.as_bytes());
        let history_json = match serde_json::to_value(&*history) {
            Ok(v) => v,
            Err(_) => {
                self.halt("falha irrecuperável (ver error! anterior)").await;
                return;
            }
        };
        let mut payouts: Vec<serde_json::Value> = res
            .payouts
            .iter()
            .map(|(pid, amount)| serde_json::json!({"player_id": pid, "amount": amount}))
            .collect();
        payouts.sort_by(|a, b| a["player_id"].as_str().cmp(&b["player_id"].as_str()));
        let settlement = serde_json::json!({
            "version": 1,
            "hand_id": history.hand_id,
            "table_id": self.table_id,
            "tournament_id": self.tournament_id,
            "pot_total": history.total_pot,
            "rake_collected": 0,
            "end_reason": history.end_reason.as_str(),
            "payouts": payouts,
        });
        let signature = match sign_settlement(&settlement, self.audit_secret.as_bytes()) {
            Ok(s) => s,
            Err(_) => {
                self.halt("falha irrecuperável (ver error! anterior)").await;
                return;
            }
        };
        let hand_uuid =
            uuid::Uuid::parse_str(&history.hand_id).unwrap_or_else(|_| uuid::Uuid::nil());
        let table_uuid =
            uuid::Uuid::parse_str(&self.table_id).unwrap_or_else(|_| uuid::Uuid::nil());
        let winner: Option<String> = history
            .results
            .iter()
            .filter(|r| r.finish_position == 1 && r.chips_won > 0)
            .map(|r| r.player_id.clone())
            .min();
        let pot_i = i64::try_from(history.total_pot).unwrap_or(i64::MAX);
        let mut tx = match self.db.begin().await {
            Ok(tx) => tx,
            Err(_) => {
                self.halt("falha irrecuperável (ver error! anterior)").await;
                return;
            }
        };
        let hand_number: i64 = sqlx::query_scalar(
            "UPDATE tables SET hand_sequence = hand_sequence + 1 WHERE id = $1 RETURNING hand_sequence",
        )
        .bind(table_uuid)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(0);
        if let Err(error) = sqlx::query(
            "INSERT INTO hand_history (id, table_id, hand_number, game_type, small_blind, big_blind, actions_json, community_cards_json, loss_deflators_json, settlement_json, settlement_signature, winner_player_id, pot_total, rake_collected, end_reason) \
             VALUES ($1, $2, $3, 'tournament', $4, $5, $6, $7, '[]', $8, $9, $10, $11, 0, $12) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(hand_uuid)
        .bind(table_uuid)
        .bind(hand_number)
        .bind(0i64)
        .bind(0i64)
        .bind(history_json["actions"].clone())
        .bind(history_json["community_cards"].clone())
        .bind(settlement)
        .bind(signature)
        .bind(winner)
        .bind(pot_i)
        .bind(format!("{:?}", history.end_reason))
        .execute(&mut *tx)
        .await
        {
            error!(?error, table_id = %self.table_id, "settle MTT: hand_history falhou; pausando");
            let _ = tx.rollback().await;
            self.halt("falha irrecuperável (ver error! anterior)").await;
            return;
        }
        // Participantes p/ VP + stacks/eliminações. Erro aqui propaga e pausa
        // a mesa com log — nunca `let _` dentro da transação.
        let mut eliminated: Vec<String> = Vec::new();
        for ((pid, chips), user_id) in memory_stacks.iter().zip(participant_uuids.iter()) {
            let chips_i = i64::try_from(*chips).unwrap_or(i64::MAX);
            if let Err(error) = sqlx::query(
                "INSERT INTO hand_participants (hand_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(hand_uuid)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            {
                error!(?error, table_id = %self.table_id, "settle MTT: participantes falhou; pausando");
                let _ = tx.rollback().await;
                self.halt("falha irrecuperável (ver error! anterior)").await;
                return;
            }
            if *chips == 0 {
                eliminated.push(pid.clone());
            }
            if let Err(error) = sqlx::query(
                "UPDATE tournament_seats SET stack = $1, status = CASE WHEN $1 = 0 THEN 'ELIMINATED' ELSE status END \
                 WHERE tournament_id = $2::uuid AND table_id = $3 AND player_id = $4",
            )
            .bind(chips_i)
            .bind(&self.tournament_id)
            .bind(table_uuid)
            .bind(pid)
            .execute(&mut *tx)
            .await
            {
                error!(?error, table_id = %self.table_id, "settle MTT: stacks falhou; pausando");
                let _ = tx.rollback().await;
                self.halt("falha irrecuperável (ver error! anterior)").await;
                return;
            }
        }
        if let Err(error) = sqlx::query(
            "DELETE FROM table_hand_recovery_guards WHERE table_id = $1 AND hand_id = $2",
        )
        .bind(table_uuid)
        .bind(hand_uuid)
        .execute(&mut *tx)
        .await
        {
            error!(?error, table_id = %self.table_id, "settle MTT: limpeza do guard falhou; pausando");
            let _ = tx.rollback().await;
            self.halt("falha irrecuperável (ver error! anterior)").await;
            return;
        }
        if tx.commit().await.is_err() {
            error!(table_id = %self.table_id, "settle MTT: commit falhou; pausando");
            self.halt("falha irrecuperável (ver error! anterior)").await;
            return;
        }
        // Espelha no motor do torneio (eliminações) e na memória.
        {
            let mut tournaments = self.tournaments.write().await;
            if let Some(store) = tournaments.get_mut(&self.tournament_id) {
                for pid in &eliminated {
                    let _ = poker_engine::tournament_engine::eliminate_player(
                        &mut store.state,
                        pid,
                        None,
                    );
                }
            }
        }
        for (pid, chips) in memory_stacks {
            if let Some(p) = self.players.iter_mut().find(|p| p.id == pid) {
                p.chips = chips;
                if chips == 0 {
                    p.is_sitting = false;
                }
            }
        }
        self.players.retain(|p| p.is_sitting || p.chips > 0);
    }

    async fn table_info_json(&self) -> serde_json::Value {
        let (level, variant) = {
            let t = self.tournaments.read().await;
            t.get(&self.tournament_id)
                .map(|s| (s.state.current_level, s.active_poker_variant().to_string()))
                .unwrap_or((0, "holdem".to_string()))
        };
        serde_json::json!({
            "type": "table_info",
            "table_id": self.table_id,
            "tournament_id": self.tournament_id,
            "name": self.name,
            "game_type": "tournament",
            "blind_level": level,
            "poker_variant": variant,
            "players": self.players,
        })
    }

    fn broadcast_state(&self) {
        let mut players_json = Vec::new();
        let mut community_cards = Vec::new();
        let mut stage = "waiting".to_string();
        let mut pots = Vec::new();
        let mut is_finished = true;
        if let Some(ref gl) = self.game_loop {
            is_finished = gl.state.is_finished;
            stage = format!("{:?}", gl.state.phase).to_lowercase();
            community_cards = gl
                .state
                .community_cards
                .iter()
                .map(crate::game_actor::card_to_string)
                .collect();
            pots.push(serde_json::json!({
                "name": "Main",
                "amount": gl.state.total_pot(),
                "eligible_players": gl.state.players.iter().map(|p| p.id.clone()).collect::<Vec<String>>()
            }));
            for gp in &gl.state.players {
                if let Some(tp) = self.players.iter().find(|p| p.id == gp.id) {
                    players_json.push(serde_json::json!({
                        "id": gp.id,
                        "name": tp.name,
                        "chips": gp.stack,
                        "bet": gp.current_bet,
                        "cards": gp.hole_cards.iter().map(crate::game_actor::card_to_string).collect::<Vec<String>>(),
                        "is_active": gl.state.active_player().map(|ap| ap.id == gp.id).unwrap_or(false),
                        "is_dealer": gl.state.dealer_index == gp.seat_index,
                        "seat": tp.seat
                    }));
                }
            }
        } else {
            for p in &self.players {
                players_json.push(serde_json::json!({
                    "id": p.id,
                    "name": p.name,
                    "chips": p.chips,
                    "bet": 0,
                    "cards": Vec::<String>::new(),
                    "is_active": false,
                    "is_dealer": false,
                    "seat": p.seat
                }));
            }
        }
        let _ = self.tx_broadcast.send(serde_json::json!({
            "type": "table_state",
            "table_id": self.table_id,
            "tournament_id": self.tournament_id,
            "stage": stage,
            "community_cards": community_cards,
            "pots": pots,
            "players": players_json,
            "current_bet_to_match": self.game_loop.as_ref().map(|g| g.state.current_bet_to_match).unwrap_or(0),
            "min_raise": self.game_loop.as_ref().map(|g| g.state.min_raise).unwrap_or(0),
            "is_finished": is_finished
        }));
    }
}

/// Garante o ator da mesa de torneio no mapa e senta os inscritos ativos.
/// Reusa `TableActorHandle` (mesmo protocolo): WS, bots e frontend funcionam.
pub async fn ensure_tournament_actor(
    state: &AppState,
    tournament_id: &str,
    table_id: &str,
    table_index: u32,
    table_name: String,
) -> TableActorHandle {
    {
        let active = state.active_tables.read().await;
        if let Some(h) = active.get(table_id) {
            return h.clone();
        }
    }
    let (tx_cmd, rx_cmd) = mpsc::channel(100);
    let (tx_broadcast, _) = broadcast::channel(100);
    let actor = TournamentActor {
        table_id: table_id.to_string(),
        tournament_id: tournament_id.to_string(),
        table_index,
        name: table_name,
        players: Vec::new(),
        game_loop: None,
        rx: rx_cmd,
        tx_broadcast: tx_broadcast.clone(),
        next_hand_at: None,
        dealer_index: 0,
        dealer_seat: None,
        antifraud: poker_engine::antifraud::AntiFraudSuite::new(),
        last_turn_start: Some(tokio::time::Instant::now()),
        turn_timeout: TURN_TIMEOUT,
        db: state.db.clone(),
        audit_secret: state.jwt_secret.clone(),
        tournaments: state.tournaments.clone(),
        active_tables: state.active_tables.clone(),
        persistence_halted: false,
    };
    let handle = TableActorHandle {
        tx_cmd,
        tx_broadcast,
    };
    state
        .active_tables
        .write()
        .await
        .insert(table_id.to_string(), handle.clone());
    // Spawna ANTES de sentar: os Sit aguardam resposta do loop do ator.
    tokio::spawn(actor.run());
    // Senta os inscritos ativos (stacks do torneio).
    let seats: Vec<(String, String, i64, i16)> = sqlx::query_as(
        "SELECT player_id, player_name, stack, seat FROM tournament_seats \
         WHERE tournament_id=$1::uuid AND table_id=$2::uuid AND status='ACTIVE'",
    )
    .bind(tournament_id)
    .bind(table_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    for (pid, pname, stack, seat) in seats {
        let (tx_resp, rx_resp) = oneshot::channel();
        let _ = handle
            .tx_cmd
            .send(PlayerCommand::Sit {
                player_id: pid,
                username: pname,
                seat: Some(seat.max(0) as usize),
                chips: stack.max(0) as u64,
                respond_to: tx_resp,
            })
            .await;
        // Timeout: um ator travado nunca pode paralisar o coordenador.
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), rx_resp).await;
    }
    handle
}
