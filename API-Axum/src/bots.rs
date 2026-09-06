//! Frota de bots da casa (base do futuro coach).
//!
//! 72 contas `bot_001..072` (is_bot) gerenciadas pelo servidor: o admin liga N
//! bots numa mesa play pelo painel, eles sentam via assento normal (buy-in em
//! PM debitado da carteira do bot) e jogam pelos comandos internos do
//! TableActor, sem WebSocket. A estrategia v1 e um LAG simples com avaliacao
//! de mao real — potes crescem, quebras acontecem, sobra 1 vencedor.
//!
//! Bots nao tem convite nem senha valida: nunca logam, nunca pontuam na
//! Minha Estrutura como beneficiarios com VP (sponsored_by NULL).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::game_actor::{PlayerCommand, TableActor};
use crate::state::TableActorHandle;

pub const BOT_POOL_SIZE: i64 = 72;
pub const BOT_NAME_PREFIX: &str = "bot_";
pub const BOT_EMAIL_DOMAIN: &str = "@bots.local";
/// Banca PM de cada bot (suficiente p/ dezenas de recompras de teste).
pub const BOT_PM_BALANCE: i64 = 1_000_000;

pub const STRATEGY_LAG_V1: &str = "lag_v1";

/// Erro de negocio da frota (vira 4xx/500 no handler admin).
#[derive(Debug)]
pub struct BotError(pub String);

impl std::fmt::Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Dependencias que a frota precisa (sem carregar o AppState inteiro).
#[derive(Clone)]
pub struct BotEnv {
    pub db: sqlx::PgPool,
    pub active_tables: Arc<RwLock<HashMap<String, TableActorHandle>>>,
    pub jwt_secret: String,
    pub redis: Option<redis::aio::ConnectionManager>,
}

/// Um deploy ativo: N bots numa mesa.
#[derive(Debug, Clone)]
pub struct BotDeployment {
    pub table_id: String,
    pub table_name: String,
    pub strategy: String,
    pub started_at: i64,
    pub hands_at_start: i64,
    pub bot_ids: Vec<String>,
}

/// Estado da frota: deploys por mesa + tasks por (mesa, bot).
pub struct BotFleet {
    env: BotEnv,
    deployments: RwLock<HashMap<String, BotDeployment>>,
    tasks: RwLock<HashMap<(String, String), JoinHandle<()>>>,
}

impl BotFleet {
    pub fn new(env: BotEnv) -> Arc<Self> {
        Arc::new(Self {
            env,
            deployments: RwLock::new(HashMap::new()),
            tasks: RwLock::new(HashMap::new()),
        })
    }

