// game_loop.rs — Máquina de Estados do Game Loop (Texas Hold'em)
// Criado em 2026-07-14 | Parte da FASE 4 — Game Loop
//
// Este módulo é o ORQUESTRADOR da mão de poker. Ele conecta todos os
// módulos do motor (deck, side_pots, rake, loss_deflator, hand_history)
// em uma máquina de estados que executa uma mão completa de Texas Hold'em
// do pré-flop ao showdown.
//
// Fluxo da mão:
//   1. start_hand()     — coleta blinds, distribui hole cards, entra em Preflop
//   2. player_action()  — processa fold/check/call/bet/raise/all-in
//   3. advance_phase()  — quando a rodada de apostas fecha, avança fase
//      Preflop → Flop (burn + 3 cartas) → Turn (burn + 1) → River (burn + 1) → Showdown
//   4. resolve_hand()    — calcula side pots, rake, loss deflator, distribui prêmios
//   5. finalize_history — grava HandHistory completo
//
// Design:
//   - O GameLoop é STATEFUL (única exceção ao motor stateless)
//   - Cada método muta o estado interno de forma determinística
//   - Todas as decisões financeiras delegam para módulos existentes
//   - O GameLoop NÃO toma decisões de IA — apenas executa ações solicitadas

use crate::deck::{compare_hands, create_deck, deal_cards, evaluate_hand, shuffle_deck, Card, HandResult};
use crate::hand_history::{
    self, Action, EndReason, GameType, HandHistory, PlayerAction, PlayerResult, TableConfig as HistoryTableConfig,
};
use crate::loss_deflator::{self, ProgressiveLossDeflatorParams};
use crate::rake::{self, RakeResult};
use crate::side_pots::{self, PlayerForPots, SidePotsResult};
use crate::types::{GamePhase, Pot, TableConfig};
use crate::utils::truncar_2_casas;
use std::collections::HashMap;

// ─── Tipos de estado ───

/// Estado individual de um jogador durante a mão
#[derive(Debug, Clone)]
pub struct PlayerState {
    /// ID único do jogador
    pub id: String,
    /// Fichas restantes (stack atual)
    pub stack: f64,
    /// Hole cards (2 cartas)
    pub hole_cards: Vec<Card>,
    /// Aposta da rodada atual (reseta a cada fase)
    pub current_bet: f64,
    /// Total apostado na mão inteira (para side pots)
    pub total_bet: f64,
    /// Se o jogador foldou
    pub has_folded: bool,
    /// Se o jogador está all-in
    pub is_all_in: bool,
    /// Se o jogador já agiu nesta rodada de apostas
    pub has_acted: bool,
    /// Índice do assento (0-based)
    pub seat_index: usize,
}

impl PlayerState {
    /// Cria um novo jogador com stack inicial
    pub fn new(id: String, stack: f64, seat_index: usize) -> Self {
        Self {
            id,
            stack,
            hole_cards: Vec::new(),
            current_bet: 0.0,
            total_bet: 0.0,
            has_folded: false,
            is_all_in: false,
            has_acted: false,
            seat_index,
        }
    }

    /// Verifica se o jogador pode agir (não foldou, não está all-in)
    pub fn can_act(&self) -> bool {
        !self.has_folded && !self.is_all_in
    }

    /// Verifica se o jogador ainda está na mão (não foldou)
    pub fn is_in_hand(&self) -> bool {
        !self.has_folded
    }

    /// Zera a aposta da rodada (chamado ao avançar de fase)
    pub fn reset_round_bet(&mut self) {
        self.current_bet = 0.0;
        self.has_acted = false;
    }
}

/// Estado completo da mão em andamento
#[derive(Debug, Clone)]
pub struct HandState {
    /// Jogadores na mão (em ordem de assento)
    pub players: Vec<PlayerState>,
    /// Índice do dealer (botão)
    pub dealer_index: usize,
    /// Cartas comunitárias reveladas
    pub community_cards: Vec<Card>,
    /// Fase atual do jogo
    pub phase: GamePhase,
    /// Baralho restante (após embaralhar e distribuir)
    pub deck: Vec<Card>,
    /// Cartas queimadas (burn pile)
    pub burn_pile: Vec<Card>,
    /// Aposta máxima da rodada atual (o que cada jogador precisa igualar)
    pub current_bet_to_match: f64,
    /// Aumento mínimo permitido (geralmente = big blind)
    pub min_raise: f64,
    /// Índice do jogador que deve agir agora
    pub active_player_index: usize,
    /// Valor do small blind
    pub small_blind: f64,
    /// Valor do big blind
    pub big_blind: f64,
    /// Se a mão já terminou
    pub is_finished: bool,
}

impl HandState {
    /// Conta jogadores ativos (não foldados, não all-in)
    pub fn active_players_count(&self) -> usize {
        self.players.iter().filter(|p| p.can_act()).count()
    }

    /// Conta jogadores ainda na mão (não foldados)
    pub fn players_in_hand_count(&self) -> usize {
        self.players.iter().filter(|p| p.is_in_hand()).count()
    }

    /// Retorna o jogador ativo atual
    pub fn active_player(&self) -> Option<&PlayerState> {
        self.players.get(self.active_player_index)
    }

    /// Retorna o jogador ativo atual (mutável)
    pub fn active_player_mut(&mut self) -> Option<&mut PlayerState> {
        self.players.get_mut(self.active_player_index)
    }

    /// Soma total apostado na mão (para o pote)
    pub fn total_pot(&self) -> f64 {
        self.players.iter().map(|p| p.total_bet).sum()
    }

    /// Encontra o próximo jogador que pode agir a partir do índice dado
    pub fn next_active_player(&self, from: usize) -> Option<usize> {
        if self.players.is_empty() {
            return None;
        }
        let n = self.players.len();
        for i in 1..=n {
            let idx = (from + i) % n;
            if self.players[idx].can_act() {
                return Some(idx);
            }
        }
        None
    }
}

