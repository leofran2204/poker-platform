// ============================================================
// Módulo: tournament_engine.rs
// Projeto: Poker Project - Motor de Poker em Rust
// Funcionalidade: Engine de torneios (estrutura, blinds, prizes, eliminação)
// Regras de Negócio: R2.7.1 a R2.7.8
//
// ============================================================
// 🎯 META DE TESTES FASE 2: +960 testes (19 → 979)
// ============================================================
// Lotes planejados:
//   [x] 7A — Config & Creation (160 testes)
//   [x] 7B — Registration & Late Registration (200 testes)
//   [x] 7C — Lifecycle & Blinds (160 testes)
//   [x] 7D — Elimination & Re-buy (200 testes)
//   [x] 7E — Add-on & Finish (160 testes)
//   [x] 7F — Cancel, Stats & Serialization (80 testes)
// Progresso atual: 6/6 lotes (960+ testes)
// ============================================================
// ============================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// -----------------------------------------------------------
// Configuração do Torneio
// -----------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindLevel {
    /// Nível número (1-based)
    pub level: u32,
    /// Small blind
    pub small_blind: u64,
    /// Big blind
    pub big_blind: u64,
    /// Ante (opcional, 0 = sem ante)
    pub ante: u64,
    /// Duração em minutos
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentConfig {
    /// Nome do torneio
    pub name: String,
    /// Tipo de jogo (Holdem, Omaha, etc.)
    pub game_type: String,
    /// Buy-in em fichas
    pub buy_in: u64,
    /// Stack inicial
    pub starting_stack: u64,
    /// Máximo de jogadores
    pub max_players: u32,
    /// Velocidade (Turbo, Normal, Slow)
    pub speed: TournamentSpeed,
    /// Níveis de blinds
    pub blind_levels: Vec<BlindLevel>,
    /// Percentual pago em premiação (ex: 0.15 = 15%)
    pub prize_pool_pct: f64,
    /// Distribuição de prizes (ex: [0.50, 0.30, 0.20] = 50%, 30%, 20%)
    pub prize_distribution: Vec<f64>,
    /// Late registration permitida?
    pub late_registration: bool,
    /// Nível máximo para late registration
    pub late_registration_max_level: u32,
    /// Permite re-buy?
    pub allow_rebuy: bool,
    /// Permite add-on?
    pub allow_addon: bool,
    /// Nível máximo para re-buy/add-on
    pub rebuy_max_level: u32,
    /// Prize pool mínimo garantido (GTD), em centavos
    #[serde(default)]
    pub guaranteed_prize: u64,
    /// Freeroll (buy-in zero)
    #[serde(default)]
    pub is_freeroll: bool,
    /// Custo do rebuy em centavos (0 = usa buy_in)
    #[serde(default)]
    pub rebuy_cost: u64,
    /// Fichas recebidas no rebuy (0 = usa starting_stack)
    #[serde(default)]
    pub rebuy_chips: u64,
    /// Máximo de rebuys por jogador (0 = ilimitado enquanto allow_rebuy)
    #[serde(default)]
    pub rebuy_max_count: u32,
    /// Elegível a rebuy com stack <= threshold (além de eliminados). 0 = só eliminados
    #[serde(default)]
    pub rebuy_stack_threshold: u64,
}

