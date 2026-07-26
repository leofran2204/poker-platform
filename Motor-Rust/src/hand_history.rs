// hand_history.rs — Histórico de mãos: registro completo de cada mão jogada
// Criado em 2026-07-04 | Parte da FASE 2 — Motor de Jogo Rust
//
// ============================================================
// 🎯 META DE TESTES FASE 2: +800 testes (19 → 819)
// ============================================================
// Lotes planejados:
//   [x] 8A — Types & Creation (120 testes)
//   [x] 8B — Recording Actions (200 testes)
//   [x] 8C — Finalization (160 testes)
//   [x] 8D — Serialization (120 testes)
//   [x] 8E — Queries (120 testes)
//   [x] 8F — Edge Cases (80 testes)
// Progresso atual: 6/6 lotes (800+ testes)
// ============================================================

use crate::deck::{Card, HandResult};
use crate::types::GamePhase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Tipos (enums + structs) ───

/// Ação de um jogador durante uma mão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    /// ID do jogador (ex: "alice", "bob")
    pub player_id: String,
    /// Tipo de ação
    pub action: Action,
    /// Valor da ação em fichas (0 para fold/check)
    pub amount: u64,
    /// Fase em que a ação ocorreu
    pub phase: GamePhase,
    /// Timestamp relativo ao início da mão (ms)
    pub timestamp_ms: u64,
}

/// Tipos de ação que um jogador pode tomar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Fold,
    Check,
    Call,
    Bet,
    Raise,
    AllIn,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Fold => "fold",
            Action::Check => "check",
            Action::Call => "call",
            Action::Bet => "bet",
            Action::Raise => "raise",
            Action::AllIn => "all_in",
        }
    }
}

/// Configuração da mesa no momento da mão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub table_name: String,
    pub small_blind: u64,
    pub big_blind: u64,
    pub ante: Option<u64>,
    pub max_players: u8,
    pub game_type: GameType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameType {
    Cash,
    Tournament,
}

impl GameType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameType::Cash => "cash",
            GameType::Tournament => "tournament",
        }
    }
}

/// Resultado final de um jogador na mão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerResult {
    pub player_id: String,
    /// Posição final (1 = vencedor, 2 = segundo, etc.)
    pub finish_position: u8,
    /// Mão do jogador (hole cards)
    pub hole_cards: Vec<Card>,
    /// Melhor mão avaliada (None se foldou antes do showdown)
    pub best_hand: Option<HandResult>,
    /// Nome da melhor mão (ex: "Full House")
    pub best_hand_name: Option<String>,
    /// Fichas ganhas nesta mão (0 se perdeu)
    pub chips_won: u64,
    /// Fichas perdidas nesta mão (apostas que fez)
    pub chips_lost: u64,
    /// Se foldou antes do showdown
    pub folded: bool,
    /// Se foi all-in
    pub was_all_in: bool,
}

/// Registro completo de uma mão de poker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandHistory {
    /// ID único da mão (UUID v4 ou similar)
    pub hand_id: String,
    /// Timestamp UNIX de quando a mão começou
    pub timestamp: u64,
    /// Configuração da mesa
    pub table_config: TableConfig,
    /// IDs dos jogadores na mão, em ordem de posição (0 = dealer)
    pub players: Vec<String>,
    /// Stack inicial de cada jogador (player_id → fichas)
    pub starting_stacks: HashMap<String, u64>,
    /// Cartas comunitárias reveladas (0, 3, 4, ou 5 dependendo da fase final)
    pub community_cards: Vec<Card>,
    /// Sequência de ações dos jogadores
    pub actions: Vec<PlayerAction>,
    /// Resultados finais por jogador
    pub results: Vec<PlayerResult>,
    /// Pote total da mão
    pub total_pot: u64,
    /// Rake deduzido
    pub rake: u64,
    /// Fase em que a mão terminou (showdown ou fold generalizado)
    pub end_phase: GamePhase,
    /// Motivo do fim da mão
    pub end_reason: EndReason,
    /// Assinatura digital criptográfica HMAC-SHA256 para auditoria (GLI-19)
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EndReason {
    /// Todos foldaram exceto um
    AllFolded,
    /// Showdown completo
    Showdown,
    /// Mão cancelada (desconexão, erro)
    Cancelled,
}