/// Resultado da resolução da mão (showdown ou fold generalizado)
#[derive(Debug, Clone)]
pub struct HandResolution {
    /// Pots calculados (antes do rake)
    pub pots: Vec<Pot>,
    /// Pagamentos finais por jogador (após rake e deflator)
    pub payouts: HashMap<String, f64>,
    /// Rake total cobrado
    pub rake: f64,
    /// Resultado do loss deflator (se aplicável)
    pub loss_deflator: Option<loss_deflator::ProgressiveLossDeflatorResult>,
    /// Resultados finais por jogador (para hand history)
    pub player_results: Vec<PlayerResult>,
    /// Fase em que a mão terminou
    pub end_phase: GamePhase,
    /// Motivo do fim da mão
    pub end_reason: EndReason,
}

/// Erros possíveis durante o game loop
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameLoopError {
    /// Jogador não encontrado
    PlayerNotFound(String),
    /// Não é a vez do jogador
    NotYourTurn(String),
    /// Jogador não pode agir (foldou ou all-in)
    PlayerCannotAct(String),
    /// Valor de aposta inválido
    InvalidBetAmount(String),
    /// Aumento menor que o mínimo
    RaiseTooSmall(String),
    /// Stack insuficiente
    InsufficientStack(String),
    /// Ação inválida para a fase atual
    InvalidActionForPhase(String),
    /// Mão já terminou
    HandAlreadyFinished,
    /// Mão não iniciada
    HandNotStarted,
    /// Menos de 2 jogadores
    NotEnoughPlayers,
}

impl std::fmt::Display for GameLoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameLoopError::PlayerNotFound(id) => write!(f, "Jogador não encontrado: {}", id),
            GameLoopError::NotYourTurn(id) => write!(f, "Não é a vez de {}", id),
            GameLoopError::PlayerCannotAct(id) => write!(f, "{} não pode agir", id),
            GameLoopError::InvalidBetAmount(msg) => write!(f, "Valor de aposta inválido: {}", msg),
            GameLoopError::RaiseTooSmall(msg) => write!(f, "Aumento menor que o mínimo: {}", msg),
            GameLoopError::InsufficientStack(msg) => write!(f, "Stack insuficiente: {}", msg),
            GameLoopError::InvalidActionForPhase(msg) => write!(f, "Ação inválida: {}", msg),
            GameLoopError::HandAlreadyFinished => write!(f, "A mão já terminou"),
            GameLoopError::HandNotStarted => write!(f, "A mão não foi iniciada"),
            GameLoopError::NotEnoughPlayers => write!(f, "Menos de 2 jogadores"),
        }
    }
}

impl std::error::Error for GameLoopError {}

// ─── Tipos de ação do jogador ───

/// Ação que um jogador pode tomar durante uma rodada de apostas
#[derive(Debug, Clone)]
pub enum PlayerMove {
    /// Abandona a mão
    Fold,
    /// Passa (apenas se ninguém apostou)
    Check,
    /// Iguala a aposta atual
    Call,
    /// Faz uma aposta inicial (quando ninguém apostou)
    Bet(f64),
    /// Aumenta a aposta atual
    Raise(f64),
    /// Vai all-in com todas as fichas restantes
    AllIn,
}

// ─── GameLoop principal ───

/// Máquina de estados que orquestra uma mão completa de Texas Hold'em
pub struct GameLoop {
    /// Estado da mão atual
    pub state: HandState,
    /// Configuração da mesa (rake)
    pub config: TableConfig,
    /// ID da mão (para hand history)
    pub hand_id: String,
    /// Nome da mesa (para hand history)
    pub table_name: String,
    /// Tipo de jogo (cash ou tournament)
    pub game_type: GameType,
    /// Ante (opcional)
    pub ante: Option<f64>,
    /// Histórico da mão (construído conforme a mão progride)
    pub history: Option<HandHistory>,
    /// Timestamp de início da mão (ms)
    pub start_timestamp: u64,
    /// Contador de ações (para timestamp relativo no histórico)
    action_counter: u64,
}

impl GameLoop {
    /// Cria um novo GameLoop com a configuração da mesa
    pub fn new(
        config: TableConfig,
        hand_id: String,
        table_name: String,
        game_type: GameType,
    ) -> Self {
        Self {
            state: HandState {
                players: Vec::new(),
                dealer_index: 0,
                community_cards: Vec::new(),
                phase: GamePhase::Preflop,
                deck: Vec::new(),
                burn_pile: Vec::new(),
                current_bet_to_match: 0.0,
                min_raise: config.big_blind,
                active_player_index: 0,
                small_blind: config.big_blind / 2.0,
                big_blind: config.big_blind,
                is_finished: false,
            },
            config,
            hand_id,
            table_name,
            game_type,
            ante: None,
            history: None,
            start_timestamp: now_timestamp_ms(),
            action_counter: 0,
        }
    }

    /// Define o ante para a mão
    pub fn with_ante(mut self, ante: f64) -> Self {
        self.ante = Some(ante);
        self
    }

    /// Adiciona um jogador à mão (antes de iniciar)
    pub fn add_player(&mut self, id: String, stack: f64) {
        let seat_index = self.state.players.len();
        self.state.players.push(PlayerState::new(id, stack, seat_index));
    }

    /// Define quem é o dealer (botão)
    pub fn set_dealer(&mut self, dealer_index: usize) {
        self.state.dealer_index = dealer_index;
    }