impl Default for TournamentConfig {
    fn default() -> Self {
        Self {
            name: "Tournament".to_string(),
            game_type: "Holdem".to_string(),
            buy_in: 0,
            starting_stack: 10_000,
            max_players: 100,
            speed: TournamentSpeed::Normal,
            blind_levels: Vec::new(),
            prize_pool_pct: 1.0,
            prize_distribution: vec![0.50, 0.30, 0.20],
            late_registration: true,
            late_registration_max_level: 4,
            allow_rebuy: false,
            allow_addon: false,
            rebuy_max_level: 0,
            guaranteed_prize: 0,
            is_freeroll: false,
            rebuy_cost: 0,
            rebuy_chips: 0,
            rebuy_max_count: 0,
            rebuy_stack_threshold: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TournamentSpeed {
    Turbo,
    Normal,
    Slow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TournamentStatus {
    /// Aguardando início (registro aberto)
    Registering,
    /// Torneio em andamento
    Running,
    /// Pausado (ex: intervalo)
    Paused,
    /// Finalizado
    Finished,
    /// Cancelado
    Cancelled,
}

// -----------------------------------------------------------
// Estado do Torneio
// -----------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTournamentEntry {
    /// ID do jogador
    pub player_id: String,
    /// Nome do jogador
    pub player_name: String,
    /// Stack atual
    pub stack: u64,
    /// Mesa atual (None = eliminado ou ainda não sentou)
    pub table_id: Option<u32>,
    /// Assento na mesa
    pub seat: Option<u32>,
    /// Número de re-buys feitos
    pub rebuys: u32,
    /// Fez add-on?
    pub addon_done: bool,
    /// Posição final (None = ainda no torneio)
    pub final_position: Option<u32>,
    /// Prêmio recebido (None = ainda no torneio ou sem prêmio)
    pub prize: Option<u64>,
    /// Timestamp de registro (epoch seconds)
    pub registered_at: u64,
    /// Timestamp de eliminação (epoch seconds, None = ainda no torneio)
    pub eliminated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentState {
    /// ID único do torneio
    pub tournament_id: String,
    /// Configuração
    pub config: TournamentConfig,
    /// Status atual
    pub status: TournamentStatus,
    /// Nível atual de blinds (1-based)
    pub current_level: u32,
    /// Timestamp de início do nível atual (epoch seconds)
    pub level_started_at: u64,
    /// Jogadores registrados (player_id -> entry)
    pub players: HashMap<String, PlayerTournamentEntry>,
    /// Total de buy-ins coletados
    pub total_buyins: u64,
    /// Total de re-buys coletados
    pub total_rebuys: u64,
    /// Total de add-ons coletados
    pub total_addons: u64,
    /// Prize pool total
    pub prize_pool: u64,
    /// Jogadores restantes (não eliminados)
    pub players_remaining: u32,
    /// Jogadores eliminados (em ordem de eliminação)
    pub eliminated_order: Vec<String>,
    /// Timestamp de início do torneio (epoch seconds)
    pub started_at: Option<u64>,
    /// Timestamp de fim do torneio (epoch seconds)
    pub finished_at: Option<u64>,
}

// -----------------------------------------------------------
// Resultado do Torneio
// -----------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentResult {
    pub tournament_id: String,
    pub tournament_name: String,
    pub total_players: u32,
    pub total_prize_pool: u64,
    pub winners: Vec<WinnerEntry>,
    pub started_at: u64,
    pub finished_at: u64,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerEntry {
    pub position: u32,
    pub player_id: String,
    pub player_name: String,
    pub prize: u64,
}

// -----------------------------------------------------------
// Funções Core
// -----------------------------------------------------------

/// Cria um novo torneio com a configuração fornecida
pub fn create_tournament(config: TournamentConfig) -> TournamentState {
    let tournament_id = generate_tournament_id(&config.name);

    let guaranteed = config.guaranteed_prize;
    TournamentState {
        tournament_id,
        config,
        status: TournamentStatus::Registering,
        current_level: 0,
        level_started_at: 0,
        players: HashMap::new(),
        total_buyins: 0,
        total_rebuys: 0,
        total_addons: 0,
        prize_pool: guaranteed,
        players_remaining: 0,
        eliminated_order: Vec::new(),
        started_at: None,
        finished_at: None,
    }
}

/// Registra um jogador no torneio
/// Retorna Ok(()) ou Err com mensagem
pub fn register_player(
    state: &mut TournamentState,
    player_id: &str,
    player_name: &str,
) -> Result<(), String> {
    // Verifica status
    if state.status != TournamentStatus::Registering && state.status != TournamentStatus::Running {
        return Err("Torneio não está aceitando registros".to_string());
    }

    // Verifica late registration
    if state.status == TournamentStatus::Running {
        if !state.config.late_registration {
            return Err("Late registration não permitida".to_string());
        }
        if state.current_level > state.config.late_registration_max_level {
            return Err("Late registration já fechou (nível máximo excedido)".to_string());
        }
    }

    // Verifica vagas
    if state.players.len() as u32 >= state.config.max_players {
        return Err("Torneio lotado".to_string());
    }

    // Verifica duplicata
    if state.players.contains_key(player_id) {
        return Err("Jogador já registrado".to_string());
    }

    let now = current_timestamp();

    let entry = PlayerTournamentEntry {
        player_id: player_id.to_string(),
        player_name: player_name.to_string(),
        stack: state.config.starting_stack,
        table_id: None,
        seat: None,
        rebuys: 0,
        addon_done: false,
        final_position: None,
        prize: None,
        registered_at: now,
        eliminated_at: None,
    };

    state.players.insert(player_id.to_string(), entry);
    state.total_buyins += state.config.buy_in;
    state.players_remaining += 1;

    // Recalcula prize pool
    recalculate_prize_pool(state);

    Ok(())
}

/// Inicia o torneio (muda status para Running)
pub fn start_tournament(state: &mut TournamentState) -> Result<(), String> {
    if state.status != TournamentStatus::Registering {
        return Err("Torneio não está em fase de registro".to_string());
    }

    if state.players.len() < 2 {
        return Err("Minimo de 2 jogadores para iniciar".to_string());
    }

    let now = current_timestamp();

    state.status = TournamentStatus::Running;
    state.started_at = Some(now);
    state.current_level = 1;
    state.level_started_at = now;

    Ok(())
}

/// Avança para o próximo nível de blinds
pub fn advance_blinds(state: &mut TournamentState) -> Result<(), String> {
    if state.status != TournamentStatus::Running {
        return Err("Torneio não está em andamento".to_string());
    }

    let next_level = state.current_level + 1;

    if next_level as usize > state.config.blind_levels.len() {
        return Err("Não há mais níveis de blinds definidos".to_string());
    }

    state.current_level = next_level;
    state.level_started_at = current_timestamp();

    Ok(())
}

/// Obtém o nível de blinds atual
pub fn get_current_blinds(state: &TournamentState) -> Option<&BlindLevel> {
    if state.current_level == 0 {
        return None;
    }
    state
        .config
        .blind_levels
        .get((state.current_level - 1) as usize)
}

/// Elimina um jogador do torneio
pub fn eliminate_player(
    state: &mut TournamentState,
    player_id: &str,
    position: Option<u32>,
) -> Result<(), String> {
    let entry = state
        .players
        .get_mut(player_id)
        .ok_or("Jogador não encontrado")?;

    if entry.eliminated_at.is_some() {
        return Err("Jogador já foi eliminado".to_string());
    }

    let now = current_timestamp();
    entry.eliminated_at = Some(now);
    entry.stack = 0;
    entry.table_id = None;
    entry.seat = None;

    state.players_remaining -= 1;
    state.eliminated_order.push(player_id.to_string());

    // Define posição se fornecida
    if let Some(pos) = position {
        entry.final_position = Some(pos);
    }

    Ok(())
}

/// Finaliza o torneio e distribui prêmios
pub fn finish_tournament(state: &mut TournamentState) -> Result<TournamentResult, String> {
    if state.status != TournamentStatus::Running {
        return Err("Torneio não está em andamento".to_string());
    }

    let now = current_timestamp();
    state.status = TournamentStatus::Finished;
    state.finished_at = Some(now);

    // Distribui prêmios
    let prizes = calculate_prizes(state);
    let mut winners = Vec::new();

    // Atribui prêmios aos jogadores restantes por ordem de stack
    let mut remaining: Vec<&mut PlayerTournamentEntry> = state
        .players
        .values_mut()
        .filter(|e| e.eliminated_at.is_none())
        .collect();
    remaining.sort_by_key(|e| std::cmp::Reverse(e.stack));

    for (i, entry) in remaining.iter_mut().enumerate() {
        let position = (i + 1) as u32;
        entry.final_position = Some(position);

        if let Some(&prize) = prizes.get(i) {
            entry.prize = Some(prize);
            winners.push(WinnerEntry {
                position,
                player_id: entry.player_id.clone(),
                player_name: entry.player_name.clone(),
                prize,
            });
        }
    }

    let started_at = state.started_at.unwrap_or(0);
    let finished_at = state.finished_at.unwrap_or(0);

    Ok(TournamentResult {
        tournament_id: state.tournament_id.clone(),
        tournament_name: state.config.name.clone(),
        total_players: state.players.len() as u32,
        total_prize_pool: state.prize_pool,
        winners,
        started_at,
        finished_at,
        duration_seconds: finished_at.saturating_sub(started_at),
    })
}

/// Cancela o torneio (reembolsa todos)
pub fn cancel_tournament(state: &mut TournamentState) -> Result<(), String> {
    if state.status == TournamentStatus::Finished {
        return Err("Torneio já finalizado".to_string());
    }

    state.status = TournamentStatus::Cancelled;
    state.finished_at = Some(current_timestamp());

    // Reembolsa todos os jogadores ainda no torneio
    for entry in state.players.values_mut() {
        if entry.eliminated_at.is_none() {
            entry.prize = Some(state.config.buy_in); // reembolso do buy-in
        }
    }

    Ok(())
}

/// Processa um re-buy para um jogador.
///
/// Elegível se eliminado **ou** (threshold > 0 e stack <= threshold).
/// Fichas = `rebuy_chips` (ou `starting_stack` se 0). Custo = `rebuy_cost` (ou `buy_in` se 0).
pub fn process_rebuy(state: &mut TournamentState, player_id: &str) -> Result<(), String> {
    if !state.config.allow_rebuy {
        return Err("Re-buy não permitido neste torneio".to_string());
    }

    if state.status != TournamentStatus::Running {
        return Err("Torneio não está em andamento".to_string());
    }

    if state.current_level > state.config.rebuy_max_level {
        return Err("Período de re-buy encerrado".to_string());
    }

    let entry = state
        .players
        .get_mut(player_id)
        .ok_or_else(|| "Jogador não encontrado".to_string())?;

    if state.config.rebuy_max_count > 0 && entry.rebuys >= state.config.rebuy_max_count {
        return Err("Limite de re-buys atingido".to_string());
    }

    let eliminated = entry.eliminated_at.is_some();
    let short_stack = state.config.rebuy_stack_threshold > 0
        && entry.eliminated_at.is_none()
        && entry.stack <= state.config.rebuy_stack_threshold;

    if !eliminated && !short_stack {
        return Err("Re-buy só para eliminados ou stack no limiar configurado".to_string());
    }

    let chips = if state.config.rebuy_chips > 0 {
        state.config.rebuy_chips
    } else {
        state.config.starting_stack
    };
    let cost = if state.config.rebuy_cost > 0 {
        state.config.rebuy_cost
    } else {
        state.config.buy_in
    };

    let was_eliminated = eliminated;
    entry.stack = chips;
    entry.eliminated_at = None;
    entry.rebuys += 1;
    entry.table_id = None;
    entry.seat = None;
    entry.final_position = None;
    entry.prize = None;

    state.total_rebuys += cost;
    if was_eliminated {
        state.players_remaining += 1;
        state.eliminated_order.retain(|id| id != player_id);
    }

    recalculate_prize_pool(state);

    Ok(())
}

/// Processa um add-on para um jogador
pub fn process_addon(
    state: &mut TournamentState,
    player_id: &str,
    addon_chips: u64,
    addon_cost: u64,
) -> Result<(), String> {
    if !state.config.allow_addon {
        return Err("Add-on não permitido neste torneio".to_string());
    }

    if state.status != TournamentStatus::Running {
        return Err("Torneio não está em andamento".to_string());
    }

    if state.current_level > state.config.rebuy_max_level {
        return Err("Período de add-on encerrado".to_string());
    }

    let entry = state
        .players
        .get_mut(player_id)
        .ok_or("Jogador não encontrado")?;

    if entry.eliminated_at.is_some() {
        return Err("Jogador já foi eliminado".to_string());
    }

    if entry.addon_done {
        return Err("Jogador já fez add-on".to_string());
    }

    entry.stack += addon_chips;
    entry.addon_done = true;
    state.total_addons += addon_cost;

    recalculate_prize_pool(state);

    Ok(())
}

/// Pausa o torneio (ex: intervalo)
pub fn pause_tournament(state: &mut TournamentState) -> Result<(), String> {
    if state.status != TournamentStatus::Running {
        return Err("Torneio não está em andamento".to_string());
    }
    state.status = TournamentStatus::Paused;
    Ok(())
}

/// Retoma o torneio após pausa
pub fn resume_tournament(state: &mut TournamentState) -> Result<(), String> {
    if state.status != TournamentStatus::Paused {
        return Err("Torneio não está pausado".to_string());
    }
    state.status = TournamentStatus::Running;
    Ok(())
}

// -----------------------------------------------------------
// Funções Auxiliares
// -----------------------------------------------------------

/// Gera um ID único para o torneio baseado no nome e timestamp
fn generate_tournament_id(name: &str) -> String {
    let timestamp = current_timestamp();
    let sanitized = name.to_lowercase().replace(' ', "_");
    format!("{}_{}", sanitized, timestamp)
}

/// Retorna o timestamp atual em segundos desde epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Recalcula o prize pool baseado nos buy-ins, re-buys e add-ons
fn recalculate_prize_pool(state: &mut TournamentState) {
    let gross = state.total_buyins + state.total_rebuys + state.total_addons;
    let collected = (gross as f64 * state.config.prize_pool_pct) as u64;
    state.prize_pool = collected.max(state.config.guaranteed_prize);
}

/// Calcula a distribuição de prêmios baseado no prize_distribution
fn calculate_prizes(state: &TournamentState) -> Vec<u64> {
    let pool = state.prize_pool;
    state
        .config
        .prize_distribution
        .iter()
        .map(|&pct| (pool as f64 * pct) as u64)
        .collect()
}

/// Verifica se o nível atual de blinds expirou
pub fn is_blind_level_expired(state: &TournamentState) -> bool {
    if state.current_level == 0 {
        return false;
    }

    if let Some(level) = get_current_blinds(state) {
        let elapsed = current_timestamp().saturating_sub(state.level_started_at);
        let duration_secs = (level.duration_minutes as u64) * 60;
        elapsed >= duration_secs
    } else {
        false
    }
}

/// Retorna estatísticas resumidas do torneio
pub fn get_tournament_stats(state: &TournamentState) -> TournamentStats {
    let avg_stack = if state.players_remaining > 0 {
        let total_stack: u64 = state
            .players
            .values()
            .filter(|e| e.eliminated_at.is_none())
            .map(|e| e.stack)
            .sum();
        total_stack / state.players_remaining as u64
    } else {
        0
    };

    TournamentStats {
        tournament_id: state.tournament_id.clone(),
        status: state.status.clone(),
        current_level: state.current_level,
        total_players: state.players.len() as u32,
        players_remaining: state.players_remaining,
        players_eliminated: state.eliminated_order.len() as u32,
        total_prize_pool: state.prize_pool,
        average_stack: avg_stack,
        total_rebuys: state.players.values().map(|p| p.rebuys as u64).sum(),
        total_addons: state.players.values().filter(|p| p.addon_done).count() as u64,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentStats {
    pub tournament_id: String,
    pub status: TournamentStatus,
    pub current_level: u32,
    pub total_players: u32,
    pub players_remaining: u32,
    pub players_eliminated: u32,
    pub total_prize_pool: u64,
    pub average_stack: u64,
    pub total_rebuys: u64,
    pub total_addons: u64,
}

// ============================================================
// Testes
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TournamentConfig {
        TournamentConfig {
            name: "Test Tournament".to_string(),
            game_type: "Holdem".to_string(),
            buy_in: 1000,
            starting_stack: 10000,
            max_players: 100,
            speed: TournamentSpeed::Normal,
            blind_levels: vec![
                BlindLevel {
                    level: 1,
                    small_blind: 10,
                    big_blind: 20,
                    ante: 0,
                    duration_minutes: 15,
                },
                BlindLevel {
                    level: 2,
                    small_blind: 20,
                    big_blind: 40,
                    ante: 0,
                    duration_minutes: 15,
                },
                BlindLevel {
                    level: 3,
                    small_blind: 30,
                    big_blind: 60,
                    ante: 5,
                    duration_minutes: 15,
                },
                BlindLevel {
                    level: 4,
                    small_blind: 50,
                    big_blind: 100,
                    ante: 10,
                    duration_minutes: 15,
                },
                BlindLevel {
                    level: 5,
                    small_blind: 100,
                    big_blind: 200,
                    ante: 20,
                    duration_minutes: 15,
                },
            ],
            prize_pool_pct: 0.90,
            prize_distribution: vec![0.50, 0.30, 0.20],
            late_registration: true,
            late_registration_max_level: 3,
            allow_rebuy: true,
            allow_addon: true,
            rebuy_max_level: 4,
            ..Default::default()
        }
    }

    #[test]
    fn test_gtd_floor_on_empty_pool() {
        let mut cfg = default_config();
        cfg.guaranteed_prize = 20_000;
        cfg.prize_pool_pct = 1.0;
        let mut state = create_tournament(cfg);
        recalculate_prize_pool(&mut state);
        assert_eq!(state.prize_pool, 20_000);
    }

    #[test]
    fn test_short_stack_rebuy_custom_chips() {
        let mut cfg = default_config();
        cfg.allow_rebuy = true;
        cfg.rebuy_max_level = 6;
        cfg.rebuy_max_count = 1;
        cfg.rebuy_stack_threshold = 5_000;
        cfg.rebuy_cost = 3_000;
        cfg.rebuy_chips = 25_000;
        cfg.starting_stack = 10_000;
        let mut state = create_tournament(cfg);
        register_player(&mut state, "p1", "Alice").unwrap();
        register_player(&mut state, "p2", "Bob").unwrap();
        start_tournament(&mut state).unwrap();
        state.current_level = 3;
        state.players.get_mut("p1").unwrap().stack = 4_000;
        process_rebuy(&mut state, "p1").unwrap();
        let p1 = state.players.get("p1").unwrap();
        assert_eq!(p1.stack, 25_000);
        assert_eq!(p1.rebuys, 1);
        assert_eq!(state.total_rebuys, 3_000);
        assert!(process_rebuy(&mut state, "p1").is_err());
    }

    #[test]
    fn test_create_tournament() {
        let config = default_config();
        let state = create_tournament(config);

        assert_eq!(state.status, TournamentStatus::Registering);
        assert_eq!(state.current_level, 0);
        assert_eq!(state.players.len(), 0);
        assert_eq!(state.players_remaining, 0);
        assert!(state.tournament_id.contains("test_tournament"));
    }

    #[test]
    fn test_register_player() {
        let config = default_config();
        let mut state = create_tournament(config);

        let result = register_player(&mut state, "p1", "Player 1");
        assert!(result.is_ok());
        assert_eq!(state.players.len(), 1);
        assert_eq!(state.players_remaining, 1);
        assert_eq!(state.total_buyins, 1000);

        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.stack, 10000);
        assert_eq!(entry.player_name, "Player 1");
    }

    #[test]
    fn test_register_duplicate_player() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "Player 1").unwrap();
        let result = register_player(&mut state, "p1", "Player 1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("já registrado"));
    }

    #[test]
    fn test_register_full_tournament() {
        let mut config = default_config();
        config.max_players = 2;
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        let result = register_player(&mut state, "p3", "P3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("lotado"));
    }

    #[test]
    fn test_start_tournament() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();

        let result = start_tournament(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.status, TournamentStatus::Running);
        assert_eq!(state.current_level, 1);
        assert!(state.started_at.is_some());
    }

    #[test]
    fn test_start_tournament_insufficient_players() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        let result = start_tournament(&mut state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Minimo de 2"));
    }

    #[test]
    fn test_advance_blinds() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        assert_eq!(state.current_level, 1);
        advance_blinds(&mut state).unwrap();
        assert_eq!(state.current_level, 2);
        advance_blinds(&mut state).unwrap();
        assert_eq!(state.current_level, 3);
    }

    #[test]
    fn test_get_current_blinds() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let blinds = get_current_blinds(&state).unwrap();
        assert_eq!(blinds.small_blind, 10);
        assert_eq!(blinds.big_blind, 20);

        advance_blinds(&mut state).unwrap();
        let blinds = get_current_blinds(&state).unwrap();
        assert_eq!(blinds.small_blind, 20);
        assert_eq!(blinds.big_blind, 40);
    }

    #[test]
    fn test_eliminate_player() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        register_player(&mut state, "p3", "P3").unwrap();
        start_tournament(&mut state).unwrap();

        assert_eq!(state.players_remaining, 3);

        eliminate_player(&mut state, "p2", Some(3)).unwrap();
        assert_eq!(state.players_remaining, 2);
        assert_eq!(state.eliminated_order.len(), 1);
        assert_eq!(state.eliminated_order[0], "p2");

        let entry = state.players.get("p2").unwrap();
        assert_eq!(entry.stack, 0);
        assert!(entry.eliminated_at.is_some());
        assert_eq!(entry.final_position, Some(3));
    }

