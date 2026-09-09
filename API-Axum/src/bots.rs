//! Frota de bots da casa (base do futuro coach).
//!
//! 72 contas `bot_001..072` (is_bot) gerenciadas pelo servidor: o admin liga N
//! bots numa mesa play pelo painel, eles sentam via assento normal (buy-in em
//! PM debitado da carteira do bot) e jogam pelos comandos internos do
//! TableActor, sem WebSocket. A estrategia unica e LAG com avaliacao de mao
//! real pelo proprio motor (as 4 variantes) — potes crescem, quebras
//! acontecem, sobra 1 vencedor.
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

/// Estrategia unica da frota: base LAG com avaliacao da mao pelo proprio
/// motor por variante (Hold'em, Short Deck, Omaha SD e Pineapple Ultimate).
pub const STRATEGY_LAG_V2: &str = "lag_v2";

pub const SUPPORTED_VARIANTS: &[&str] = &[
    "holdem",
    "short_deck",
    "short_deck_omaha",
    "ultimate_pineapple",
];

pub fn all_strategies() -> Vec<String> {
    vec![STRATEGY_LAG_V2.to_string()]
}

/// Expande grupos [(personalidade, qtd)] na ordem dos bots sentados.
/// Soma ≠ count → erro. Vazio → todos Lag (compatível com deploys antigos).
pub fn expand_groups(
    groups: &[(Personality, usize)],
    count: usize,
) -> Result<Vec<Personality>, BotError> {
    if groups.is_empty() {
        return Ok(vec![Personality::Lag; count]);
    }
    let total: usize = groups.iter().map(|(_, n)| n).sum();
    if total != count {
        return Err(BotError(format!("grupos somam {total}, esperado {count}")));
    }
    let mut out = Vec::with_capacity(count);
    for (personality, n) in groups {
        out.extend(std::iter::repeat_n(*personality, *n));
    }
    Ok(out)
}

/// Parse "tag:2,lag:4,nit:1,station:1" (caixa de opção do painel).
pub fn parse_groups(spec: &str) -> Result<Vec<(Personality, usize)>, BotError> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Ok(Vec::new());
    }
    spec.split(',')
        .map(|part| {
            let (name, n) = part
                .split_once(':')
                .ok_or_else(|| BotError(format!("grupo inválido (use nome:qtd): {part}")))?;
            let personality = Personality::parse(name)
                .ok_or_else(|| BotError(format!("personalidade desconhecida: {name}")))?;
            let n: usize = n
                .trim()
                .parse()
                .map_err(|_| BotError(format!("qtd inválida em: {part}")))?;
            if n == 0 {
                return Err(BotError(format!("qtd zerada em: {part}")));
            }
            Ok((personality, n))
        })
        .collect()
}

/// Personalidades jogáveis (meio-termo): tendências sobrepostas, nunca
/// caricaturas. Nenhuma frequência em 0% ou 100%: todo bot blefa e folda
/// às vezes; a diferença aparece na média de centenas de mãos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Personality {
    Nit,
    Tag,
    Lag,
    Station,
}

impl Personality {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "nit" | "rock" => Some(Self::Nit),
            "tag" => Some(Self::Tag),
            "lag" => Some(Self::Lag),
            "station" | "calling-station" | "calling_station" => Some(Self::Station),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nit => "nit",
            Self::Tag => "tag",
            Self::Lag => "lag",
            Self::Station => "station",
        }
    }

    pub fn all() -> Vec<String> {
        ["nit", "tag", "lag", "station"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }
}

/// Faixa base (min..=max, em %) de cada botão de decisão por personalidade.
/// Meio-termo deliberado: bandas sobrepostas, pisos ≥5%, tetos ≤95%.
#[derive(Debug, Clone, Copy)]
struct ProfileBands {
    bet_strong: (u8, u8),
    bet_medium: (u8, u8),
    raise_strong: (u8, u8),
    /// Multiplicador de pot-odds ×100 a partir do qual paga (110 = 1,1x).
    call_odds_bp: (u16, u16),
    /// Paga sem pensar até este múltiplo do BB.
    call_free_bb: u64,
    /// Shove com algo na mão até este stack (em BB). 0 = nunca shoveia.
    shove_bb: u64,
    /// % de fold com lixo pré-flop sem aposta (pedir mesa é grátis).
    trash_fold: (u8, u8),
}

fn bands_of(personality: Personality) -> ProfileBands {
    match personality {
        Personality::Nit => ProfileBands {
            bet_strong: (75, 95),
            bet_medium: (5, 15),
            raise_strong: (70, 90),
            call_odds_bp: (140, 170),
            call_free_bb: 1,
            shove_bb: 6,
            trash_fold: (75, 95),
        },
        Personality::Tag => ProfileBands {
            bet_strong: (65, 85),
            bet_medium: (10, 25),
            raise_strong: (60, 80),
            call_odds_bp: (120, 140),
            call_free_bb: 2,
            shove_bb: 10,
            trash_fold: (60, 80),
        },
        Personality::Lag => ProfileBands {
            bet_strong: (55, 75),
            bet_medium: (25, 40),
            raise_strong: (45, 65),
            call_odds_bp: (95, 115),
            call_free_bb: 3,
            shove_bb: 8,
            trash_fold: (20, 40),
        },
        Personality::Station => ProfileBands {
            bet_strong: (15, 30),
            bet_medium: (5, 15),
            raise_strong: (5, 15),
            call_odds_bp: (60, 80),
            call_free_bb: 6,
            shove_bb: 0,
            trash_fold: (10, 30),
        },
    }
}

/// Parâmetros efetivos de UMA decisão (sorteados da faixa + jitter + humor).
#[derive(Debug, Clone)]
pub struct BotParams {
    pub bet_strong: u8,
    pub bet_medium: u8,
    pub raise_strong: u8,
    pub call_odds_bp: u16,
    pub call_free_bb: u64,
    pub shove_bb: u64,
    pub trash_fold: u8,
}

/// Perfil vivo de um bot: preset + jitter individual + humor da sessão.
/// O jitter (±15, fixo por seed) separa bots iguais; o humor (±10, deriva
/// lenta com reversão à média) impede jogo engessado. Tudo limitado a
/// [5, 95] para percentuais — nunca 0% nem 100%.
#[derive(Debug, Clone)]
pub struct BotProfile {
    pub personality: Personality,
    jitter: i8,
    mood: i8,
}