    /// Inicia a mão: coleta blinds, distribui hole cards, define primeira ação
    pub fn start_hand(&mut self) -> Result<(), GameLoopError> {
        if self.state.players.len() < 2 {
            return Err(GameLoopError::NotEnoughPlayers);
        }
        if self.state.is_finished {
            return Err(GameLoopError::HandAlreadyFinished);
        }

        // 1. Criar e embaralhar baralho
        let full_deck = create_deck();
        self.state.deck = shuffle_deck(&full_deck);

        // 2. Coletar ante (se configurado)
        if let Some(ante) = self.ante {
            if ante > 0.0 {
                for player in &mut self.state.players {
                    let ante_amount = ante.min(player.stack);
                    player.stack -= ante_amount;
                    player.total_bet += ante_amount;
                }
            }
        }

        // 3. Coletar blinds
        let sb_index = self.small_blind_index();
        let bb_index = self.big_blind_index();

        // Small blind
        let sb_amount = self.state.small_blind.min(self.state.players[sb_index].stack);
        self.state.players[sb_index].stack -= sb_amount;
        self.state.players[sb_index].current_bet = sb_amount;
        self.state.players[sb_index].total_bet = sb_amount;

        // Big blind
        let bb_amount = self.state.big_blind.min(self.state.players[bb_index].stack);
        self.state.players[bb_index].stack -= bb_amount;
        self.state.players[bb_index].current_bet = bb_amount;
        self.state.players[bb_index].total_bet = bb_amount;

        // Verificar all-in por blinds
        if self.state.players[sb_index].stack == 0.0 {
            self.state.players[sb_index].is_all_in = true;
        }
        if self.state.players[bb_index].stack == 0.0 {
            self.state.players[bb_index].is_all_in = true;
        }

        // 4. Distribuir hole cards (2 por jogador)
        for _ in 0..2 {
            for player in &mut self.state.players {
                let (cards, remaining) = deal_cards(&self.state.deck, 1);
                player.hole_cards.extend(cards);
                self.state.deck = remaining;
            }
        }

        // 5. Definir aposta a igualar = big blind
        self.state.current_bet_to_match = self.state.big_blind;
        self.state.min_raise = self.state.big_blind;

        // 6. Primeiro a agir = UTG (após o big blind)
        // Heads-up: o dealer (SB) age primeiro pré-flop
        if self.state.players.len() == 2 {
            self.state.active_player_index = sb_index;
        } else {
            self.state.active_player_index = self.next_player_after(bb_index);
        }

        // 7. Inicializar hand history
        let players_ids: Vec<String> = self.state.players.iter().map(|p| p.id.clone()).collect();
        let starting_stacks: HashMap<String, u64> = self
            .state
            .players
            .iter()
            .map(|p| (p.id.clone(), p.stack as u64 + p.total_bet as u64))
            .collect();

        let history_config = HistoryTableConfig {
            table_name: self.table_name.clone(),
            small_blind: self.state.small_blind as u64,
            big_blind: self.state.big_blind as u64,
            ante: self.ante.map(|a| a as u64),
            max_players: 9,
            game_type: self.game_type,
        };

        let mut history = hand_history::create_hand_history(
            self.hand_id.clone(),
            history_config,
            players_ids,
            starting_stacks,
        );

        // Registrar blinds no histórico
        let sb_id = self.state.players[sb_index].id.clone();
        let bb_id = self.state.players[bb_index].id.clone();
        self.record_history_action(&mut history, &sb_id, Action::Call, sb_amount as u64);
        self.record_history_action(&mut history, &bb_id, Action::Raise, bb_amount as u64);

        self.history = Some(history);

        Ok(())
    }

