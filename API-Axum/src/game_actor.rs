// game_actor.rs — Ator para orquestração da mesa de Poker em tempo real.
//
// Gerencia a conexão de múltiplos jogadores, as transições do GameLoop
// e a transmissão das cartas e apostas via WebSockets.

use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{error, info};

use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::types::TableConfig;

const DEFAULT_TURN_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(30);

/// Ações que podem ser solicitadas pelos jogadores.
#[derive(Debug)]
pub enum PlayerCommand {
    Sit {
        player_id: String,
        username: String,
        seat: Option<usize>,
        chips: u64,
        respond_to: oneshot::Sender<usize>,
    },
    Leave {
        player_id: String,
    },
    /// Cash-out is accepted only between hands. `Some(chips)` means the actor
    /// had the current stack in memory; `None` lets the API use persisted escrow.
    CashOut {
        player_id: String,
        respond_to: oneshot::Sender<Result<Option<u64>, String>>,
    },
    Action {
        player_id: String,
        action: String,
        amount: u64,
    },
    GetTableInfo {
        respond_to: mpsc::Sender<serde_json::Value>,
    },
}

/// Estado do jogador persistente na mesa.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TablePlayer {
    pub id: String,
    pub name: String,
    pub chips: u64,
    pub seat: usize,
    pub is_sitting: bool,
}

pub struct TableActor {
    pub table_id: String,
    pub name: String,
    pub config: TableConfig,
    pub players: Vec<TablePlayer>,
    pub game_loop: Option<GameLoop>,
    pub rx: mpsc::Receiver<PlayerCommand>,
    pub tx_broadcast: broadcast::Sender<serde_json::Value>,
    pub next_hand_at: Option<tokio::time::Instant>,
    /// Index in the current GameLoop player list, used only by that hand.
    pub dealer_index: usize,
    /// Physical seat of the most recent dealer. Unlike `dealer_index`, this
    /// survives players leaving and joining between hands.
    pub dealer_seat: Option<usize>,
    pub antifraud: poker_engine::antifraud::AntiFraudSuite,
    pub last_turn_start: Option<tokio::time::Instant>,
    pub turn_timeout: tokio::time::Duration,
    pub db: Option<sqlx::PgPool>,
    pub redis: Option<redis::aio::ConnectionManager>,
    pub audit_secret: Option<String>,
    pub persistence_halted: bool,
}

struct HandHistoryRecord {
    hand_id: uuid::Uuid,
    table_id: uuid::Uuid,
    participants: Vec<uuid::Uuid>,
    actions: serde_json::Value,
    community_cards: serde_json::Value,
    loss_deflators: serde_json::Value,
    pot: i64,
    rake: i64,
    reason: String,
    small_blind: i64,
    big_blind: i64,
}