impl BotProfile {
    pub fn new(personality: Personality, seed: u64) -> Self {
        let mut rng = seed.max(1);
        let mut xorshift = move || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };
        let jitter = (xorshift() % 31) as i8 - 15;
        Self {
            personality,
            jitter,
            mood: 0,
        }
    }

    /// Deriva lenta do humor: ±1 por chamada com reversão à média.
    pub fn tick_mood(&mut self, rnd: &mut dyn FnMut() -> u64) {
        let roll = rnd() % 100;
        if roll < 8 {
            if self.mood > 0 {
                self.mood -= 1;
            } else if self.mood < 0 {
                self.mood += 1;
            } else if rnd().is_multiple_of(2) {
                self.mood = 1;
            } else {
                self.mood = -1;
            }
        }
        self.mood = self.mood.clamp(-10, 10);
    }

    fn shift(&self, rnd: &mut dyn FnMut() -> u64, lo: u8, hi: u8) -> u8 {
        let span = u64::from(hi.saturating_sub(lo));
        let sampled = u64::from(lo) + if span > 0 { rnd() % (span + 1) } else { 0 };
        (sampled as i16 + self.jitter as i16 + self.mood as i16).clamp(5, 95) as u8
    }

    /// Sorteia os parâmetros efetivos: faixa da personalidade + jitter + humor.
    pub fn params(&self, rnd: &mut dyn FnMut() -> u64) -> BotParams {
        let bands = bands_of(self.personality);
        let bet_strong = self.shift(rnd, bands.bet_strong.0, bands.bet_strong.1);
        let bet_medium = self.shift(rnd, bands.bet_medium.0, bands.bet_medium.1);
        let raise_strong = self.shift(rnd, bands.raise_strong.0, bands.raise_strong.1);
        let trash_fold = self.shift(rnd, bands.trash_fold.0, bands.trash_fold.1);
        BotParams {
            bet_strong,
            bet_medium,
            raise_strong,
            call_odds_bp: {
                let span = bands.call_odds_bp.1.saturating_sub(bands.call_odds_bp.0) as u64;
                let sampled =
                    u64::from(bands.call_odds_bp.0) + if span > 0 { rnd() % (span + 1) } else { 0 };
                (sampled as i16 + self.jitter as i16 + self.mood as i16).clamp(50, 200) as u16
            },
            call_free_bb: bands.call_free_bb,
            shove_bb: bands.shove_bb,
            trash_fold,
        }
    }

    /// Rótulo p/ painel: `tag·loose`, `nit·steady`, ...
    pub fn mood_label(&self) -> &'static str {
        if self.mood > 3 {
            "loose"
        } else if self.mood < -3 {
            "tight"
        } else {
            "steady"
        }
    }

    pub fn label(&self) -> String {
        format!("{}·{}", self.personality.as_str(), self.mood_label())
    }
}

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
    pub tournaments: Arc<RwLock<HashMap<String, crate::tournament_store::TournamentStore>>>,
    pub jwt_secret: String,
    pub redis: Option<redis::aio::ConnectionManager>,
}

/// Um deploy ativo: N bots numa mesa.
#[derive(Debug, Clone)]
pub struct BotDeployment {
    pub table_id: String,
    pub table_name: String,
    pub strategy: String,
    pub variant: String,
    pub started_at: i64,
    pub hands_at_start: i64,
    pub bot_ids: Vec<String>,
    /// Personalidade por bot (padrão: lag). Ex.: "tag·loose" no painel.
    pub personalities: HashMap<String, String>,
}

/// Um deploy ativo num torneio: N bots inscritos que jogam nas 3 mesas.
/// Bots têm `sponsored_by` NULL: nunca pontuam nem geram rede — o fee deles
/// é 100% da casa. Só torneios play money (banca PM dos bots).
#[derive(Debug, Clone)]
pub struct BotTournamentDeployment {
    pub tournament_id: String,
    pub tournament_name: String,
    pub strategy: String,
    pub money_mode: String,
    pub started_at: i64,
    pub bot_ids: Vec<String>,
    /// Personalidade por bot (padrão: lag).
    pub personalities: HashMap<String, String>,
}

/// Estado da frota: deploys por mesa + tasks por (mesa, bot).
pub struct BotFleet {
    env: BotEnv,
    deployments: RwLock<HashMap<String, BotDeployment>>,
    tasks: RwLock<HashMap<(String, String), JoinHandle<()>>>,
    tournament_deployments: RwLock<HashMap<String, BotTournamentDeployment>>,
    tournament_tasks: RwLock<HashMap<(String, String), JoinHandle<()>>>,
}