    /// Processa a ação de um jogador
    pub fn player_action(&mut self, player_id: &str, move_type: PlayerMove) -> Result<(), GameLoopError> {
        if self.state.is_finished {
            return Err(GameLoopError::HandAlreadyFinished);
        }

        // Validar que é a vez do jogador
        let active_idx = self.state.active_player_index;
        if self.state.players[active_idx].id != player_id {
            return Err(GameLoopError::NotYourTurn(player_id.to_string()));
        }

        let player = &self.state.players[active_idx];
        if !player.can_act() {
            return Err(GameLoopError::PlayerCannotAct(player_id.to_string()));
        }

        let current_bet_to_match = self.state.current_bet_to_match;
        let player_current = self.state.players[active_idx].current_bet;
        let to_call = current_bet_to_match - player_current;

        match move_type {
            PlayerMove::Fold => {
                self.state.players[active_idx].has_folded = true;
                self.state.players[active_idx].has_acted = true;
                self.record_history_action_id(active_idx, Action::Fold, 0);
                // Se apenas 1 jogador (ou menos) ainda está na mão, a mão termina
                if self.state.players_in_hand_count() <= 1 {
                    self.state.is_finished = true;
                    return Ok(());
                }
            }
            PlayerMove::Check => {
                if to_call > 0.0 {
                    return Err(GameLoopError::InvalidActionForPhase(
                        "Check não permitido quando há aposta a igualar".to_string(),
                    ));
                }
                self.state.players[active_idx].has_acted = true;
                self.record_history_action_id(active_idx, Action::Check, 0);
            }
            PlayerMove::Call => {
                if to_call <= 0.0 {
                    return Err(GameLoopError::InvalidActionForPhase(
                        "Call não permitido quando não há aposta a igualar".to_string(),
                    ));
                }
                let call_amount = to_call.min(self.state.players[active_idx].stack);
                self.state.players[active_idx].stack -= call_amount;
                self.state.players[active_idx].current_bet += call_amount;
                self.state.players[active_idx].total_bet += call_amount;
                if self.state.players[active_idx].stack == 0.0 {
                    self.state.players[active_idx].is_all_in = true;
                }
                self.state.players[active_idx].has_acted = true;
                self.record_history_action_id(active_idx, Action::Call, call_amount as u64);
            }
            PlayerMove::Bet(amount) => {
                if to_call > 0.0 {
                    return Err(GameLoopError::InvalidActionForPhase(
                        "Bet não permitido quando há aposta — use Raise".to_string(),
                    ));
                }
                if amount <= 0.0 {
                    return Err(GameLoopError::InvalidBetAmount("Aposta deve ser positiva".to_string()));
                }
                if amount < self.state.min_raise {
                    return Err(GameLoopError::RaiseTooSmall(format!(
                        "Aposta mínima: {}",
                        self.state.min_raise
                    )));
                }
                if amount > self.state.players[active_idx].stack {
                    return Err(GameLoopError::InsufficientStack(player_id.to_string()));
                }
                self.state.players[active_idx].stack -= amount;
                self.state.players[active_idx].current_bet = amount;
                self.state.players[active_idx].total_bet += amount;
                self.state.current_bet_to_match = amount;
                self.state.min_raise = amount;
                if self.state.players[active_idx].stack == 0.0 {
                    self.state.players[active_idx].is_all_in = true;
                }
                // Resetar has_acted dos outros jogadores (nova rodada de apostas)
                self.reset_other_players_acted(active_idx);
                self.state.players[active_idx].has_acted = true;
                self.record_history_action_id(active_idx, Action::Bet, amount as u64);
            }
            PlayerMove::Raise(amount) => {
                // Raise é permitido quando:
                // - Há aposta a igualar (to_call > 0), OU
                // - O jogador já igualou mas quer aumentar (current_bet_to_match > 0)
                if current_bet_to_match <= 0.0 {
                    return Err(GameLoopError::InvalidActionForPhase(
                        "Raise não permitido quando não há aposta — use Bet".to_string(),
                    ));
                }
                // amount = valor total da aposta (não o incremento)
                let raise_increment = amount - current_bet_to_match;
                if raise_increment < self.state.min_raise {
                    return Err(GameLoopError::RaiseTooSmall(format!(
                        "Aumento mínimo: {} (raise de {})",
                        self.state.min_raise,
                        raise_increment
                    )));
                }
                let total_needed = amount - player_current;
                if total_needed > self.state.players[active_idx].stack {
                    // All-in raise (menor que o mínimo, mas válido se all-in)
                    let all_in_amount = player_current + self.state.players[active_idx].stack;
                    self.state.players[active_idx].stack = 0.0;
                    self.state.players[active_idx].current_bet = all_in_amount;
                    self.state.players[active_idx].total_bet += all_in_amount - player_current;
                    self.state.players[active_idx].is_all_in = true;
                    if all_in_amount > current_bet_to_match {
                        self.state.current_bet_to_match = all_in_amount;
                        self.reset_other_players_acted(active_idx);
                    }
                    self.record_history_action_id(active_idx, Action::AllIn, all_in_amount as u64);
                } else {
                    self.state.players[active_idx].stack -= total_needed;
                    self.state.players[active_idx].current_bet = amount;
                    self.state.players[active_idx].total_bet += total_needed;
                    self.state.current_bet_to_match = amount;
                    self.state.min_raise = raise_increment;
                    if self.state.players[active_idx].stack == 0.0 {
                        self.state.players[active_idx].is_all_in = true;
                    }
                    self.reset_other_players_acted(active_idx);
                    self.record_history_action_id(active_idx, Action::Raise, amount as u64);
                }
                self.state.players[active_idx].has_acted = true;
            }
            PlayerMove::AllIn => {
                let all_in_amount = self.state.players[active_idx].stack;
                let new_total_bet = player_current + all_in_amount;
                self.state.players[active_idx].stack = 0.0;
                self.state.players[active_idx].current_bet = new_total_bet;
                self.state.players[active_idx].total_bet += all_in_amount;
                self.state.players[active_idx].is_all_in = true;
                self.state.players[active_idx].has_acted = true;

                if new_total_bet > current_bet_to_match {
                    let raise_increment = new_total_bet - current_bet_to_match;
                    if raise_increment >= self.state.min_raise {
                        self.state.min_raise = raise_increment;
                    }
                    self.state.current_bet_to_match = new_total_bet;
                    self.reset_other_players_acted(active_idx);
                }
                self.record_history_action_id(active_idx, Action::AllIn, all_in_amount as u64);
            }
        }

        // Verificar se a rodada de apostas terminou
        if self.is_betting_round_complete() {
            // Se só sobrou 1 jogador, a mão termina
            if self.state.players_in_hand_count() <= 1 {
                self.state.is_finished = true;
            } else {
                // Avançar para a próxima fase
                self.advance_phase()?;
            }
        } else {
            // Passar a vez para o próximo jogador ativo
            self.advance_to_next_player();
        }

        Ok(())
    }