    /// Construtor p/ testes (pool preguiçoso, sem Redis, sem mesas).
    pub fn for_test() -> Arc<Self> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost:5432/unused_test_db")
            .expect("URL de teste invalida");
        Self::new(BotEnv {
            db,
            active_tables: Arc::new(RwLock::new(HashMap::new())),
            jwt_secret: "test-secret".to_string(),
            redis: None,
        })
    }

    fn now_epoch() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    // ─── Pool ───

    /// Cria as contas bot_001..072 que faltarem. Retorna (criadas, total).
    pub async fn ensure_pool(&self) -> Result<(i64, i64), BotError> {
        let mut created = 0i64;
        for n in 1..=BOT_POOL_SIZE {
            let username = format!("{BOT_NAME_PREFIX}{n:03}");
            let email = format!("{username}{BOT_EMAIL_DOMAIN}");
            let r = sqlx::query(
                "INSERT INTO users (username, email, password_hash, role, status, \
                 balance_pm_cash, balance_pm_mtt, is_bot, email_verified_at) \
                 VALUES ($1, $2, 'BOT_NO_LOGIN', 'player', 'active', $3, $3, TRUE, \
                 EXTRACT(epoch FROM now())::BIGINT) \
                 ON CONFLICT (username) DO NOTHING",
            )
            .bind(&username)
            .bind(&email)
            .bind(BOT_PM_BALANCE)
            .execute(&self.env.db)
            .await
            .map_err(|e| BotError(format!("pool insert: {e}")))?;
            created += r.rows_affected() as i64;
        }
        // Garante marca/banca mesmo p/ nomes pre-existentes (ex. teste manual).
        sqlx::query(
            "UPDATE users SET is_bot = TRUE, status = 'active', \
             balance_pm_cash = GREATEST(balance_pm_cash, $1) \
             WHERE username LIKE 'bot\\_%'",
        )
        .bind(BOT_PM_BALANCE)
        .execute(&self.env.db)
        .await
        .map_err(|e| BotError(format!("pool normalize: {e}")))?;
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_bot")
            .fetch_one(&self.env.db)
            .await
            .map_err(|e| BotError(format!("pool count: {e}")))?;
        Ok((created, total))
    }

    /// Bots livres (não destacados em nenhuma mesa).
    pub async fn free_bots(&self, limit: i64) -> Result<Vec<(String, String)>, BotError> {
        let busy: Vec<String> = {
            let deps = self.deployments.read().await;
            deps.values().flat_map(|d| d.bot_ids.clone()).collect()
        };
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT id::TEXT, username FROM users WHERE is_bot AND status = 'active' \
             ORDER BY username LIMIT $1",
        )
        .bind(limit + busy.len() as i64 + 8)
        .fetch_all(&self.env.db)
        .await
        .map_err(|e| BotError(format!("free bots: {e}")))?;
        let busy_set: std::collections::HashSet<String> = busy.into_iter().collect();
        Ok(rows
            .into_iter()
            .filter(|(id, _)| !busy_set.contains(id))
            .take(limit as usize)
            .collect())
    }

    // ─── Deploy ───

    /// Liga `count` bots na mesa (buy-in = min da mesa, play money).
    pub async fn start(
        self: &Arc<Self>,
        table_id: &str,
        count: usize,
        strategy: &str,
    ) -> Result<BotDeployment, BotError> {
        if strategy != STRATEGY_LAG_V1 {
            return Err(BotError(format!("estrategia desconhecida: {strategy}")));
        }
        if count == 0 || count > 9 {
            return Err(BotError("count deve ser 1..9".to_string()));
        }
        if self.deployments.read().await.contains_key(table_id) {
            return Err(BotError("mesa ja tem deploy de bots (pare antes)".to_string()));
        }
        let table_id_uuid =
            uuid::Uuid::parse_str(table_id).map_err(|_| BotError("table_id invalido".to_string()))?;

        // Mesa precisa estar aberta, publica e play money.
        let row: Option<(String, i64, i64, i64, i64, i16, String)> = sqlx::query_as(
            "SELECT name, small_blind, big_blind, min_buy_in, max_buy_in, max_players, \
             COALESCE(poker_variant, 'holdem') \
             FROM tables WHERE id = $1 AND visibility = 'public' AND status = 'OPEN' \
             AND COALESCE(money_mode, 'play') = 'play' AND game_type = 'cash'",
        )
        .bind(table_id_uuid)
        .fetch_optional(&self.env.db)
        .await
        .map_err(|e| BotError(format!("mesa: {e}")))?;
        let Some((table_name, _sb, bb, min_buy_in, _max_buy_in, max_players, variant)) = row else {
            return Err(BotError("mesa play indisponivel".to_string()));
        };
        if variant != "holdem" {
            return Err(BotError(
                "frota v1 so joga Texas Hold'em (estrategia usa 2 cartas)".to_string(),
            ));
        }
        if bb <= 0 || min_buy_in <= 0 {
            return Err(BotError("config de blinds/buy-in invalida".to_string()));
        }

        let occupied: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM cash_game_seats WHERE table_id = $1 AND status = 'ACTIVE'",
        )
        .bind(table_id_uuid)
        .fetch_one(&self.env.db)
        .await
        .map_err(|e| BotError(format!("assentos: {e}")))?;
        let free_seats = (max_players as i64 - occupied).max(0) as usize;
        if count > free_seats {
            return Err(BotError(format!(
                "so ha {free_seats} assentos livres (ocupados: {occupied})"
            )));
        }

        let bots = self.free_bots(count as i64).await?;
        if bots.len() < count {
            return Err(BotError(format!(
                "so ha {} bots livres no elenco de {BOT_POOL_SIZE}",
                bots.len()
            )));
        }

        let handle = self.ensure_actor(table_id).await?;
        let buy_in = min_buy_in as u64;
        let mut seated = Vec::new();
        for (bot_id, bot_name) in bots {
            self.seat_bot(table_id_uuid, &bot_id, buy_in).await?;
            // Registra no ator (assento auto).
            let (tx, rx) = tokio::sync::oneshot::channel();
            handle
                .tx_cmd
                .send(PlayerCommand::Sit {
                    player_id: bot_id.clone(),
                    username: bot_name.clone(),
                    seat: None,
                    chips: buy_in,
                    respond_to: tx,
                })
                .await
                .map_err(|_| BotError("ator indisponivel (Sit)".to_string()))?;
            rx.await
                .map_err(|_| BotError("ator nao respondeu ao Sit".to_string()))?;
            // Task jogadora.
            let task = tokio::spawn(bot_loop(
                handle.clone(),
                bot_id.clone(),
                table_id.to_string(),
                strategy.to_string(),
            ));
            self.tasks
                .write()
                .await
                .insert((table_id.to_string(), bot_id.clone()), task);
            seated.push(bot_id);
        }

        let hands_at_start: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM hand_history WHERE table_id = $1",
        )
        .bind(table_id_uuid)
        .fetch_one(&self.env.db)
        .await
        .unwrap_or(0);
        let dep = BotDeployment {
            table_id: table_id.to_string(),
            table_name,
            strategy: strategy.to_string(),
            started_at: Self::now_epoch(),
            hands_at_start,
            bot_ids: seated,
        };
        self.deployments
            .write()
            .await
            .insert(table_id.to_string(), dep.clone());
        Ok(dep)
    }

    /// Para o deploy da mesa: fold imediato (sitting=false), cashout e baixa.
    /// Retorna fichas devolvidas a wallets.
    pub async fn stop(&self, table_id: &str) -> Result<i64, BotError> {
        let dep = self
            .deployments
            .write()
            .await
            .remove(table_id)
            .ok_or_else(|| BotError("mesa sem deploy de bots".to_string()))?;
        let table_uuid =
            uuid::Uuid::parse_str(table_id).map_err(|_| BotError("table_id invalido".to_string()))?;

        // 1. Aborta tasks + tira do jogo (fold imediato na proxima vez).
        {
            let mut tasks = self.tasks.write().await;
            for bot_id in &dep.bot_ids {
                if let Some(h) = tasks.remove(&(table_id.to_string(), bot_id.clone())) {
                    h.abort();
                }
            }
        }
        if let Some(handle) = self.env.active_tables.read().await.get(table_id).cloned() {
            for bot_id in &dep.bot_ids {
                let _ = handle
                    .tx_cmd
                    .send(PlayerCommand::SetSitting {
                        player_id: bot_id.clone(),
                        sitting: false,
                    })
                    .await;
            }
            // 2. Cashout com retry (falha enquanto a mao corre).
            for bot_id in &dep.bot_ids {
                for _ in 0..40 {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let sent = handle
                        .tx_cmd
                        .send(PlayerCommand::CashOut {
                            player_id: bot_id.clone(),
                            respond_to: tx,
                        })
                        .await
                        .is_ok();
                    if !sent {
                        break; // ator morto: vai direto p/ baixa contabil
                    }
                    match rx.await {
                        // Entre maos: fichas (ou ausencia) confirmadas.
                        Ok(Ok(_)) | Err(_) => break,
                        // Mao em andamento: espera a proxima janela.
                        Ok(Err(_)) => tokio::time::sleep(Duration::from_secs(3)).await,
                    }
                }
                // 3. Baixa contabil: devolve ao wallet o que o ator (ou a mesa) tem.
                let actor_chips = None;
                let _ = crate::cash_seats::persist_cash_out_seat(
                    &self.env.db,
                    table_uuid,
                    bot_id,
                    actor_chips,
                )
                .await;
            }
        }
        // Soma devolvida nesta parada (assentos que acabaram de sair).
        let refunded: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(s.chips)::BIGINT, 0) FROM cash_game_seats s \
             JOIN users u ON u.id = s.user_id \
             WHERE s.table_id = $1 AND s.status = 'CASHED_OUT' AND u.is_bot",
        )
        .bind(table_uuid)
        .fetch_one(&self.env.db)
        .await
        .unwrap_or(0);
        Ok(refunded)
    }

    pub async fn status(&self) -> Vec<BotDeployment> {
        self.deployments.read().await.values().cloned().collect()
    }

    // ─── Internos ───

    /// Replica o essencial do join (bots tem banca propria, sem reset diario).
    async fn seat_bot(&self, table_id: uuid::Uuid, bot_id: &str, buy_in: u64) -> Result<(), BotError> {
        let mut tx = self
            .env
            .db
            .begin()
            .await
            .map_err(|e| BotError(format!("tx: {e}")))?;
        let t: Option<(i16, String, String)> = sqlx::query_as(
            "SELECT max_players, status, visibility FROM tables WHERE id = $1 FOR UPDATE",
        )
        .bind(table_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| BotError(format!("lock mesa: {e}")))?;
        let Some((max_players, status, visibility)) = t else {
            return Err(BotError("mesa sumiu".to_string()));
        };
        if status != "OPEN" || visibility != "public" {
            return Err(BotError("mesa fechou no meio do deploy".to_string()));
        }
        // Idempotente: bot ja sentado volta sem novo debito.
        let existing: Option<(i16, i64)> = sqlx::query_as(
            "SELECT seat, chips FROM cash_game_seats \
             WHERE table_id = $1 AND user_id = $2::uuid AND status = 'ACTIVE' FOR UPDATE",
        )
        .bind(table_id)
        .bind(bot_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| BotError(format!("seat existente: {e}")))?;
        if existing.is_some() {
            tx.commit()
                .await
                .map_err(|e| BotError(format!("commit: {e}")))?;
            return Ok(());
        }
        let occupied: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM cash_game_seats WHERE table_id = $1 AND status = 'ACTIVE'",
        )
        .bind(table_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| BotError(format!("conta assentos: {e}")))?;
        if occupied >= max_players as i64 {
            return Err(BotError("mesa lotou no meio do deploy".to_string()));
        }
        let seat: Option<i16> = sqlx::query_scalar(
            "SELECT s::INT2 FROM generate_series(0, $1) s WHERE NOT EXISTS ( \
                 SELECT 1 FROM cash_game_seats \
                 WHERE table_id = $2 AND seat = s AND status = 'ACTIVE') \
             ORDER BY s LIMIT 1",
        )
        .bind(max_players - 1)
        .bind(table_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| BotError(format!("assento livre: {e}")))?;
        let Some(seat) = seat else {
            return Err(BotError("sem assento livre".to_string()));
        };
        // Garante banca e debita.
        sqlx::query(
            "UPDATE users SET balance_pm_cash = GREATEST(balance_pm_cash, $2) WHERE id = $1::uuid",
        )
        .bind(bot_id)
        .bind(BOT_PM_BALANCE)
        .execute(&mut *tx)
        .await
        .map_err(|e| BotError(format!("banca: {e}")))?;
        let debited = sqlx::query(
            "UPDATE users SET balance_pm_cash = balance_pm_cash - $2 \
             WHERE id = $1::uuid AND balance_pm_cash >= $2",
        )
        .bind(bot_id)
        .bind(buy_in as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| BotError(format!("debito: {e}")))?;
        if debited.rows_affected() != 1 {
            return Err(BotError("saldo insuficiente do bot".to_string()));
        }
        let seat_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO cash_game_seats (table_id, user_id, seat, chips, buy_in, wallet_kind) \
             VALUES ($1, $2::uuid, $3, $4, $4, 'pm_cash') RETURNING id",
        )
        .bind(table_id)
        .bind(bot_id)
        .bind(seat)
        .bind(buy_in as i64)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| BotError(format!("insert seat: {e}")))?;
        sqlx::query(
            "INSERT INTO cash_game_ledger (user_id, table_id, seat_id, entry_type, amount) \
             VALUES ($1::uuid, $2, $3, 'BUY_IN', $4)",
        )
        .bind(bot_id)
        .bind(table_id)
        .bind(seat_id)
        .bind(buy_in as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| BotError(format!("ledger: {e}")))?;
        tx.commit()
            .await
            .map_err(|e| BotError(format!("commit: {e}")))?;
        Ok(())
    }

    /// Get-or-spawn do TableActor (mesmo fluxo do WS).
    async fn ensure_actor(&self, table_id: &str) -> Result<TableActorHandle, BotError> {
        if let Some(h) = self.env.active_tables.read().await.get(table_id).cloned() {
            return Ok(h);
        }
        let table_uuid =
            uuid::Uuid::parse_str(table_id).map_err(|_| BotError("table_id invalido".to_string()))?;
        let row: Option<(
            String, i64, i64, i16, i64, Option<i64>, Option<i64>, Option<i64>, String,
        )> = sqlx::query_as(
            "SELECT name, small_blind, big_blind, rake_basis_points, rake_cap, \
             rake_cap_heads_up, rake_cap_three_to_four, rake_cap_five_plus, \
             COALESCE(poker_variant, 'holdem') \
             FROM tables WHERE id = $1",
        )
        .bind(table_uuid)
        .fetch_optional(&self.env.db)
        .await
        .map_err(|e| BotError(format!("config mesa: {e}")))?;
        let Some((table_name, small_blind, big_blind, rake_bp, rake_cap, cap_hu, cap_34, cap_5p, variant)) =
            row
        else {
            return Err(BotError("mesa nao encontrada".to_string()));
        };
        let mut active = self.env.active_tables.write().await;
        if let Some(h) = active.get(table_id) {
            return Ok(h.clone());
        }
        let mut config = poker_engine::types::TableConfig::new(
            big_blind as u64,
            rake_bp as u16,
            rake_cap as u64,
        )
        .with_small_blind(small_blind as u64)
        .with_poker_variant(poker_engine::types::PokerVariant::parse(&variant));
        if let (Some(hu), Some(t34), Some(f5p)) = (cap_hu, cap_34, cap_5p) {
            config = config.with_rake_cap_schedule(poker_engine::types::RakeCapSchedule {
                heads_up: hu as u64,
                three_to_four: t34 as u64,
                five_plus: f5p as u64,
            });
        }
        let (tx_cmd, rx_cmd) = tokio::sync::mpsc::channel(100);
        let (tx_broadcast, _) = tokio::sync::broadcast::channel(100);
        let mut actor = TableActor::new(
            table_id.to_string(),
            table_name,
            rx_cmd,
            tx_broadcast.clone(),
        )
        .with_db(self.env.db.clone())
        .with_audit_secret(self.env.jwt_secret.clone())
        .with_config(config);
        if let Some(ref redis) = self.env.redis {
            actor = actor.with_redis(redis.clone());
        }
        tokio::spawn(actor.run());
        let handle = TableActorHandle {
            tx_cmd,
            tx_broadcast,
        };
        active.insert(table_id.to_string(), handle.clone());
        Ok(handle)
    }
}