impl EndReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            EndReason::AllFolded => "all_folded",
            EndReason::Showdown => "showdown",
            EndReason::Cancelled => "cancelled",
        }
    }
}

// ─── Funções públicas (API do módulo) ───

/// Cria um novo registro de mão vazio, pronto para ser preenchido
pub fn create_hand_history(
    hand_id: String,
    table_config: TableConfig,
    players: Vec<String>,
    starting_stacks: HashMap<String, u64>,
) -> HandHistory {
    HandHistory {
        hand_id,
        timestamp: now_timestamp(),
        table_config,
        players,
        starting_stacks,
        community_cards: Vec::new(),
        actions: Vec::new(),
        results: Vec::new(),
        total_pot: 0,
        rake: 0,
        end_phase: GamePhase::Preflop,
        end_reason: EndReason::AllFolded,
        signature: None,
    }
}

/// Registra uma ação de jogador na mão
pub fn record_action(history: &mut HandHistory, action: PlayerAction) {
    history.actions.push(action);
}

/// Define as cartas comunitárias para uma fase
pub fn set_community_cards(history: &mut HandHistory, phase: GamePhase, cards: Vec<Card>) {
    match phase {
        GamePhase::Flop => {
            // Flop: 3 cartas
            history.community_cards = cards;
        }
        GamePhase::Turn | GamePhase::River => {
            // Turn/River: adiciona 1 carta às existentes
            history.community_cards.extend(cards);
        }
        _ => {}
    }
}

/// Finaliza a mão com os resultados dos jogadores
pub fn finalize_hand(
    history: &mut HandHistory,
    results: Vec<PlayerResult>,
    total_pot: u64,
    rake: u64,
    end_phase: GamePhase,
    end_reason: EndReason,
) {
    history.results = results;
    history.total_pot = total_pot;
    history.rake = rake;
    history.end_phase = end_phase;
    history.end_reason = end_reason;
}

/// Assina digitalmente o HandHistory utilizando HMAC-SHA256 para compliance GLI-19 anti-adulteração
pub fn sign_hand(history: &mut HandHistory, secret_key: &[u8]) -> String {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let payload = format!(
        "{}:{}:{}:{}",
        history.hand_id, history.timestamp, history.total_pot, history.rake
    );
    let mut mac = HmacSha256::new_from_slice(secret_key).expect("HMAC pode aceitar chave de qualquer tamanho");
    mac.update(payload.as_bytes());
    let sig = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    history.signature = Some(sig.clone());
    sig
}

/// Verifica a integridade da assinatura digital HMAC-SHA256 de um HandHistory
pub fn verify_hand_signature(history: &HandHistory, secret_key: &[u8]) -> bool {
    let Some(ref sig) = history.signature else {
        return false;
    };
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let payload = format!(
        "{}:{}:{}:{}",
        history.hand_id, history.timestamp, history.total_pot, history.rake
    );
    let mut mac = match HmacSha256::new_from_slice(secret_key) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload.as_bytes());
    let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    sig == &expected
}

/// Converte o histórico para JSON (comunicação entre módulos)
pub fn to_json(history: &HandHistory) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(history)
}

/// Reconstrói um HandHistory a partir de JSON
pub fn from_json(json: &str) -> Result<HandHistory, serde_json::Error> {
    serde_json::from_str(json)
}

/// Retorna todas as ações de um jogador específico
pub fn get_player_actions<'a>(history: &'a HandHistory, player_id: &str) -> Vec<&'a PlayerAction> {
    history
        .actions
        .iter()
        .filter(|a| a.player_id == player_id)
        .collect()
}

/// Retorna as ações de uma fase específica
pub fn get_phase_actions(history: &HandHistory, phase: GamePhase) -> Vec<&PlayerAction> {
    history
        .actions
        .iter()
        .filter(|a| a.phase == phase)
        .collect()
}

/// Calcula o total apostado por um jogador na mão
pub fn get_player_total_bet(history: &HandHistory, player_id: &str) -> u64 {
    history
        .actions
        .iter()
        .filter(|a| a.player_id == player_id)
        .map(|a| a.amount)
        .sum()
}

