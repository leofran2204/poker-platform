// game_actor.rs — Ator para orquestração da mesa de Poker em tempo real.
//
// Gerencia a conexão de múltiplos jogadores, as transições do GameLoop
// e a transmissão das cartas e apostas via WebSockets.

use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{info, error};

use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::types::TableConfig;
use poker_engine::hand_history::GameType;

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
    pub dealer_index: usize,
    pub antifraud: poker_engine::antifraud::AntiFraudSuite,
    pub last_turn_start: Option<tokio::time::Instant>,
    pub db: Option<sqlx::PgPool>,
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
            config: TableConfig::new(2000, 0.05, 10000), // Default BB=2000 centavos (R$ 20,00), rake=5%, cap=10000 centavos (R$ 100,00)
            players: Vec::new(),
            game_loop: None,
            rx,
            tx_broadcast,
            next_hand_at: None,
            dealer_index: 0,
            antifraud: poker_engine::antifraud::AntiFraudSuite::new(),
            last_turn_start: Some(tokio::time::Instant::now()),
            db: None,
        }
    }

    pub fn with_db(mut self, db: sqlx::PgPool) -> Self {
        self.db = Some(db);
        self
    }

    pub async fn run(mut self) {
        info!("Table actor started for table: {}", self.table_id);
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
            PlayerCommand::Sit { player_id, username, seat, chips, respond_to } => {
                let assigned_seat = self.handle_sit(player_id, username, seat, chips);
                let _ = respond_to.send(assigned_seat);
            }
            PlayerCommand::Leave { player_id } => {
                self.handle_leave(player_id);
            }
            PlayerCommand::Action { player_id, action, amount } => {
                self.handle_action(player_id, action, amount);
            }
            PlayerCommand::GetTableInfo { respond_to } => {
                let info = self.get_table_info_json();
                let _ = respond_to.send(info).await;
            }
        }
    }

    async fn tick(&mut self) {
        if let Some(next_at) = self.next_hand_at {
            if tokio::time::Instant::now() >= next_at {
                self.next_hand_at = None;
                self.game_loop = None;
                self.start_new_hand();
            }
        } else if self.game_loop.is_none() && self.players.iter().filter(|p| p.is_sitting && p.chips > 0).count() >= 2 {
            // Auto-start hand if we have enough players
            self.start_new_hand();
        }
    }

    fn handle_sit(&mut self, player_id: String, username: String, seat: Option<usize>, chips: u64) -> usize {
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

        info!("Player sat at table {} in seat {}", self.table_id, assigned_seat);
        self.broadcast_state();
        assigned_seat
    }

    fn handle_leave(&mut self, player_id: String) {
        self.players.retain(|p| p.id != player_id);
        info!("Player {} left table {}", player_id, self.table_id);
        
        // Se o jogador sair e a mão estiver em andamento, devemos dar fold nele
        if let Some(ref mut gl) = self.game_loop {
            if !gl.state.is_finished {
                let active_idx = gl.state.active_player_index;
                let is_active_turn = gl.state.players.get(active_idx).map(|p| p.id.as_str()) == Some(player_id.as_str());

                if is_active_turn {
                    let _ = gl.player_action(&player_id, PlayerMove::Fold);
                } else if let Some(p) = gl.state.players.iter_mut().find(|p| p.id == player_id) {
                    p.has_folded = true;
                    if gl.state.players_in_hand_count() <= 1 {
                        gl.state.is_finished = true;
                    }
                }
            }
        }

        self.broadcast_state();
    }

    fn handle_action(&mut self, player_id: String, action: String, amount: u64) {
        let elapsed_ms = self
            .last_turn_start
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(500);
        self.last_turn_start = Some(tokio::time::Instant::now());

        let risk_score = self.antifraud.process_action(&player_id, elapsed_ms);
        if risk_score.recommendation == poker_engine::antifraud::RiskRecommendation::BlockSession {
            error!("Action blocked by AntiFraud for player {}: risk score {}", player_id, risk_score.total_score);
            return;
        }

        let game_loop = match &mut self.game_loop {
            Some(gl) => gl,
            None => {
                error!("Attempted action but no game loop running at table: {}", self.table_id);
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

        if game_loop.state.is_finished {
            // Resolver a mão
            if let Ok(res) = game_loop.resolve_hand() {
                game_loop.finalize_history(&res);

                // Persistir histórico da mão no PostgreSQL (operação desacoplada em background)
                if let Some(ref mut history) = game_loop.history {
                    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "poker_platform_audit_secret_key_2026".to_string());
                    poker_engine::hand_history::sign_hand(history, secret.as_bytes());
                    if let Ok(history_json) = serde_json::to_value(&history) {
                        let db_opt = self.db.clone();
                        let hand_id = history.hand_id.clone();
                        let pot = history.total_pot;
                        let rake = history.rake;
                        let reason = format!("{:?}", history.end_reason);

                        tokio::spawn(async move {
                            if let Some(db) = db_opt {
                                let uuid_val = uuid::Uuid::parse_str(&hand_id).unwrap_or_else(|_| uuid::Uuid::new_v4());
                                let _ = sqlx::query(
                                    "INSERT INTO hand_history (id, table_id, hand_number, game_type, small_blind, big_blind, actions_json, community_cards_json, pot_total, rake_collected, end_reason)
                                     VALUES ($1, NULL, 1, 'cash', 10, 20, $2, $3, $4, $5, $6)
                                     ON CONFLICT (id) DO NOTHING"
                                )
                                .bind(uuid_val)
                                .bind(&history_json["actions"])
                                .bind(&history_json["community_cards"])
                                .bind(pot as i64)
                                .bind(rake as i64)
                                .bind(&reason)
                                .execute(&db)
                                .await;
                            }
                        });
                    }
                }

                // Atualizar os saldos das fichas na mesa baseado nos stacks e payouts
                for gp in &game_loop.state.players {
                    if let Some(tp) = self.players.iter_mut().find(|p| p.id == gp.id) {
                        let payout = res.payouts.get(&gp.id).copied().unwrap_or(0);
                        tp.chips = gp.stack + payout;
                    }
                }

                // Disparar evento global do Loss Deflator se foi ativado
                if let Some(deflator) = &res.loss_deflator {
                    let loser_name = self.players.iter().find(|p| p.id == deflator.loser_id).map(|p| p.name.clone()).unwrap_or_else(|| deflator.loser_id.clone());
                    let winner_name = self.players.iter().find(|p| p.id == deflator.winner_id).map(|p| p.name.clone()).unwrap_or_else(|| deflator.winner_id.clone());
                    
                    let final_loser_chips = self.players.iter().find(|p| p.id == deflator.loser_id).map(|p| p.chips).unwrap_or(0);
                    let prevented_elimination = final_loser_chips == deflator.cashback;
                    
                    // Como game_actor.rs atualmente orquestra Cash, deixamos fixo como false para torneio (torneio usa tournament_engine)
                    let is_tournament = false;
                    
                    // Como a regra de negócio atual baseia o deflator na fase e não nas odds exatas, 
                    // vamos enviar a fase (ex: 15%, 25%, 35%) como representação do "quão quebrado" foi o river.
                    // O UX pediu para mostrar "(18%)". Vamos mapear as probabilidades estimadas daquela fase.
                    // Se deflator.odds foi calculado (> 0), usamos a chance do VENCEDOR (1 - loser_equity)
                    // Exemplo: perdedor tinha 82% -> vencedor tinha 18% (odds_broken = 18%)
                    let odds_broken = if deflator.odds > 0.0 {
                        ((1.0 - deflator.odds) * 100.0).round() as u8
                    } else {
                        match deflator.tier {
                            poker_engine::loss_deflator::LossDeflatorTier::SevenPercent => 7,
                            poker_engine::loss_deflator::LossDeflatorTier::FifteenPercent => 15,
                            poker_engine::loss_deflator::LossDeflatorTier::TwentyFivePercent => 25,
                            poker_engine::loss_deflator::LossDeflatorTier::ThirtyFivePercent => 35,
                        }
                    };

                    let event_payload = serde_json::json!({
                        "type": "deflator_triggered",
                        "loser_name": loser_name,
                        "winner_name": winner_name,
                        "cashback_amount": deflator.cashback,
                        "odds_broken": odds_broken,
                        "prevented_elimination": prevented_elimination,
                        "is_tournament": is_tournament
                    });
                    
                    let _ = self.tx_broadcast.send(event_payload);
                }
            }
            self.broadcast_state();
            // Iniciar próxima mão depois de 6 segundos
            self.next_hand_at = Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(6));
        } else {
            self.broadcast_state();
        }
    }

    fn start_new_hand(&mut self) {
        let mut active_players: Vec<&mut TablePlayer> = self
            .players
            .iter_mut()
            .filter(|p| p.is_sitting && p.chips > 0)
            .collect();

        if active_players.len() < 2 {
            return;
        }

        active_players.sort_by_key(|p| p.seat);

        let mut gl = GameLoop::new(
            self.config.clone(),
            format!("hand_{}", uuid::Uuid::new_v4()),
            self.name.clone(),
            GameType::Cash,
        );

        for p in &active_players {
            gl.add_player(p.id.clone(), p.chips);
        }

        // Escolhe o dealer
        self.dealer_index = (self.dealer_index + 1) % active_players.len();
        gl.set_dealer(self.dealer_index);

        if let Err(e) = gl.start_hand() {
            error!("Failed to start hand: {}", e);
            return;
        }

        self.game_loop = Some(gl);
        info!("Started new hand at table {}", self.table_id);
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
            community_cards = gl.state.community_cards.iter().map(card_to_string).collect();

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
            "is_finished": is_finished
        });

        let _ = self.tx_broadcast.send(state_payload);
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