// ─── Loop do bot ───

/// Ouve o broadcast da mesa e age quando tem a vez.
async fn bot_loop(
    handle: TableActorHandle,
    bot_id: String,
    table_id: String,
    strategy: String,
) {
    let mut rx = handle.tx_broadcast.subscribe();
    // Anti-loop: nao repete decisao no mesmo snapshot.
    let mut last_sig: Option<(String, u64, u64, u64)> = None;
    // RNG proprio (xorshift, sem deps): pensa como gente, 0.4–1.5s.
    let mut rng = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B9)
        ^ bot_id.len() as u64
        ^ 0x85EBCA6B)
        .max(1);
    let mut rnd = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };

    loop {
        let msg = match rx.recv().await {
            Ok(m) => m,
            Err(_) => {
                tracing::info!(bot = %bot_id, table = %table_id, "bot task saiu (broadcast fechou)");
                return; // broadcast fechou: mesa caiu.
            }
        };
        if msg.get("type").and_then(|v| v.as_str()) != Some("table_state") {
            continue;
        }
        let Some((action, amount)) = decide_for(&msg, &bot_id, &strategy, &mut rnd) else {
            continue;
        };
        let players = msg.get("players").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let me = players.iter().find(|p| {
            p.get("id").and_then(|v| v.as_str()).unwrap_or("") == bot_id
        });
        let sig_src = me.map(|p| {
            (
                msg.get("stage").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                msg.get("pots")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|p| p.get("amount"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                p.get("bet").and_then(|v| v.as_u64()).unwrap_or(0),
                p.get("chips").and_then(|v| v.as_u64()).unwrap_or(0),
            )
        });
        if sig_src.is_some() && sig_src == last_sig {
            continue;
        }
        // Pausa humana antes de agir.
        tokio::time::sleep(Duration::from_millis(400 + rnd() % 1100)).await;
        if handle
            .tx_cmd
            .send(PlayerCommand::Action {
                player_id: bot_id.clone(),
                action,
                amount,
            })
            .await
            .is_err()
        {
            return; // ator morto.
        }
        last_sig = sig_src;
    }
}