    /// Avança para a próxima fase do jogo (flop, turn, river, showdown)
    pub fn advance_phase(&mut self) -> Result<(), GameLoopError> {
        // Resetar apostas da rodada
        for player in &mut self.state.players {
            player.reset_round_bet();
        }
        self.state.current_bet_to_match = 0.0;
        self.state.min_raise = self.state.big_blind;

        let next_phase = self.state.phase.next().ok_or(GameLoopError::InvalidActionForPhase(
            "Já está em showdown".to_string(),
        ))?;

        match next_phase {
            GamePhase::Flop => {
                // Burn 1, deal 3
                let (burn, d1) = deal_cards(&self.state.deck, 1);
                self.state.burn_pile.extend(burn);
                self.state.deck = d1;
                let (flop, d2) = deal_cards(&self.state.deck, 3);
                self.state.community_cards.extend(flop.clone());
                self.state.deck = d2;
                if let Some(h) = &mut self.history {
                    hand_history::set_community_cards(h, GamePhase::Flop, flop);
                }
            }
            GamePhase::Turn => {
                let (burn, d1) = deal_cards(&self.state.deck, 1);
                self.state.burn_pile.extend(burn);
                self.state.deck = d1;
                let (turn_card, d2) = deal_cards(&self.state.deck, 1);
                self.state.community_cards.extend(turn_card.clone());
                self.state.deck = d2;
                if let Some(h) = &mut self.history {
                    hand_history::set_community_cards(h, GamePhase::Turn, turn_card);
                }
            }
            GamePhase::River => {
                let (burn, d1) = deal_cards(&self.state.deck, 1);
                self.state.burn_pile.extend(burn);
                self.state.deck = d1;
                let (river_card, d2) = deal_cards(&self.state.deck, 1);
                self.state.community_cards.extend(river_card.clone());
                self.state.deck = d2;
                if let Some(h) = &mut self.history {
                    hand_history::set_community_cards(h, GamePhase::River, river_card);
                }
            }
            GamePhase::Showdown => {
                // No showdown, não há mais apostas
                self.state.is_finished = true;
            }
            GamePhase::Preflop => {
                return Err(GameLoopError::InvalidActionForPhase(
                    "Não é possível voltar para preflop".to_string(),
                ));
            }
        }

        self.state.phase = next_phase;

        // Se não for showdown, definir primeiro a agir
        if next_phase != GamePhase::Showdown {
            self.set_first_to_act_postflop();
        }

        // Se todos estão all-in ou só sobrou 1 ativo, avançar automaticamente até showdown
        if self.state.active_players_count() <= 1 && !self.state.is_finished {
            // Se há apenas 1 ou 0 jogadores ativos, distribuir as cartas restantes e ir ao showdown
            self.run_out_board()?;
        }

        Ok(())
    }

    /// Resolve a mão: calcula side pots, rake, loss deflator, distribui prêmios
    pub fn resolve_hand(&mut self) -> Result<HandResolution, GameLoopError> {
        if !self.state.is_finished {
            return Err(GameLoopError::HandNotStarted);
        }

        let players_in_hand: Vec<&PlayerState> = self.state.players.iter().filter(|p| p.is_in_hand()).collect();

        // Se todos foldaram exceto um → vencedor por fold
        if players_in_hand.len() == 1 {
            return self.resolve_fold_win();
        }

        // Showdown: calcular side pots e distribuir
        self.resolve_showdown()
    }

    /// Finaliza o hand history com os resultados da resolução
    pub fn finalize_history(&mut self, resolution: &HandResolution) {
        if let Some(h) = &mut self.history {
            let total_pot = self.state.total_pot();
            hand_history::finalize_hand(
                h,
                resolution.player_results.clone(),
                total_pot as u64,
                resolution.rake as u64,
                resolution.end_phase,
                resolution.end_reason,
            );
        }
    }

    /// Retorna o hand history finalizado (se disponível)
    pub fn get_history(&self) -> Option<&HandHistory> {
        self.history.as_ref()
    }

    // ─── Métodos internos ───

    /// Índice do small blind (após o dealer)
    fn small_blind_index(&self) -> usize {
        // Heads-up: o dealer é o small blind
        if self.state.players.len() == 2 {
            self.state.dealer_index
        } else {
            self.next_player_after(self.state.dealer_index)
        }
    }

    /// Índice do big blind (após o small blind)
    fn big_blind_index(&self) -> usize {
        self.next_player_after(self.small_blind_index())
    }

    /// Próximo jogador ativo após o índice dado
    fn next_player_after(&self, from: usize) -> usize {
        let n = self.state.players.len();
        (from + 1) % n
    }

    /// Avança o active_player_index para o próximo jogador que pode agir
    fn advance_to_next_player(&mut self) {
        if let Some(next) = self.state.next_active_player(self.state.active_player_index) {
            self.state.active_player_index = next;
        }
    }

    /// Define o primeiro a agir no pós-flop (primeiro ativo à esquerda do dealer)
    fn set_first_to_act_postflop(&mut self) {
        let start = self.state.dealer_index;
        if let Some(idx) = self.state.next_active_player(start) {
            self.state.active_player_index = idx;
        }
    }

    /// Reseta has_acted de todos os jogadores exceto o atual (nova rodada de apostas)
    fn reset_other_players_acted(&mut self, current_idx: usize) {
        for (i, player) in self.state.players.iter_mut().enumerate() {
            if i != current_idx && player.can_act() {
                player.has_acted = false;
            }
        }
    }

    /// Verifica se a rodada de apostas atual está completa
    fn is_betting_round_complete(&self) -> bool {
        let active_players: Vec<&PlayerState> = self
            .state
            .players
            .iter()
            .filter(|p| p.is_in_hand() && !p.is_all_in)
            .collect();

        // Se não há jogadores ativos, a rodada está completa
        if active_players.is_empty() {
            return true;
        }

        // Todos os jogadores ativos devem ter agido E igualado a aposta
        for player in &active_players {
            if !player.has_acted {
                return false;
            }
            if player.current_bet != self.state.current_bet_to_match {
                return false;
            }
        }

        true
    }