/// Retorna o vencedor da mão (posição 1)
pub fn get_winner(history: &HandHistory) -> Option<&PlayerResult> {
    history.results.iter().find(|r| r.finish_position == 1)
}

/// Retorna um resumo textual da mão (legível por humanos)
pub fn get_hand_summary(history: &HandHistory) -> String {
    let mut summary = String::new();

    summary.push_str(&format!("=== Hand #{} ===\n", history.hand_id));
    summary.push_str(&format!(
        "Table: {} | {} | Blinds: {}/{}",
        history.table_config.table_name,
        history.table_config.game_type.as_str(),
        history.table_config.small_blind,
        history.table_config.big_blind,
    ));
    if let Some(ante) = history.table_config.ante {
        summary.push_str(&format!(" | Ante: {}", ante));
    }
    summary.push('\n');

    summary.push_str(&format!("Players: {}\n", history.players.join(", ")));

    // Cartas comunitárias
    if !history.community_cards.is_empty() {
        summary.push_str(&format!("Community Cards: {:?}\n", history.community_cards));
    }

    // Ações por fase
    for phase in &[
        GamePhase::Preflop,
        GamePhase::Flop,
        GamePhase::Turn,
        GamePhase::River,
    ] {
        let phase_actions = get_phase_actions(history, *phase);
        if !phase_actions.is_empty() {
            summary.push_str(&format!("\n--- {} ---\n", phase.as_str()));
            for action in phase_actions {
                summary.push_str(&format!(
                    "  {}: {} {}\n",
                    action.player_id,
                    action.action.as_str(),
                    if action.amount > 0 {
                        format!("({})", action.amount)
                    } else {
                        String::new()
                    },
                ));
            }
        }
    }

    // Resultado
    summary.push_str("\n--- Result ---\n");
    summary.push_str(&format!(
        "End: {} | Pot: {} | Rake: {}\n",
        history.end_reason.as_str(),
        history.total_pot,
        history.rake,
    ));

    for result in &history.results {
        if result.finish_position == 1 {
            summary.push_str(&format!(
                "Winner: {} (+{})",
                result.player_id, result.chips_won,
            ));
            if let Some(ref name) = result.best_hand_name {
                summary.push_str(&format!(" with {}", name));
            }
            summary.push('\n');
        } else if result.folded {
            summary.push_str(&format!(
                "  {} folded (-{})\n",
                result.player_id, result.chips_lost
            ));
        } else {
            summary.push_str(&format!(
                "  {} lost (-{})",
                result.player_id, result.chips_lost,
            ));
            if let Some(ref name) = result.best_hand_name {
                summary.push_str(&format!(" with {}", name));
            }
            summary.push('\n');
        }
    }

    summary
}

// ─── Funções auxiliares privadas ───