// ─── Estrategia lag_v1 ───

#[derive(Debug, Clone, Copy, PartialEq)]
struct Card {
    rank: u8, // 2..14
    suit: u8, // 0..3
}

fn parse_card(s: &str) -> Option<Card> {
    let mut ch = s.chars();
    let r = ch.next()?;
    let su = ch.next()?;
    let rank = match r {
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'T' => 10,
        'J' => 11,
        'Q' => 12,
        'K' => 13,
        'A' => 14,
        _ => return None,
    };
    let suit = match su {
        'h' => 0,
        'd' => 1,
        'c' => 2,
        's' => 3,
        _ => return None,
    };
    Some(Card { rank, suit })
}

/// Forca da mao: 0 fraca, 1 media, 2 forte.
fn hand_tier(hole: &[Card], community: &[Card]) -> u8 {
    if community.is_empty() {
        return preflop_tier(hole);
    }
    postflop_tier(hole, community)
}

fn preflop_tier(hole: &[Card]) -> u8 {
    if hole.len() < 2 {
        return 0;
    }
    let (a, b) = (hole[0], hole[1]);
    let (hi, lo) = if a.rank >= b.rank {
        (a.rank, b.rank)
    } else {
        (b.rank, a.rank)
    };
    let suited = a.suit == b.suit;
    if a.rank == b.rank {
        if hi >= 11 {
            return 2;
        }
        if hi >= 7 {
            return 1;
        }
        return 1; // pares baixos: especulativos, pagam barato
    }
    if hi == 14 && lo >= 11 {
        return 2; // AK/AQ/AJ
    }
    if hi == 14 || (hi >= 12 && lo >= 10) {
        return 1;
    }
    if suited && hi - lo <= 4 && hi >= 8 {
        return 1; // suited conectivos altos
    }
    if hi - lo <= 2 && hi >= 9 {
        return 1; // conectivos off altos
    }
    0
}