    /// Distribui as cartas comunitárias restantes (quando todos estão all-in)
    fn run_out_board(&mut self) -> Result<(), GameLoopError> {
        // Avançar até showdown distribuindo todas as cartas
        while self.state.phase != GamePhase::Showdown {
            let next = self.state.phase.next().ok_or(GameLoopError::InvalidActionForPhase(
                "Erro ao avançar para showdown".to_string(),
            ))?;

            match next {
                GamePhase::Flop => {
                    let (burn, d1) = deal_cards(&self.state.deck, 1);
                    self.state.burn_pile.extend(burn);
                    self.state.deck = d1;
                    let (flop, d2) = deal_cards(&self.state.deck, 3);
                    self.state.community_cards.extend(flop.clone());
                    self.state.deck = d2;
                    if let Some(h) = &mut self.history {
                        hand_history::set_community_cards(h, GamePhase::Flop, flop);
                    }
                }
                GamePhase::Turn => {
                    let (burn, d1) = deal_cards(&self.state.deck, 1);
                    self.state.burn_pile.extend(burn);
                    self.state.deck = d1;
                    let (turn_card, d2) = deal_cards(&self.state.deck, 1);
                    self.state.community_cards.extend(turn_card.clone());
                    self.state.deck = d2;
                    if let Some(h) = &mut self.history {
                        hand_history::set_community_cards(h, GamePhase::Turn, turn_card);
                    }
                }
                GamePhase::River => {
                    let (burn, d1) = deal_cards(&self.state.deck, 1);
                    self.state.burn_pile.extend(burn);
                    self.state.deck = d1;
                    let (river_card, d2) = deal_cards(&self.state.deck, 1);
                    self.state.community_cards.extend(river_card.clone());
                    self.state.deck = d2;
                    if let Some(h) = &mut self.history {
                        hand_history::set_community_cards(h, GamePhase::River, river_card);
                    }
                }
                _ => {}
            }

            self.state.phase = next;
        }

        self.state.is_finished = true;
        Ok(())
    }

    /// Resolve vitória por fold (todos foldaram exceto um)
    fn resolve_fold_win(&mut self) -> Result<HandResolution, GameLoopError> {
        let winner_idx = self
            .state
            .players
            .iter()
            .position(|p| p.is_in_hand())
            .ok_or(GameLoopError::InvalidActionForPhase("Nenhum jogador na mão".to_string()))?;

        let winner_id = self.state.players[winner_idx].id.clone();
        let total_pot = self.state.total_pot();

        // Sem showdown → sem side pots complexos, sem rake sobre fold win
        // (convenção comum: rake só em pots que vão ao showdown)
        let mut payouts = HashMap::new();
        payouts.insert(winner_id.clone(), truncar_2_casas(total_pot));

        // Resultados dos jogadores para hand history
        let mut player_results = Vec::new();
        for (i, player) in self.state.players.iter().enumerate() {
            let is_winner = i == winner_idx;
            player_results.push(PlayerResult {
                player_id: player.id.clone(),
                finish_position: if is_winner { 1 } else { 2 },
                hole_cards: player.hole_cards.clone(),
                best_hand: None,
                best_hand_name: None,
                chips_won: if is_winner { total_pot as u64 } else { 0 },
                chips_lost: player.total_bet as u64,
                folded: player.has_folded,
                was_all_in: player.is_all_in,
            });
        }

        Ok(HandResolution {
            pots: vec![Pot::new(total_pot, vec![winner_id.clone()])],
            payouts,
            rake: 0.0,
            loss_deflator: None,
            player_results,
            end_phase: self.state.phase,
            end_reason: EndReason::AllFolded,
        })
    }

    /// Resolve showdown: calcula side pots, rake, loss deflator, distribui
    fn resolve_showdown(&mut self) -> Result<HandResolution, GameLoopError> {
        // 1. Construir PlayerForPots para side_pots
        let players_for_pots: Vec<PlayerForPots> = self
            .state
            .players
            .iter()
            .map(|p| PlayerForPots {
                id: p.id.clone(),
                total_bet: p.total_bet,
                has_folded: p.has_folded,
                cards: p.hole_cards.clone(),
            })
            .collect();

        // 2. Calcular side pots e distribuir
        let side_pots_result: SidePotsResult =
            side_pots::resolve_side_pots(&players_for_pots, &self.state.community_cards);

        let pots = side_pots_result.pots.clone();
        let mut payouts = side_pots_result.payouts.clone();

        // 3. Deduzir rake
        let rake_result: RakeResult = rake::deduct_rake(&pots, &self.config, None);
        let total_rake = rake_result.total_rake;

        // Aplicar rake: subtrair rake proporcionalmente dos payouts
        for pot_entry in &rake_result.per_pot {
            let pot_idx = pot_entry.pot_index;
            let rake_from_pot = pot_entry.rake;
            // Subtrair rake proporcionalmente dos vencedores deste pot
            let pot = &pots[pot_idx];
            let pot_total = pot.amount;
            if pot_total > 0.0 {
                for eligible_id in &pot.eligible_players {
                    if let Some(winnings) = payouts.get_mut(eligible_id) {
                        let proportion = *winnings / pot_total;
                        *winnings -= truncar_2_casas(rake_from_pot * proportion);
                    }
                }
            }
        }

        // 4. Loss deflator (se houve all-in pré-showdown)
        let loss_deflator_result = self.calculate_loss_deflator(&pots, &payouts);

        // Se houver cashback, adicioná-lo aos payouts
        if let Some(ref deflator) = loss_deflator_result {
            let loser_id = &deflator.loser_id;
            if let Some(loser_payout) = payouts.get_mut(loser_id) {
                *loser_payout += deflator.cashback;
            } else {
                payouts.insert(loser_id.clone(), deflator.cashback);
            }
        }

        // 5. Truncar todos os payouts para 2 casas
        for (_, amount) in payouts.iter_mut() {
            *amount = truncar_2_casas(*amount);
        }

        // 6. Construir player_results para hand history
        let player_results = self.build_player_results(&payouts, &side_pots_result);

        Ok(HandResolution {
            pots,
            payouts,
            rake: total_rake,
            loss_deflator: loss_deflator_result,
            player_results,
            end_phase: GamePhase::Showdown,
            end_reason: EndReason::Showdown,
        })
    }