async fn persist_completed_hand(
    db: sqlx::PgPool,
    record: HandHistoryRecord,
    settled_stacks: &[(uuid::Uuid, i64)],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    let hand_number: i64 = sqlx::query_scalar(
        "UPDATE tables SET hand_sequence = hand_sequence + 1 WHERE id = $1 RETURNING hand_sequence",
    )
    .bind(record.table_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO hand_history (id, table_id, hand_number, game_type, small_blind, big_blind, actions_json, community_cards_json, loss_deflators_json, pot_total, rake_collected, end_reason) \
         VALUES ($1, $2, $3, 'cash', $4, $5, $6, $7, $8, $9, $10, $11) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(record.hand_id)
    .bind(record.table_id)
    .bind(hand_number)
    .bind(record.small_blind)
    .bind(record.big_blind)
    .bind(record.actions)
    .bind(record.community_cards)
    .bind(record.loss_deflators)
    .bind(record.pot)
    .bind(record.rake)
    .bind(record.reason)
    .execute(&mut *tx)
    .await?;

    // FASE 2: Ledger B2B SaaS
    // Deposita a fatia do Clube no saldo administrativo se a mesa for privada.
    if record.rake > 0 {
        let club_id: Option<uuid::Uuid> =
            sqlx::query_scalar("SELECT club_id FROM tables WHERE id = $1")
                .bind(record.table_id)
                .fetch_optional(&mut *tx)
                .await?
                .flatten();

        if let Some(c_id) = club_id {
            let platform_fee = (record.rake * 15) / 100;
            let club_rake = record.rake.saturating_sub(platform_fee);
            sqlx::query("UPDATE clubs SET balance = balance + $1 WHERE id = $2")
                .bind(club_rake)
                .bind(c_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    for user_id in record.participants {
        sqlx::query(
            "INSERT INTO hand_participants (hand_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(record.hand_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }
    for (user_id, chips) in settled_stacks {
        let updated = sqlx::query(
            "UPDATE cash_game_seats SET chips = $1 \
             WHERE table_id = $2 AND user_id = $3 AND status = 'ACTIVE'",
        )
        .bind(chips)
        .bind(record.table_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() != 1 {
            return Err(sqlx::Error::RowNotFound);
        }
    }
    let cleared =
        sqlx::query("DELETE FROM table_hand_recovery_guards WHERE table_id = $1 AND hand_id = $2")
            .bind(record.table_id)
            .bind(record.hand_id)
            .execute(&mut *tx)
            .await?;
    if cleared.rows_affected() != 1 {
        return Err(sqlx::Error::RowNotFound);
    }
    tx.commit().await
}

impl TableActor {
    pub fn new(
        table_id: String,
        name: String,
        rx: mpsc::Receiver<PlayerCommand>,
        tx_broadcast: broadcast::Sender<serde_json::Value>,
    ) -> Self {
        Self {
            table_id,
            name,
            config: TableConfig::new(2000, 500, 10000), // Default BB=2000 centavos (R$ 20,00), rake=5%, cap=10000 centavos (R$ 100,00)
            players: Vec::new(),
            game_loop: None,
            rx,
            tx_broadcast,
            next_hand_at: None,
            dealer_index: 0,
            dealer_seat: None,
            antifraud: poker_engine::antifraud::AntiFraudSuite::new(),
            last_turn_start: Some(tokio::time::Instant::now()),
            turn_timeout: DEFAULT_TURN_TIMEOUT,
            db: None,
            redis: None,
            persistence_halted: false,
            audit_secret: None,
        }
    }

    pub fn with_db(mut self, db: sqlx::PgPool) -> Self {
        self.db = Some(db);
        self
    }

    pub fn with_config(mut self, config: TableConfig) -> Self {
        self.config = config;
        self
    }

    #[cfg(test)]
    pub fn with_turn_timeout(mut self, turn_timeout: tokio::time::Duration) -> Self {
        self.turn_timeout = turn_timeout;
        self
    }

    pub fn with_redis(mut self, redis: redis::aio::ConnectionManager) -> Self {
        self.redis = Some(redis);
        self
    }

    pub fn with_audit_secret(mut self, audit_secret: String) -> Self {
        self.audit_secret = Some(audit_secret);
        self
    }

    pub async fn run(mut self) {
        info!("Table actor started for table: {}", self.table_id);

        // FASE 1: Recovery Snapshot do Redis no Boot
        if let Some(ref mut redis) = self.redis {
            use redis::AsyncCommands;
            let key = format!("poker:table:state:{}", self.table_id);
            if let Ok(Some(_json_str)) = redis.get::<_, Option<String>>(&key).await {
                info!(
                    "Recovered table state from Redis for table: {}",
                    self.table_id
                );
                // MVP: snapshot preservado no Redis; unmarshal completo fica para ownership distribuído.
            }
        }

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(250));

        loop {
            tokio::select! {
                cmd = self.rx.recv() => {
                    match cmd {
                        Some(command) => self.handle_command(command).await,
                        None => {
                            info!("Command channel closed for table: {}", self.table_id);
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    self.tick().await;
                }
            }
        }
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
                let assigned_seat = self.handle_sit(player_id, username, seat, chips);
                let _ = respond_to.send(assigned_seat);
                self.save_snapshot().await;
            }
            PlayerCommand::Leave { player_id } => {
                self.handle_leave(player_id);
                self.save_snapshot().await;
            }
            PlayerCommand::CashOut {
                player_id,
                respond_to,
            } => {
                let chips = self.handle_cash_out(&player_id);
                let _ = respond_to.send(chips);
                self.save_snapshot().await;
            }
            PlayerCommand::Action {
                player_id,
                action,
                amount,
            } => {
                self.handle_action(player_id, action, amount).await;
                self.save_snapshot().await;
            }
            PlayerCommand::GetTableInfo { respond_to } => {
                let info = self.get_table_info_json();
                let _ = respond_to.send(info).await;
            }
        }
    }

    async fn tick(&mut self) {
        if self.persistence_halted {
            return;
        }

        let timed_out_player = self.game_loop.as_ref().and_then(|game_loop| {
            (!game_loop.state.is_finished)
                .then_some(())
                .and(self.last_turn_start)
                .filter(|started_at| started_at.elapsed() >= self.turn_timeout)
                .and_then(|_| {
                    game_loop
                        .state
                        .active_player()
                        .map(|player| player.id.clone())
                })
        });
        if let Some(player_id) = timed_out_player {
            tracing::warn!(
                table_id = %self.table_id,
                player_id = %player_id,
                timeout_seconds = self.turn_timeout.as_secs(),
                "Player action timed out; applying automatic fold"
            );
            self.handle_action(player_id, "fold".to_string(), 0).await;
            self.save_snapshot().await;
            return;
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
            // Auto-start hand if we have enough players
            self.start_new_hand().await;
        }
    }

    fn handle_sit(
        &mut self,
        player_id: String,
        username: String,
        seat: Option<usize>,
        chips: u64,
    ) -> usize {
        // Remover se já estiver na mesa (para evitar duplicatas)
        self.players.retain(|p| p.id != player_id);

        let assigned_seat = match seat {
            Some(s) => s,
            None => {
                // Encontrar o próximo assento livre
                let mut found_seat = 0;
                for s in 0..9 {
                    if !self.players.iter().any(|p| p.seat == s) {
                        found_seat = s;
                        break;
                    }
                }
                found_seat
            }
        };

        self.players.push(TablePlayer {
            id: player_id,
            name: username,
            chips,
            seat: assigned_seat,
            is_sitting: true,
        });

        info!(
            "Player sat at table {} in seat {}",
            self.table_id, assigned_seat
        );
        self.broadcast_state();
        assigned_seat
    }

    fn handle_leave(&mut self, player_id: String) {
        self.players.retain(|p| p.id != player_id);
        info!("Player {} left table {}", player_id, self.table_id);

        // Se o jogador sair e a mão estiver em andamento, devemos dar fold nele
        let mut reset_turn_timer = false;
        if let Some(ref mut gl) = self.game_loop {
            if !gl.state.is_finished {
                let active_idx = gl.state.active_player_index;
                let is_active_turn = gl.state.players.get(active_idx).map(|p| p.id.as_str())
                    == Some(player_id.as_str());

                if is_active_turn {
                    reset_turn_timer = gl.player_action(&player_id, PlayerMove::Fold).is_ok()
                        && !gl.state.is_finished;
                } else if let Some(p) = gl.state.players.iter_mut().find(|p| p.id == player_id) {
                    p.has_folded = true;
                    if gl.state.players_in_hand_count() <= 1 {
                        gl.state.is_finished = true;
                    }
                }
            }
        }
        if reset_turn_timer {
            self.last_turn_start = Some(tokio::time::Instant::now());
        }

        self.broadcast_state();
    }

    fn handle_cash_out(&mut self, player_id: &str) -> Result<Option<u64>, String> {
        if let Some(game_loop) = &self.game_loop {
            if !game_loop.state.is_finished
                && game_loop
                    .state
                    .players
                    .iter()
                    .any(|player| player.id == player_id)
            {
                return Err("Cannot cash out while a hand is in progress".to_string());
            }
        }

        let chips = self
            .players
            .iter()
            .find(|player| player.id == player_id)
            .map(|player| player.chips);
        if chips.is_some() {
            self.players.retain(|player| player.id != player_id);
            self.broadcast_state();
        }
        Ok(chips)
    }

    async fn handle_action(&mut self, player_id: String, action: String, amount: u64) {
        let elapsed_ms = self
            .last_turn_start
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(500);
        let risk_score = self.antifraud.process_action(&player_id, elapsed_ms);
        if risk_score.recommendation == poker_engine::antifraud::RiskRecommendation::BlockSession {
            error!(
                "Action blocked by AntiFraud for player {}: risk score {}",
                player_id, risk_score.total_score
            );
            return;
        }

        let history_db = self.db.clone();
        let audit_secret = self.audit_secret.clone();
        let table_id = self.table_id.clone();
        let table_big_blind = self.config.big_blind;

        let game_loop = match &mut self.game_loop {
            Some(gl) => gl,
            None => {
                error!(
                    "Attempted action but no game loop running at table: {}",
                    self.table_id
                );
                return;
            }
        };

        let m = action.to_lowercase();
        let player_move = match m.as_str() {
            "fold" => PlayerMove::Fold,
            "check" => PlayerMove::Check,
            "call" => PlayerMove::Call,
            "bet" => PlayerMove::Bet(amount),
            "raise" => PlayerMove::Raise(amount),
            "all-in" | "allin" => PlayerMove::AllIn,
            _ => {
                error!("Invalid action string: {}", action);
                return;
            }
        };

        if let Err(e) = game_loop.player_action(&player_id, player_move) {
            error!("Error processing action for player {}: {}", player_id, e);
            return;
        }

        self.last_turn_start =
            (!game_loop.state.is_finished).then_some(tokio::time::Instant::now());

        if game_loop.state.is_finished {
            // Resolver a mão
            if let Ok(res) = game_loop.resolve_hand() {
                game_loop.finalize_history(&res);

                // The hand history, resulting stacks, and recovery guard are
                // committed in one transaction. A crash before commit leaves
                // the pre-hand escrow intact and a guard that pauses the table.
                let memory_stacks: Vec<(String, u64)> = game_loop
                    .state
                    .players
                    .iter()
                    .map(|player| {
                        let payout = res.payouts.get(&player.id).copied().unwrap_or(0);
                        (player.id.clone(), player.stack + payout)
                    })
                    .collect();
                let durable_stacks = memory_stacks
                    .iter()
                    .map(|(player_id, chips)| {
                        Ok((
                            uuid::Uuid::parse_str(player_id)
                                .map_err(|_| "Invalid player id while settling hand".to_string())?,
                            i64::try_from(*chips)
                                .map_err(|_| "Player stack exceeds database range".to_string())?,
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>();
                let persistence_result = match (
                    game_loop.history.as_mut(),
                    audit_secret.as_deref(),
                    history_db,
                    durable_stacks.as_ref(),
                ) {
                    (Some(history), Some(audit_secret), Some(db), Ok(durable_stacks)) => {
                        poker_engine::hand_history::sign_hand(history, audit_secret.as_bytes());
                        let record = (|| -> Result<HandHistoryRecord, String> {
                            let history_json = serde_json::to_value(&*history)
                                .map_err(|_| "Could not serialize hand history".to_string())?;
                            Ok(HandHistoryRecord {
                                hand_id: uuid::Uuid::parse_str(&history.hand_id).map_err(|_| {
                                    "Invalid hand id while settling hand".to_string()
                                })?,
                                table_id: uuid::Uuid::parse_str(&table_id).map_err(|_| {
                                    "Invalid table id while settling hand".to_string()
                                })?,
                                participants: history
                                    .players
                                    .iter()
                                    .map(|player_id| uuid::Uuid::parse_str(player_id))
                                    .collect::<Result<Vec<_>, _>>()
                                    .map_err(|_| "Invalid hand participant id".to_string())?,
                                actions: history_json["actions"].clone(),
                                community_cards: history_json["community_cards"].clone(),
                                loss_deflators: history_json
                                    .get("loss_deflators")
                                    .cloned()
                                    .unwrap_or_else(|| serde_json::json!([])),
                                pot: i64::try_from(history.total_pot)
                                    .map_err(|_| "Hand pot exceeds database range".to_string())?,
                                rake: i64::try_from(history.rake)
                                    .map_err(|_| "Hand rake exceeds database range".to_string())?,
                                reason: format!("{:?}", history.end_reason),
                                small_blind: i64::try_from(table_big_blind / 2).map_err(|_| {
                                    "Small blind exceeds database range".to_string()
                                })?,
                                big_blind: i64::try_from(table_big_blind)
                                    .map_err(|_| "Big blind exceeds database range".to_string())?,
                            })
                        })();
                        match record {
                            Ok(record) => persist_completed_hand(db, record, durable_stacks)
                                .await
                                .map_err(|error| error.to_string()),
                            Err(error) => Err(error),
                        }
                    }
                    (None, _, _, _) => Err("Hand history is unavailable".to_string()),
                    (_, None, _, _) => {
                        Err("Hand-history signing secret is unavailable".to_string())
                    }
                    (_, _, None, _) => Err("Database persistence is unavailable".to_string()),
                    (_, _, _, Err(error)) => Err(error.clone()),
                };
                if let Err(error) = persistence_result {
                    error!(?error, table_id = %self.table_id, "Hand settlement was not committed atomically");
                    self.pause_for_recovery("hand settlement persistence failed")
                        .await;
                    return;
                }
                for (player_id, chips) in memory_stacks {
                    if let Some(table_player) = self
                        .players
                        .iter_mut()
                        .find(|player| player.id == player_id)
                    {
                        table_player.chips = chips;
                    }
                }

                // Um evento público por perdedor elegível (multi-all-in).
                for deflator in &res.loss_deflators {
                    let loser_name = self
                        .players
                        .iter()
                        .find(|p| p.id == deflator.loser_id)
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| deflator.loser_id.clone());
                    let winner_name = self
                        .players
                        .iter()
                        .find(|p| p.id == deflator.winner_id)
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| deflator.winner_id.clone());

                    let final_loser_chips = self
                        .players
                        .iter()
                        .find(|p| p.id == deflator.loser_id)
                        .map(|p| p.chips)
                        .unwrap_or(0);
                    let prevented_elimination = final_loser_chips == deflator.cashback;

                    // Cash games only in this actor path.
                    let is_tournament = false;

                    let deflator_percent = match deflator.tier {
                        poker_engine::loss_deflator::LossDeflatorTier::SevenPercent => 7,
                        poker_engine::loss_deflator::LossDeflatorTier::FifteenPercent => 15,
                        poker_engine::loss_deflator::LossDeflatorTier::TwentyFivePercent => 25,
                        poker_engine::loss_deflator::LossDeflatorTier::ThirtyFivePercent => 35,
                    };
                    let loser_equity_percent = (deflator.loser_equity * 10_000.0).round() / 100.0;
                    let winner_upset_percent = ((1.0 - deflator.loser_equity) * 100.0)
                        .round()
                        .clamp(0.0, 100.0) as u8;

                    let event_payload = serde_json::json!({
                        "type": "deflator_triggered",
                        "loser_name": loser_name,
                        "winner_name": winner_name,
                        "cashback_amount": deflator.cashback,
                        "deflator_percent": deflator_percent,
                        "loser_equity_percent": loser_equity_percent,
                        // Compat: approx chance the winner had of causing the upset.
                        "odds_broken": winner_upset_percent,
                        "opponents_counted": deflator.opponents_counted,
                        "prevented_elimination": prevented_elimination,
                        "is_tournament": is_tournament
                    });

                    let _ = self.tx_broadcast.send(event_payload);
                }
            }
            self.broadcast_state();
            // Iniciar próxima mão depois de 6 segundos
            self.next_hand_at =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(6));
        } else {
            self.broadcast_state();
        }
    }

    async fn start_new_hand(&mut self) {
        if self.persistence_halted {
            return;
        }

        let mut active_players: Vec<(String, u64, usize)> = self
            .players
            .iter()
            .filter(|player| player.is_sitting && player.chips > 0)
            .map(|player| (player.id.clone(), player.chips, player.seat))
            .collect();
        if active_players.len() < 2 {
            return;
        }
        active_players.sort_by_key(|(_, _, seat)| *seat);

        let hand_id = uuid::Uuid::new_v4();
        if let Some(db) = self.db.clone() {
            let table_id = match uuid::Uuid::parse_str(&self.table_id) {
                Ok(table_id) => table_id,
                Err(_) => {
                    self.pause_for_recovery("table id is not a UUID").await;
                    return;
                }
            };
            match sqlx::query(
                "INSERT INTO table_hand_recovery_guards (table_id, hand_id) \
                 VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(table_id)
            .bind(hand_id)
            .execute(&db)
            .await
            {
                Ok(result) if result.rows_affected() == 1 => {}
                Ok(_) => {
                    self.pause_for_recovery("an unrecovered hand guard already exists")
                        .await;
                    return;
                }
                Err(error) => {
                    error!(?error, table_id = %self.table_id, "Failed to persist hand recovery guard");
                    self.pause_for_recovery("could not persist hand recovery guard")
                        .await;
                    return;
                }
            }
        }

        let mut gl = GameLoop::new(
            self.config.clone(),
            hand_id.to_string(),
            self.name.clone(),
            GameType::Cash,
        );
        for (player_id, chips, _) in &active_players {
            gl.add_player(player_id.clone(), *chips);
        }

        // Pick the first occupied seat to the left of the previous physical
        // dealer. Keeping a physical seat avoids repeating or skipping a
        // dealer when the active player list shrinks between hands.
        self.dealer_index = match self.dealer_seat {
            Some(previous_seat) => active_players
                .iter()
                .position(|(_, _, seat)| *seat > previous_seat)
                .unwrap_or(0),
            None => 0,
        };
        self.dealer_seat = Some(active_players[self.dealer_index].2);
        gl.set_dealer(self.dealer_index);

        if let Err(error) = gl.start_hand() {
            error!(?error, table_id = %self.table_id, "Failed to start hand after recovery guard");
            self.pause_for_recovery("could not initialize guarded hand")
                .await;
            return;
        }

        self.game_loop = Some(gl);
        self.last_turn_start = Some(tokio::time::Instant::now());
        info!(table_id = %self.table_id, hand_id = %hand_id, "Started guarded hand");
        self.broadcast_state();
    }

    async fn pause_for_recovery(&mut self, reason: &str) {
        self.persistence_halted = true;
        self.next_hand_at = None;
        error!(table_id = %self.table_id, reason, "Table halted pending recovery review");
        if let (Some(db), Ok(table_id)) = (self.db.clone(), uuid::Uuid::parse_str(&self.table_id)) {
            if let Err(error) = sqlx::query("UPDATE tables SET status = 'PAUSED' WHERE id = $1")
                .bind(table_id)
                .execute(&db)
                .await
            {
                error!(?error, table_id = %self.table_id, "Failed to pause table after persistence failure");
            }
        }
        self.broadcast_state();
    }

    fn get_table_info_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "table_info",
            "table_id": self.table_id,
            "name": self.name,
            "small_blind": self.config.big_blind / 2,
            "big_blind": self.config.big_blind,
            "game_type": "cash",
            "players": self.players
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
                .map(card_to_string)
                .collect();

            // Mapeia os potes
            let main_pot_amount = gl.state.total_pot();
            pots.push(serde_json::json!({
                "name": "Main",
                "amount": main_pot_amount,
                "eligible_players": gl.state.players.iter().map(|p| p.id.clone()).collect::<Vec<String>>()
            }));

            // Jogadores do game loop
            for gp in &gl.state.players {
                if let Some(tp) = self.players.iter().find(|p| p.id == gp.id) {
                    players_json.push(serde_json::json!({
                        "id": gp.id,
                        "name": tp.name,
                        "chips": gp.stack,
                        "bet": gp.current_bet,
                        "cards": gp.hole_cards.iter().map(card_to_string).collect::<Vec<String>>(),
                        "is_active": gl.state.active_player().map(|ap| ap.id == gp.id).unwrap_or(false),
                        "is_dealer": gl.state.dealer_index == gp.seat_index,
                        "seat": tp.seat
                    }));
                }
            }
        } else {
            // Fora da mão (lobby ativo)
            for p in &self.players {
                players_json.push(serde_json::json!({
                    "id": p.id,
                    "name": p.name,
                    "chips": p.chips,
                    "bet": 0.0,
                    "cards": Vec::<String>::new(),
                    "is_active": false,
                    "is_dealer": false,
                    "seat": p.seat
                }));
            }
        }

        let state_payload = serde_json::json!({
            "type": "table_state",
            "table_id": self.table_id,
            "stage": stage,
            "community_cards": community_cards,
            "pots": pots,
            "players": players_json,
            "current_bet_to_match": self.game_loop.as_ref().map(|g| g.state.current_bet_to_match).unwrap_or(0),
            "min_raise": self.game_loop.as_ref().map(|g| g.state.min_raise).unwrap_or(self.config.big_blind),
            "is_finished": is_finished
        });

        let _ = self.tx_broadcast.send(state_payload);
    }

    /// Salva um snapshot do estado atual da mesa no Redis (TTL de 1 hora)
    pub async fn save_snapshot(&mut self) {
        if let Some(ref mut redis) = self.redis {
            use redis::AsyncCommands;
            let key = format!("poker:table:state:{}", self.table_id);
            let snapshot = serde_json::json!({
                "table_id": self.table_id,
                "name": self.name,
                "dealer_index": self.dealer_index,
                "players": self.players,
                "is_finished": self.game_loop.as_ref().map(|g| g.state.is_finished).unwrap_or(true),
                "updated_at": chrono::Utc::now().to_rfc3339()
            });

            if let Ok(json_str) = serde_json::to_string(&snapshot) {
                let _: Result<(), _> = redis.set_ex(key, json_str, 3600).await;
            }
        }
    }
}

fn card_to_string(card: &poker_engine::deck::Card) -> String {
    let rank_str = match card.rank {
        poker_engine::deck::Rank::Ace => "A",
        poker_engine::deck::Rank::King => "K",
        poker_engine::deck::Rank::Queen => "Q",
        poker_engine::deck::Rank::Jack => "J",
        poker_engine::deck::Rank::Ten => "T",
        poker_engine::deck::Rank::Nine => "9",
        poker_engine::deck::Rank::Eight => "8",
        poker_engine::deck::Rank::Seven => "7",
        poker_engine::deck::Rank::Six => "6",
        poker_engine::deck::Rank::Five => "5",
        poker_engine::deck::Rank::Four => "4",
        poker_engine::deck::Rank::Three => "3",
        poker_engine::deck::Rank::Two => "2",
    };
    let suit_str = match card.suit {
        poker_engine::deck::Suit::Hearts => "h",
        poker_engine::deck::Suit::Diamonds => "d",
        poker_engine::deck::Suit::Clubs => "c",
        poker_engine::deck::Suit::Spades => "s",
    };
    format!("{}{}", rank_str, suit_str)
}

#[cfg(test)]
mod tests {
    use super::{TableActor, TablePlayer};
    use tokio::sync::{broadcast, mpsc};

    fn player(id: &str, seat: usize) -> TablePlayer {
        TablePlayer {
            id: id.to_string(),
            name: id.to_string(),
            chips: 10_000,
            seat,
            is_sitting: true,
        }
    }

    #[test]
    fn cash_out_returns_persistable_stack_between_hands() {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        );
        actor.players.push(TablePlayer {
            chips: 12_345,
            ..player("player", 0)
        });

        assert_eq!(actor.handle_cash_out("player"), Ok(Some(12_345)));
        assert!(actor.players.is_empty());
    }

    #[tokio::test]
    async fn dealer_rotation_follows_physical_seats_after_a_player_leaves() {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        );
        actor.players = vec![player("a", 0), player("b", 3), player("c", 7)];

        actor.start_new_hand().await;
        assert_eq!(actor.dealer_seat, Some(0));
        actor.game_loop = None;

        actor.start_new_hand().await;
        assert_eq!(actor.dealer_seat, Some(3));
        actor.game_loop = None;

        actor.players.retain(|table_player| table_player.seat != 3);
        actor.start_new_hand().await;
        assert_eq!(actor.dealer_seat, Some(7));
    }

    #[tokio::test]
    async fn overdue_turn_is_folded_by_the_actor() {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        )
        .with_turn_timeout(tokio::time::Duration::from_millis(1));
        actor.players = vec![player("a", 0), player("b", 1), player("c", 2)];
        actor.start_new_hand().await;
        let active_before = actor
            .game_loop
            .as_ref()
            .expect("hand should start")
            .state
            .active_player()
            .expect("hand should have an active player")
            .id
            .clone();
        actor.last_turn_start = Some(
            tokio::time::Instant::now()
                .checked_sub(tokio::time::Duration::from_secs(1))
                .expect("instant supports one second subtraction"),
        );

        actor.tick().await;

        let game_loop = actor
            .game_loop
            .as_ref()
            .expect("hand should remain available");
        assert!(
            game_loop.state.is_finished
                || game_loop
                    .state
                    .active_player()
                    .is_some_and(|player| player.id != active_before)
        );
    }
}
