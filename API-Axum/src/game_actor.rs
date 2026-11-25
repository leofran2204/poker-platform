// game_actor.rs — Ator para orquestração da mesa de Poker em tempo real.
//
// Gerencia a conexão de múltiplos jogadores, as transições do GameLoop
// e a transmissão das cartas e apostas via WebSockets.

use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{error, info};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use poker_engine::game_loop::{GameLoop, GameLoopError, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::types::TableConfig;

const DEFAULT_TURN_TIMEOUT: tokio::time::Duration = crate::time_bank::TURN;
/// After the socket drops, keep the seat briefly so a refresh can reconnect.
/// Then cash out so the lobby cannot stay full of ghost players.
const DEFAULT_DISCONNECT_CASH_OUT_AFTER: tokio::time::Duration =
    tokio::time::Duration::from_secs(45);

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
    /// Socket caiu: reserva o assento (cash-out em 45s) sem fold imediato.
    Disconnect {
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
    /// Mantém o assento; não joga a próxima mão. Sem timer de cash-out.
    SetSitting {
        player_id: String,
        sitting: bool,
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
    #[serde(skip)]
    pub disconnected_since: Option<tokio::time::Instant>,
    /// Leave explícito no meio da mão: fold imediato no próximo tick.
    /// Drop de socket (`Disconnect`) não marca: graça sem fold imediato.
    #[serde(skip)]
    pub left_hand: bool,
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
    pub time_banks: crate::time_bank::TimeBanks,
    pub disconnect_cash_out_after: tokio::time::Duration,
    pub db: Option<sqlx::PgPool>,
    pub redis: Option<redis::aio::ConnectionManager>,
    pub audit_secret: Option<String>,
    pub persistence_halted: bool,
    /// Vencedores da última mão liquidada (ids, p/ revelar no showdown).
    pub last_winners: Vec<String>,
}

struct HandHistoryRecord {
    hand_id: uuid::Uuid,
    table_id: uuid::Uuid,
    participants: Vec<uuid::Uuid>,
    actions: serde_json::Value,
    community_cards: serde_json::Value,
    loss_deflators: serde_json::Value,
    settlement: serde_json::Value,
    settlement_signature: String,
    winner_player_id: Option<String>,
    pot: i64,
    rake: i64,
    reason: String,
    small_blind: i64,
    big_blind: i64,
}

pub(crate) fn sign_settlement(
    settlement: &serde_json::Value,
    secret: &[u8],
) -> Result<String, String> {
    let payload = serde_json::to_vec(settlement)
        .map_err(|error| format!("Could not serialize settlement: {error}"))?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|_| "Invalid settlement signing secret".to_string())?;
    mac.update(&payload);
    Ok(mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Monta o bloco `showdown` do broadcast a partir do hand history finalizado:
/// só quem não foldou e tem melhor mão avaliada, com as cartas do jogo.
/// Nada vaza: o filtro do WS já revela esses mesmos jogadores.
pub(crate) fn showdown_reveal(
    history: &poker_engine::hand_history::HandHistory,
    name_of: &dyn Fn(&str) -> String,
) -> Vec<serde_json::Value> {
    history
        .results
        .iter()
        .filter(|result| !result.folded)
        .filter_map(|result| {
            let best = result.best_hand.as_ref()?;
            let mut cards: Vec<String> = best.cards.iter().map(card_to_string).collect();
            cards.extend(best.kickers.iter().map(card_to_string));
            Some(serde_json::json!({
                "player_id": result.player_id,
                "player_name": name_of(&result.player_id),
                "hand_name": result.best_hand_name,
                "cards": cards,
            }))
        })
        .collect()
}

pub(crate) fn settlement_signature_valid(
    settlement: &serde_json::Value,
    signature: &str,
    secret: &[u8],
) -> bool {
    sign_settlement(settlement, secret)
        .is_ok_and(|expected| expected.as_bytes().ct_eq(signature.as_bytes()).into())
}

fn validate_chip_conservation(
    starting_total: u128,
    final_total: u128,
    pot_total: u128,
    payout_total: u128,
    rake: u128,
) -> Result<(), String> {
    if payout_total.checked_add(rake) != Some(pot_total) {
        return Err("Payouts plus rake do not equal the pot".to_string());
    }
    if final_total.checked_add(rake) != Some(starting_total) {
        return Err("Final stacks plus rake do not equal starting stacks".to_string());
    }
    Ok(())
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
    let inserted = sqlx::query(
        "INSERT INTO hand_history (id, table_id, hand_number, game_type, small_blind, big_blind, actions_json, community_cards_json, loss_deflators_json, settlement_json, settlement_signature, winner_player_id, pot_total, rake_collected, end_reason) \
         VALUES ($1, $2, $3, 'cash', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
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
    .bind(record.settlement)
    .bind(record.settlement_signature)
    .bind(record.winner_player_id)
    .bind(record.pot)
    .bind(record.rake)
    .bind(record.reason)
    .execute(&mut *tx)
    .await?;
    if inserted.rows_affected() == 0 {
        // Uma repetição da persistência não pode creditar novamente clube/agente.
        tx.rollback().await?;
        return Ok(());
    }

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
            let platform_fee = crate::estrutura::commission(record.rake, 15);
            let club_rake = record.rake.saturating_sub(platform_fee);
            sqlx::query("UPDATE clubs SET balance = balance + $1 WHERE id = $2")
                .bind(club_rake)
                .bind(c_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    let participants = record.participants.clone();
    let hand_id = record.hand_id;
    let rake = record.rake;
    for user_id in record.participants {
        sqlx::query(
            "INSERT INTO hand_participants (hand_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(record.hand_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }
    if rake > 0 && !participants.is_empty() {
        crate::estrutura::distribute_hand_rake(&mut tx, hand_id, &participants, rake).await?;
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
            time_banks: crate::time_bank::TimeBanks::default(),
            disconnect_cash_out_after: DEFAULT_DISCONNECT_CASH_OUT_AFTER,
            db: None,
            redis: None,
            persistence_halted: false,
            audit_secret: None,
            last_winners: Vec::new(),
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
    /// Mesa sem banco de tempo: o turno acaba no tempo normal.
    pub fn without_time_bank(mut self) -> Self {
        self.time_banks = crate::time_bank::TimeBanks::disabled();
        self
    }

    pub fn with_turn_timeout(mut self, turn_timeout: tokio::time::Duration) -> Self {
        self.turn_timeout = turn_timeout;
        self
    }

    #[cfg(test)]
    pub fn with_disconnect_cash_out_after(mut self, after: tokio::time::Duration) -> Self {
        self.disconnect_cash_out_after = after;
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
            PlayerCommand::Disconnect { player_id } => {
                self.handle_disconnect(player_id);
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
            PlayerCommand::SetSitting { player_id, sitting } => {
                self.handle_set_sitting(player_id, sitting);
                self.save_snapshot().await;
            }
        }
    }

    async fn tick(&mut self) {
        if self.persistence_halted {
            return;
        }

        let leaver_active_player = self.game_loop.as_ref().and_then(|game_loop| {
            (!game_loop.state.is_finished)
                .then(|| {
                    game_loop
                        .state
                        .active_player()
                        .map(|player| player.id.clone())
                })
                .flatten()
                .filter(|player_id| {
                    self.players
                        .iter()
                        .find(|player| player.id == *player_id)
                        .is_none_or(|player| {
                            !player.is_sitting
                                && (player.disconnected_since.is_none() || player.left_hand)
                        })
                })
        });
        if let Some(player_id) = leaver_active_player {
            if let Some(player) = self
                .players
                .iter_mut()
                .find(|player| player.id == player_id)
            {
                player.left_hand = false;
            }
            // Saída voluntária da mesa: desiste da mão em qualquer modalidade
            // (o check automático é para quem ficou sem tempo ou caiu da conexão).
            tracing::warn!(
                table_id = %self.table_id,
                player_id = %player_id,
                action = "fold",
                "Player out of seat during active hand; folding"
            );
            self.handle_action(player_id, "fold".to_string(), 0).await;
            return;
        }
        let disconnected_active_player = self.game_loop.as_ref().and_then(|game_loop| {
            (!game_loop.state.is_finished)
                .then(|| {
                    game_loop
                        .state
                        .active_player()
                        .map(|player| player.id.clone())
                })
                .flatten()
                .filter(|player_id| {
                    self.players
                        .iter()
                        .find(|player| player.id == *player_id)
                        .is_none_or(|player| {
                            !player.is_sitting && player.disconnected_since.is_none()
                        })
                })
        });
        if let Some(player_id) = disconnected_active_player {
            let automatic = self.automatic_timeout_action();
            tracing::warn!(
                table_id = %self.table_id,
                player_id = %player_id,
                action = automatic,
                "Disconnected active player; applying automatic action"
            );
            self.handle_automatic_action(player_id).await;
            return;
        }

        // O turno vai até o tempo normal mais o tempo extra de quem está
        // conectado.
        let timed_out_player = self.game_loop.as_ref().and_then(|game_loop| {
            if game_loop.state.is_finished {
                return None;
            }
            let started_at = self.last_turn_start?;
            let player_id = game_loop.state.active_player()?.id.clone();
            let limit = self
                .time_banks
                .turn_limit(self.turn_timeout, self.may_use_time_bank(&player_id));
            (started_at.elapsed() >= limit).then_some(player_id)
        });
        if let Some(player_id) = timed_out_player {
            // Tempo esgotado: check sem aposta a pagar, desistência com aposta
            // (no Brazilian Pineapple, mantém as cartas antes do check).
            let automatic = self.automatic_timeout_action();
            tracing::warn!(
                table_id = %self.table_id,
                player_id = %player_id,
                timeout_seconds = self.turn_timeout.as_secs(),
                action = automatic,
                "Player action timed out; applying automatic action"
            );
            self.handle_automatic_action(player_id).await;
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

        self.cash_out_expired_disconnects().await;
    }

    fn handle_sit(
        &mut self,
        player_id: String,
        username: String,
        seat: Option<usize>,
        chips: u64,
    ) -> usize {
        if let Some(existing) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            existing.name = username;
            existing.chips = chips;
            if let Some(seat) = seat {
                existing.seat = seat;
            }
            existing.is_sitting = true;
            existing.disconnected_since = None;
            existing.left_hand = false;
            let assigned_seat = existing.seat;
            info!(
                "Player reconnected at table {} in seat {}",
                self.table_id, assigned_seat
            );
            self.broadcast_state();
            return assigned_seat;
        }

        let assigned_seat = match seat {
            Some(s) => s,
            None => {
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
            disconnected_since: None,
            left_hand: false,
        });

        info!(
            "Player sat at table {} in seat {}",
            self.table_id, assigned_seat
        );
        self.broadcast_state();
        assigned_seat
    }

    fn handle_leave(&mut self, player_id: String) {
        let player_is_in_active_hand = self.game_loop.as_ref().is_some_and(|game_loop| {
            !game_loop.state.is_finished
                && game_loop
                    .state
                    .players
                    .iter()
                    .any(|player| player.id == player_id)
        });
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            player.is_sitting = false;
            // Leave explícito: fold imediato (consome no tick) + graça de
            // assento para o cash-out. Drop de socket usa handle_disconnect.
            player.left_hand = true;
            if player.disconnected_since.is_none() {
                player.disconnected_since = Some(tokio::time::Instant::now());
            }
        }
        if player_is_in_active_hand {
            info!(
                "Player {} disconnected from active hand at table {}; automatic fold scheduled",
                player_id, self.table_id
            );
        } else {
            info!(
                "Player {} disconnected from table {}; seat reserved until cash-out grace",
                player_id, self.table_id
            );
        }

        self.broadcast_state();
    }

    fn handle_disconnect(&mut self, player_id: String) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            player.is_sitting = false;
            if player.disconnected_since.is_none() {
                player.disconnected_since = Some(tokio::time::Instant::now());
            }
        }
        info!(
            "Player {} socket dropped at table {}; seat reserved until cash-out grace",
            player_id, self.table_id
        );
        self.broadcast_state();
    }

    fn handle_set_sitting(&mut self, player_id: String, sitting: bool) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            player.is_sitting = sitting;
            if sitting {
                player.disconnected_since = None;
            }
        }
        self.broadcast_state();
    }

    async fn cash_out_expired_disconnects(&mut self) {
        if self.persistence_halted {
            return;
        }
        let in_unfinished_hand: std::collections::HashSet<String> = self
            .game_loop
            .as_ref()
            .filter(|game_loop| !game_loop.state.is_finished)
            .map(|game_loop| {
                game_loop
                    .state
                    .players
                    .iter()
                    .map(|player| player.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        let now = tokio::time::Instant::now();
        let due: Vec<(String, u64)> = self
            .players
            .iter()
            .filter(|player| {
                !in_unfinished_hand.contains(&player.id)
                    && player.disconnected_since.is_some_and(|since| {
                        now.saturating_duration_since(since) >= self.disconnect_cash_out_after
                    })
            })
            .map(|player| (player.id.clone(), player.chips))
            .collect();
        if due.is_empty() {
            return;
        }

        let table_uuid = uuid::Uuid::parse_str(&self.table_id).ok();
        for (player_id, chips) in due {
            if let (Some(db), Some(table_id)) = (self.db.as_ref(), table_uuid) {
                match crate::cash_seats::persist_cash_out_seat(
                    db,
                    table_id,
                    &player_id,
                    Some(chips),
                )
                .await
                {
                    Ok(_) => {}
                    Err(error) => {
                        error!(
                            ?error,
                            table_id = %self.table_id,
                            player_id = %player_id,
                            "Failed to persist disconnect cash-out"
                        );
                        continue;
                    }
                }
            }
            let _ = self.handle_cash_out(&player_id);
            info!(
                table_id = %self.table_id,
                player_id = %player_id,
                "Cashed out disconnected player after grace period"
            );
        }
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

    pub(crate) async fn handle_action(&mut self, player_id: String, action: String, amount: u64) {
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
        let table_small_blind = self.config.effective_small_blind();

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
            "keep" => PlayerMove::Keep,
            // Brazilian Pineapple: `amount` carrega a posição da carta (0 a 2).
            "swap" => PlayerMove::Swap(amount as usize),
            "discard" => PlayerMove::Discard(Some(amount as usize)),
            // Descarte sem escolha: o motor guarda as duas melhores cartas.
            "auto_discard" => PlayerMove::Discard(None),
            _ => {
                error!("Invalid action string: {}", action);
                return;
            }
        };

        let draw_step = matches!(
            player_move,
            PlayerMove::Keep | PlayerMove::Swap(_) | PlayerMove::Discard(_)
        );
        if let Err(e) = game_loop.player_action(&player_id, player_move) {
            // Mão travada all-in (comum quando blinds superam stacks): quem não
            // pode agir nunca completa fold — corre o board e cai no fluxo de
            // liquidação abaixo em vez de girar em erro para sempre.
            let stalled =
                matches!(&e, GameLoopError::PlayerCannotAct(_)) && game_loop.run_out_stalled_hand();
            if !stalled {
                error!("Error processing action for player {}: {}", player_id, e);
                return;
            }
        }

        // Depois da troca (ou do descarte) a vez continua com o mesmo jogador
        // e o relógio segue correndo até a aposta.
        if !continues_same_turn(game_loop, &player_id, draw_step) {
            self.last_turn_start =
                (!game_loop.state.is_finished).then_some(tokio::time::Instant::now());
        }

        if game_loop.state.is_finished {
            // Resolver a mão fora das threads assíncronas (ver `resolve_hand_off_runtime`).
            let resolution = resolve_hand_off_runtime(&mut self.game_loop).await;
            let Some(game_loop) = self.game_loop.as_mut() else {
                return;
            };
            if let Some(Ok(res)) = resolution {
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
                self.last_winners = res.payouts.keys().cloned().collect::<Vec<_>>();
                let starting_total = game_loop
                    .history
                    .as_ref()
                    .map(|history| {
                        history
                            .starting_stacks
                            .values()
                            .map(|chips| u128::from(*chips))
                            .sum()
                    })
                    .ok_or_else(|| "Hand history is unavailable".to_string());
                let final_total: u128 = memory_stacks
                    .iter()
                    .map(|(_, chips)| u128::from(*chips))
                    .sum();
                let payout_total: u128 = res.payouts.values().map(|chips| u128::from(*chips)).sum();
                let settlement_validation = starting_total.and_then(|starting_total| {
                    validate_chip_conservation(
                        starting_total,
                        final_total,
                        u128::from(game_loop.state.total_pot()),
                        payout_total,
                        u128::from(res.rake),
                    )
                });
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
                    settlement_validation.as_ref(),
                ) {
                    (Some(history), Some(audit_secret), Some(db), Ok(durable_stacks), Ok(())) => {
                        poker_engine::hand_history::sign_hand(history, audit_secret.as_bytes());
                        let record = (|| -> Result<HandHistoryRecord, String> {
                            let history_json = serde_json::to_value(&*history)
                                .map_err(|_| "Could not serialize hand history".to_string())?;
                            let mut payouts = res
                                .payouts
                                .iter()
                                .map(|(player_id, amount)| {
                                    serde_json::json!({
                                        "player_id": player_id,
                                        "amount": amount,
                                    })
                                })
                                .collect::<Vec<_>>();
                            payouts.sort_by(|left, right| {
                                left["player_id"].as_str().cmp(&right["player_id"].as_str())
                            });
                            let mut players = history
                                .results
                                .iter()
                                .map(|result| {
                                    let final_stack = memory_stacks
                                        .iter()
                                        .find(|(player_id, _)| player_id == &result.player_id)
                                        .map(|(_, chips)| *chips)
                                        .ok_or_else(|| {
                                            "Final stack missing from settlement".to_string()
                                        })?;
                                    Ok(serde_json::json!({
                                        "player_id": result.player_id,
                                        "finish_position": result.finish_position,
                                        "chips_won": result.chips_won,
                                        "chips_lost": result.chips_lost,
                                        "folded": result.folded,
                                        "was_all_in": result.was_all_in,
                                        "best_hand_name": result.best_hand_name,
                                        "final_stack": final_stack,
                                    }))
                                })
                                .collect::<Result<Vec<_>, String>>()?;
                            players.sort_by(|left, right| {
                                left["player_id"].as_str().cmp(&right["player_id"].as_str())
                            });
                            let loss_deflators = history_json
                                .get("loss_deflators")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!([]));
                            let settlement = serde_json::json!({
                                "version": 1,
                                "betting_rule_version": history.betting_rule_version,
                                "hand_id": history.hand_id,
                                "table_id": table_id,
                                "pot_total": history.total_pot,
                                "rake_collected": history.rake,
                                "end_reason": history.end_reason.as_str(),
                                "payouts": payouts,
                                "players": players,
                                "loss_deflators": loss_deflators,
                            });
                            let settlement_signature =
                                sign_settlement(&settlement, audit_secret.as_bytes())?;
                            let winner_player_id = history
                                .results
                                .iter()
                                .filter(|result| {
                                    result.finish_position == 1 && result.chips_won > 0
                                })
                                .map(|result| result.player_id.clone())
                                .min();
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
                                loss_deflators,
                                settlement,
                                settlement_signature,
                                winner_player_id,
                                pot: i64::try_from(history.total_pot)
                                    .map_err(|_| "Hand pot exceeds database range".to_string())?,
                                rake: i64::try_from(history.rake)
                                    .map_err(|_| "Hand rake exceeds database range".to_string())?,
                                reason: format!("{:?}", history.end_reason),
                                small_blind: i64::try_from(table_small_blind).map_err(|_| {
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
                    (None, _, _, _, _) => Err("Hand history is unavailable".to_string()),
                    (_, None, _, _, _) => {
                        Err("Hand-history signing secret is unavailable".to_string())
                    }
                    (_, _, None, _, _) => Err("Database persistence is unavailable".to_string()),
                    (_, _, _, Err(error), _) => Err(error.clone()),
                    (_, _, _, _, Err(error)) => Err(error.clone()),
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
                self.players
                    .retain(|player| player.is_sitting || player.disconnected_since.is_none());
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
        self.last_winners.clear();
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

    /// Ação aplicada quando o tempo do jogador da vez acaba.
    /// Só quem está conectado à mesa ganha o tempo extra.
    fn may_use_time_bank(&self, player_id: &str) -> bool {
        self.players
            .iter()
            .find(|player| player.id == player_id)
            .is_some_and(|player| player.is_sitting && player.disconnected_since.is_none())
    }

    /// Segundos de tempo extra que o jogador da vez ainda tem neste instante.
    fn active_time_bank_reserve_secs(&self) -> u64 {
        let Some(game_loop) = self.game_loop.as_ref().filter(|g| !g.state.is_finished) else {
            return 0;
        };
        let (Some(player), Some(started)) = (game_loop.state.active_player(), self.last_turn_start)
        else {
            return 0;
        };
        if !self.may_use_time_bank(&player.id) {
            return 0;
        }
        self.time_banks
            .live(started.elapsed(), self.turn_timeout)
            .as_secs()
    }

    fn automatic_timeout_action(&self) -> &'static str {
        automatic_timeout_action(self.game_loop.as_ref())
    }

    /// Aplica a ação automática completa da vez: no Brazilian Pineapple, o
    /// passo de troca (manter ou descarte automático) e logo depois o check ou
    /// a desistência, sem esperar outro tempo esgotado.
    pub(crate) async fn handle_automatic_action(&mut self, player_id: String) {
        // Até três passos na mesma vez: manter, descartar (river) e check.
        for _ in 0..3 {
            let automatic = self.automatic_timeout_action();
            self.handle_action(player_id.clone(), automatic.to_string(), 0)
                .await;
            if !needs_second_automatic_step(self.game_loop.as_ref(), &player_id, automatic) {
                break;
            }
        }
    }

    fn get_table_info_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "table_info",
            "table_id": self.table_id,
            "name": self.name,
            "small_blind": self.config.effective_small_blind(),
            "big_blind": self.config.big_blind,
            "game_type": "cash",
            "players": self.players
        })
    }

    pub(crate) fn broadcast_state(&self) {
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
                        "_legal_actions": gl.legal_actions(&gp.id),
                        "cards": gp.hole_cards.iter().map(card_to_string).collect::<Vec<String>>(),
                        "folded": gp.has_folded,
                        "draw_count": gp.draw_count,
                        "river_discard": gp.river_discard.as_ref().map(card_to_string),
                        "is_active": gl.state.active_player().map(|ap| ap.id == gp.id).unwrap_or(false),
                        "is_dealer": gl.state.dealer_index == gp.seat_index,
                        "seat": tp.seat,
                        "is_sitting": tp.is_sitting
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
                    "seat": p.seat,
                    "is_sitting": p.is_sitting
                }));
            }
        }

        let state_payload = serde_json::json!({
            "type": "table_state",
            "table_id": self.table_id,
            "stage": stage,
            "drawing": self.game_loop.as_ref().is_some_and(|g| g.state.drawing && !g.state.is_finished),
            "discarding": self.game_loop.as_ref().is_some_and(|g| g.state.discarding && !g.state.is_finished),
            "betting_structure": self.game_loop.as_ref().map(|g| g.betting_structure()),
            "ante": self.game_loop.as_ref().map(|g| g.nominal_ante()).unwrap_or(0),
            "ante_paid": self.game_loop.as_ref().map(|g| g.ante_paid()).unwrap_or(0),
            "ante_player_id": self.game_loop.as_ref().and_then(|g| g.ante_player_id()),
            "hand_id": self.game_loop.as_ref().map(|g| &g.hand_id),
            "community_cards": community_cards,
            "pots": pots,
            "players": players_json,
            "winners": if is_finished { self.last_winners.clone() } else { Vec::<String>::new() },
            "showdown": self.game_loop.as_ref().and_then(|gl| {
                if !is_finished {
                    return None;
                }
                gl.history.as_ref().map(|history| {
                    showdown_reveal(history, &|player_id| {
                        self.players
                            .iter()
                            .find(|player| player.id == player_id)
                            .map(|player| player.name.clone())
                            .unwrap_or_else(|| player_id.to_string())
                    })
                })
            }).unwrap_or_default(),
            "current_bet_to_match": self.game_loop.as_ref().map(|g| g.state.current_bet_to_match).unwrap_or(0),
            "min_raise": self.game_loop.as_ref().map(|g| g.state.min_raise).unwrap_or(self.config.big_blind),
            "is_finished": is_finished,
            "time_bank": self.last_turn_start.map(|started| {
                self.turn_timeout.saturating_sub(started.elapsed()).as_secs()
            }).unwrap_or(0),
            "time_bank_reserve": self.active_time_bank_reserve_secs()
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
                "updated_at": crate::clock::now_rfc3339()
            });

            if let Ok(json_str) = serde_json::to_string(&snapshot) {
                let _: Result<(), _> = redis.set_ex(key, json_str, 3600).await;
            }
        }
    }
}

/// Resolve a mão numa thread de bloqueio (`spawn_blocking`). No Omaha e no
/// Brazilian Pineapple, o Loss Deflator de um all-in pré-flop multiway pode levar
/// segundos; assim o cálculo não ocupa as threads que atendem as outras mesas e
/// as conexões. A própria mesa espera o resultado para pagar a mão. `None` só se
/// não houver mão ou se o servidor estiver desligando.
pub(crate) async fn resolve_hand_off_runtime(
    slot: &mut Option<GameLoop>,
) -> Option<Result<poker_engine::game_loop::HandResolution, GameLoopError>> {
    let mut game_loop = slot.take()?;
    let joined = tokio::task::spawn_blocking(move || {
        let resolution = game_loop.resolve_hand();
        (game_loop, resolution)
    })
    .await;
    match joined {
        Ok((game_loop, resolution)) => {
            *slot = Some(game_loop);
            Some(resolution)
        }
        // Mesmo comportamento de antes: um pânico no motor derruba o ator.
        Err(error) => match error.try_into_panic() {
            Ok(panic) => std::panic::resume_unwind(panic),
            Err(_) => None,
        },
    }
}

/// Ação automática (tempo esgotado, saída ou desconexão), igual em todas as
/// modalidades: com aposta a pagar, desiste; sem aposta, dá check. No
/// Brazilian Pineapple, com aposta a pagar, desiste já no passo de troca; sem
/// aposta, mantém as cartas, no river descarta deixando as duas melhores (o
/// motor escolhe) e em seguida dá check.
pub(crate) fn automatic_timeout_action(
    game_loop: Option<&poker_engine::game_loop::GameLoop>,
) -> &'static str {
    let Some(gl) = game_loop.filter(|gl| !gl.state.is_finished) else {
        return "fold";
    };
    let Some(active) = gl.state.active_player() else {
        return "fold";
    };
    if gl.state.current_bet_to_match > active.current_bet {
        "fold"
    } else if gl.state.drawing {
        "keep"
    } else if gl.state.discarding {
        "auto_discard"
    } else {
        "check"
    }
}

/// Brazilian Pineapple: depois da troca (ou do descarte) a vez continua com o
/// mesmo jogador, agora nas apostas.
pub(crate) fn continues_same_turn(
    game_loop: &poker_engine::game_loop::GameLoop,
    player_id: &str,
    draw_step: bool,
) -> bool {
    draw_step
        && !game_loop.state.is_finished
        && game_loop
            .state
            .active_player()
            .is_some_and(|p| p.id == player_id)
}

/// A ação automática foi só o passo de troca (ou de descarte) e o jogador
/// ainda tem a vez: falta o descarte do river ou o check (ou a desistência).
pub(crate) fn needs_second_automatic_step(
    game_loop: Option<&poker_engine::game_loop::GameLoop>,
    player_id: &str,
    automatic: &str,
) -> bool {
    matches!(automatic, "keep" | "auto_discard")
        && game_loop.is_some_and(|gl| continues_same_turn(gl, player_id, true))
}

pub(crate) fn card_to_string(card: &poker_engine::deck::Card) -> String {
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
    use super::{
        automatic_timeout_action, settlement_signature_valid, sign_settlement,
        validate_chip_conservation, TableActor, TablePlayer,
    };
    use tokio::sync::{broadcast, mpsc};

    fn player(id: &str, seat: usize) -> TablePlayer {
        TablePlayer {
            id: id.to_string(),
            name: id.to_string(),
            chips: 10_000,
            seat,
            is_sitting: true,
            disconnected_since: None,
            left_hand: false,
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
        .with_turn_timeout(tokio::time::Duration::from_millis(1))
        .without_time_bank();
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

    /// Hold'em com três jogadores: os dois primeiros pagam o big blind e a vez
    /// fica com o big blind, sem aposta a pagar.
    async fn holdem_big_blind_option() -> (TableActor, String) {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        )
        .with_turn_timeout(tokio::time::Duration::from_millis(1))
        .without_time_bank();
        actor.players = vec![player("a", 0), player("b", 1), player("c", 2)];
        actor.start_new_hand().await;
        for _ in 0..2 {
            let caller = active_id(&actor).expect("hand should have an active player");
            assert_eq!(automatic_timeout_action(actor.game_loop.as_ref()), "fold");
            actor.handle_action(caller, "call".to_string(), 0).await;
        }
        let big_blind = active_id(&actor).expect("big blind has the option");
        (actor, big_blind)
    }

    fn has_folded(actor: &TableActor, player_id: &str) -> bool {
        actor
            .game_loop
            .as_ref()
            .and_then(|gl| gl.state.players.iter().find(|p| p.id == player_id))
            .is_some_and(|p| p.has_folded)
    }

    #[tokio::test]
    async fn overdue_turn_without_a_bet_to_pay_checks_in_every_variant() {
        let (mut actor, big_blind) = holdem_big_blind_option().await;
        assert_eq!(automatic_timeout_action(actor.game_loop.as_ref()), "check");
        actor.last_turn_start = Some(
            tokio::time::Instant::now()
                .checked_sub(tokio::time::Duration::from_secs(1))
                .expect("instant supports one second subtraction"),
        );

        actor.tick().await;

        let gl = actor.game_loop.as_ref().expect("hand continues");
        assert_eq!(gl.state.phase, poker_engine::types::GamePhase::Flop);
        assert!(!has_folded(&actor, &big_blind));
    }

    #[tokio::test]
    async fn leaving_the_table_folds_even_without_a_bet_to_pay() {
        let (mut actor, big_blind) = holdem_big_blind_option().await;

        actor.handle_leave(big_blind.clone());
        actor.tick().await;

        assert!(has_folded(&actor, &big_blind));
    }

    /// Mesa com mão iniciada e o turno do jogador da vez aberto há `turn_age` segundos.
    async fn actor_mid_turn(turn_secs: u64, turn_age: u64) -> (TableActor, String) {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        )
        .with_turn_timeout(tokio::time::Duration::from_secs(turn_secs));
        actor.players = vec![player("a", 0), player("b", 1), player("c", 2)];
        actor.start_new_hand().await;
        let active = active_id(&actor).expect("hand should have an active player");
        actor.last_turn_start = Some(
            tokio::time::Instant::now()
                .checked_sub(tokio::time::Duration::from_secs(turn_age))
                .expect("instant supports the subtraction"),
        );
        (actor, active)
    }

    fn active_id(actor: &TableActor) -> Option<String> {
        let game_loop = actor.game_loop.as_ref()?;
        (!game_loop.state.is_finished)
            .then(|| game_loop.state.active_player().map(|p| p.id.clone()))
            .flatten()
    }

    #[tokio::test]
    async fn extra_time_keeps_a_connected_player_in_the_hand_past_the_normal_time() {
        // Turno de 10 s aberto há 13 s: 3 s além do normal, dentro dos 5 s extras.
        let (mut actor, active) = actor_mid_turn(10, 13).await;

        actor.tick().await;

        assert_eq!(active_id(&actor).as_deref(), Some(active.as_str()));
        assert!((1..=2).contains(&actor.active_time_bank_reserve_secs()));

        // Ao agir, a próxima vez começa com os 5 s extras inteiros: não acumula
        // nem é descontado.
        actor
            .handle_action(active.clone(), "fold".to_string(), 0)
            .await;
        let next = active_id(&actor).expect("hand continues with two players");
        assert_ne!(next, active);
        assert_eq!(actor.active_time_bank_reserve_secs(), 5);
    }

    #[tokio::test]
    async fn extra_time_runs_out_and_the_actor_acts_for_the_player() {
        // 10 s normais + 5 s extras: aos 16 s o turno acabou.
        let (mut actor, active) = actor_mid_turn(10, 16).await;

        actor.tick().await;

        assert_ne!(active_id(&actor).as_deref(), Some(active.as_str()));
        // O tempo extra renova: a próxima vez dele volta a ter 10 + 5 s.
        assert_eq!(
            actor
                .time_banks
                .turn_limit(actor.turn_timeout, actor.may_use_time_bank(&active)),
            actor.turn_timeout + crate::time_bank::EXTRA
        );
    }

    #[tokio::test]
    async fn disconnected_player_does_not_get_extra_time() {
        let (mut actor, active) = actor_mid_turn(10, 11).await;
        let seat = actor
            .players
            .iter_mut()
            .find(|p| p.id == active)
            .expect("active player is seated");
        seat.is_sitting = false;
        seat.disconnected_since = Some(tokio::time::Instant::now());

        actor.tick().await;

        assert_ne!(active_id(&actor).as_deref(), Some(active.as_str()));
    }

    #[tokio::test]
    async fn table_state_reports_the_turn_time_and_the_extra_time() {
        let (tx_cmd, rx_cmd) = mpsc::channel(1);
        drop(tx_cmd);
        let (tx_broadcast, mut rx_broadcast) = broadcast::channel(8);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        );
        actor.players = vec![player("a", 0), player("b", 1), player("c", 2)];
        actor.start_new_hand().await;
        let mut state = serde_json::Value::Null;
        while let Ok(message) = rx_broadcast.try_recv() {
            state = message;
        }
        assert!((14..=15).contains(&state["time_bank"].as_u64().unwrap()));
        assert_eq!(state["time_bank_reserve"], 5);
    }

    #[test]
    fn settlement_signature_changes_when_payment_changes() {
        let first = serde_json::json!({
            "hand_id": "hand",
            "payouts": [{"player_id": "a", "amount": 300}],
        });
        let second = serde_json::json!({
            "hand_id": "hand",
            "payouts": [{"player_id": "a", "amount": 299}],
        });
        let first_signature = sign_settlement(&first, b"audit-secret").unwrap();
        let second_signature = sign_settlement(&second, b"audit-secret").unwrap();
        assert_eq!(first_signature.len(), 64);
        assert_ne!(first_signature, second_signature);
        assert!(settlement_signature_valid(
            &first,
            &first_signature,
            b"audit-secret"
        ));
        assert!(!settlement_signature_valid(
            &second,
            &first_signature,
            b"audit-secret"
        ));
    }

    #[test]
    fn settlement_conservation_rejects_wrong_pot_or_final_stack() {
        assert!(validate_chip_conservation(1_000, 990, 300, 290, 10).is_ok());
        assert!(validate_chip_conservation(1_000, 990, 300, 289, 10).is_err());
        assert!(validate_chip_conservation(1_000, 989, 300, 290, 10).is_err());
    }

    #[tokio::test]
    async fn disconnected_active_player_is_folded_without_waiting_for_timeout() {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        );
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

        actor.handle_leave(active_before.clone());
        assert!(actor
            .players
            .iter()
            .find(|player| player.id == active_before)
            .is_some_and(|player| !player.is_sitting));

        actor.tick().await;

        let active_after = actor
            .game_loop
            .as_ref()
            .expect("hand should remain available")
            .state
            .active_player()
            .expect("another player should become active")
            .id
            .clone();
        assert_ne!(active_after, active_before);
    }

    #[tokio::test]
    async fn reconnect_before_grace_keeps_the_seat() {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        )
        .with_disconnect_cash_out_after(tokio::time::Duration::from_secs(30));
        actor.players = vec![player("a", 0), player("b", 1)];
        actor.handle_leave("a".to_string());
        assert!(
            !actor
                .players
                .iter()
                .find(|p| p.id == "a")
                .unwrap()
                .is_sitting
        );
        assert!(actor
            .players
            .iter()
            .find(|p| p.id == "a")
            .unwrap()
            .disconnected_since
            .is_some());

        let seat = actor.handle_sit("a".to_string(), "a".to_string(), Some(0), 10_000);
        assert_eq!(seat, 0);
        let again = actor.players.iter().find(|p| p.id == "a").unwrap();
        assert!(again.is_sitting);
        assert!(again.disconnected_since.is_none());
        actor.tick().await;
        assert_eq!(actor.players.iter().filter(|p| p.id == "a").count(), 1);
    }

    #[tokio::test]
    async fn disconnect_between_hands_cashes_out_after_grace() {
        let (_tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_broadcast, _) = broadcast::channel(1);
        let mut actor = TableActor::new(
            "table".to_string(),
            "Test".to_string(),
            rx_cmd,
            tx_broadcast,
        )
        .with_disconnect_cash_out_after(tokio::time::Duration::from_millis(1));
        actor.players = vec![player("a", 0), player("b", 1)];
        actor.handle_leave("a".to_string());
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        actor.tick().await;
        assert!(
            !actor.players.iter().any(|p| p.id == "a"),
            "disconnected player should cash out after grace"
        );
        assert_eq!(actor.players.len(), 1);
    }
}