    /// Calcula o loss deflator para o maior perdedor do showdown
    fn calculate_loss_deflator(
        &self,
        pots: &[Pot],
        payouts: &HashMap<String, f64>,
    ) -> Option<loss_deflator::ProgressiveLossDeflatorResult> {
        // Encontrar o maior perdedor (jogador que apostou mais e recebeu menos)
        // que esteve em all-in
        let mut biggest_loser: Option<(String, f64)> = None; // (id, net_loss)
        let mut winner_id = String::new();

        for player in &self.state.players {
            if player.has_folded || !player.is_in_hand() {
                continue;
            }
            let won = payouts.get(&player.id).copied().unwrap_or(0.0);
            let net = won - player.total_bet;
            if net < 0.0 {
                match &biggest_loser {
                    None => {
                        biggest_loser = Some((player.id.clone(), net));
                    }
                    Some((_, loss)) if net < *loss => {
                        biggest_loser = Some((player.id.clone(), net));
                    }
                    _ => {}
                }
            } else if winner_id.is_empty() || net > 0.0 {
                winner_id = player.id.clone();
            }
        }

        let (loser_id, _) = biggest_loser?;

        // Determinar a fase do all-in (usar a fase atual como aproximação)
        // Em uma implementação completa, rastrearíamos a fase exata do all-in
        let phase = self.state.phase;

        let params = ProgressiveLossDeflatorParams {
            pots: pots.to_vec(),
            loser_id: loser_id.clone(),
            winner_id,
            phase,
        };

        loss_deflator::calculate_progressive_loss_deflator(params)
    }

    /// Constrói a lista de PlayerResult para hand history
    fn build_player_results(
        &self,
        payouts: &HashMap<String, f64>,
        _side_pots_result: &SidePotsResult,
    ) -> Vec<PlayerResult> {
        // Avaliar mãos de todos os jogadores que não foldaram
        let mut hand_evals: HashMap<String, HandResult> = HashMap::new();
        for player in &self.state.players {
            if player.is_in_hand() && !player.hole_cards.is_empty() {
                let eval = evaluate_hand(&player.hole_cards, &self.state.community_cards);
                hand_evals.insert(player.id.clone(), eval);
            }
        }

        // Ordenar jogadores por força da mão (para finish_position)
        let mut ranked: Vec<(String, HandResult)> = hand_evals
            .iter()
            .map(|(id, hr)| (id.clone(), hr.clone()))
            .collect();
        ranked.sort_by(|a, b| compare_hands(&a.1, &b.1).reverse());

        let mut player_results = Vec::new();
        for player in &self.state.players {
            let won = payouts.get(&player.id).copied().unwrap_or(0.0);
            let eval = hand_evals.get(&player.id);
            let finish_position = if player.has_folded {
                ranked.len() + 1
            } else {
                ranked.iter().position(|(id, _)| id == &player.id).map(|i| i + 1).unwrap_or(ranked.len() + 1)
            };

            player_results.push(PlayerResult {
                player_id: player.id.clone(),
                finish_position: finish_position as u8,
                hole_cards: player.hole_cards.clone(),
                best_hand: eval.cloned(),
                best_hand_name: eval.map(|e| get_hand_rank_name(e.rank)),
                chips_won: won as u64,
                chips_lost: player.total_bet as u64,
                folded: player.has_folded,
                was_all_in: player.is_all_in,
            });
        }

        player_results
    }

    /// Registra uma ação no hand history (por índice do jogador)
    fn record_history_action_id(&mut self, player_idx: usize, action: Action, amount: u64) {
        if let Some(h) = &mut self.history {
            let player_id = self.state.players[player_idx].id.clone();
            self.action_counter += 1;
            let pa = PlayerAction {
                player_id,
                action,
                amount,
                phase: self.state.phase,
                timestamp_ms: self.action_counter * 100,
            };
            hand_history::record_action(h, pa);
        }
    }

    /// Registra uma ação no hand history (por ID do jogador)
    fn record_history_action(&mut self, history: &mut HandHistory, player_id: &str, action: Action, amount: u64) {
        self.action_counter += 1;
        let pa = PlayerAction {
            player_id: player_id.to_string(),
            action,
            amount,
            phase: self.state.phase,
            timestamp_ms: self.action_counter * 100,
        };
        hand_history::record_action(history, pa);
    }
}

// ─── Funções auxiliares ───

/// Retorna o nome legível de um HandRank
fn get_hand_rank_name(rank: crate::deck::HandRank) -> String {
    use crate::deck::HandRank;
    match rank {
        HandRank::HighCard => "High Card".to_string(),
        HandRank::OnePair => "One Pair".to_string(),
        HandRank::TwoPair => "Two Pair".to_string(),
        HandRank::ThreeOfAKind => "Three of a Kind".to_string(),
        HandRank::Straight => "Straight".to_string(),
        HandRank::Flush => "Flush".to_string(),
        HandRank::FullHouse => "Full House".to_string(),
        HandRank::FourOfAKind => "Four of a Kind".to_string(),
        HandRank::StraightFlush => "Straight Flush".to_string(),
        HandRank::RoyalFlush => "Royal Flush".to_string(),
    }
}