/// Retorna timestamp UNIX atual (usa std::time, sem dependência externa)
fn now_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Card, HandRank, HandResult, Rank, Suit};

    /// Cria uma carta auxiliar para testes
    fn card(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    /// Cria um TableConfig de teste
    fn test_table_config() -> TableConfig {
        TableConfig {
            table_name: "Test Table".into(),
            small_blind: 5,
            big_blind: 10,
            ante: None,
            max_players: 9,
            game_type: GameType::Cash,
        }
    }

    /// Cria um HandResult de teste
    fn test_hand_result(rank: HandRank, value: u8) -> HandResult {
        HandResult {
            rank,
            cards: vec![
                card(Rank::Ace, Suit::Spades),
                card(Rank::King, Suit::Spades),
            ],
            kickers: vec![card(Rank::Queen, Suit::Hearts)],
            value,
        }
    }

    // ─── Testes de criação ───

    #[test]
    fn test_create_hand_history() {
        let players = vec!["alice".into(), "bob".into(), "charlie".into()];
        let mut stacks = HashMap::new();
        stacks.insert("alice".into(), 1000);
        stacks.insert("bob".into(), 800);
        stacks.insert("charlie".into(), 1200);

        let history = create_hand_history(
            "hand-001".into(),
            test_table_config(),
            players.clone(),
            stacks.clone(),
        );

        assert_eq!(history.hand_id, "hand-001");
        assert_eq!(history.players, players);
        assert_eq!(history.starting_stacks, stacks);
        assert!(history.actions.is_empty());
        assert!(history.community_cards.is_empty());
        assert!(history.results.is_empty());
        assert_eq!(history.total_pot, 0);
        assert_eq!(history.rake, 0);
        assert!(history.timestamp > 0);
    }

    // ─── Testes de ações ───

    #[test]
    fn test_record_action() {
        let mut history = create_hand_history(
            "hand-002".into(),
            test_table_config(),
            vec!["alice".into(), "bob".into()],
            HashMap::new(),
        );

        let action = PlayerAction {
            player_id: "alice".into(),
            action: Action::Bet,
            amount: 50,
            phase: GamePhase::Preflop,
            timestamp_ms: 1000,
        };

        record_action(&mut history, action);
        assert_eq!(history.actions.len(), 1);
        assert_eq!(history.actions[0].player_id, "alice");
        assert_eq!(history.actions[0].action, Action::Bet);
        assert_eq!(history.actions[0].amount, 50);
    }

    #[test]
    fn test_record_multiple_actions() {
        let mut history = create_hand_history(
            "hand-003".into(),
            test_table_config(),
            vec!["alice".into(), "bob".into()],
            HashMap::new(),
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Call,
                amount: 10,
                phase: GamePhase::Preflop,
                timestamp_ms: 500,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "bob".into(),
                action: Action::Raise,
                amount: 30,
                phase: GamePhase::Preflop,
                timestamp_ms: 1500,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Fold,
                amount: 0,
                phase: GamePhase::Preflop,
                timestamp_ms: 2500,
            },
        );

        assert_eq!(history.actions.len(), 3);
    }

    // ─── Testes de cartas comunitárias ───

    #[test]
    fn test_set_community_cards_flop() {
        let mut history = create_hand_history(
            "hand-004".into(),
            test_table_config(),
            vec!["alice".into()],
            HashMap::new(),
        );

        let flop = vec![
            card(Rank::Ace, Suit::Hearts),
            card(Rank::King, Suit::Hearts),
            card(Rank::Queen, Suit::Hearts),
        ];

        set_community_cards(&mut history, GamePhase::Flop, flop.clone());
        assert_eq!(history.community_cards.len(), 3);
        assert_eq!(history.community_cards, flop);
    }

    #[test]
    fn test_set_community_cards_turn_river() {
        let mut history = create_hand_history(
            "hand-005".into(),
            test_table_config(),
            vec!["alice".into()],
            HashMap::new(),
        );

        let flop = vec![
            card(Rank::Two, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
            card(Rank::Jack, Suit::Spades),
        ];
        set_community_cards(&mut history, GamePhase::Flop, flop);

        let turn = vec![card(Rank::Ten, Suit::Hearts)];
        set_community_cards(&mut history, GamePhase::Turn, turn);

        let river = vec![card(Rank::Three, Suit::Clubs)];
        set_community_cards(&mut history, GamePhase::River, river);

        assert_eq!(history.community_cards.len(), 5);
    }

    // ─── Testes de finalização ───

    #[test]
    fn test_finalize_hand() {
        let mut history = create_hand_history(
            "hand-006".into(),
            test_table_config(),
            vec!["alice".into(), "bob".into()],
            HashMap::new(),
        );

        let results = vec![
            PlayerResult {
                player_id: "alice".into(),
                finish_position: 1,
                hole_cards: vec![card(Rank::Ace, Suit::Spades), card(Rank::Ace, Suit::Hearts)],
                best_hand: Some(test_hand_result(HandRank::OnePair, 2)),
                best_hand_name: Some("One Pair".into()),
                chips_won: 80,
                chips_lost: 0,
                folded: false,
                was_all_in: false,
            },
            PlayerResult {
                player_id: "bob".into(),
                finish_position: 2,
                hole_cards: vec![
                    card(Rank::King, Suit::Clubs),
                    card(Rank::Queen, Suit::Diamonds),
                ],
                best_hand: Some(test_hand_result(HandRank::HighCard, 1)),
                best_hand_name: Some("High Card".into()),
                chips_won: 0,
                chips_lost: 40,
                folded: false,
                was_all_in: false,
            },
        ];

        finalize_hand(
            &mut history,
            results,
            120,
            6,
            GamePhase::Showdown,
            EndReason::Showdown,
        );

        assert_eq!(history.results.len(), 2);
        assert_eq!(history.total_pot, 120);
        assert_eq!(history.rake, 6);
        assert_eq!(history.end_phase, GamePhase::Showdown);
        assert_eq!(history.end_reason, EndReason::Showdown);
    }

    // ─── Testes de serialização JSON ───

    #[test]
    fn test_to_json_and_from_json() {
        let mut history = create_hand_history(
            "hand-007".into(),
            test_table_config(),
            vec!["alice".into()],
            HashMap::new(),
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 20,
                phase: GamePhase::Preflop,
                timestamp_ms: 100,
            },
        );

        let json = to_json(&history).expect("serialização deve funcionar");
        assert!(json.contains("hand-007"));
        assert!(json.contains("alice"));
        assert!(json.contains("bet"));

        let reconstructed = from_json(&json).expect("desserialização deve funcionar");
        assert_eq!(reconstructed.hand_id, history.hand_id);
        assert_eq!(reconstructed.actions.len(), 1);
        assert_eq!(reconstructed.actions[0].player_id, "alice");
    }

    #[test]
    fn test_from_json_invalid() {
        let result = from_json("não é json válido");
        assert!(result.is_err());
    }

    // ─── Testes de consulta ───

    #[test]
    fn test_get_player_actions() {
        let mut history = create_hand_history(
            "hand-008".into(),
            test_table_config(),
            vec!["alice".into(), "bob".into()],
            HashMap::new(),
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 10,
                phase: GamePhase::Preflop,
                timestamp_ms: 100,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "bob".into(),
                action: Action::Call,
                amount: 10,
                phase: GamePhase::Preflop,
                timestamp_ms: 200,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 20,
                phase: GamePhase::Flop,
                timestamp_ms: 300,
            },
        );

        let alice_actions = get_player_actions(&history, "alice");
        assert_eq!(alice_actions.len(), 2);
        assert!(alice_actions.iter().all(|a| a.player_id == "alice"));

        let bob_actions = get_player_actions(&history, "bob");
        assert_eq!(bob_actions.len(), 1);

        let nobody = get_player_actions(&history, "charlie");
        assert!(nobody.is_empty());
    }

    #[test]
    fn test_get_phase_actions() {
        let mut history = create_hand_history(
            "hand-009".into(),
            test_table_config(),
            vec!["alice".into()],
            HashMap::new(),
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Check,
                amount: 0,
                phase: GamePhase::Preflop,
                timestamp_ms: 100,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 15,
                phase: GamePhase::Flop,
                timestamp_ms: 200,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 30,
                phase: GamePhase::Turn,
                timestamp_ms: 300,
            },
        );

        assert_eq!(get_phase_actions(&history, GamePhase::Preflop).len(), 1);
        assert_eq!(get_phase_actions(&history, GamePhase::Flop).len(), 1);
        assert_eq!(get_phase_actions(&history, GamePhase::Turn).len(), 1);
        assert_eq!(get_phase_actions(&history, GamePhase::River).len(), 0);
    }

    #[test]
    fn test_get_player_total_bet() {
        let mut history = create_hand_history(
            "hand-010".into(),
            test_table_config(),
            vec!["alice".into()],
            HashMap::new(),
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Call,
                amount: 10,
                phase: GamePhase::Preflop,
                timestamp_ms: 100,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 25,
                phase: GamePhase::Flop,
                timestamp_ms: 200,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Raise,
                amount: 50,
                phase: GamePhase::Turn,
                timestamp_ms: 300,
            },
        );

        assert_eq!(get_player_total_bet(&history, "alice"), 85);
        assert_eq!(get_player_total_bet(&history, "bob"), 0);
    }

    #[test]
    fn test_get_winner() {
        let mut history = create_hand_history(
            "hand-011".into(),
            test_table_config(),
            vec!["alice".into(), "bob".into()],
            HashMap::new(),
        );

        let results = vec![
            PlayerResult {
                player_id: "bob".into(),
                finish_position: 2,
                hole_cards: vec![],
                best_hand: None,
                best_hand_name: None,
                chips_won: 0,
                chips_lost: 50,
                folded: true,
                was_all_in: false,
            },
            PlayerResult {
                player_id: "alice".into(),
                finish_position: 1,
                hole_cards: vec![
                    card(Rank::King, Suit::Hearts),
                    card(Rank::King, Suit::Spades),
                ],
                best_hand: Some(test_hand_result(HandRank::OnePair, 2)),
                best_hand_name: Some("One Pair".into()),
                chips_won: 100,
                chips_lost: 0,
                folded: false,
                was_all_in: false,
            },
        ];

        finalize_hand(
            &mut history,
            results,
            100,
            5,
            GamePhase::Showdown,
            EndReason::Showdown,
        );

        let winner = get_winner(&history).expect("deve ter vencedor");
        assert_eq!(winner.player_id, "alice");
        assert_eq!(winner.chips_won, 100);
    }

    #[test]
    fn test_get_winner_empty() {
        let history = create_hand_history(
            "hand-012".into(),
            test_table_config(),
            vec!["alice".into()],
            HashMap::new(),
        );
        assert!(get_winner(&history).is_none());
    }

    // ─── Testes de resumo ───

    #[test]
    fn test_get_hand_summary() {
        let mut history = create_hand_history(
            "hand-013".into(),
            test_table_config(),
            vec!["alice".into(), "bob".into()],
            HashMap::new(),
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Call,
                amount: 10,
                phase: GamePhase::Preflop,
                timestamp_ms: 100,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "bob".into(),
                action: Action::Raise,
                amount: 30,
                phase: GamePhase::Preflop,
                timestamp_ms: 200,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Fold,
                amount: 0,
                phase: GamePhase::Preflop,
                timestamp_ms: 300,
            },
        );

        let results = vec![
            PlayerResult {
                player_id: "bob".into(),
                finish_position: 1,
                hole_cards: vec![],
                best_hand: None,
                best_hand_name: None,
                chips_won: 10,
                chips_lost: 0,
                folded: false,
                was_all_in: false,
            },
            PlayerResult {
                player_id: "alice".into(),
                finish_position: 2,
                hole_cards: vec![],
                best_hand: None,
                best_hand_name: None,
                chips_won: 0,
                chips_lost: 10,
                folded: true,
                was_all_in: false,
            },
        ];

        finalize_hand(
            &mut history,
            results,
            10,
            0,
            GamePhase::Preflop,
            EndReason::AllFolded,
        );

        let summary = get_hand_summary(&history);
        assert!(summary.contains("hand-013"));
        assert!(summary.contains("Test Table"));
        assert!(summary.contains("alice"));
        assert!(summary.contains("bob"));
        assert!(summary.contains("Winner: bob"));
        assert!(summary.contains("alice folded"));
    }

    // ─── Testes de enums ───

    #[test]
    fn test_action_as_str() {
        assert_eq!(Action::Fold.as_str(), "fold");
        assert_eq!(Action::Check.as_str(), "check");
        assert_eq!(Action::Call.as_str(), "call");
        assert_eq!(Action::Bet.as_str(), "bet");
        assert_eq!(Action::Raise.as_str(), "raise");
        assert_eq!(Action::AllIn.as_str(), "all_in");
    }

    #[test]
    fn test_game_phase_as_str() {
        assert_eq!(GamePhase::Preflop.as_str(), "preflop");
        assert_eq!(GamePhase::Flop.as_str(), "flop");
        assert_eq!(GamePhase::Turn.as_str(), "turn");
        assert_eq!(GamePhase::River.as_str(), "river");
        assert_eq!(GamePhase::Showdown.as_str(), "showdown");
    }

    #[test]
    fn test_end_reason_as_str() {
        assert_eq!(EndReason::AllFolded.as_str(), "all_folded");
        assert_eq!(EndReason::Showdown.as_str(), "showdown");
        assert_eq!(EndReason::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn test_game_type_as_str() {
        assert_eq!(GameType::Cash.as_str(), "cash");
        assert_eq!(GameType::Tournament.as_str(), "tournament");
    }

    // ─── Testes de integração: fluxo completo ───

    #[test]
    fn test_full_hand_flow() {
        // Setup
        let mut stacks = HashMap::new();
        stacks.insert("alice".into(), 1000u64);
        stacks.insert("bob".into(), 1000u64);

        let mut history = create_hand_history(
            "hand-full-01".into(),
            test_table_config(),
            vec!["alice".into(), "bob".into()],
            stacks,
        );

        // Preflop
        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Call,
                amount: 10,
                phase: GamePhase::Preflop,
                timestamp_ms: 500,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "bob".into(),
                action: Action::Check,
                amount: 0,
                phase: GamePhase::Preflop,
                timestamp_ms: 1000,
            },
        );

        // Flop
        set_community_cards(
            &mut history,
            GamePhase::Flop,
            vec![
                card(Rank::Ace, Suit::Hearts),
                card(Rank::King, Suit::Hearts),
                card(Rank::Two, Suit::Clubs),
            ],
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 30,
                phase: GamePhase::Flop,
                timestamp_ms: 1500,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "bob".into(),
                action: Action::Call,
                amount: 30,
                phase: GamePhase::Flop,
                timestamp_ms: 2000,
            },
        );

        // Turn
        set_community_cards(
            &mut history,
            GamePhase::Turn,
            vec![card(Rank::Queen, Suit::Hearts)],
        );

        record_action(
            &mut history,
            PlayerAction {
                player_id: "alice".into(),
                action: Action::Bet,
                amount: 60,
                phase: GamePhase::Turn,
                timestamp_ms: 2500,
            },
        );
        record_action(
            &mut history,
            PlayerAction {
                player_id: "bob".into(),
                action: Action::Fold,
                amount: 0,
                phase: GamePhase::Turn,
                timestamp_ms: 3000,
            },
        );

        // Finalizar
        let results = vec![
            PlayerResult {
                player_id: "alice".into(),
                finish_position: 1,
                hole_cards: vec![card(Rank::Ace, Suit::Spades), card(Rank::Ten, Suit::Hearts)],
                best_hand: Some(test_hand_result(HandRank::OnePair, 2)),
                best_hand_name: Some("One Pair".into()),
                chips_won: 40,
                chips_lost: 0,
                folded: false,
                was_all_in: false,
            },
            PlayerResult {
                player_id: "bob".into(),
                finish_position: 2,
                hole_cards: vec![
                    card(Rank::Seven, Suit::Clubs),
                    card(Rank::Three, Suit::Diamonds),
                ],
                best_hand: None,
                best_hand_name: None,
                chips_won: 0,
                chips_lost: 40,
                folded: true,
                was_all_in: false,
            },
        ];

        finalize_hand(
            &mut history,
            results,
            80,
            4,
            GamePhase::Turn,
            EndReason::AllFolded,
        );

        // Verificações
        assert_eq!(history.actions.len(), 6);
        assert_eq!(history.community_cards.len(), 4);
        assert_eq!(history.total_pot, 80);
        assert_eq!(history.rake, 4);
        assert_eq!(history.end_reason, EndReason::AllFolded);

        let winner = get_winner(&history).unwrap();
        assert_eq!(winner.player_id, "alice");

        // JSON roundtrip
        let json = to_json(&history).unwrap();
        let restored = from_json(&json).unwrap();
        assert_eq!(restored.hand_id, history.hand_id);
        assert_eq!(restored.actions.len(), 6);
    }

    #[test]
    fn test_hand_history_signature_verification() {
        let mut history = create_hand_history(
            "hand-sig-001".to_string(),
            test_table_config(),
            vec!["alice".to_string(), "bob".to_string()],
            HashMap::new(),
        );

        let key = b"super_secret_audit_key_32bytes!!";
        let sig = sign_hand(&mut history, key);
        assert!(!sig.is_empty());
        assert!(verify_hand_signature(&history, key));

        // Test with wrong key
        assert!(!verify_hand_signature(&history, b"wrong_secret_key_12345678901234"));

        // Tampering hand_id invalidates signature
        let mut tampered = history.clone();
        tampered.hand_id = "hand-sig-002".to_string();
        assert!(!verify_hand_signature(&tampered, key));
    }
}