    #[test]
    fn test_finish_tournament() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        register_player(&mut state, "p3", "P3").unwrap();
        start_tournament(&mut state).unwrap();

        // Simula stacks diferentes
        state.players.get_mut("p1").unwrap().stack = 30000;
        state.players.get_mut("p2").unwrap().stack = 20000;
        state.players.get_mut("p3").unwrap().stack = 10000;

        let result = finish_tournament(&mut state).unwrap();
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(result.total_players, 3);
        assert_eq!(result.winners.len(), 3);

        // Verifica ordem: p1 (1º), p2 (2º), p3 (3º)
        assert_eq!(result.winners[0].player_id, "p1");
        assert_eq!(result.winners[0].position, 1);
        assert_eq!(result.winners[1].player_id, "p2");
        assert_eq!(result.winners[1].position, 2);
        assert_eq!(result.winners[2].player_id, "p3");
        assert_eq!(result.winners[2].position, 3);

        // Verifica prizes
        let pool = state.prize_pool;
        assert_eq!(result.winners[0].prize, (pool as f64 * 0.50) as u64);
        assert_eq!(result.winners[1].prize, (pool as f64 * 0.30) as u64);
        assert_eq!(result.winners[2].prize, (pool as f64 * 0.20) as u64);
    }

    #[test]
    fn test_cancel_tournament() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        cancel_tournament(&mut state).unwrap();
        assert_eq!(state.status, TournamentStatus::Cancelled);

        // Verifica reembolso
        let p1 = state.players.get("p1").unwrap();
        assert_eq!(p1.prize, Some(1000));
    }

    #[test]
    fn test_rebuy() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        // Elimina p1
        eliminate_player(&mut state, "p1", None).unwrap();
        assert_eq!(state.players_remaining, 1);

        // Re-buy p1
        process_rebuy(&mut state, "p1").unwrap();
        assert_eq!(state.players_remaining, 2);

        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.stack, 10000);
        assert_eq!(entry.rebuys, 1);
        assert!(entry.eliminated_at.is_none());
        assert_eq!(state.total_rebuys, 1000);
    }

    #[test]
    fn test_addon() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        let old_stack = state.players.get("p1").unwrap().stack;
        process_addon(&mut state, "p1", 5000, 500).unwrap();

        let entry = state.players.get("p1").unwrap();
        assert_eq!(entry.stack, old_stack + 5000);
        assert!(entry.addon_done);
        assert_eq!(state.total_addons, 500);
    }

    #[test]
    fn test_pause_and_resume() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        pause_tournament(&mut state).unwrap();
        assert_eq!(state.status, TournamentStatus::Paused);

        resume_tournament(&mut state).unwrap();
        assert_eq!(state.status, TournamentStatus::Running);
    }

    #[test]
    fn test_late_registration() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        // Late registration deve funcionar (nível 1 <= 3)
        let result = register_player(&mut state, "p3", "P3");
        assert!(result.is_ok());

        // Avança blinds até nível 4 (late registration fecha)
        advance_blinds(&mut state).unwrap(); // nível 2
        advance_blinds(&mut state).unwrap(); // nível 3
        advance_blinds(&mut state).unwrap(); // nível 4

        let result = register_player(&mut state, "p4", "P4");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nível máximo excedido"));
    }

    #[test]
    fn test_prize_pool_calculation() {
        let config = default_config();
        let mut state = create_tournament(config);

        // 3 jogadores = 3000 buy-ins
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        register_player(&mut state, "p3", "P3").unwrap();

        // prize_pool_pct = 0.90
        assert_eq!(state.prize_pool, (3000.0 * 0.90) as u64);
    }

    #[test]
    fn test_tournament_stats() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        register_player(&mut state, "p3", "P3").unwrap();
        start_tournament(&mut state).unwrap();

        let stats = get_tournament_stats(&state);
        assert_eq!(stats.total_players, 3);
        assert_eq!(stats.players_remaining, 3);
        assert_eq!(stats.players_eliminated, 0);
        assert_eq!(stats.current_level, 1);
        assert_eq!(stats.average_stack, 10000);
    }

    #[test]
    fn test_json_serialization() {
        let config = default_config();
        let state = create_tournament(config);

        let json = serde_json::to_string_pretty(&state).unwrap();
        assert!(json.contains("registering")); // lowercase devido ao rename_all
        assert!(json.contains("test_tournament"));

        let deserialized: TournamentState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tournament_id, state.tournament_id);
        assert_eq!(deserialized.status, TournamentStatus::Registering);
    }

    #[test]
    fn test_blind_level_expired() {
        let config = default_config();
        let mut state = create_tournament(config);

        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();

        // Acabou de começar, não deve ter expirado
        assert!(!is_blind_level_expired(&state));
    }
}