fn postflop_tier(hole: &[Card], community: &[Card]) -> u8 {
    let mut ranks = [0u8; 15];
    let mut suits = [0u8; 4];
    for c in hole.iter().chain(community.iter()) {
        ranks[c.rank as usize] += 1;
        suits[c.suit as usize] += 1;
    }
    let pairs = ranks.iter().filter(|&&n| n == 2).count();
    let trips = ranks.iter().any(|&n| n >= 3);
    let four = ranks.iter().any(|&n| n >= 4);
    let flush = suits.iter().any(|&n| n >= 5);
    // Straight: 5 ranks seguidos entre as 7 cartas.
    let mut present = [false; 15];
    for r in 2..=14 {
        present[r] = ranks[r] > 0;
    }
    if ranks[14] > 0 {
        present[1] = true; // A joga baixo
    }
    let straight = (1..=10).any(|s| (s..s + 5).all(|r| present[r]));
    if four || (trips && pairs > 0) || flush || straight {
        return 2;
    }
    if trips {
        return 2;
    }
    // Draws: 4 do mesmo naipe ou 4 em sequencia (OESD simplificado).
    let flush_draw = suits.iter().any(|&n| n == 4);
    let mut best_run = 0;
    let mut run = 0;
    for r in 1..=14 {
        if present[r] {
            run += 1;
            best_run = best_run.max(run);
        } else {
            run = 0;
        }
    }
    // Top pair / overpair / middle pair com kicker decente.
    let board_hi = community.iter().map(|c| c.rank).max().unwrap_or(0);
    let hole_hi = hole.iter().map(|c| c.rank).max().unwrap_or(0);
    let hole_lo = hole.iter().map(|c| c.rank).min().unwrap_or(0);
    let pair_on_board = hole.iter().any(|c| community.iter().any(|b| b.rank == c.rank));
    let pocket_pair = hole.len() == 2 && hole[0].rank == hole[1].rank;
    if pairs >= 2 {
        return 2; // dois pares
    }
    if pair_on_board && (hole_hi >= board_hi || pocket_pair && hole_hi > board_hi) {
        return 1; // top pair / overpair
    }
    if pair_on_board || pocket_pair {
        return 1; // par medio: paga barato
    }
    if flush_draw || best_run >= 4 {
        return 1;
    }
    if hole_hi >= 12 {
        return if community.len() <= 3 { 1 } else { 0 }; // overcards so prestam no flop
    }
    let _ = hole_lo;
    0
}