/// Retorna o timestamp atual em milissegundos (UNIX epoch)
fn now_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> TableConfig {
        TableConfig::new(10.0, 5.0, 5.0) // BB=10, rake=5%, cap=5
    }

    fn make_game_loop_with_2_players() -> GameLoop {
        let config = make_config();
        let mut gl = GameLoop::new(config, "hand-001".to_string(), "Test Table".to_string(), GameType::Cash);
        gl.add_player("alice".to_string(), 1000.0);
        gl.add_player("bob".to_string(), 1000.0);
        gl.set_dealer(0);
        gl
    }

    #[test]
    fn test_start_hand_deals_hole_cards() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        assert_eq!(gl.state.players.len(), 2);
        assert_eq!(gl.state.players[0].hole_cards.len(), 2);
        assert_eq!(gl.state.players[1].hole_cards.len(), 2);
        assert_eq!(gl.state.phase, GamePhase::Preflop);
        assert!(!gl.state.is_finished);
    }

    #[test]
    fn test_start_hand_collects_blinds() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Heads-up: dealer = SB, outro = BB
        // Alice (idx 0) é dealer → SB = 5
        // Bob (idx 1) → BB = 10
        assert_eq!(gl.state.players[0].current_bet, 5.0, "Alice SB");
        assert_eq!(gl.state.players[1].current_bet, 10.0, "Bob BB");
        assert_eq!(gl.state.current_bet_to_match, 10.0);
    }

    #[test]
    fn test_fold_ends_hand() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Heads-up: Alice (SB) age primeiro
        assert_eq!(gl.state.active_player_index, 0);
        gl.player_action("alice", PlayerMove::Fold).unwrap();

        assert!(gl.state.is_finished);
        assert_eq!(gl.state.players_in_hand_count(), 1);
    }

    #[test]
    fn test_call_then_check_completes_preflop() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Alice (SB) calls (iguala BB)
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // Bob (BB) checks
        gl.player_action("bob", PlayerMove::Check).unwrap();

        // Deve ter avançado para flop
        assert_eq!(gl.state.phase, GamePhase::Flop);
        assert_eq!(gl.state.community_cards.len(), 3);
    }

    #[test]
    fn test_full_hand_to_showdown() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Preflop: Alice (SB) calls, Bob (BB) checks → Flop
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);

        // Flop: Bob (BB) age primeiro em heads-up pós-flop, depois Alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Turn);
        assert_eq!(gl.state.community_cards.len(), 4);

        // Turn: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::River);
        assert_eq!(gl.state.community_cards.len(), 5);

        // River: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Showdown);
        assert!(gl.state.is_finished);
    }

    #[test]
    fn test_resolve_fold_win() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        gl.player_action("alice", PlayerMove::Fold).unwrap();

        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.end_reason, EndReason::AllFolded);
        assert_eq!(resolution.rake, 0.0);

        // Bob ganha o pot (SB + BB = 15)
        let bob_payout = resolution.payouts.get("bob").copied().unwrap_or(0.0);
        assert!(bob_payout > 0.0, "Bob deve receber o pot");
    }

    #[test]
    fn test_resolve_showdown_distributes_pot() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Joga até showdown (all check)
        // Preflop: Alice (SB) calls, Bob (BB) checks
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // Turn: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // River: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();

        assert!(gl.state.is_finished);

        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.end_reason, EndReason::Showdown);

        // Um dos dois deve ter ganho
        let total_payouts: f64 = resolution.payouts.values().sum();
        assert!(total_payouts > 0.0, "Deve haver pagamento");
    }

    #[test]
    fn test_bet_and_raise() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Alice (SB) calls
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // Bob (BB) raises to 30
        gl.player_action("bob", PlayerMove::Raise(30.0)).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 30.0);

        // Alice calls the raise
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // Bob has already acted, round should be complete
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn test_all_in() {
        let mut gl = GameLoop::new(make_config(), "hand-ai".to_string(), "AI Table".to_string(), GameType::Cash);
        gl.add_player("alice".to_string(), 50.0); // stack pequeno
        gl.add_player("bob".to_string(), 1000.0);
        gl.set_dealer(0);
        gl.start_hand().unwrap();

        // Alice (SB=5) goes all-in
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // Bob must call
        gl.player_action("bob", PlayerMove::Call).unwrap();

        // Alice está all-in, Bob call → deve ir direto ao showdown
        assert_eq!(gl.state.phase, GamePhase::Showdown);
        assert!(gl.state.is_finished);
        assert!(gl.state.players[0].is_all_in);
    }

    #[test]
    fn test_not_your_turn_error() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Tentar agir com o jogador errado
        let result = gl.player_action("bob", PlayerMove::Fold);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GameLoopError::NotYourTurn("bob".to_string()));
    }

    #[test]
    fn test_check_when_bet_exists_fails() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Alice (SB) tenta dar check quando há aposta (BB)
        let result = gl.player_action("alice", PlayerMove::Check);
        assert!(result.is_err());
    }

    #[test]
    fn test_hand_history_recorded() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Preflop
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // Turn: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // River: Bob age primeiro
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();

        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);

        let history = gl.get_history().unwrap();
        assert_eq!(history.hand_id, "hand-001");
        assert!(!history.actions.is_empty());
        assert_eq!(history.community_cards.len(), 5);
    }

    #[test]
    fn test_three_players_preflop_order() {
        let mut gl = GameLoop::new(make_config(), "hand-3p".to_string(), "3P Table".to_string(), GameType::Cash);
        gl.add_player("alice".to_string(), 1000.0);
        gl.add_player("bob".to_string(), 1000.0);
        gl.add_player("carol".to_string(), 1000.0);
        gl.set_dealer(0);
        gl.start_hand().unwrap();

        // 3 players: dealer=0 (Alice), SB=1 (Bob), BB=2 (Carol)
        // UTG = primeiro após BB = Alice (dealer, wraps around)
        assert_eq!(gl.state.players[1].current_bet, 5.0, "Bob SB");
        assert_eq!(gl.state.players[2].current_bet, 10.0, "Carol BB");
        // Primeiro a agir = UTG = índice 0 (Alice)
        assert_eq!(gl.state.active_player_index, 0);
    }

    #[test]
    fn test_insufficient_stack_error() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Alice tenta bet maior que stack
        let result = gl.player_action("alice", PlayerMove::Bet(10000.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_raise_too_small_error() {
        let mut gl = make_game_loop_with_2_players();
        gl.start_hand().unwrap();

        // Alice calls
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // Bob tenta raise de apenas 1 (min_raise = BB = 10)
        let result = gl.player_action("bob", PlayerMove::Raise(11.0));
        assert!(result.is_err());
    }
}