impl BotFleet {
    pub fn new(env: BotEnv) -> Arc<Self> {
        Arc::new(Self {
            env,
            deployments: RwLock::new(HashMap::new()),
            tasks: RwLock::new(HashMap::new()),
            tournament_deployments: RwLock::new(HashMap::new()),
            tournament_tasks: RwLock::new(HashMap::new()),
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
            tournaments: Arc::new(RwLock::new(HashMap::new())),
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

    /// Bots livres (não destacados em nenhuma mesa nem torneio).
    pub async fn free_bots(&self, limit: i64) -> Result<Vec<(String, String)>, BotError> {
        let busy: Vec<String> = {
            let deps = self.deployments.read().await;
            let tdeps = self.tournament_deployments.read().await;
            deps.values()
                .flat_map(|d| d.bot_ids.clone())
                .chain(tdeps.values().flat_map(|d| d.bot_ids.clone()))
                .collect()
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
    /// `groups` distribui personalidades na ordem dos sentados; vazio = tudo Lag.
    pub async fn start(
        self: &Arc<Self>,
        table_id: &str,
        count: usize,
        strategy: &str,
        groups: &[(Personality, usize)],
    ) -> Result<BotDeployment, BotError> {
        if strategy != STRATEGY_LAG_V2 {
            return Err(BotError(format!("estrategia desconhecida: {strategy}")));
        }
        if count == 0 || count > 9 {
            return Err(BotError("count deve ser 1..9".to_string()));
        }
        if self.deployments.read().await.contains_key(table_id) {
            return Err(BotError(
                "mesa ja tem deploy de bots (pare antes)".to_string(),
            ));
        }
        let table_id_uuid = uuid::Uuid::parse_str(table_id)
            .map_err(|_| BotError("table_id invalido".to_string()))?;

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
        if !SUPPORTED_VARIANTS.contains(&variant.as_str()) {
            return Err(BotError(format!(
                "variante sem suporte na frota: {variant}"
            )));
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
        let profiles = expand_groups(groups, count)?;

        let handle = self.ensure_actor(table_id).await?;
        let buy_in = min_buy_in as u64;
        let mut seated = Vec::new();
        let mut personalities = HashMap::new();
        for (index, (bot_id, bot_name)) in bots.into_iter().enumerate() {
            let personality = profiles.get(index).copied().unwrap_or(Personality::Lag);
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
            // Task jogadora (perfil próprio por bot).
            let task = tokio::spawn(bot_loop(
                handle.clone(),
                bot_id.clone(),
                table_id.to_string(),
                variant.clone(),
                personality,
            ));
            self.tasks
                .write()
                .await
                .insert((table_id.to_string(), bot_id.clone()), task);
            personalities.insert(bot_id.clone(), personality.as_str().to_string());
            seated.push(bot_id);
        }

        let hands_at_start: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM hand_history WHERE table_id = $1")
                .bind(table_id_uuid)
                .fetch_one(&self.env.db)
                .await
                .unwrap_or(0);
        let dep = BotDeployment {
            table_id: table_id.to_string(),
            table_name,
            strategy: strategy.to_string(),
            variant: variant.clone(),
            started_at: Self::now_epoch(),
            hands_at_start,
            bot_ids: seated,
            personalities,
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
        let table_uuid = uuid::Uuid::parse_str(table_id)
            .map_err(|_| BotError("table_id invalido".to_string()))?;

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

    pub async fn tournament_status(&self) -> Vec<BotTournamentDeployment> {
        self.tournament_deployments
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    // ─── Torneios ───

    /// Inscreve `count` bots no torneio (play money) e liga o supervisor que
    /// os coloca para jogar nas 3 mesas quando houver ator vivo.
    pub async fn start_tournament(
        self: &Arc<Self>,
        tournament_id: &str,
        count: usize,
        strategy: &str,
        groups: &[(Personality, usize)],
    ) -> Result<BotTournamentDeployment, BotError> {
        if strategy != STRATEGY_LAG_V2 {
            return Err(BotError(format!("estrategia desconhecida: {strategy}")));
        }
        if count == 0 || count > BOT_POOL_SIZE as usize {
            return Err(BotError("count deve ser 1..72".to_string()));
        }
        if strategy != STRATEGY_LAG_V2 {
            return Err(BotError(format!("estrategia desconhecida: {strategy}")));
        }
        if count == 0 || count > BOT_POOL_SIZE as usize {
            return Err(BotError("count deve ser 1..72".to_string()));
        }
        if self
            .tournament_deployments
            .read()
            .await
            .contains_key(tournament_id)
        {
            return Err(BotError(
                "torneio ja tem deploy de bots (pare antes)".to_string(),
            ));
        }
        let (tname, mode) = {
            let t = self.env.tournaments.read().await;
            let store = t
                .get(tournament_id)
                .ok_or_else(|| BotError("torneio nao encontrado".to_string()))?;
            (store.state.config.name.clone(), store.money_mode.clone())
        };
        if !mode.eq_ignore_ascii_case("play") {
            return Err(BotError("bots so jogam torneios play money".to_string()));
        }
        let free = {
            let deps = self.deployments.read().await;
            let tdeps = self.tournament_deployments.read().await;
            let mut busy = std::collections::HashSet::new();
            for d in deps.values() {
                busy.extend(d.bot_ids.iter().cloned());
            }
            for d in tdeps.values() {
                busy.extend(d.bot_ids.iter().cloned());
            }
            let rows: Vec<(String, String)> = sqlx::query_as(
                "SELECT id::TEXT, username FROM users WHERE is_bot AND status = 'active' \
                 ORDER BY username LIMIT $1",
            )
            .bind(count as i64 + busy.len() as i64 + 8)
            .fetch_all(&self.env.db)
            .await
            .map_err(|e| BotError(format!("free bots: {e}")))?;
            rows.into_iter()
                .filter(|(id, _)| !busy.contains(id))
                .take(count)
                .collect::<Vec<_>>()
        };
        if free.len() < count {
            return Err(BotError(format!(
                "so ha {} bots livres no elenco de {BOT_POOL_SIZE}",
                free.len()
            )));
        }
        let profiles = expand_groups(groups, count)?;
        let mut seated = Vec::new();
        let mut personalities = HashMap::new();
        let mut profile_iter = profiles.into_iter();
        for (bot_id, bot_name) in free {
            let Some(personality) = profile_iter.next() else {
                break;
            };
            match self
                .register_bot_in_tournament(tournament_id, &bot_id, &bot_name)
                .await
            {
                Ok(()) => {
                    personalities.insert(bot_id.clone(), personality.as_str().to_string());
                    seated.push(bot_id);
                }
                Err(e) => {
                    tracing::warn!(bot = %bot_name, "bot pulado no MTT: {e}");
                    continue;
                }
            }
            if seated.len() >= count {
                break;
            }
        }
        if seated.is_empty() {
            return Err(BotError("nenhum bot conseguiu se inscrever".to_string()));
        }
        let dep = BotTournamentDeployment {
            tournament_id: tournament_id.to_string(),
            tournament_name: tname,
            strategy: strategy.to_string(),
            money_mode: mode,
            started_at: Self::now_epoch(),
            bot_ids: seated,
            personalities,
        };
        self.tournament_deployments
            .write()
            .await
            .insert(tournament_id.to_string(), dep.clone());
        // Supervisor: mantém um loop de jogo por bot sentado.
        let fleet = Arc::clone(self);
        let tid = tournament_id.to_string();
        tokio::spawn(async move {
            fleet.supervise_tournament(tid).await;
        });
        Ok(dep)
    }

    /// Para o deploy do torneio: aborta tasks e coloca os bots em sit-out
    /// (fichas seguem no torneio até blindar/eliminar — MTT não tem cash-out).
    /// Retorna quantos bots foram desligados.
    pub async fn stop_tournament(&self, tournament_id: &str) -> Result<usize, BotError> {
        let dep = self
            .tournament_deployments
            .write()
            .await
            .remove(tournament_id)
            .ok_or_else(|| BotError("torneio sem deploy de bots".to_string()))?;
        {
            let mut tasks = self.tournament_tasks.write().await;
            for bot_id in &dep.bot_ids {
                if let Some(h) = tasks.remove(&(tournament_id.to_string(), bot_id.clone())) {
                    h.abort();
                }
            }
        }
        // Sit-out nas mesas vivas (best effort).
        let tables: Vec<String> = {
            let t = self.env.tournaments.read().await;
            t.get(tournament_id)
                .map(|s| s.live_table_ids.clone())
                .unwrap_or_default()
        };
        let active = self.env.active_tables.read().await;
        for table_id in tables {
            if let Some(handle) = active.get(&table_id) {
                for bot_id in &dep.bot_ids {
                    let _ = handle
                        .tx_cmd
                        .send(PlayerCommand::SetSitting {
                            player_id: bot_id.clone(),
                            sitting: false,
                        })
                        .await;
                }
            }
        }
        Ok(dep.bot_ids.len())
    }

    /// Inscreve um bot via fluxo interno (sem JWT): engine + débito buy-in e
    /// fee na banca PM + ledger de fee + linha em tournament_players.
    async fn register_bot_in_tournament(
        &self,
        tournament_id: &str,
        bot_id: &str,
        bot_name: &str,
    ) -> Result<(), BotError> {
        use poker_engine::tournament_engine as engine;
        // Snapshot p/ desfazer no motor se o banco falhar.
        let (buy_in, starting_stack, snapshot) = {
            let mut t = self.env.tournaments.write().await;
            let store = t
                .get_mut(tournament_id)
                .ok_or_else(|| BotError("torneio sumiu".to_string()))?;
            let snapshot = (
                store.state.total_buyins,
                store.state.total_fees,
                store.state.prize_pool,
                store.state.players_remaining,
            );
            engine::register_player(&mut store.state, bot_id, bot_name)
                .map_err(|e| BotError(format!("inscricao: {e}")))?;
            (
                store.state.config.buy_in,
                store.state.config.starting_stack,
                snapshot,
            )
        };
        let fee = engine::entry_fee_cents(buy_in);
        let mut tx = self
            .env
            .db
            .begin()
            .await
            .map_err(|e| BotError(format!("tx: {e}")))?;
        if buy_in > 0 {
            let buy_in_i =
                i64::try_from(buy_in).map_err(|_| BotError("buy-in invalido".to_string()))?;
            if let Err(e) = crate::wallet::debit_wallet(
                &mut *tx,
                bot_id,
                buy_in_i,
                crate::wallet::WalletKind::PmMtt,
            )
            .await
            {
                rollback_bot_registration(&self.env.tournaments, tournament_id, bot_id, snapshot)
                    .await;
                return Err(BotError(format!("debito buy-in: {e:?}")));
            }
        }
        if fee > 0 {
            let fee_i = i64::try_from(fee).map_err(|_| BotError("fee invalido".to_string()))?;
            if let Err(e) = crate::wallet::debit_wallet(
                &mut *tx,
                bot_id,
                fee_i,
                crate::wallet::WalletKind::PmMtt,
            )
            .await
            {
                rollback_bot_registration(&self.env.tournaments, tournament_id, bot_id, snapshot)
                    .await;
                return Err(BotError(format!("debito fee: {e:?}")));
            }
            let payer = uuid::Uuid::parse_str(bot_id)
                .map_err(|_| BotError("bot id invalido".to_string()))?;
            let week_start: i64 = sqlx::query_scalar(
                "SELECT EXTRACT(EPOCH FROM date_trunc('week', timezone('America/Sao_Paulo', now())))::BIGINT",
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| BotError(format!("relogio semanal: {e}")))?;
            if let Err(e) =
                crate::estrutura::distribute_fee(&mut tx, payer, fee_i, week_start).await
            {
                rollback_bot_registration(&self.env.tournaments, tournament_id, bot_id, snapshot)
                    .await;
                return Err(BotError(format!("split fee: {e}")));
            }
        }
        if let Err(e) = sqlx::query(
            "INSERT INTO tournament_players (tournament_id, player_id, player_name, stack, registered_at) \
             VALUES ($1::uuid, $2, $3, $4, EXTRACT(EPOCH FROM NOW())::BIGINT) \
             ON CONFLICT (tournament_id, player_id) DO NOTHING",
        )
        .bind(tournament_id)
        .bind(bot_id)
        .bind(bot_name)
        .bind(starting_stack as i64)
        .execute(&mut *tx)
        .await
        {
            rollback_bot_registration(&self.env.tournaments, tournament_id, bot_id, snapshot).await;
            return Err(BotError(format!("tournament_players: {e}")));
        }
        // Espelha contadores do motor na linha do torneio.
        {
            let t = self.env.tournaments.read().await;
            if let Some(store) = t.get(tournament_id) {
                if let Err(e) = sqlx::query(
                    "UPDATE tournaments SET prize_pool = $2, players_remaining = $3, total_buyins = $4, total_fees = $5 WHERE id = $1::uuid",
                )
                .bind(tournament_id)
                .bind(store.state.prize_pool as i64)
                .bind(store.state.players_remaining as i32)
                .bind(store.state.players.len() as i32)
                .bind(store.state.total_fees as i64)
                .execute(&mut *tx)
                .await
                {
                    rollback_bot_registration(&self.env.tournaments, tournament_id, bot_id, snapshot).await;
                    return Err(BotError(format!("contadores: {e}")));
                }
            }
        }
        tx.commit()
            .await
            .map_err(|e| BotError(format!("commit: {e}")))?;
        Ok(())
    }

    async fn supervise_tournament(self: Arc<Self>, tournament_id: String) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            let bots = match self.tournament_deployments.read().await.get(&tournament_id) {
                Some(dep) => dep
                    .bot_ids
                    .iter()
                    .map(|bot_id| {
                        let personality = dep
                            .personalities
                            .get(bot_id)
                            .and_then(|s| Personality::parse(s))
                            .unwrap_or(Personality::Lag);
                        (bot_id.clone(), personality)
                    })
                    .collect::<Vec<_>>(),
                None => return, // deploy removido: encerra.
            };
            // Torneio fora do ar: encerra o supervisor (assentos viram blind-out).
            let alive = {
                let t = self.env.tournaments.read().await;
                t.get(&tournament_id).is_some_and(|s| {
                    matches!(
                        s.state.status,
                        poker_engine::tournament_engine::TournamentStatus::Registering
                            | poker_engine::tournament_engine::TournamentStatus::Running
                            | poker_engine::tournament_engine::TournamentStatus::Paused
                    )
                })
            };
            if !alive {
                let mut tasks = self.tournament_tasks.write().await;
                for (bot_id, _) in &bots {
                    if let Some(h) = tasks.remove(&(tournament_id.clone(), bot_id.clone())) {
                        h.abort();
                    }
                }
                return;
            }
            for (bot_id, personality) in bots {
                let key = (tournament_id.clone(), bot_id.clone());
                if self.tournament_tasks.read().await.contains_key(&key) {
                    continue;
                }
                // Só liga loop p/ bot com assento ACTIVE.
                let seated: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM tournament_seats WHERE tournament_id = $1::uuid AND player_id = $2 AND status = 'ACTIVE')",
                )
                .bind(&tournament_id)
                .bind(&bot_id)
                .fetch_one(&self.env.db)
                .await
                .unwrap_or(false);
                if !seated {
                    continue;
                }
                let fleet = Arc::clone(&self);
                let tid = tournament_id.clone();
                let handle = tokio::spawn(async move {
                    fleet.bot_tournament_loop(tid, bot_id, personality).await;
                });
                self.tournament_tasks.write().await.insert(key, handle);
            }
        }
    }

    /// Loop de jogo do bot no torneio: resolve a mesa viva atual a cada
    /// iteração (sobrevive a rebalance/consolidação) e joga lag_v2 com o
    /// perfil vivo da personalidade. Sai quando o deploy acaba, o bot
    /// elimina ou o torneio fecha.
    async fn bot_tournament_loop(
        self: Arc<Self>,
        tournament_id: String,
        bot_id: String,
        personality: Personality,
    ) {
        let seed = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B9)
            ^ bot_id.len() as u64
            ^ 0x85EBCA6B)
            .max(1);
        let mut profile = BotProfile::new(personality, seed);
        let mut rng = seed;
        let mut rnd = move || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };
        let mut last_table = String::new();
        let mut rx_opt: Option<tokio::sync::broadcast::Receiver<serde_json::Value>> = None;
        let mut last_sig: Option<(String, u64, u64, u64)> = None;
        loop {
            // Deploy removido? Sai.
            let deployed = self
                .tournament_deployments
                .read()
                .await
                .get(&tournament_id)
                .is_some_and(|d| d.bot_ids.iter().any(|b| b == &bot_id));
            if !deployed {
                return;
            }
            // Mesa viva atual do bot + variante vigente.
            let table_id: Option<String> = sqlx::query_as(
                "SELECT s.table_id::text FROM tournament_seats s \
                 WHERE s.tournament_id = $1::uuid AND s.player_id = $2 AND s.status = 'ACTIVE'",
            )
            .bind(&tournament_id)
            .bind(&bot_id)
            .fetch_optional(&self.env.db)
            .await
            .unwrap_or(None)
            .map(|(table_id,)| table_id);
            let variant = {
                let t = self.env.tournaments.read().await;
                t.get(&tournament_id)
                    .map(|s| s.active_poker_variant().to_string())
                    .unwrap_or_else(|| "holdem".to_string())
            };
            let Some(table_id) = table_id else {
                // Sem assento: eliminado ou torneio fechado.
                return;
            };
            if table_id != last_table {
                let handle = self.env.active_tables.read().await.get(&table_id).cloned();
                match handle {
                    Some(h) => {
                        rx_opt = Some(h.tx_broadcast.subscribe());
                        last_table = table_id.clone();
                        last_sig = None;
                    }
                    None => {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        continue;
                    }
                }
            }
            let rx = match rx_opt.as_mut() {
                Some(rx) => rx,
                None => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            };
            let msg = match rx.recv().await {
                Ok(m) => m,
                Err(_) => {
                    last_table.clear();
                    rx_opt = None;
                    continue;
                }
            };
            if msg.get("type").and_then(|v| v.as_str()) != Some("table_state") {
                continue;
            }
            profile.tick_mood(&mut rnd);
            let params = profile.params(&mut rnd);
            let Some((action, amount)) = decide_for(&msg, &bot_id, &variant, &params, &mut rnd)
            else {
                continue;
            };
            let players = msg
                .get("players")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let me = players
                .iter()
                .find(|p| p.get("id").and_then(|v| v.as_str()).unwrap_or("") == bot_id);
            let sig_src = me.map(|p| {
                (
                    msg.get("stage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
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
            tokio::time::sleep(tokio::time::Duration::from_millis(400 + rnd() % 1100)).await;
            // Re-resolve o ator (pode ter mudado de mesa no intervalo).
            let players = msg
                .get("players")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let me = players
                .iter()
                .find(|p| p.get("id").and_then(|v| v.as_str()).unwrap_or("") == bot_id);
            let sig_src = me.map(|p| {
                (
                    msg.get("stage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
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
            tokio::time::sleep(tokio::time::Duration::from_millis(400 + rnd() % 1100)).await;
            // Re-resolve o ator (pode ter mudado de mesa no intervalo).
            let goes: Option<tokio::sync::mpsc::Sender<PlayerCommand>> = self
                .env
                .active_tables
                .read()
                .await
                .get(&table_id)
                .map(|h| h.tx_cmd.clone());
            if let Some(tx) = goes {
                let _ = tx
                    .send(PlayerCommand::Action {
                        player_id: bot_id.clone(),
                        action,
                        amount,
                    })
                    .await;
            }
            last_sig = sig_src;
        }
    }

    // ─── Internos ───

    /// Replica o essencial do join (bots tem banca propria, sem reset diario).
    async fn seat_bot(
        &self,
        table_id: uuid::Uuid,
        bot_id: &str,
        buy_in: u64,
    ) -> Result<(), BotError> {
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
        let table_uuid = uuid::Uuid::parse_str(table_id)
            .map_err(|_| BotError("table_id invalido".to_string()))?;
        let row: Option<(
            String,
            i64,
            i64,
            i16,
            i64,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            String,
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
        let Some((
            table_name,
            small_blind,
            big_blind,
            rake_bp,
            rake_cap,
            cap_hu,
            cap_34,
            cap_5p,
            variant,
        )) = row
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
    variant: String,
    personality: Personality,
) {
    let mut rx = handle.tx_broadcast.subscribe();
    // Anti-loop: nao repete decisao no mesmo snapshot.
    let mut last_sig: Option<(String, u64, u64, u64)> = None;
    // RNG proprio (xorshift, sem deps): pensa como gente, 0.4–1.5s.
    let seed = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B9)
        ^ bot_id.len() as u64
        ^ 0x85EBCA6B)
        .max(1);
    let mut profile = BotProfile::new(personality, seed);
    let mut rng = seed;
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
        profile.tick_mood(&mut rnd);
        let params = profile.params(&mut rnd);
        let Some((action, amount)) = decide_for(&msg, &bot_id, &variant, &params, &mut rnd) else {
            continue;
        };
        let players = msg
            .get("players")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let me = players
            .iter()
            .find(|p| p.get("id").and_then(|v| v.as_str()).unwrap_or("") == bot_id);
        let sig_src = me.map(|p| {
            (
                msg.get("stage")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
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

// ─── Base compartilhada: tiers pre-flop/pos-flop (usados pela lag_v2) ───

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
    let pair_on_board = hole
        .iter()
        .any(|c| community.iter().any(|b| b.rank == c.rank));
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

// ─── lag_v2: avaliacao com o proprio motor, por variante ───

use poker_engine::deck::{
    evaluate_hand, evaluate_hand_short_deck, evaluate_hand_short_deck_omaha,
    evaluate_hand_ultimate_pineapple, HandRank, Rank as EngineRank, Suit as EngineSuit,
};

fn to_engine_card(c: Card) -> Option<poker_engine::deck::Card> {
    let rank = match c.rank {
        2 => EngineRank::Two,
        3 => EngineRank::Three,
        4 => EngineRank::Four,
        5 => EngineRank::Five,
        6 => EngineRank::Six,
        7 => EngineRank::Seven,
        8 => EngineRank::Eight,
        9 => EngineRank::Nine,
        10 => EngineRank::Ten,
        11 => EngineRank::Jack,
        12 => EngineRank::Queen,
        13 => EngineRank::King,
        14 => EngineRank::Ace,
        _ => return None,
    };
    let suit = match c.suit {
        0 => EngineSuit::Hearts,
        1 => EngineSuit::Diamonds,
        2 => EngineSuit::Clubs,
        3 => EngineSuit::Spades,
        _ => return None,
    };
    Some(poker_engine::deck::Card { rank, suit })
}

/// Categoria da melhor mao de 5 cartas, com as regras exatas da variante
/// (Omaha/Pineapple: 2 da mao + 3 do bordo; Short Deck: flush>FH, trips>straight).
fn evaluate_rank_for_variant(variant: &str, hole: &[Card], community: &[Card]) -> Option<HandRank> {
    let eh: Vec<poker_engine::deck::Card> =
        hole.iter().filter_map(|c| to_engine_card(*c)).collect();
    let ec: Vec<poker_engine::deck::Card> = community
        .iter()
        .filter_map(|c| to_engine_card(*c))
        .collect();
    if eh.len() != hole.len() || ec.len() != community.len() || community.len() < 3 {
        return None;
    }
    let res = match variant {
        "short_deck" => evaluate_hand_short_deck(&eh, &ec),
        "short_deck_omaha" => evaluate_hand_short_deck_omaha(&eh, &ec),
        "ultimate_pineapple" => evaluate_hand_ultimate_pineapple(&eh, &ec),
        _ => evaluate_hand(&eh, &ec),
    };
    Some(res.rank)
}

/// Pre-flop por quantidade de cartas: 2 = Hold'em/SD, 3 = Pineapple (melhor
/// par de 2), 4+ = Omaha (melhor par de 2 + bonus por coordenacao).
fn preflop_tier_v2(hole: &[Card]) -> u8 {
    match hole.len() {
        0 | 1 => 0,
        2 => preflop_tier(hole),
        3 => {
            let mut best = 0;
            for i in 0..3 {
                for j in (i + 1)..3 {
                    best = best.max(preflop_tier(&[hole[i], hole[j]]));
                }
            }
            best
        }
        _ => {
            let n = hole.len().min(4);
            let mut best = 0;
            let mut good = 0;
            for i in 0..n {
                for j in (i + 1)..n {
                    let t = preflop_tier(&[hole[i], hole[j]]);
                    if t >= 1 {
                        good += 1;
                    }
                    best = best.max(t);
                }
            }
            // Mao coordenada (2+ combos jogaveis, ex. double-suited): sobe p/ media.
            if best == 0 && good >= 2 {
                return 1;
            }
            best
        }
    }
}

/// Draws simples, valendo p/ qualquer baralho. Com short_deck=true conta
/// tambem a roda A-6-7-8-9 (flush draw e OESD).
fn has_bot_draw(hole: &[Card], community: &[Card], short_deck: bool) -> bool {
    let mut suits = [0u8; 4];
    let mut present = [false; 15];
    for c in hole.iter().chain(community.iter()) {
        suits[c.suit as usize] += 1;
        present[c.rank as usize] = true;
    }
    if suits.iter().any(|&n| n == 4) {
        return true;
    }
    if short_deck {
        let wheel: [usize; 5] = [14, 9, 8, 7, 6];
        let have = wheel.iter().filter(|&&r| present[r]).count();
        if have >= 4 {
            return true;
        }
    } else {
        present[1] = present[14]; // A joga baixo
    }
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
    best_run >= 4
}

/// Par simples (top pair/overpair/par medio): vale p/ Hold'em e Short Deck.
/// Em Omaha/Pineapple um par isolado quase nao vale: retorna 0.
fn pair_tier_v2(variant: &str, hole: &[Card], community: &[Card]) -> u8 {
    if community.is_empty() {
        return 0;
    }
    let board_hi = community.iter().map(|c| c.rank).max().unwrap_or(0);
    let hole_hi = hole.iter().map(|c| c.rank).max().unwrap_or(0);
    let paired = hole
        .iter()
        .any(|c| community.iter().any(|b| b.rank == c.rank));
    let pocket = hole.len() == 2 && hole[0].rank == hole[1].rank;
    match variant {
        "short_deck_omaha" | "ultimate_pineapple" => 0,
        _ => {
            if paired && (hole_hi >= board_hi || (pocket && hole_hi > board_hi)) {
                1
            } else if paired || pocket {
                1
            } else {
                0
            }
        }
    }
}

fn postflop_tier_v2(variant: &str, hole: &[Card], community: &[Card]) -> u8 {
    use HandRank::*;
    let short = variant != "holdem";
    let rank = evaluate_rank_for_variant(variant, hole, community);
    match rank {
        Some(FourOfAKind | StraightFlush | RoyalFlush) => return 2,
        Some(FullHouse | Flush) => return 2,
        Some(ThreeOfAKind) => {
            // Em Omaha trips sem full e vulneravel; nas outras e forte.
            if variant == "short_deck_omaha" || variant == "ultimate_pineapple" {
                return 1;
            }
            return 2;
        }
        Some(Straight) => {
            // Straight raramente e nuts em Omaha/Pineapple.
            if variant == "short_deck_omaha" || variant == "ultimate_pineapple" {
                return 1;
            }
            return if variant == "holdem" { 2 } else { 1 };
        }
        Some(TwoPair) => return 1,
        _ => {}
    }
    let pair = pair_tier_v2(variant, hole, community);
    if pair > 0 {
        return pair;
    }
    if has_bot_draw(hole, community, short) {
        return 1;
    }
    0
}

fn hand_tier_v2(variant: &str, hole: &[Card], community: &[Card]) -> u8 {
    if community.is_empty() {
        return preflop_tier_v2(hole);
    }
    // Hold'em mantem o comportamento v1 (paridade total).
    if variant == "holdem" {
        return postflop_tier(hole, community);
    }
    postflop_tier_v2(variant, hole, community)
}

/// Decide (acao, valor). None = nao e minha vez / sem fichas.
/// Os 6 botões de agressividade vêm do `params` (perfil vivo); a avaliação
/// da mão (tier) continua no motor por variante.
fn decide_for(
    state: &serde_json::Value,
    bot_id: &str,
    variant: &str,
    params: &BotParams,
    rnd: &mut dyn FnMut() -> u64,
) -> Option<(String, u64)> {
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
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str())
                .filter_map(parse_card)
                .collect()
        })
        .unwrap_or_default();
    if cards.len() < 2 {
        return None; // cartas ainda nao distribuidas
    }
    let community: Vec<Card> = state
        .get("community_cards")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str())
                .filter_map(parse_card)
                .collect()
        })
        .unwrap_or_default();
    let to_match = state
        .get("current_bet_to_match")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let min_raise = state
        .get("min_raise")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .max(1);
    let pot: u64 = state
        .get("pots")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|p| p.get("amount"))
                .filter_map(|v| v.as_u64())
                .sum()
        })
        .unwrap_or(0);
    let to_call = to_match.saturating_sub(my_bet).min(stack);
    let bb = min_raise.max(1);
    let tier = hand_tier_v2(variant, &cards, &community);
    let roll = rnd() % 100;

    // Short stack com algo na mao: shove (station nunca: shove_bb 0).
    if params.shove_bb > 0 && stack <= params.shove_bb * bb && tier >= 1 && to_call > 0 {
        return Some(("allin".to_string(), 0));
    }

    if to_call == 0 {
        // Posso pedir mesa gratis — mas nit folda lixo mesmo de graça.
        if community.is_empty() && tier == 0 && (rnd() % 100) < params.trash_fold as u64 {
            return Some(("fold".to_string(), 0));
        }
        match tier {
            2 => {
                // Forte: aposta 2/3 do pote (bet_strong%) ou mesa traiçoeira.
                if roll < params.bet_strong as u64 {
                    let amt = (pot * 2 / 3)
                        .clamp(min_raise, stack)
                        .max(min_raise)
                        .min(stack);
                    return Some(("bet".to_string(), amt));
                }
                return Some(("check".to_string(), 0));
            }
            1 => {
                if roll < params.bet_medium as u64 {
                    let amt = (pot / 2).clamp(min_raise, stack).max(min_raise).min(stack);
                    return Some(("bet".to_string(), amt));
                }
                return Some(("check".to_string(), 0));
            }
            _ => return Some(("check".to_string(), 0)),
        }
    }

    // Tem aposta: matematica de pote com o tempero do perfil.
    let pot_odds = to_call as f64 / (pot + to_call).max(1) as f64;
    let equity = match tier {
        2 => 0.65,
        1 => 0.35,
        _ => 0.08,
    };
    let odds_mult = params.call_odds_bp as f64 / 100.0;
    if tier == 2 && roll < params.raise_strong as u64 {
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
    if equity > pot_odds * odds_mult || to_call <= params.call_free_bb * bb {
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
        // Flush (5 do mesmo naipe)
        assert_eq!(
            postflop_tier(&[c("Ah"), c("2h")], &[c("5h"), c("9h"), c("Qh")]),
            2
        );
        // Nada
        assert_eq!(
            postflop_tier(&[c("7c"), c("2d")], &[c("As"), c("Kd"), c("Qh")]),
            0
        );
    }

    #[test]
    fn v2_engine_ranks_short_deck() {
        use poker_engine::deck::HandRank;
        // Flush vale mais que full house no Short Deck.
        assert_eq!(
            evaluate_rank_for_variant(
                "short_deck",
                &[c("Ah"), c("Kh")],
                &[c("Qh"), c("Jh"), c("9h")]
            ),
            Some(HandRank::Flush)
        );
        // Roda A-6-7-8-9 vale straight.
        assert_eq!(
            evaluate_rank_for_variant(
                "short_deck",
                &[c("Ah"), c("Kd")],
                &[c("9c"), c("8d"), c("7h"), c("6s"), c("2c")]
            ),
            Some(HandRank::Straight)
        );
    }

    #[test]
    fn v2_omaha_needs_two_plus_three() {
        use poker_engine::deck::HandRank;
        // 4 do mesmo naipe na mao + 1 no bordo NAO e flush (precisa 2+3).
        assert_ne!(
            evaluate_rank_for_variant(
                "short_deck_omaha",
                &[c("Ah"), c("Kh"), c("Qh"), c("Jh")],
                &[c("9h"), c("7c"), c("6d")]
            ),
            Some(HandRank::Flush)
        );
        // Com 2 do naipe na mao + 3 no bordo, flush existe.
        assert_eq!(
            evaluate_rank_for_variant(
                "short_deck_omaha",
                &[c("Ah"), c("Kh"), c("7c"), c("6d")],
                &[c("Qh"), c("Jh"), c("9h")]
            ),
            Some(HandRank::Flush)
        );
    }

    #[test]
    fn v2_preflop_hole_counts() {
        // Pineapple: melhor par de 2 (AAx = forte).
        assert_eq!(preflop_tier_v2(&[c("Ah"), c("Ad"), c("7c")]), 2);
        // Omaha: AA + coordenacao = forte.
        assert_eq!(preflop_tier_v2(&[c("Ah"), c("Ad"), c("Kh"), c("Qd")]), 2);
        // Lixo continua lixo mesmo com 4 cartas.
        assert_eq!(preflop_tier_v2(&[c("7c"), c("2d"), c("8h"), c("3s")]), 0);
    }

    // ─── Motor de personalidades ───

    fn seeded_rng(seed: u64) -> impl FnMut() -> u64 {
        let mut rng = seed.max(1);
        move || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        }
    }

    #[test]
    fn personalities_parse_and_list() {
        assert_eq!(Personality::parse("TAG"), Some(Personality::Tag));
        assert_eq!(
            Personality::parse("calling-station"),
            Some(Personality::Station)
        );
        assert_eq!(Personality::parse("rock"), Some(Personality::Nit));
        assert!(Personality::parse("maniac").is_none());
        assert_eq!(Personality::all().len(), 4);
    }

    #[test]
    fn params_stay_in_human_bands() {
        // 200 seeds × 4 perfis: nunca 0% nem 100%, call_odds com sanidade.
        for personality in [
            Personality::Nit,
            Personality::Tag,
            Personality::Lag,
            Personality::Station,
        ] {
            for seed in 1..=200u64 {
                let profile = BotProfile::new(personality, seed);
                let mut rnd = seeded_rng(seed * 7919 + 13);
                for _ in 0..5 {
                    let p = profile.params(&mut rnd);
                    for v in [p.bet_strong, p.bet_medium, p.raise_strong, p.trash_fold] {
                        assert!((5..=95).contains(&v), "{personality:?} fora da banda: {v}");
                    }
                    assert!((50..=200).contains(&p.call_odds_bp));
                }
            }
        }
    }

    #[test]
    fn same_personality_differs_by_seed() {
        // Jitter individual: dois TAGs não são clones.
        let mut seen = std::collections::HashSet::new();
        for seed in 1..=30u64 {
            let profile = BotProfile::new(Personality::Tag, seed);
            let mut rnd = seeded_rng(seed);
            let p = profile.params(&mut rnd);
            seen.insert((p.bet_strong, p.bet_medium, p.raise_strong));
        }
        assert!(seen.len() > 1, "jitter não diferencia bots iguais");
    }

    #[test]
    fn ordering_holds_on_average_with_overlap() {
        // Meio-termo: Nit folda lixo MAIS que LAG na média, mas as
        // distribuições se sobrepõem (nenhum robô 100%).
        let mut nit_folds = 0u32;
        let mut lag_folds = 0u32;
        let mut station_folds = 0u32;
        const N: u32 = 400;
        for i in 0..N {
            let mut rnd = seeded_rng(i as u64 * 131 + 7);
            let nit = BotProfile::new(Personality::Nit, i as u64 + 1).params(&mut rnd);
            let lag = BotProfile::new(Personality::Lag, i as u64 + 10001).params(&mut rnd);
            let station = BotProfile::new(Personality::Station, i as u64 + 20001).params(&mut rnd);
            let mut r2 = seeded_rng(i as u64 * 17 + 3);
            if r2() % 100 < nit.trash_fold as u64 {
                nit_folds += 1;
            }
            if r2() % 100 < lag.trash_fold as u64 {
                lag_folds += 1;
            }
            if r2() % 100 < station.trash_fold as u64 {
                station_folds += 1;
            }
        }
        assert!(
            nit_folds > lag_folds,
            "nit {nit_folds} deveria foldar mais que lag {lag_folds}"
        );
        assert!(lag_folds > station_folds);
        assert!(station_folds > 0, "station também folda às vezes");
        assert!(nit_folds < N, "nit também joga às vezes");
    }

    #[test]
    fn mood_stays_bounded_and_labeled() {
        let mut profile = BotProfile::new(Personality::Tag, 42);
        let mut rnd = seeded_rng(9);
        for _ in 0..500 {
            profile.tick_mood(&mut rnd);
            assert!((-10..=10).contains(&profile.mood));
        }
        assert_eq!(BotProfile::new(Personality::Lag, 1).mood_label(), "steady");
    }

    #[test]
    fn groups_expand_and_validate() {
        let g = expand_groups(&[], 6).unwrap();
        assert!(g.iter().all(|p| *p == Personality::Lag));
        let g = expand_groups(&[(Personality::Tag, 2), (Personality::Nit, 1)], 3).unwrap();
        assert_eq!(
            g,
            vec![Personality::Tag, Personality::Tag, Personality::Nit]
        );
        assert!(expand_groups(&[(Personality::Tag, 2)], 3).is_err());
        assert_eq!(
            parse_groups("tag:2,lag:4").unwrap(),
            vec![(Personality::Tag, 2), (Personality::Lag, 4)]
        );
        assert!(parse_groups("maniac:2").is_err());
        assert!(parse_groups("tag:0").is_err());
        assert!(parse_groups("").unwrap().is_empty());
    }

    #[test]
    fn station_never_shoves_and_nit_folds_trash() {
        let station = BotProfile::new(Personality::Station, 5);
        let mut rnd = seeded_rng(5);
        assert_eq!(station.params(&mut rnd).shove_bb, 0);
        // Nit com lixo pré-flop sem aposta: alta chance de fold.
        let nit = BotProfile::new(Personality::Nit, 7);
        let mut rnd = seeded_rng(7);
        let p = nit.params(&mut rnd);
        assert!((75..=95).contains(&p.trash_fold));
    }
}

/// Desfaz a inscrição do bot no motor (o banco já deu rollback sozinho).
async fn rollback_bot_registration(
    tournaments: &Arc<RwLock<HashMap<String, crate::tournament_store::TournamentStore>>>,
    tournament_id: &str,
    bot_id: &str,
    snapshot: (u64, u64, u64, u32),
) {
    let mut m = tournaments.write().await;
    if let Some(store) = m.get_mut(tournament_id) {
        store.state.players.remove(bot_id);
        store.state.total_buyins = snapshot.0;
        store.state.total_fees = snapshot.1;
        store.state.prize_pool = snapshot.2;
        store.state.players_remaining = snapshot.3;
    }
}