/// Decide (acao, valor). None = nao e minha vez / sem fichas.
fn decide_for(
    state: &serde_json::Value,
    bot_id: &str,
    strategy: &str,
    rnd: &mut dyn FnMut() -> u64,
) -> Option<(String, u64)> {
    if strategy != STRATEGY_LAG_V1 {
        return None;
    }
    let players = state.get("players")?.as_array()?;
    let me = players
        .iter()
        .find(|p| p.get("id").and_then(|v| v.as_str()) == Some(bot_id))?;
    if me.get("is_active").and_then(|v| v.as_bool()) != Some(true) {
        return None;
    }
    let stack = me.get("chips").and_then(|v| v.as_u64()).unwrap_or(0);
    if stack == 0 {
        return None;
    }
    let my_bet = me.get("bet").and_then(|v| v.as_u64()).unwrap_or(0);
    let cards: Vec<Card> = me
        .get("cards")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|c| c.as_str()).filter_map(parse_card).collect())
        .unwrap_or_default();
    if cards.len() < 2 {
        return None; // cartas ainda nao distribuidas
    }
    let community: Vec<Card> = state
        .get("community_cards")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|c| c.as_str()).filter_map(parse_card).collect())
        .unwrap_or_default();
    let to_match = state
        .get("current_bet_to_match")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let min_raise = state.get("min_raise").and_then(|v| v.as_u64()).unwrap_or(0).max(1);
    let pot: u64 = state
        .get("pots")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|p| p.get("amount")).filter_map(|v| v.as_u64()).sum())
        .unwrap_or(0);
    let to_call = to_match.saturating_sub(my_bet).min(stack);
    let bb = min_raise.max(1);
    let tier = hand_tier(&cards, &community);
    let roll = rnd() % 100;

    // Short stack com algo na mao: shove.
    if stack <= 8 * bb && tier >= 1 && to_call > 0 {
        return Some(("allin".to_string(), 0));
    }

    if to_call == 0 {
        // Posso pedir mesa gratis.
        match tier {
            2 => {
                // Forte: aposta 2/3 do pote (70%) ou mesa traiçoeira (30%).
                if roll < 70 {
                    let amt = (pot * 2 / 3).clamp(min_raise, stack).max(min_raise).min(stack);
                    return Some(("bet".to_string(), amt));
                }
                return Some(("check".to_string(), 0));
            }
            1 => {
                if roll < 30 {
                    let amt = (pot / 2).clamp(min_raise, stack).max(min_raise).min(stack);
                    return Some(("bet".to_string(), amt));
                }
                return Some(("check".to_string(), 0));
            }
            _ => return Some(("check".to_string(), 0)),
        }
    }

    // Tem aposta: matematica de pote simples (LAG paga leve).
    let pot_odds = to_call as f64 / (pot + to_call).max(1) as f64;
    let equity = match tier {
        2 => 0.65,
        1 => 0.35,
        _ => 0.08,
    };
    if tier == 2 && roll < 55 {
        // Forte: sobe (total = mesa + min_raise ou pote).
        let total = (to_match + min_raise.max(pot / 2)).max(to_match + min_raise);
        let affordable = my_bet + stack;
        if total <= affordable {
            return Some(("raise".to_string(), total));
        }
        if stack > to_call {
            return Some(("call".to_string(), 0));
        }
        return Some(("allin".to_string(), 0));
    }
    if equity > pot_odds * 1.1 || to_call <= 3 * bb {
        return Some(("call".to_string(), 0));
    }
    Some(("fold".to_string(), 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(s: &str) -> Card {
        parse_card(s).unwrap()
    }

    #[test]
    fn preflop_tiers() {
        assert_eq!(preflop_tier(&[c("Ah"), c("Ad")]), 2);
        assert_eq!(preflop_tier(&[c("Ah"), c("Kc")]), 2);
        assert_eq!(preflop_tier(&[c("Qh"), c("Jh")]), 1);
        assert_eq!(preflop_tier(&[c("7h"), c("2c")]), 0);
    }

    #[test]
    fn postflop_reads_board() {
        // Top pair
        assert_eq!(
            postflop_tier(&[c("Ah"), c("7c")], &[c("As"), c("9d"), c("4h")]),
            1
        );
        // Flush
        assert_eq!(
            postflop_tier(&[c("Ah"), c("2h")], &[c("5h"), c("9h"), c("Kd")]),
            2
        );
        // Nada
        assert_eq!(
            postflop_tier(&[c("7c"), c("2d")], &[c("As"), c("Kd"), c("Qh")]),
            0
        );
    }
}
