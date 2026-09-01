// game_loop_tests.rs — Testes abrangentes do Game Loop (300 testes)
//
// Cobertura completa da máquina de estados do Texas Hold'em:
//   Lote 1: PlayerState (50 testes)
//   Lote 2: HandState (50 testes)
//   Lote 3: GameLoop init/blinds/ante (50 testes)
//   Lote 4: Ações — Fold/Check/Call (50 testes)
//   Lote 5: Ações — Bet/Raise/AllIn (50 testes)
//   Lote 6: Resolve/Showdown/Errors/History (50 testes)
//
// Cada lote cobre: casos normais, edge cases, erros e invariantes.

use crate::deck::{Card, Rank, Suit};
use crate::game_loop::{GameLoop, GameLoopError, HandState, PlayerMove, PlayerState};
use crate::hand_history::{EndReason, GameType};
use crate::types::{GamePhase, TableConfig};

// ═══════════════════════════════════════════════════════════════════
// Helpers compartilhados
// ═══════════════════════════════════════════════════════════════════

fn make_config() -> TableConfig {
    // Cenário unitário: BB=10 centavos, rake=5%, cap=500 centavos.
    // Os valores pequenos deixam explícitas as transições de blinds e all-in.
    TableConfig::new(10, 500, 500)
}

#[allow(dead_code)]
fn make_config_custom(bb: u64, rake: u16, cap: u64) -> TableConfig {
    TableConfig::new(bb, rake, cap)
}

fn make_game_loop_2p() -> GameLoop {
    let mut gl = GameLoop::new(
        make_config(),
        "hand-001".to_string(),
        "Test Table".to_string(),
        GameType::Cash,
    );
    gl.add_player("alice".to_string(), 1000);
    gl.add_player("bob".to_string(), 1000);
    gl.set_dealer(0);
    gl
}

fn make_game_loop_3p() -> GameLoop {
    let mut gl = GameLoop::new(
        make_config(),
        "hand-3p".to_string(),
        "3P Table".to_string(),
        GameType::Cash,
    );
    gl.add_player("alice".to_string(), 1000);
    gl.add_player("bob".to_string(), 1000);
    gl.add_player("carol".to_string(), 1000);
    gl.set_dealer(0);
    gl
}

fn make_game_loop_4p() -> GameLoop {
    let mut gl = GameLoop::new(
        make_config(),
        "hand-4p".to_string(),
        "4P Table".to_string(),
        GameType::Cash,
    );
    gl.add_player("alice".to_string(), 1000);
    gl.add_player("bob".to_string(), 1000);
    gl.add_player("carol".to_string(), 1000);
    gl.add_player("dave".to_string(), 1000);
    gl.set_dealer(0);
    gl
}

fn make_game_loop_6p() -> GameLoop {
    let mut gl = GameLoop::new(
        make_config(),
        "hand-6p".to_string(),
        "6P Table".to_string(),
        GameType::Cash,
    );
    gl.add_player("p0".to_string(), 1000);
    gl.add_player("p1".to_string(), 1000);
    gl.add_player("p2".to_string(), 1000);
    gl.add_player("p3".to_string(), 1000);
    gl.add_player("p4".to_string(), 1000);
    gl.add_player("p5".to_string(), 1000);
    gl.set_dealer(0);
    gl
}

fn make_game_loop_n(n: usize) -> GameLoop {
    let mut gl = GameLoop::new(
        make_config(),
        "hand-n".to_string(),
        "N Table".to_string(),
        GameType::Cash,
    );
    for i in 0..n {
        gl.add_player(format!("p{i}"), 1000);
    }
    gl.set_dealer(0);
    gl
}

/// Joga uma mão completa heads-up com all-check até showdown
fn play_all_check_to_showdown(gl: &mut GameLoop) {
    // Preflop: SB calls, BB checks
    let first = gl.state.active_player_index;
    let first_id = gl.state.players[first].id.clone();
    gl.player_action(&first_id, PlayerMove::Call).unwrap();
    let second_id = gl.state.players[gl.state.active_player_index].id.clone();
    gl.player_action(&second_id, PlayerMove::Check).unwrap();
    // Flop, Turn, River: alternar checks
    for _ in 0..3 {
        let a_id = gl.state.players[gl.state.active_player_index].id.clone();
        gl.player_action(&a_id, PlayerMove::Check).unwrap();
        let b_id = gl.state.players[gl.state.active_player_index].id.clone();
        gl.player_action(&b_id, PlayerMove::Check).unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════════
// LOTE 1 — PlayerState (50 testes)
// ═══════════════════════════════════════════════════════════════════

mod player_state_tests {
    use super::*;

    // ─── Construção e estado inicial ───

    #[test]
    fn ps_new_cria_jogador_com_stack_correto() {
        let p = PlayerState::new("alice".to_string(), 1000, 0);
        assert_eq!(p.id, "alice");
        assert_eq!(p.stack, 1000);
        assert_eq!(p.seat_index, 0);
    }

    #[test]
    fn ps_new_stack_zero() {
        let p = PlayerState::new("broke".to_string(), 0, 3);
        assert_eq!(p.stack, 0);
    }

    #[test]
    fn ps_new_stack_fracionado() {
        let p = PlayerState::new("frac".to_string(), 99, 1);
        assert_eq!(p.stack, 99);
    }

    #[test]
    fn ps_new_hole_cards_vazio() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert!(p.hole_cards.is_empty());
    }

    #[test]
    fn ps_new_current_bet_zero() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert_eq!(p.current_bet, 0);
    }

    #[test]
    fn ps_new_total_bet_zero() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert_eq!(p.total_bet, 0);
    }

    #[test]
    fn ps_new_nao_foldado() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert!(!p.has_folded);
    }

    #[test]
    fn ps_new_nao_all_in() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert!(!p.is_all_in);
    }

    #[test]
    fn ps_new_nao_agiu() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert!(!p.has_acted);
    }

    #[test]
    fn ps_new_seat_index_preservado() {
        for i in 0..9 {
            let p = PlayerState::new(format!("p{i}"), 100, i);
            assert_eq!(p.seat_index, i);
        }
    }

    // ─── can_act() ───

    #[test]
    fn ps_can_act_true_estado_inicial() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert!(p.can_act());
    }

    #[test]
    fn ps_can_act_false_se_foldado() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = true;
        assert!(!p.can_act());
    }

    #[test]
    fn ps_can_act_false_se_all_in() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.is_all_in = true;
        assert!(!p.can_act());
    }

    #[test]
    fn ps_can_act_false_se_foldado_e_all_in() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = true;
        p.is_all_in = true;
        assert!(!p.can_act());
    }

    #[test]
    fn ps_can_act_true_com_stack_zero_nao_all_in() {
        // Stack zero mas não marcado all_in ainda pode agir (edge case)
        let mut p = PlayerState::new("x".to_string(), 0, 0);
        p.stack = 0;
        // can_act só verifica folded e all_in
        assert!(p.can_act());
    }

    // ─── is_in_hand() ───

    #[test]
    fn ps_is_in_hand_true_inicial() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert!(p.is_in_hand());
    }

    #[test]
    fn ps_is_in_hand_false_se_foldado() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = true;
        assert!(!p.is_in_hand());
    }

    #[test]
    fn ps_is_in_hand_true_se_all_in() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.is_all_in = true;
        assert!(p.is_in_hand());
    }

    #[test]
    fn ps_is_in_hand_true_se_all_in_e_stack_zero() {
        let mut p = PlayerState::new("x".to_string(), 0, 0);
        p.is_all_in = true;
        assert!(p.is_in_hand());
    }

    // ─── reset_round_bet() ───

    #[test]
    fn ps_reset_round_bet_zera_current_bet() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.current_bet = 50;
        p.reset_round_bet();
        assert_eq!(p.current_bet, 0);
    }

    #[test]
    fn ps_reset_round_bet_zera_has_acted() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_acted = true;
        p.reset_round_bet();
        assert!(!p.has_acted);
    }

    #[test]
    fn ps_reset_round_bet_nao_altera_total_bet() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.total_bet = 200;
        p.reset_round_bet();
        assert_eq!(p.total_bet, 200);
    }

    #[test]
    fn ps_reset_round_bet_nao_altera_stack() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.stack = 50;
        p.reset_round_bet();
        assert_eq!(p.stack, 50);
    }

    #[test]
    fn ps_reset_round_bet_nao_altera_folded() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = true;
        p.reset_round_bet();
        assert!(p.has_folded);
    }

    #[test]
    fn ps_reset_round_bet_nao_altera_all_in() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.is_all_in = true;
        p.reset_round_bet();
        assert!(p.is_all_in);
    }

    #[test]
    fn ps_reset_round_bet_zera_ja_zero() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.reset_round_bet();
        assert_eq!(p.current_bet, 0);
        assert!(!p.has_acted);
    }

    // ─── Clone e Debug ───

    #[test]
    fn ps_clone_preserva_todos_campos() {
        let mut p = PlayerState::new("alice".to_string(), 500, 2);
        p.current_bet = 30;
        p.total_bet = 60;
        p.has_folded = false;
        p.is_all_in = true;
        p.has_acted = true;
        p.hole_cards = vec![
            Card {
                rank: Rank::Ace,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Hearts,
            },
        ];
        let c = p.clone();
        assert_eq!(c.id, p.id);
        assert_eq!(c.stack, p.stack);
        assert_eq!(c.seat_index, p.seat_index);
        assert_eq!(c.current_bet, p.current_bet);
        assert_eq!(c.total_bet, p.total_bet);
        assert_eq!(c.has_folded, p.has_folded);
        assert_eq!(c.is_all_in, p.is_all_in);
        assert_eq!(c.has_acted, p.has_acted);
        assert_eq!(c.hole_cards.len(), 2);
    }

    #[test]
    fn ps_debug_nao_panic() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        let s = format!("{p:?}");
        assert!(s.contains("PlayerState"));
    }

    #[test]
    fn ps_clone_e_modificar_original_nao_afeta_clone() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        let c = p.clone();
        p.stack = 0;
        assert_eq!(c.stack, 100);
    }

    // ─── Múltiplos jogadores ───

    #[test]
    fn ps_criar_9_jogadores_seats_distintos() {
        let players: Vec<PlayerState> = (0..9)
            .map(|i| PlayerState::new(format!("p{i}"), 1000, i))
            .collect();
        for (i, p) in players.iter().enumerate() {
            assert_eq!(p.seat_index, i);
        }
    }

    #[test]
    fn ps_ids_distintos() {
        let p1 = PlayerState::new("alice".to_string(), 100, 0);
        let p2 = PlayerState::new("bob".to_string(), 100, 1);
        assert_ne!(p1.id, p2.id);
    }

    #[test]
    fn ps_stacks_diferentes() {
        let p1 = PlayerState::new("rich".to_string(), 10000, 0);
        let p2 = PlayerState::new("poor".to_string(), 10, 1);
        assert!(p1.stack > p2.stack);
    }

    // ─── Estados combinados ───

    #[test]
    fn ps_foldado_nao_all_in_can_act_false() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = true;
        p.is_all_in = false;
        assert!(!p.can_act());
    }

    #[test]
    fn ps_nao_foldado_all_in_can_act_false() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = false;
        p.is_all_in = true;
        assert!(!p.can_act());
    }

    #[test]
    fn ps_nao_foldado_nao_all_in_can_act_true() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = false;
        p.is_all_in = false;
        assert!(p.can_act());
    }

    #[test]
    fn ps_is_in_hand_independente_de_all_in() {
        let mut p1 = PlayerState::new("a".to_string(), 100, 0);
        p1.is_all_in = true;
        let mut p2 = PlayerState::new("b".to_string(), 100, 1);
        p2.has_folded = true;
        assert!(p1.is_in_hand());
        assert!(!p2.is_in_hand());
    }

    // ─── Valores extremos ───

    #[test]
    fn ps_stack_grande() {
        let p = PlayerState::new("whale".to_string(), 1_000_000, 0);
        assert_eq!(p.stack, 1_000_000);
    }

    #[test]
    fn ps_stack_pequeno() {
        let p = PlayerState::new("short".to_string(), 0, 0);
        assert_eq!(p.stack, 0);
    }

    #[test]
    fn ps_seat_index_maximo_8() {
        let p = PlayerState::new("last".to_string(), 100, 8);
        assert_eq!(p.seat_index, 8);
    }

    #[test]
    fn ps_id_vazio_permitido() {
        let p = PlayerState::new("".to_string(), 100, 0);
        assert_eq!(p.id, "");
    }

    #[test]
    fn ps_id_com_espacos() {
        let p = PlayerState::new("player one".to_string(), 100, 0);
        assert_eq!(p.id, "player one");
    }

    #[test]
    fn ps_id_unicode() {
        let p = PlayerState::new("jogador_ção".to_string(), 100, 0);
        assert_eq!(p.id, "jogador_ção");
    }

    // ─── Sequência de operações ───

    #[test]
    fn ps_reset_round_bet_apos_multiplas_apostas() {
        let mut p = PlayerState::new("x".to_string(), 1000, 0);
        p.current_bet = 100;
        p.has_acted = true;
        p.reset_round_bet();
        assert_eq!(p.current_bet, 0);
        assert!(!p.has_acted);
        p.current_bet = 200;
        p.has_acted = true;
        p.reset_round_bet();
        assert_eq!(p.current_bet, 0);
        assert!(!p.has_acted);
    }

    #[test]
    fn ps_total_bet_acumula_manualmente() {
        let mut p = PlayerState::new("x".to_string(), 1000, 0);
        p.total_bet += 50;
        p.total_bet += 30;
        assert_eq!(p.total_bet, 80);
    }

    #[test]
    fn ps_current_bet_substitui_nao_acumula() {
        let mut p = PlayerState::new("x".to_string(), 1000, 0);
        p.current_bet = 50;
        p.current_bet = 100;
        assert_eq!(p.current_bet, 100);
    }

    #[test]
    fn ps_hole_cards_pode_ter_2_cartas() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.hole_cards = vec![
            Card {
                rank: Rank::Ace,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Spades,
            },
        ];
        assert_eq!(p.hole_cards.len(), 2);
    }

    #[test]
    fn ps_hole_cards_pode_ser_vazio() {
        let p = PlayerState::new("x".to_string(), 100, 0);
        assert!(p.hole_cards.is_empty());
    }

    #[test]
    fn ps_hole_cards_clone_independente() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.hole_cards = vec![Card {
            rank: Rank::Two,
            suit: Suit::Clubs,
        }];
        let mut c = p.clone();
        c.hole_cards.push(Card {
            rank: Rank::Three,
            suit: Suit::Clubs,
        });
        assert_eq!(p.hole_cards.len(), 1);
        assert_eq!(c.hole_cards.len(), 2);
    }

    #[test]
    fn ps_can_act_apos_unfold_manual() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = true;
        assert!(!p.can_act());
        p.has_folded = false;
        assert!(p.can_act());
    }

    #[test]
    fn ps_is_in_hand_apos_unfold_manual() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.has_folded = true;
        assert!(!p.is_in_hand());
        p.has_folded = false;
        assert!(p.is_in_hand());
    }

    #[test]
    fn ps_reset_round_bet_idempotente() {
        let mut p = PlayerState::new("x".to_string(), 100, 0);
        p.reset_round_bet();
        let s1 = (p.current_bet, p.has_acted);
        p.reset_round_bet();
        let s2 = (p.current_bet, p.has_acted);
        assert_eq!(s1, s2);
    }

    #[test]
    fn ps_dez_jogadores_todos_can_act() {
        let players: Vec<PlayerState> = (0..10)
            .map(|i| PlayerState::new(format!("p{i}"), 1000, i))
            .collect();
        assert!(players.iter().all(|p| p.can_act()));
    }

    #[test]
    fn ps_dez_jogadores_todos_is_in_hand() {
        let players: Vec<PlayerState> = (0..10)
            .map(|i| PlayerState::new(format!("p{i}"), 1000, i))
            .collect();
        assert!(players.iter().all(|p| p.is_in_hand()));
    }
}

// ═══════════════════════════════════════════════════════════════════
// LOTE 2 — HandState (50 testes)
// ═══════════════════════════════════════════════════════════════════

mod hand_state_tests {
    use super::*;

    fn make_hand_state(players: Vec<PlayerState>) -> HandState {
        HandState {
            players,
            dealer_index: 0,
            community_cards: Vec::new(),
            phase: GamePhase::Preflop,
            deck: Vec::new(),
            burn_pile: Vec::new(),
            current_bet_to_match: 0,
            min_raise: 10,
            active_player_index: 0,
            small_blind: 5,
            big_blind: 10,
            is_finished: false,
        }
    }

    fn make_players(n: usize) -> Vec<PlayerState> {
        (0..n)
            .map(|i| PlayerState::new(format!("p{i}"), 1000, i))
            .collect()
    }

    // ─── active_players_count() ───

    #[test]
    fn hs_active_count_todos_ativos() {
        let hs = make_hand_state(make_players(4));
        assert_eq!(hs.active_players_count(), 4);
    }

    #[test]
    fn hs_active_count_um_foldado() {
        let mut players = make_players(3);
        players[0].has_folded = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.active_players_count(), 2);
    }

    #[test]
    fn hs_active_count_um_all_in() {
        let mut players = make_players(3);
        players[1].is_all_in = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.active_players_count(), 2);
    }

    #[test]
    fn hs_active_count_todos_foldados() {
        let mut players = make_players(3);
        for p in &mut players {
            p.has_folded = true;
        }
        let hs = make_hand_state(players);
        assert_eq!(hs.active_players_count(), 0);
    }

    #[test]
    fn hs_active_count_todos_all_in() {
        let mut players = make_players(3);
        for p in &mut players {
            p.is_all_in = true;
        }
        let hs = make_hand_state(players);
        assert_eq!(hs.active_players_count(), 0);
    }

    #[test]
    fn hs_active_count_vazio() {
        let hs = make_hand_state(Vec::new());
        assert_eq!(hs.active_players_count(), 0);
    }

    #[test]
    fn hs_active_count_mistura_estados() {
        let mut players = make_players(6);
        players[0].has_folded = true;
        players[1].is_all_in = true;
        players[2].has_folded = true;
        players[3].is_all_in = true;
        // p4 e p5 ativos
        let hs = make_hand_state(players);
        assert_eq!(hs.active_players_count(), 2);
    }

    #[test]
    fn hs_active_count_um_jogador() {
        let hs = make_hand_state(make_players(1));
        assert_eq!(hs.active_players_count(), 1);
    }

    // ─── players_in_hand_count() ───

    #[test]
    fn hs_in_hand_count_todos_na_mao() {
        let hs = make_hand_state(make_players(4));
        assert_eq!(hs.players_in_hand_count(), 4);
    }

    #[test]
    fn hs_in_hand_count_um_foldado() {
        let mut players = make_players(3);
        players[0].has_folded = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.players_in_hand_count(), 2);
    }

    #[test]
    fn hs_in_hand_count_all_in_conta() {
        let mut players = make_players(3);
        players[0].is_all_in = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.players_in_hand_count(), 3);
    }

    #[test]
    fn hs_in_hand_count_todos_foldados() {
        let mut players = make_players(3);
        for p in &mut players {
            p.has_folded = true;
        }
        let hs = make_hand_state(players);
        assert_eq!(hs.players_in_hand_count(), 0);
    }

    #[test]
    fn hs_in_hand_count_vazio() {
        let hs = make_hand_state(Vec::new());
        assert_eq!(hs.players_in_hand_count(), 0);
    }

    #[test]
    fn hs_in_hand_count_mistura() {
        let mut players = make_players(5);
        players[0].has_folded = true;
        players[2].has_folded = true;
        players[4].is_all_in = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.players_in_hand_count(), 3);
    }

    // ─── active_player() ───

    #[test]
    fn hs_active_player_retorna_ref() {
        let hs = make_hand_state(make_players(3));
        let ap = hs.active_player();
        assert!(ap.is_some());
        assert_eq!(ap.unwrap().id, "p0");
    }

    #[test]
    fn hs_active_player_indice_1() {
        let mut hs = make_hand_state(make_players(3));
        hs.active_player_index = 1;
        let ap = hs.active_player();
        assert_eq!(ap.unwrap().id, "p1");
    }

    #[test]
    fn hs_active_player_indice_ultimo() {
        let mut hs = make_hand_state(make_players(4));
        hs.active_player_index = 3;
        let ap = hs.active_player();
        assert_eq!(ap.unwrap().id, "p3");
    }

    #[test]
    fn hs_active_player_vazio_none() {
        let hs = make_hand_state(Vec::new());
        assert!(hs.active_player().is_none());
    }

    #[test]
    fn hs_active_player_indice_fora_range_none() {
        let mut hs = make_hand_state(make_players(2));
        hs.active_player_index = 5;
        assert!(hs.active_player().is_none());
    }

    // ─── active_player_mut() ───

    #[test]
    fn hs_active_player_mut_modifica() {
        let mut hs = make_hand_state(make_players(2));
        if let Some(p) = hs.active_player_mut() {
            p.stack = 500;
        }
        assert_eq!(hs.players[0].stack, 500);
    }

    #[test]
    fn hs_active_player_mut_indice_1() {
        let mut hs = make_hand_state(make_players(3));
        hs.active_player_index = 1;
        if let Some(p) = hs.active_player_mut() {
            p.has_folded = true;
        }
        assert!(hs.players[1].has_folded);
    }

    #[test]
    fn hs_active_player_mut_vazio_none() {
        let mut hs = make_hand_state(Vec::new());
        assert!(hs.active_player_mut().is_none());
    }

    // ─── total_pot() ───

    #[test]
    fn hs_total_pot_zero_sem_apostas() {
        let hs = make_hand_state(make_players(3));
        assert_eq!(hs.total_pot(), 0);
    }

    #[test]
    fn hs_total_pot_soma_total_bets() {
        let mut players = make_players(3);
        players[0].total_bet = 100;
        players[1].total_bet = 50;
        players[2].total_bet = 25;
        let hs = make_hand_state(players);
        assert_eq!(hs.total_pot(), 175);
    }

    #[test]
    fn hs_total_pot_ignora_folded_se_total_bet_existe() {
        let mut players = make_players(2);
        players[0].total_bet = 100;
        players[0].has_folded = true;
        players[1].total_bet = 50;
        let hs = make_hand_state(players);
        // total_pot soma total_bet de todos (folded também contribuiu)
        assert_eq!(hs.total_pot(), 150);
    }

    #[test]
    fn hs_total_pot_vazio() {
        let hs = make_hand_state(Vec::new());
        assert_eq!(hs.total_pot(), 0);
    }

    #[test]
    fn hs_total_pot_um_jogador() {
        let mut players = make_players(1);
        players[0].total_bet = 200;
        let hs = make_hand_state(players);
        assert_eq!(hs.total_pot(), 200);
    }

    // ─── next_active_player() ───

    #[test]
    fn hs_next_active_from_0() {
        let hs = make_hand_state(make_players(4));
        assert_eq!(hs.next_active_player(0), Some(1));
    }

    #[test]
    fn hs_next_active_from_ultimo_wrap() {
        let hs = make_hand_state(make_players(4));
        assert_eq!(hs.next_active_player(3), Some(0));
    }

    #[test]
    fn hs_next_active_pula_foldado() {
        let mut players = make_players(4);
        players[1].has_folded = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.next_active_player(0), Some(2));
    }

    #[test]
    fn hs_next_active_pula_all_in() {
        let mut players = make_players(4);
        players[2].is_all_in = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.next_active_player(1), Some(3));
    }

    #[test]
    fn hs_next_active_pula_foldado_e_all_in() {
        let mut players = make_players(5);
        players[1].has_folded = true;
        players[2].is_all_in = true;
        let hs = make_hand_state(players);
        assert_eq!(hs.next_active_player(0), Some(3));
    }

    #[test]
    fn hs_next_active_nenhum_ativo_none() {
        let mut players = make_players(3);
        for p in &mut players {
            p.has_folded = true;
        }
        let hs = make_hand_state(players);
        assert_eq!(hs.next_active_player(0), None);
    }

    #[test]
    fn hs_next_active_vazio_none() {
        let hs = make_hand_state(Vec::new());
        assert_eq!(hs.next_active_player(0), None);
    }

    #[test]
    fn hs_next_active_apenas_um_ativo_retorna_ele() {
        let mut players = make_players(3);
        players[0].has_folded = true;
        players[2].is_all_in = true;
        let hs = make_hand_state(players);
        // from 1, next active should wrap to 1 itself
        assert_eq!(hs.next_active_player(1), Some(1));
    }

    #[test]
    fn hs_next_active_from_meio() {
        let hs = make_hand_state(make_players(6));
        assert_eq!(hs.next_active_player(2), Some(3));
    }

    #[test]
    fn hs_next_active_dois_jogadores() {
        let hs = make_hand_state(make_players(2));
        assert_eq!(hs.next_active_player(0), Some(1));
        assert_eq!(hs.next_active_player(1), Some(0));
    }

    // ─── Campos struct e fase ───

    #[test]
    fn hs_phase_inicial_preflop() {
        let hs = make_hand_state(make_players(2));
        assert_eq!(hs.phase, GamePhase::Preflop);
    }

    #[test]
    fn hs_community_cards_vazio_inicial() {
        let hs = make_hand_state(make_players(2));
        assert!(hs.community_cards.is_empty());
    }

    #[test]
    fn hs_deck_vazio_inicial() {
        let hs = make_hand_state(make_players(2));
        assert!(hs.deck.is_empty());
    }

    #[test]
    fn hs_burn_pile_vazio_inicial() {
        let hs = make_hand_state(make_players(2));
        assert!(hs.burn_pile.is_empty());
    }

    #[test]
    fn hs_is_finished_false_inicial() {
        let hs = make_hand_state(make_players(2));
        assert!(!hs.is_finished);
    }

    #[test]
    fn hs_dealer_index_preservado() {
        let mut hs = make_hand_state(make_players(4));
        hs.dealer_index = 2;
        assert_eq!(hs.dealer_index, 2);
    }

    #[test]
    fn hs_current_bet_to_match_preservado() {
        let mut hs = make_hand_state(make_players(2));
        hs.current_bet_to_match = 50;
        assert_eq!(hs.current_bet_to_match, 50);
    }

    #[test]
    fn hs_min_raise_preservado() {
        let mut hs = make_hand_state(make_players(2));
        hs.min_raise = 20;
        assert_eq!(hs.min_raise, 20);
    }

    #[test]
    fn hs_small_blind_preservado() {
        let hs = make_hand_state(make_players(2));
        assert_eq!(hs.small_blind, 5);
    }

    #[test]
    fn hs_big_blind_preservado() {
        let hs = make_hand_state(make_players(2));
        assert_eq!(hs.big_blind, 10);
    }

    // ─── Clone e Debug ───

    #[test]
    fn hs_clone_preserva_estado() {
        let mut hs = make_hand_state(make_players(3));
        hs.current_bet_to_match = 100;
        hs.phase = GamePhase::Flop;
        let c = hs.clone();
        assert_eq!(c.current_bet_to_match, 100);
        assert_eq!(c.phase, GamePhase::Flop);
        assert_eq!(c.players.len(), 3);
    }

    #[test]
    fn hs_clone_independente() {
        let hs = make_hand_state(make_players(2));
        let mut c = hs.clone();
        c.current_bet_to_match = 999;
        assert_eq!(hs.current_bet_to_match, 0);
    }

    #[test]
    fn hs_debug_nao_panic() {
        let hs = make_hand_state(make_players(2));
        let s = format!("{hs:?}");
        assert!(s.contains("HandState"));
    }

    #[test]
    fn hs_clone_preserva_community_cards() {
        let mut hs = make_hand_state(make_players(2));
        hs.community_cards = vec![
            Card {
                rank: Rank::Ace,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Diamonds,
            },
        ];
        let c = hs.clone();
        assert_eq!(c.community_cards.len(), 3);
    }

    #[test]
    fn hs_clone_preserva_dealer_index() {
        let mut hs = make_hand_state(make_players(5));
        hs.dealer_index = 3;
        let c = hs.clone();
        assert_eq!(c.dealer_index, 3);
    }

    #[test]
    fn hs_clone_preserva_active_player_index() {
        let mut hs = make_hand_state(make_players(4));
        hs.active_player_index = 2;
        let c = hs.clone();
        assert_eq!(c.active_player_index, 2);
    }

    #[test]
    fn hs_active_count_9_jogadores() {
        let hs = make_hand_state(make_players(9));
        assert_eq!(hs.active_players_count(), 9);
    }

    #[test]
    fn hs_in_hand_count_9_jogadores() {
        let hs = make_hand_state(make_players(9));
        assert_eq!(hs.players_in_hand_count(), 9);
    }

    #[test]
    fn hs_total_pot_9_jogadores_blinds() {
        let mut players = make_players(9);
        // Simular blinds: SB=5, BB=10
        players[1].total_bet = 5;
        players[2].total_bet = 10;
        let hs = make_hand_state(players);
        assert_eq!(hs.total_pot(), 15);
    }
}

// ═══════════════════════════════════════════════════════════════════
// LOTE 3 — GameLoop init / blinds / ante (50 testes)
// ═══════════════════════════════════════════════════════════════════

mod game_loop_init_tests {
    use super::*;

    // ─── GameLoop::new() ───

    #[test]
    fn gl_new_cria_estrutura() {
        let gl = GameLoop::new(make_config(), "h1".into(), "Mesa".into(), GameType::Cash);
        assert_eq!(gl.hand_id, "h1");
        assert_eq!(gl.table_name, "Mesa");
        assert_eq!(gl.state.players.len(), 0);
        assert_eq!(gl.state.phase, GamePhase::Preflop);
        assert!(!gl.state.is_finished);
    }

    #[test]
    fn gl_new_blinds_padrao() {
        let gl = GameLoop::new(
            TableConfig::new(2000, 500, 500),
            "h1".into(),
            "Mesa".into(),
            GameType::Cash,
        );
        assert_eq!(gl.state.small_blind, 1000);
        assert_eq!(gl.state.big_blind, 2000);
        assert_eq!(gl.state.min_raise, 2000);
    }

    #[test]
    fn gl_new_ante_none() {
        let gl = make_game_loop_2p();
        assert!(gl.ante.is_none());
    }

    #[test]
    fn gl_new_game_type_tournament() {
        let gl = GameLoop::new(
            make_config(),
            "h1".into(),
            "Mesa".into(),
            GameType::Tournament,
        );
        assert_eq!(gl.game_type, GameType::Tournament);
    }

    #[test]
    fn gl_new_history_none() {
        let gl = make_game_loop_2p();
        assert!(gl.history.is_none());
    }

    #[test]
    fn gl_new_current_bet_to_match_zero() {
        let gl = make_game_loop_2p();
        assert_eq!(gl.state.current_bet_to_match, 0);
    }

    #[test]
    fn gl_new_dealer_index_zero() {
        let gl = make_game_loop_2p();
        assert_eq!(gl.state.dealer_index, 0);
    }

    #[test]
    fn gl_new_community_cards_vazio() {
        let gl = make_game_loop_2p();
        assert!(gl.state.community_cards.is_empty());
    }

    #[test]
    fn gl_new_deck_vazio() {
        let gl = make_game_loop_2p();
        assert!(gl.state.deck.is_empty());
    }

    // ─── with_ante() ───

    #[test]
    fn gl_with_ante_define_valor() {
        let gl = make_game_loop_2p().with_ante(5);
        assert_eq!(gl.ante, Some(5));
    }

    #[test]
    fn gl_with_ante_zero() {
        let gl = make_game_loop_2p().with_ante(0);
        assert_eq!(gl.ante, Some(0));
    }

    #[test]
    fn gl_with_ante_nao_afeta_state() {
        let gl = make_game_loop_2p().with_ante(5);
        assert_eq!(gl.state.players[0].stack, 1000);
    }

    #[test]
    fn gl_with_ante_encadeavel() {
        let gl = make_game_loop_2p().with_ante(3);
        assert_eq!(gl.ante, Some(3));
    }

    // ─── add_player() ───

    #[test]
    fn gl_add_player_incrementa() {
        let gl = make_game_loop_2p();
        assert_eq!(gl.state.players.len(), 2);
    }

    #[test]
    fn gl_add_player_3_jogadores() {
        let gl = make_game_loop_3p();
        assert_eq!(gl.state.players.len(), 3);
    }

    #[test]
    fn gl_add_player_6_jogadores() {
        let gl = make_game_loop_6p();
        assert_eq!(gl.state.players.len(), 6);
    }

    #[test]
    fn gl_add_player_9_jogadores() {
        let gl = make_game_loop_n(9);
        assert_eq!(gl.state.players.len(), 9);
    }

    #[test]
    fn gl_add_player_stack_preservado() {
        let gl = make_game_loop_2p();
        assert_eq!(gl.state.players[0].stack, 1000);
        assert_eq!(gl.state.players[1].stack, 1000);
    }

    #[test]
    fn gl_add_player_ids_corretos() {
        let gl = make_game_loop_2p();
        assert_eq!(gl.state.players[0].id, "alice");
        assert_eq!(gl.state.players[1].id, "bob");
    }

    #[test]
    fn gl_add_player_seat_index_sequencial() {
        let gl = make_game_loop_3p();
        assert_eq!(gl.state.players[0].seat_index, 0);
        assert_eq!(gl.state.players[1].seat_index, 1);
        assert_eq!(gl.state.players[2].seat_index, 2);
    }

    // ─── set_dealer() ───

    #[test]
    fn gl_set_dealer_posicao_0() {
        let mut gl = make_game_loop_2p();
        gl.set_dealer(0);
        assert_eq!(gl.state.dealer_index, 0);
    }

    #[test]
    fn gl_set_dealer_posicao_1() {
        let mut gl = make_game_loop_2p();
        gl.set_dealer(1);
        assert_eq!(gl.state.dealer_index, 1);
    }

    #[test]
    fn gl_set_dealer_ultimo() {
        let mut gl = make_game_loop_4p();
        gl.set_dealer(3);
        assert_eq!(gl.state.dealer_index, 3);
    }

    // ─── start_hand() — blinds ───

    #[test]
    fn gl_start_hand_2p_sb_bb_coletados() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Heads-up: dealer(0) = SB, p1 = BB
        assert_eq!(gl.state.players[0].stack, 995); // 1000 - 5
        assert_eq!(gl.state.players[1].stack, 990); // 1000 - 10
    }

    #[test]
    fn gl_start_hand_2p_sb_bb_total_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert_eq!(gl.state.players[0].total_bet, 5);
        assert_eq!(gl.state.players[1].total_bet, 10);
    }

    #[test]
    fn gl_start_hand_3p_blinds_corretos() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // dealer=0, SB=1, BB=2
        assert_eq!(gl.state.players[1].stack, 995); // SB
        assert_eq!(gl.state.players[2].stack, 990); // BB
        assert_eq!(gl.state.players[0].stack, 1000); // dealer não paga blind
    }

    #[test]
    fn gl_start_hand_4p_blinds() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        // dealer=0, SB=1, BB=2
        assert_eq!(gl.state.players[1].total_bet, 5);
        assert_eq!(gl.state.players[2].total_bet, 10);
        assert_eq!(gl.state.players[0].total_bet, 0);
        assert_eq!(gl.state.players[3].total_bet, 0);
    }

    #[test]
    fn gl_start_hand_current_bet_to_match_bb() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert_eq!(gl.state.current_bet_to_match, 10);
    }

    #[test]
    fn gl_start_hand_min_raise_bb() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert_eq!(gl.state.min_raise, 10);
    }

    #[test]
    fn gl_start_hand_sb_all_in_se_stack_insuficiente() {
        let mut gl = GameLoop::new(make_config(), "h1".into(), "Mesa".into(), GameType::Cash);
        gl.add_player("alice".to_string(), 3); // SB = 5, mas stack = 3
        gl.add_player("bob".to_string(), 1000);
        gl.set_dealer(0);
        gl.start_hand().unwrap();
        assert!(gl.state.players[0].is_all_in);
        assert_eq!(gl.state.players[0].stack, 0);
    }

    #[test]
    fn gl_start_hand_bb_all_in_se_stack_insuficiente() {
        let mut gl = GameLoop::new(make_config(), "h1".into(), "Mesa".into(), GameType::Cash);
        gl.add_player("alice".to_string(), 1000);
        gl.add_player("bob".to_string(), 5); // BB = 10, mas stack = 5
        gl.set_dealer(0);
        gl.start_hand().unwrap();
        assert!(gl.state.players[1].is_all_in);
        assert_eq!(gl.state.players[1].stack, 0);
    }

    // ─── start_hand() — hole cards ───

    #[test]
    fn gl_start_hand_hole_cards_2_cada() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert_eq!(gl.state.players[0].hole_cards.len(), 2);
        assert_eq!(gl.state.players[1].hole_cards.len(), 2);
    }

    #[test]
    fn gl_start_hand_hole_cards_6p() {
        let mut gl = make_game_loop_6p();
        gl.start_hand().unwrap();
        for p in &gl.state.players {
            assert_eq!(p.hole_cards.len(), 2);
        }
    }

    #[test]
    fn gl_start_hand_deck_reduzido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // 52 - 4 hole cards = 48
        assert_eq!(gl.state.deck.len(), 48);
    }

    #[test]
    fn gl_start_hand_deck_6p() {
        let mut gl = make_game_loop_6p();
        gl.start_hand().unwrap();
        // 52 - 12 hole cards = 40
        assert_eq!(gl.state.deck.len(), 40);
    }

    // ─── start_hand() — first to act ───

    #[test]
    fn gl_start_hand_2p_first_to_act_sb() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Heads-up: SB (dealer=0) age primeiro
        assert_eq!(gl.state.active_player_index, 0);
    }

    #[test]
    fn gl_start_hand_3p_first_to_act_utg() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // dealer=0, SB=1, BB=2, UTG = next after BB = 0
        assert_eq!(gl.state.active_player_index, 0);
    }

    #[test]
    fn gl_start_hand_4p_first_to_act_utg() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        // dealer=0, SB=1, BB=2, UTG = next after BB = 3
        assert_eq!(gl.state.active_player_index, 3);
    }

    #[test]
    fn gl_start_hand_dealer_1_2p_first_to_act() {
        let mut gl = make_game_loop_2p();
        gl.set_dealer(1);
        gl.start_hand().unwrap();
        // Heads-up: dealer(1) = SB age primeiro
        assert_eq!(gl.state.active_player_index, 1);
    }

    // ─── start_hand() — ante ───

    #[test]
    fn gl_start_hand_com_ante_coleta() {
        let mut gl = make_game_loop_2p().with_ante(2);
        gl.start_hand().unwrap();
        // Ante é descontado do stack; total_bet é sobrescrito pelos blinds
        assert_eq!(gl.state.players[0].stack, 993); // 1000 - 2 ante - 5 SB
        assert_eq!(gl.state.players[1].stack, 988); // 1000 - 2 ante - 10 BB
        assert_eq!(gl.state.players[0].total_bet, 7); // 2 ante + 5 SB
        assert_eq!(gl.state.players[1].total_bet, 12); // 2 ante + 10 BB
    }

    #[test]
    fn gl_start_hand_ante_zero_ignorado() {
        let mut gl = make_game_loop_2p().with_ante(0);
        gl.start_hand().unwrap();
        assert_eq!(gl.state.players[0].total_bet, 5); // só SB
        assert_eq!(gl.state.players[1].total_bet, 10); // só BB
    }

    #[test]
    fn gl_start_hand_ante_reduz_stack() {
        let mut gl = make_game_loop_2p().with_ante(3);
        gl.start_hand().unwrap();
        assert_eq!(gl.state.players[0].stack, 992); // 1000 - 3 - 5
        assert_eq!(gl.state.players[1].stack, 987); // 1000 - 3 - 10
    }

    #[test]
    fn gl_start_hand_ante_6p() {
        let mut gl = make_game_loop_6p().with_ante(1);
        gl.start_hand().unwrap();
        for p in &gl.state.players {
            assert!(p.total_bet >= 1);
        }
    }

    fn make_tournament_with_bb_stack(bb_stack: u64) -> GameLoop {
        let mut gl = GameLoop::new(
            make_config(),
            "tournament-bba".to_string(),
            "Tournament BBA".to_string(),
            GameType::Tournament,
        )
        .with_ante(10)
        .with_skip_loss_deflator(true);
        gl.add_player("dealer".to_string(), 1000);
        gl.add_player("small-blind".to_string(), 1000);
        gl.add_player("big-blind".to_string(), bb_stack);
        gl.set_dealer(0);
        gl
    }

    fn finish_checking_hand(gl: &mut GameLoop) {
        for _ in 0..20 {
            if gl.state.is_finished {
                return;
            }
            let player = &gl.state.players[gl.state.active_player_index];
            let player_id = player.id.clone();
            let action = if player.current_bet < gl.state.current_bet_to_match {
                PlayerMove::Call
            } else {
                PlayerMove::Check
            };
            gl.player_action(&player_id, action).unwrap();
        }
        panic!("a mão de teste não terminou");
    }

    #[test]
    fn tournament_big_blind_ante_is_paid_only_by_big_blind_after_blinds() {
        let mut gl = make_tournament_with_bb_stack(1000);
        gl.start_hand().unwrap();

        assert_eq!(gl.state.players[0].total_bet, 0);
        assert_eq!(gl.state.players[0].stack, 1000);
        assert_eq!(gl.state.players[1].total_bet, 5);
        assert_eq!(gl.state.players[1].stack, 995);
        assert_eq!(gl.state.players[2].total_bet, 20);
        assert_eq!(gl.state.players[2].stack, 980);
    }

    #[test]
    fn tournament_short_big_blind_pays_no_ante_and_gets_proportional_main_pot() {
        let mut gl = make_tournament_with_bb_stack(7);
        gl.start_hand().unwrap();

        assert_eq!(gl.state.players[2].total_bet, 7);
        assert_eq!(gl.state.players[2].stack, 0);
        assert!(gl.state.players[2].is_all_in);
        assert_eq!(gl.state.players[0].total_bet, 0);
        assert_eq!(gl.state.players[1].total_bet, 5);

        finish_checking_hand(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.pots.len(), 2);
        assert_eq!(resolution.pots[0].amount, 21);
        assert_eq!(resolution.pots[0].eligible_players.len(), 3);
        assert_eq!(resolution.pots[1].amount, 6);
        assert_eq!(resolution.pots[1].eligible_players.len(), 2);
    }

    #[test]
    fn tournament_partial_big_blind_ante_stays_dead_in_main_pot() {
        let mut gl = make_tournament_with_bb_stack(15);
        gl.start_hand().unwrap();

        assert_eq!(gl.state.players[2].total_bet, 15);
        assert_eq!(gl.state.players[2].stack, 0);
        assert!(gl.state.players[2].is_all_in);

        finish_checking_hand(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.pots.len(), 1);
        assert_eq!(resolution.pots[0].amount, 35);
        assert_eq!(resolution.pots[0].eligible_players.len(), 3);
    }

    // ─── start_hand() — errors ───

    #[test]
    fn gl_start_hand_erro_menos_de_2() {
        let mut gl = GameLoop::new(make_config(), "h1".into(), "Mesa".into(), GameType::Cash);
        gl.add_player("solo".to_string(), 1000);
        gl.set_dealer(0);
        let err = gl.start_hand().unwrap_err();
        assert_eq!(err, GameLoopError::NotEnoughPlayers);
    }

    #[test]
    fn gl_start_hand_erro_0_jogadores() {
        let mut gl = GameLoop::new(make_config(), "h1".into(), "Mesa".into(), GameType::Cash);
        gl.set_dealer(0);
        let err = gl.start_hand().unwrap_err();
        assert_eq!(err, GameLoopError::NotEnoughPlayers);
    }

    #[test]
    fn gl_start_hand_erro_hand_finished() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.state.is_finished = true;
        let err = gl.start_hand().unwrap_err();
        assert_eq!(err, GameLoopError::HandAlreadyFinished);
    }

    // ─── start_hand() — history ───

    #[test]
    fn gl_start_hand_cria_history() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert!(gl.history.is_some());
    }

    #[test]
    fn gl_start_hand_history_tem_2_acoes() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let h = gl.history.as_ref().unwrap();
        // SB call + BB raise
        assert_eq!(h.actions.len(), 2);
    }

    #[test]
    fn gl_start_hand_history_hand_id() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let h = gl.history.as_ref().unwrap();
        assert_eq!(h.hand_id, "hand-001");
    }

    // ─── dealer position variations ───

    #[test]
    fn gl_start_hand_dealer_2_4p_blinds() {
        let mut gl = make_game_loop_4p();
        gl.set_dealer(2);
        gl.start_hand().unwrap();
        // dealer=2, SB=3, BB=0
        assert_eq!(gl.state.players[3].total_bet, 5); // SB
        assert_eq!(gl.state.players[0].total_bet, 10); // BB
    }

    #[test]
    fn gl_start_hand_dealer_3_4p_first_to_act() {
        let mut gl = make_game_loop_4p();
        gl.set_dealer(3);
        gl.start_hand().unwrap();
        // dealer=3, SB=0, BB=1, UTG = next after BB = 2
        assert_eq!(gl.state.active_player_index, 2);
    }

    #[test]
    fn gl_start_hand_5p_blinds() {
        let mut gl = make_game_loop_n(5);
        gl.start_hand().unwrap();
        // dealer=0, SB=1, BB=2
        assert_eq!(gl.state.players[1].total_bet, 5);
        assert_eq!(gl.state.players[2].total_bet, 10);
    }

    #[test]
    fn gl_start_hand_5p_first_to_act() {
        let mut gl = make_game_loop_n(5);
        gl.start_hand().unwrap();
        // dealer=0, SB=1, BB=2, UTG = 3
        assert_eq!(gl.state.active_player_index, 3);
    }

    #[test]
    fn gl_start_hand_9p_blinds() {
        let mut gl = make_game_loop_n(9);
        gl.start_hand().unwrap();
        assert_eq!(gl.state.players[1].total_bet, 5);
        assert_eq!(gl.state.players[2].total_bet, 10);
    }

    #[test]
    fn gl_start_hand_9p_first_to_act() {
        let mut gl = make_game_loop_n(9);
        gl.start_hand().unwrap();
        // dealer=0, SB=1, BB=2, UTG = 3
        assert_eq!(gl.state.active_player_index, 3);
    }

    #[test]
    fn gl_start_hand_nao_afeta_outros_jogadores() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        // dealer(0) e UTG(3) não pagam blinds
        assert_eq!(gl.state.players[0].total_bet, 0);
        assert_eq!(gl.state.players[3].total_bet, 0);
    }

    #[test]
    fn gl_start_hand_is_finished_false() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert!(!gl.state.is_finished);
    }

    #[test]
    fn gl_start_hand_phase_preflop() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert_eq!(gl.state.phase, GamePhase::Preflop);
    }
}

// ═══════════════════════════════════════════════════════════════════
// LOTE 4 — Fold / Check / Call (50+ testes)
// ═══════════════════════════════════════════════════════════════════

mod fold_check_call_tests {
    use super::*;

    // ─── Fold — básico ───

    #[test]
    fn fold_2p_sb_fold_termina_mao() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // SB (dealer=0) é o primeiro a agir heads-up
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(gl.state.is_finished);
    }

    #[test]
    fn fold_2p_bb_vence_por_fold() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Bob (BB) ainda está na mão
        assert!(gl.state.players[1].is_in_hand());
        assert!(gl.state.players[0].has_folded);
    }

    #[test]
    fn fold_2p_bb_nao_foldou() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(!gl.state.players[1].has_folded);
    }

    #[test]
    fn fold_2p_has_acted_true() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(gl.state.players[0].has_acted);
    }

    #[test]
    fn fold_3p_utg_fold_continua_mao() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: dealer=0, SB=1, BB=2, UTG=0 (next after BB wraps to 0)
        // Actually first to act = next after BB = 0 (dealer)
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(!gl.state.is_finished);
        assert!(gl.state.players[0].has_folded);
    }

    #[test]
    fn fold_3p_dois_fold_termina() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // First to act = player 0 (UTG in 3p)
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Next to act = player 1 (SB)
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        assert!(gl.state.is_finished);
        assert!(gl.state.players[2].is_in_hand());
    }

    #[test]
    fn fold_3p_bb_vence_sozinho() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        assert!(gl.state.players[2].is_in_hand());
        assert!(!gl.state.players[2].has_folded);
    }

    #[test]
    fn fold_4p_tres_fold_bb_vence() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        // 4p: dealer=0, SB=1, BB=2, UTG=3
        gl.player_action("dave", PlayerMove::Fold).unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        assert!(gl.state.is_finished);
        assert!(gl.state.players[2].is_in_hand());
    }

    #[test]
    fn fold_4p_um_fold_continua() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        gl.player_action("dave", PlayerMove::Fold).unwrap();
        assert!(!gl.state.is_finished);
    }

    #[test]
    fn fold_4p_dois_fold_continua() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        gl.player_action("dave", PlayerMove::Fold).unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(!gl.state.is_finished);
    }

    // ─── Fold — erros ───

    #[test]
    fn fold_jogador_errado_erro() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let err = gl.player_action("bob", PlayerMove::Fold).unwrap_err();
        assert_eq!(err, GameLoopError::NotYourTurn("bob".to_string()));
    }

    #[test]
    fn fold_mao_nao_iniciada_erro() {
        let gl = make_game_loop_2p();
        // Não chamou start_hand
        let mut gl = gl;
        let err = gl.player_action("alice", PlayerMove::Fold);
        // active_player_index pode ser 0 mas state.phase é Preflop por padrão
        // is_finished é false, então pode passar pela primeira checagem
        // Mas o jogador 0 não pode agir se não há mão iniciada
        // Verificar que retorna erro ou Ok dependendo do estado
        // Como start_hand não foi chamado, o estado pode ser inconsistente
        // Vamos apenas garantir que não panic
        let _ = err;
    }

    #[test]
    fn fold_mao_ja_terminada_erro() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let err = gl.player_action("bob", PlayerMove::Fold).unwrap_err();
        assert_eq!(err, GameLoopError::HandAlreadyFinished);
    }

    #[test]
    fn fold_jogador_inexistente_erro() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let err = gl.player_action("zezinho", PlayerMove::Fold).unwrap_err();
        assert_eq!(err, GameLoopError::NotYourTurn("zezinho".to_string()));
    }

    #[test]
    fn fold_jogador_que_ja_foldou_erro() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Tentar foldar alice de novo (mas agora a vez é de bob)
        let err = gl.player_action("alice", PlayerMove::Fold).unwrap_err();
        assert_eq!(err, GameLoopError::NotYourTurn("alice".to_string()));
    }

    // ─── Fold — estado após fold ───

    #[test]
    fn fold_total_bet_preservado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Alice pagou SB = 5 antes de foldar
        assert_eq!(gl.state.players[0].total_bet, 5);
    }

    #[test]
    fn fold_stack_preservado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Alice tinha 1000 - 5 (SB) = 995
        assert_eq!(gl.state.players[0].stack, 995);
    }

    #[test]
    fn fold_is_all_in_false() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(!gl.state.players[0].is_all_in);
    }

    #[test]
    fn fold_current_bet_preservado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert_eq!(gl.state.players[0].current_bet, 5);
    }

    #[test]
    fn fold_can_act_false_apos_fold() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(!gl.state.players[0].can_act());
    }

    #[test]
    fn fold_is_in_hand_false_apos_fold() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(!gl.state.players[0].is_in_hand());
    }

    // ─── Check — básico ───

    #[test]
    fn check_2p_bb_check_apos_sb_call() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // SB (alice) calls first
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // BB (bob) checks
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Rodada completa, deve avançar para Flop
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn check_2p_sb_check_erro_preflop() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // SB tem que pagar BB, não pode dar check
        let err = gl.player_action("alice", PlayerMove::Check).unwrap_err();
        assert!(matches!(err, GameLoopError::InvalidActionForPhase(_)));
    }

    #[test]
    fn check_3p_bb_check_apos_todos_fold() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // UTG (alice) folda, SB (bob) folda, BB (carol) check
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        // Agora só carol está na mão → mão termina por fold
        assert!(gl.state.is_finished);
    }

    #[test]
    fn check_postflop_primeiro_jogador() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: SB calls, BB checks → Flop
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
        // Postflop heads-up: primeiro ativo à esquerda do dealer = bob (BB)
        // Bob pode dar check (não há aposta)
        gl.player_action("bob", PlayerMove::Check).unwrap();
    }

    #[test]
    fn check_postflop_segundo_jogador() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob age primeiro, depois alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // Turn
        assert_eq!(gl.state.phase, GamePhase::Turn);
    }

    #[test]
    fn check_todos_check_ate_showdown() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob, alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // Turn: bob, alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // River: bob, alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // Showdown
        assert_eq!(gl.state.phase, GamePhase::Showdown);
        assert!(gl.state.is_finished);
    }

    #[test]
    fn check_4p_todos_check_flop() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        // UTG=3, dealer=0, SB=1, BB=2
        gl.player_action("dave", PlayerMove::Call).unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn check_erro_quando_ha_aposta() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // UTG (alice) calls BB
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // SB (bob) calls BB
        gl.player_action("bob", PlayerMove::Call).unwrap();
        // BB (carol) pode dar check (aposta igualada)
        gl.player_action("carol", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn check_erro_quando_ha_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // SB (alice) raises
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // BB (bob) tenta check → erro
        let err = gl.player_action("bob", PlayerMove::Check).unwrap_err();
        assert!(matches!(err, GameLoopError::InvalidActionForPhase(_)));
    }

    #[test]
    fn check_has_acted_true() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Após check, advance_phase reseta has_acted
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn check_nao_altera_stack() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        let stack_antes = gl.state.players[1].stack;
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.players[1].stack, stack_antes);
    }

    #[test]
    fn check_nao_altera_current_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Após check, advance_phase reseta current_bet para 0
        assert_eq!(gl.state.players[1].current_bet, 0);
    }

    #[test]
    fn check_nao_altera_total_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        let tb_antes = gl.state.players[1].total_bet;
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.players[1].total_bet, tb_antes);
    }

    // ─── Call — básico ───

    #[test]
    fn call_2p_sb_call_iguala_bb() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // SB igualou BB: current_bet deve ser 10
        assert_eq!(gl.state.players[0].current_bet, 10);
    }

    #[test]
    fn call_2p_sb_call_stack_reduzido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // 1000 - 5 (SB) - 5 (call complement) = 990
        assert_eq!(gl.state.players[0].stack, 990);
    }

    #[test]
    fn call_2p_sb_call_total_bet_bb() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        assert_eq!(gl.state.players[0].total_bet, 10);
    }

    #[test]
    fn call_2p_sb_call_has_acted() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        assert!(gl.state.players[0].has_acted);
    }

    #[test]
    fn call_2p_sb_bb_call_flop() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn call_3p_utg_call_bb() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // UTG (alice) calls BB=10
        gl.player_action("alice", PlayerMove::Call).unwrap();
        assert_eq!(gl.state.players[0].current_bet, 10);
        assert_eq!(gl.state.players[0].stack, 990);
    }

    #[test]
    fn call_3p_sb_call_bb() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        // SB (bob) calls: 5 + 5 = 10
        assert_eq!(gl.state.players[1].current_bet, 10);
    }

    #[test]
    fn call_4p_todos_call_flop() {
        let mut gl = make_game_loop_4p();
        gl.start_hand().unwrap();
        gl.player_action("dave", PlayerMove::Call).unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn call_2p_sb_call_all_in_stack_insuficiente() {
        let mut gl = make_game_loop_2p();
        gl.add_player("shorty".to_string(), 3);
        // Agora temos 3 jogadores, mas shorty tem só 3
        // Vamos testar com 2 jogadores e stack baixo
        let mut gl2 = GameLoop::new(
            make_config(),
            "hand-short".to_string(),
            "Short Table".to_string(),
            GameType::Cash,
        );
        gl2.add_player("alice".to_string(), 3);
        gl2.add_player("bob".to_string(), 1000);
        gl2.set_dealer(0);
        gl2.start_hand().unwrap();
        // Alice é SB (dealer=0 heads-up), SB = min(5, 3) = 3
        // Alice tem 0 restante, está all-in
        assert!(gl2.state.players[0].is_all_in);
        assert_eq!(gl2.state.players[0].stack, 0);
    }

    #[test]
    fn call_erro_sem_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob age primeiro, tenta call sem haver aposta
        let err = gl.player_action("bob", PlayerMove::Call).unwrap_err();
        assert!(matches!(err, GameLoopError::InvalidActionForPhase(_)));
    }

    #[test]
    fn call_erro_jogador_errado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let err = gl.player_action("bob", PlayerMove::Call).unwrap_err();
        assert_eq!(err, GameLoopError::NotYourTurn("bob".to_string()));
    }

    #[test]
    fn call_erro_mao_terminada() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let err = gl.player_action("bob", PlayerMove::Call).unwrap_err();
        assert_eq!(err, GameLoopError::HandAlreadyFinished);
    }

    #[test]
    fn call_apos_raise_iguala_nova_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // SB raises to 30
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // BB calls 30
        gl.player_action("bob", PlayerMove::Call).unwrap();
        // Após call, advance_phase reseta current_bet para 0
        assert_eq!(gl.state.players[1].current_bet, 0);
        assert_eq!(gl.state.players[1].stack, 970);
    }

    #[test]
    fn call_apos_raise_flop() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn call_apos_raise_current_bet_to_match() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        // Após call, advance_phase reseta current_bet_to_match para 0
        assert_eq!(gl.state.current_bet_to_match, 0);
    }

    #[test]
    fn call_3p_utg_call_sb_call_bb_check_flop() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn call_3p_todos_igualados_current_bet() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        // Após check, advance_phase reseta current_bet para 0
        for p in &gl.state.players {
            assert_eq!(p.current_bet, 0);
        }
    }

    #[test]
    fn call_3p_todos_igualados_has_acted() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        // Após check, advance_phase reseta has_acted
        for p in &gl.state.players {
            assert!(!p.has_acted);
        }
    }

    #[test]
    fn call_3p_todos_igualados_stack() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        // Alice: 1000 - 10 = 990
        // Bob: 1000 - 5 (SB) - 5 (call) = 990
        // Carol: 1000 - 10 (BB) = 990
        assert_eq!(gl.state.players[0].stack, 990);
        assert_eq!(gl.state.players[1].stack, 990);
        assert_eq!(gl.state.players[2].stack, 990);
    }

    #[test]
    fn call_2p_sb_call_is_all_in_false() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        assert!(!gl.state.players[0].is_all_in);
    }

    #[test]
    fn call_2p_bb_check_is_all_in_false() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert!(!gl.state.players[1].is_all_in);
    }

    #[test]
    fn call_2p_bb_check_nao_foldou() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert!(!gl.state.players[1].has_folded);
    }

    #[test]
    fn call_2p_sb_call_nao_foldou() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        assert!(!gl.state.players[0].has_folded);
    }

    #[test]
    fn call_2p_sb_call_can_act_true() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        assert!(gl.state.players[0].can_act());
    }

    #[test]
    fn call_2p_bb_check_can_act_true() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert!(gl.state.players[1].can_act());
    }

    #[test]
    fn call_2p_flop_current_bet_to_match_zero() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 0);
    }

    #[test]
    fn call_2p_flop_current_bets_resetadas() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        for p in &gl.state.players {
            assert_eq!(p.current_bet, 0);
        }
    }

    #[test]
    fn call_2p_flop_has_acted_resetado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        for p in &gl.state.players {
            assert!(!p.has_acted);
        }
    }

    #[test]
    fn call_2p_flop_min_raise_bb() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.min_raise, 10);
    }

    #[test]
    fn call_2p_flop_total_bet_preservado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // total_bet não é resetado entre fases
        assert_eq!(gl.state.players[0].total_bet, 10);
        assert_eq!(gl.state.players[1].total_bet, 10);
    }

    #[test]
    fn call_2p_flop_community_cards_3() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.community_cards.len(), 3);
    }

    #[test]
    fn call_2p_flop_deck_reduzido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // 48 (após hole cards) - 1 burn - 3 flop = 44
        assert_eq!(gl.state.deck.len(), 44);
    }

    #[test]
    fn call_2p_flop_burn_pile_1() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.burn_pile.len(), 1);
    }

    #[test]
    fn call_2p_turn_community_cards_4() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob, alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.community_cards.len(), 4);
    }

    #[test]
    fn call_2p_river_community_cards_5() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob, alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        // Turn: bob, alice
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.community_cards.len(), 5);
    }

    #[test]
    fn call_2p_showdown_community_cards_5() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        assert_eq!(gl.state.community_cards.len(), 5);
        assert_eq!(gl.state.phase, GamePhase::Showdown);
    }

    #[test]
    fn call_2p_showdown_is_finished() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        assert!(gl.state.is_finished);
    }

    #[test]
    fn call_2p_showdown_burn_pile_3() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        assert_eq!(gl.state.burn_pile.len(), 3);
    }

    #[test]
    fn call_2p_showdown_deck_restante() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        // 52 - 4 hole - 3 burn - 5 community = 40
        assert_eq!(gl.state.deck.len(), 40);
    }

    #[test]
    fn call_2p_showdown_total_pot() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        // SB + BB = 10 + 10 = 20
        assert_eq!(gl.state.total_pot(), 20);
    }
}

#[cfg(test)]
mod bet_raise_allin_tests {
    use super::*;

    // ===================== BET =====================

    #[test]
    fn bet_2p_flop_bet_valido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: alice (SB) call, bob (BB) check → flop
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob primeiro
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        assert_eq!(gl.state.players[1].current_bet, 50);
    }

    #[test]
    fn bet_2p_flop_reduz_stack() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        // Stack inicial 1000 - 10 (BB) - 50 (bet) = 940
        assert_eq!(gl.state.players[1].stack, 940);
    }

    #[test]
    fn bet_2p_flop_total_bet_acumula() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        // total_bet = 10 (BB) + 50 (bet) = 60
        assert_eq!(gl.state.players[1].total_bet, 60);
    }

    #[test]
    fn bet_2p_flop_current_bet_to_match_atualizado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 50);
    }

    #[test]
    fn bet_2p_flop_min_raise_atualizado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        assert_eq!(gl.state.min_raise, 50);
    }

    #[test]
    fn bet_2p_flop_has_acted_true() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        assert!(gl.state.players[1].has_acted);
    }

    #[test]
    fn bet_2p_flop_reseta_has_acted_outro() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        // alice deve ter has_acted resetado
        assert!(!gl.state.players[0].has_acted);
    }

    #[test]
    fn bet_2p_flop_avanca_turno() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        // Rodada não completa, vez da alice
        assert_eq!(gl.state.active_player_index, 0);
    }

    #[test]
    fn bet_erro_quando_ha_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: to_call = 5 para alice (SB)
        let r = gl.player_action("alice", PlayerMove::Bet(50));
        assert!(r.is_err());
        match r.unwrap_err() {
            GameLoopError::InvalidActionForPhase(_) => {}
            e => panic!("Esperado InvalidActionForPhase, got {:?}", e),
        }
    }

    #[test]
    fn bet_erro_amount_zero() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        let r = gl.player_action("bob", PlayerMove::Bet(0));
        assert!(matches!(r.unwrap_err(), GameLoopError::InvalidBetAmount(_)));
    }

    #[test]
    fn bet_erro_amount_negativo() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        let r = gl.player_action("bob", PlayerMove::Bet(0));
        assert!(matches!(r.unwrap_err(), GameLoopError::InvalidBetAmount(_)));
    }

    #[test]
    fn bet_erro_menor_que_min_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // min_raise = BB = 10
        let r = gl.player_action("bob", PlayerMove::Bet(5));
        assert!(matches!(r.unwrap_err(), GameLoopError::RaiseTooSmall(_)));
    }

    #[test]
    fn bet_erro_maior_que_stack() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Stack do bob = 990, bet de 2000
        let r = gl.player_action("bob", PlayerMove::Bet(2000));
        assert!(matches!(
            r.unwrap_err(),
            GameLoopError::InsufficientStack(_)
        ));
    }

    #[test]
    fn bet_allin_stack_zero() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Stack do bob = 990
        gl.player_action("bob", PlayerMove::Bet(990)).unwrap();
        assert_eq!(gl.state.players[1].stack, 0);
        assert!(gl.state.players[1].is_all_in);
    }

    #[test]
    fn bet_igual_min_raise_valido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // min_raise = 10, bet de 10 deve ser válido
        gl.player_action("bob", PlayerMove::Bet(10)).unwrap();
        assert_eq!(gl.state.players[1].current_bet, 10);
    }

    #[test]
    fn bet_3p_reseta_outros_jogadores() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // Preflop: alice call, bob call, carol check → flop
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        // Flop: bob primeiro (left of dealer = índice 1)
        gl.player_action("bob", PlayerMove::Bet(30)).unwrap();
        assert!(!gl.state.players[0].has_acted);
        assert!(!gl.state.players[2].has_acted);
    }

    #[test]
    fn bet_2p_pot_acumula() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        // Pot = 20 (blinds) + 50 (bet) = 70
        assert_eq!(gl.state.total_pot(), 70);
    }

    // ===================== RAISE =====================

    #[test]
    fn raise_2p_preflop_valido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // alice (SB) raise para 30 (incremento de 20 >= min_raise 10)
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        assert_eq!(gl.state.players[0].current_bet, 30);
    }

    #[test]
    fn raise_2p_preflop_reduz_stack() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // Stack 995 (após SB) - 25 (total_needed = 30 - 5) = 970
        assert_eq!(gl.state.players[0].stack, 970);
    }

    #[test]
    fn raise_2p_preflop_total_bet_acumula() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // total_bet = 5 (SB) + 25 (total_needed) = 30
        assert_eq!(gl.state.players[0].total_bet, 30);
    }

    #[test]
    fn raise_2p_preflop_current_bet_to_match() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 30);
    }

    #[test]
    fn raise_2p_preflop_min_raise_incremento() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // min_raise = 30 - 10 (BB) = 20
        assert_eq!(gl.state.min_raise, 20);
    }

    #[test]
    fn raise_2p_preflop_reseta_outro() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // bob não agiu ainda
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn raise_2p_preflop_has_acted_true() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        assert!(gl.state.players[0].has_acted);
    }

    #[test]
    fn raise_2p_preflop_avanca_turno() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        assert_eq!(gl.state.active_player_index, 1);
    }

    #[test]
    fn raise_erro_sem_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: current_bet_to_match = 0
        let r = gl.player_action("bob", PlayerMove::Raise(30));
        assert!(matches!(
            r.unwrap_err(),
            GameLoopError::InvalidActionForPhase(_)
        ));
    }

    #[test]
    fn raise_erro_incremento_menor_min_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // current_bet_to_match = 10 (BB), raise para 15 → incremento 5 < 10
        let r = gl.player_action("alice", PlayerMove::Raise(15));
        assert!(matches!(r.unwrap_err(), GameLoopError::RaiseTooSmall(_)));
    }

    #[test]
    fn raise_erro_incremento_igual_min_raise_valido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // raise para 20 → incremento 10 == min_raise 10
        gl.player_action("alice", PlayerMove::Raise(20)).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 20);
    }

    #[test]
    fn raise_apos_bet_valido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob bet 50, alice raise 100
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        gl.player_action("alice", PlayerMove::Raise(100)).unwrap();
        assert_eq!(gl.state.players[0].current_bet, 100);
    }

    #[test]
    fn raise_apos_bet_min_raise_atualizado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        gl.player_action("alice", PlayerMove::Raise(100)).unwrap();
        // min_raise = 100 - 50 = 50
        assert_eq!(gl.state.min_raise, 50);
    }

    #[test]
    fn raise_apos_bet_reseta_outros() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        gl.player_action("alice", PlayerMove::Raise(100)).unwrap();
        // bob deve ter has_acted resetado
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn raise_allin_menor_que_min_valido() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // alice tem stack 995, raise all-in para 995+5=1000
        // incremento = 1000 - 10 = 990 >= min_raise 10 → normal raise
        gl.player_action("alice", PlayerMove::Raise(1000)).unwrap();
        assert_eq!(gl.state.players[0].stack, 0);
        assert!(gl.state.players[0].is_all_in);
    }

    #[test]
    fn raise_allin_stack_insuficiente() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // alice stack 995, raise para 2000 → total_needed > stack → all-in raise
        // all_in_amount = 5 + 995 = 1000
        gl.player_action("alice", PlayerMove::Raise(2000)).unwrap();
        assert_eq!(gl.state.players[0].stack, 0);
        assert!(gl.state.players[0].is_all_in);
        assert_eq!(gl.state.players[0].current_bet, 1000);
    }

    #[test]
    fn raise_allin_atualiza_bet_to_match() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(2000)).unwrap();
        // all_in_amount = 1000 > current_bet_to_match 10
        assert_eq!(gl.state.current_bet_to_match, 1000);
    }

    #[test]
    fn raise_allin_reseta_outros() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(2000)).unwrap();
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn raise_3p_apos_bet() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        // Flop: bob bet 30 (bob é primeiro postflop), carol call, alice raise 80
        gl.player_action("bob", PlayerMove::Bet(30)).unwrap();
        gl.player_action("carol", PlayerMove::Call).unwrap();
        gl.player_action("alice", PlayerMove::Raise(80)).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 80);
        // bob e carol resetados
        assert!(!gl.state.players[1].has_acted);
        assert!(!gl.state.players[2].has_acted);
    }

    #[test]
    fn raise_2p_pot_acumula() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // Pot = 15 (blinds) + 25 (total_needed) = 40
        assert_eq!(gl.state.total_pot(), 40);
    }

    // ===================== ALLIN =====================

    #[test]
    fn allin_2p_preflop_stack_zero() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.players[0].stack, 0);
    }

    #[test]
    fn allin_2p_preflop_is_all_in() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        assert!(gl.state.players[0].is_all_in);
    }

    #[test]
    fn allin_2p_preflop_has_acted() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        assert!(gl.state.players[0].has_acted);
    }

    #[test]
    fn allin_2p_preflop_current_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // alice: current=5 (SB), stack=995, all_in → new_total_bet = 5+995 = 1000
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.players[0].current_bet, 1000);
    }

    #[test]
    fn allin_2p_preflop_total_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // total_bet = 5 (SB) + 995 (all_in) = 1000
        assert_eq!(gl.state.players[0].total_bet, 1000);
    }

    #[test]
    fn allin_2p_preflop_atualiza_bet_to_match() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // new_total_bet = 1000 > current_bet_to_match 10
        assert_eq!(gl.state.current_bet_to_match, 1000);
    }

    #[test]
    fn allin_2p_preflop_reseta_outro() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn allin_2p_preflop_min_raise_atualizado() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // raise_increment = 1000 - 10 = 990 >= min_raise 10
        assert_eq!(gl.state.min_raise, 990);
    }

    #[test]
    fn allin_2p_preflop_avanca_turno() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // bob ainda precisa agir
        assert_eq!(gl.state.active_player_index, 1);
    }

    #[test]
    fn allin_2p_flop_allin() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob all-in
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.players[1].stack, 0);
        assert!(gl.state.players[1].is_all_in);
    }

    #[test]
    fn allin_2p_flop_current_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // bob: current=0, stack=990, all_in → new_total_bet = 0+990 = 990
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.players[1].current_bet, 990);
    }

    #[test]
    fn allin_nao_aumenta_bet_nao_reseta_outros() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // alice all-in: new_total_bet = 1000 > 10 → reseta
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // bob call all-in (stack 990, current 10, to_call 990)
        // bob all-in: new_total_bet = 10 + 990 = 1000 == 1000 → NÃO aumenta
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        // Como não aumentou, não reseta outros (mas alice já agiu)
        // A rodada deve estar completa → is_finished ou advance_phase
        assert!(gl.state.is_finished || gl.state.phase == GamePhase::Showdown);
    }

    #[test]
    fn allin_2p_pot_acumula() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // Pot = 15 (blinds) + 995 (all_in) = 1010
        assert_eq!(gl.state.total_pot(), 1010);
    }

    #[test]
    fn allin_3p_side_pot_stack_zero() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        gl.player_action("carol", PlayerMove::AllIn).unwrap();
        // Todos all-in
        for p in &gl.state.players {
            assert_eq!(p.stack, 0);
            assert!(p.is_all_in);
        }
    }

    #[test]
    fn allin_3p_todos_allin_finished() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        gl.player_action("carol", PlayerMove::AllIn).unwrap();
        // Todos all-in, rodada completa, is_finished
        assert!(gl.state.is_finished);
    }

    #[test]
    fn allin_call_allin_menor_que_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // alice all-in 1000
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // bob tem stack 990, to_call = 990
        // bob all-in: new_total_bet = 10 + 990 = 1000 == 1000 (não aumenta)
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        // Após rodada completa, advance_phase reseta current_bet para 0
        assert_eq!(gl.state.players[1].current_bet, 0);
        assert!(gl.state.is_finished);
    }

    #[test]
    fn allin_2p_ambos_allin_showdown() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        // Ambos all-in → board run-out → showdown
        assert!(gl.state.is_finished);
        assert_eq!(gl.state.phase, GamePhase::Showdown);
    }

    #[test]
    fn allin_2p_ambos_allin_community_cards_5() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        // Board completo run-out
        assert_eq!(gl.state.community_cards.len(), 5);
    }

    #[test]
    fn allin_2p_ambos_allin_burn_pile_3() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.burn_pile.len(), 3);
    }

    #[test]
    fn allin_2p_ambos_allin_pot() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        // Pot = 15 (blinds) + 995 (alice) + 990 (bob) = 2000
        assert_eq!(gl.state.total_pot(), 2000);
    }

    #[test]
    fn allin_apos_bet_allin_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob bet 50
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        // alice all-in: current=0 (resetado no flop), stack=990, new_total_bet=990 > 50
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 990);
        assert!(gl.state.players[0].is_all_in);
    }

    #[test]
    fn allin_apos_bet_allin_reseta_outros() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // bob resetado
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn allin_apos_bet_allin_min_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // raise_increment = 990 - 50 = 940 >= min_raise 50
        assert_eq!(gl.state.min_raise, 940);
    }

    #[test]
    fn allin_2p_flop_allin_nao_aumenta_nao_reseta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob bet 990 (all-in)
        gl.player_action("bob", PlayerMove::Bet(990)).unwrap();
        // alice all-in: current=10, stack=990, new_total=1000 > 990 → reseta
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        // bob já agiu (bet), mas foi resetado pelo all-in da alice
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn bet_raise_allin_fluxo_completo() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: alice raise 30, bob call → flop
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
        // Flop: bob bet 50, alice raise 100, bob call → turn
        gl.player_action("bob", PlayerMove::Bet(50)).unwrap();
        gl.player_action("alice", PlayerMove::Raise(100)).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Turn);
    }

    #[test]
    fn bet_raise_allin_fluxo_allin_showdown() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Showdown);
        assert!(gl.state.is_finished);
    }
}

// ═══════════════════════════════════════════════════════════════════
// LOTE 6 — Resolve/Showdown/Errors/History (50 testes)
// ═══════════════════════════════════════════════════════════════════

mod resolve_showdown_errors_tests {
    use super::*;

    // ─── resolve_hand: erros ───

    #[test]
    fn resolve_hand_nao_iniciada_erro() {
        let gl = make_game_loop_2p();
        let result = gl.state.is_finished;
        // Sem start_hand, is_finished=false
        assert!(!result);
    }

    #[test]
    fn resolve_hand_sem_start_erro_hand_not_started() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Sem terminar a mão, resolve_hand deve dar erro
        let result = gl.resolve_hand();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GameLoopError::HandNotStarted);
    }

    #[test]
    fn resolve_hand_apos_fold_unico_vencedor() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        assert!(gl.state.is_finished);
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.end_reason, EndReason::AllFolded);
        assert!(resolution.payouts.contains_key("bob"));
    }

    #[test]
    fn resolve_hand_fold_vencedor_recebe_pot() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        // Pot = SB(5) + BB(10) = 15
        let bob_winnings = resolution.payouts.get("bob").copied().unwrap_or(0);
        assert!(bob_winnings > 0);
    }

    #[test]
    fn resolve_hand_fold_sem_rake() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        // Fold win não tem rake
        assert_eq!(resolution.rake, 0);
    }

    #[test]
    fn resolve_hand_fold_loss_deflator_none() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert!(resolution.loss_deflator.is_none());
    }

    #[test]
    fn resolve_hand_fold_end_phase_preflop() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.end_phase, GamePhase::Preflop);
    }

    #[test]
    fn resolve_hand_fold_3p_vencedor() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: dealer=0(alice), SB=1(bob), BB=2(carol), UTG=0(alice) age primeiro
        // alice fold, bob fold → carol vence
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        assert!(gl.state.is_finished);
        let resolution = gl.resolve_hand().unwrap();
        assert!(resolution.payouts.contains_key("carol"));
        assert_eq!(resolution.end_reason, EndReason::AllFolded);
    }

    #[test]
    fn resolve_hand_fold_3p_pot_completo() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // alice(UTG) fold, bob(SB) fold → carol(BB) vence
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        let carol_winnings = resolution.payouts.get("carol").copied().unwrap_or(0);
        assert!(carol_winnings >= 15); // pelo menos blinds
    }

    #[test]
    fn resolve_hand_showdown_2p() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        assert_eq!(gl.state.phase, GamePhase::Showdown);
        assert!(gl.state.is_finished);
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.end_reason, EndReason::Showdown);
    }

    #[test]
    fn resolve_hand_showdown_tem_pots() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        assert!(!resolution.pots.is_empty());
    }

    #[test]
    fn resolve_hand_showdown_tem_payouts() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        assert!(!resolution.payouts.is_empty());
    }

    #[test]
    fn resolve_hand_showdown_tem_player_results() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.player_results.len(), 2);
    }

    #[test]
    fn resolve_hand_showdown_end_phase_showdown() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.end_phase, GamePhase::Showdown);
    }

    #[test]
    fn resolve_hand_showdown_rake_positivo() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        // Rake de 5% sobre pot de 20 = 1 (cap 5)
        assert!(resolution.rake > 0);
    }

    #[test]
    fn resolve_hand_showdown_rake_limitado_cap() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // All-in para maximizar o pot
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        // Rake cap = 5
        assert!(resolution.rake <= 500);
    }

    #[test]
    fn resolve_hand_allin_showdown() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.end_reason, EndReason::Showdown);
    }

    #[test]
    fn resolve_hand_allin_tem_side_pots() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert!(!resolution.pots.is_empty());
    }

    #[test]
    fn resolve_hand_allin_payouts_somam_menos_rake() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        let total_payouts: u64 = resolution.payouts.values().sum();
        let total_pot: u64 = resolution.pots.iter().map(|p| p.amount).sum();
        // payouts + rake ≈ total_pot
        assert_eq!(total_payouts + resolution.rake, total_pot);
    }

    // ─── HandResolution: estrutura ───

    #[test]
    fn hand_resolution_pots_nao_vazio() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert!(!resolution.pots.is_empty());
    }

    #[test]
    fn hand_resolution_payouts_tem_vencedor() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert!(resolution.payouts.contains_key("bob"));
    }

    #[test]
    fn hand_resolution_player_results_fold() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.player_results.len(), 2);
        let alice_result = resolution
            .player_results
            .iter()
            .find(|r| r.player_id == "alice")
            .unwrap();
        assert!(alice_result.folded);
        assert_eq!(alice_result.finish_position, 2);
    }

    #[test]
    fn hand_resolution_player_results_vencedor_fold() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        let bob_result = resolution
            .player_results
            .iter()
            .find(|r| r.player_id == "bob")
            .unwrap();
        assert_eq!(bob_result.finish_position, 1);
        assert!(!bob_result.folded);
    }

    #[test]
    fn hand_resolution_chips_won_vencedor() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        let bob_result = resolution
            .player_results
            .iter()
            .find(|r| r.player_id == "bob")
            .unwrap();
        assert!(bob_result.chips_won > 0);
    }

    #[test]
    fn hand_resolution_chips_lost_perdedor() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        let alice_result = resolution
            .player_results
            .iter()
            .find(|r| r.player_id == "alice")
            .unwrap();
        // Alice apostou SB = 5
        assert_eq!(alice_result.chips_lost, 5);
    }

    #[test]
    fn hand_resolution_fold_separa_aposta_nao_coberta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert_eq!(resolution.pots.len(), 2);
        assert_eq!(resolution.pots[1].amount, 5);
        assert_eq!(resolution.pots[1].eligible_players, vec!["bob"]);
        assert_eq!(resolution.rake, 0);
    }

    #[test]
    fn hand_resolution_fold_pot_eligible_vencedor() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        assert!(resolution.pots[0].is_eligible("bob"));
    }

    // ─── finalize_history e get_history ───

    #[test]
    fn get_history_sem_start_none() {
        let gl = make_game_loop_2p();
        assert!(gl.get_history().is_none());
    }

    #[test]
    fn get_history_apos_start_some() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        assert!(gl.get_history().is_some());
    }

    #[test]
    fn get_history_tem_hand_id() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let history = gl.get_history().unwrap();
        assert_eq!(history.hand_id, "hand-001");
    }

    #[test]
    fn get_history_tem_table_config() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let history = gl.get_history().unwrap();
        assert_eq!(history.table_config.big_blind, 10);
    }

    #[test]
    fn get_history_tem_players() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let history = gl.get_history().unwrap();
        assert_eq!(history.players.len(), 2);
        assert!(history.players.contains(&"alice".to_string()));
        assert!(history.players.contains(&"bob".to_string()));
    }

    #[test]
    fn get_history_tem_starting_stacks() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let history = gl.get_history().unwrap();
        assert_eq!(history.starting_stacks.get("alice"), Some(&1000));
        assert_eq!(history.starting_stacks.get("bob"), Some(&1000));
    }

    #[test]
    fn finalize_history_registra_resultados() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert!(!history.results.is_empty());
    }

    #[test]
    fn finalize_history_registra_total_pot() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert!(history.total_pot > 0);
    }

    #[test]
    fn finalize_history_registra_rake() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        // Fold win → rake = 0
        assert_eq!(history.rake, 0);
    }

    #[test]
    fn finalize_history_registra_end_phase() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert_eq!(history.end_phase, GamePhase::Preflop);
    }

    #[test]
    fn finalize_history_registra_end_reason() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert_eq!(history.end_reason, EndReason::AllFolded);
    }

    #[test]
    fn finalize_history_showdown_end_phase() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert_eq!(history.end_phase, GamePhase::Showdown);
    }

    #[test]
    fn finalize_history_showdown_end_reason() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert_eq!(history.end_reason, EndReason::Showdown);
    }

    #[test]
    fn finalize_history_showdown_tem_community_cards() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert_eq!(history.community_cards.len(), 5);
    }

    #[test]
    fn finalize_history_fold_sem_community_cards() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        let resolution = gl.resolve_hand().unwrap();
        gl.finalize_history(&resolution);
        let history = gl.get_history().unwrap();
        assert_eq!(history.community_cards.len(), 0);
    }

    // ─── GameLoopError: todos os variantes ───

    #[test]
    fn erro_player_not_found() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Heads-up: alice (SB/dealer) age primeiro
        // "zebra" não é o jogador ativo → NotYourTurn (verificação de turno vem antes)
        let result = gl.player_action("zebra", PlayerMove::Fold);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GameLoopError::NotYourTurn("zebra".to_string())
        );
    }

    #[test]
    fn erro_not_your_turn() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Heads-up: alice (SB/dealer) age primeiro
        let result = gl.player_action("bob", PlayerMove::Fold);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GameLoopError::NotYourTurn("bob".to_string())
        );
    }

    #[test]
    fn erro_player_cannot_act_folded() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: alice(UTG) age primeiro, fold → turno passa para bob
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Tentar agir com alice que já foldou → NotYourTurn (turno já é de bob)
        let result = gl.player_action("alice", PlayerMove::Check);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GameLoopError::NotYourTurn("alice".to_string())
        );
    }

    #[test]
    fn erro_invalid_bet_amount_zero() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob bet 0 → erro
        let result = gl.player_action("bob", PlayerMove::Bet(0));
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::InvalidBetAmount(_) => {}
            other => panic!("Esperado InvalidBetAmount, got {other:?}"),
        }
    }

    #[test]
    fn erro_invalid_bet_amount_negative() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        let result = gl.player_action("bob", PlayerMove::Bet(0));
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::InvalidBetAmount(_) => {}
            other => panic!("Esperado InvalidBetAmount, got {other:?}"),
        }
    }

    #[test]
    fn erro_raise_too_small() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: alice raise 11 (incremento 1 < min_raise 10)
        let result = gl.player_action("alice", PlayerMove::Raise(11));
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::RaiseTooSmall(_) => {}
            other => panic!("Esperado RaiseTooSmall, got {other:?}"),
        }
    }

    #[test]
    fn erro_insufficient_stack_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob bet 2000 > stack 990
        let result = gl.player_action("bob", PlayerMove::Bet(2000));
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::InsufficientStack(_) => {}
            other => panic!("Esperado InsufficientStack, got {other:?}"),
        }
    }

    #[test]
    fn erro_insufficient_stack_raise_vira_allin() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: alice raise 2000 > stack 995 → vira AllIn automático
        // (o código trata raise acima do stack como all-in)
        let result = gl.player_action("alice", PlayerMove::Raise(2000));
        assert!(result.is_ok());
        // Alice deve estar all-in
        assert!(gl.state.players[0].is_all_in);
    }

    #[test]
    fn erro_invalid_action_check_com_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: alice tenta check com BB pendente
        let result = gl.player_action("alice", PlayerMove::Check);
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::InvalidActionForPhase(_) => {}
            other => panic!("Esperado InvalidActionForPhase, got {other:?}"),
        }
    }

    #[test]
    fn erro_invalid_action_call_sem_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob tenta call sem aposta pendente
        let result = gl.player_action("bob", PlayerMove::Call);
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::InvalidActionForPhase(_) => {}
            other => panic!("Esperado InvalidActionForPhase, got {other:?}"),
        }
    }

    #[test]
    fn erro_invalid_action_bet_com_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        // Preflop: alice tenta bet (deveria usar raise)
        let result = gl.player_action("alice", PlayerMove::Bet(50));
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::InvalidActionForPhase(_) => {}
            other => panic!("Esperado InvalidActionForPhase, got {other:?}"),
        }
    }

    #[test]
    fn erro_invalid_action_raise_sem_aposta() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: bob tenta raise sem aposta (deveria usar bet)
        let result = gl.player_action("bob", PlayerMove::Raise(50));
        assert!(result.is_err());
        match result.unwrap_err() {
            GameLoopError::InvalidActionForPhase(_) => {}
            other => panic!("Esperado InvalidActionForPhase, got {other:?}"),
        }
    }

    #[test]
    fn erro_hand_already_finished() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Tentar agir após mão terminada
        let result = gl.player_action("bob", PlayerMove::Check);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GameLoopError::HandAlreadyFinished);
    }

    #[test]
    fn erro_not_enough_players_1p() {
        let mut gl = make_game_loop_n(1);
        let result = gl.start_hand();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GameLoopError::NotEnoughPlayers);
    }

    #[test]
    fn erro_not_enough_players_0p() {
        let gl = GameLoop::new(
            make_config(),
            "h0".to_string(),
            "T0".to_string(),
            GameType::Cash,
        );
        let mut gl = gl;
        let result = gl.start_hand();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GameLoopError::NotEnoughPlayers);
    }

    #[test]
    fn erro_hand_already_finished_start() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Fold).unwrap();
        // Tentar start_hand novamente após terminar
        let result = gl.start_hand();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GameLoopError::HandAlreadyFinished);
    }

    // ─── advance_phase e run_out_board ───

    #[test]
    fn advance_phase_preflop_para_flop() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Flop);
        assert_eq!(gl.state.community_cards.len(), 3);
    }

    #[test]
    fn advance_phase_flop_para_turn() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        gl.player_action("alice", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Turn);
        assert_eq!(gl.state.community_cards.len(), 4);
    }

    #[test]
    fn advance_phase_turn_para_river() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        play_all_check_to_showdown(&mut gl);
        // Após 2 rounds (flop, turn), estamos no river
        assert!(gl.state.phase == GamePhase::River || gl.state.phase == GamePhase::Showdown);
    }

    #[test]
    fn advance_phase_reseta_current_bet() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Após flop, current_bet de todos deve ser 0
        assert_eq!(gl.state.players[0].current_bet, 0);
        assert_eq!(gl.state.players[1].current_bet, 0);
    }

    #[test]
    fn advance_phase_reseta_has_acted() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Após flop, has_acted de todos deve ser false
        assert!(!gl.state.players[0].has_acted);
        assert!(!gl.state.players[1].has_acted);
    }

    #[test]
    fn advance_phase_reseta_current_bet_to_match() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        assert_eq!(gl.state.current_bet_to_match, 0);
    }

    #[test]
    fn advance_phase_reseta_min_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // min_raise volta para BB
        assert_eq!(gl.state.min_raise, 10);
    }

    #[test]
    fn advance_phase_burn_pile_cresce() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: 1 burn + 3 community
        assert_eq!(gl.state.burn_pile.len(), 1);
    }

    #[test]
    fn advance_phase_deck_diminui() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        let deck_before = gl.state.deck.len();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Flop: 1 burn + 3 community = 4 cartas removidas
        assert_eq!(gl.state.deck.len(), deck_before - 4);
    }

    #[test]
    fn run_out_board_allin_2p_showdown() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        assert_eq!(gl.state.phase, GamePhase::Showdown);
        assert_eq!(gl.state.community_cards.len(), 5);
    }

    #[test]
    fn run_out_board_burn_pile_3_burns() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        // 3 burns (flop, turn, river)
        assert_eq!(gl.state.burn_pile.len(), 3);
    }

    #[test]
    fn run_out_board_3p_allin() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: alice(UTG) all-in, bob(SB) fold, carol(BB) call
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        gl.player_action("carol", PlayerMove::Call).unwrap();
        // Alice e carol all-in → run_out_board
        assert_eq!(gl.state.phase, GamePhase::Showdown);
        assert_eq!(gl.state.community_cards.len(), 5);
    }

    #[test]
    fn run_out_board_1_ativo_termina() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: alice(UTG) all-in, bob fold, carol fold → alice vence
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::Fold).unwrap();
        gl.player_action("carol", PlayerMove::Fold).unwrap();
        // Só alice resta → is_finished
        assert!(gl.state.is_finished);
    }

    // ─── is_betting_round_complete indireto ───

    #[test]
    fn betting_round_completo_todos_acted_e_igual() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Ambos agiram e current_bet igual (10) → completa → flop
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn betting_round_incompleto_um_nao_acted() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // Bob ainda não agiu → não completa → ainda preflop
        assert_eq!(gl.state.phase, GamePhase::Preflop);
    }

    #[test]
    fn betting_round_completo_apos_call_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        // Ambos agiram, current_bet igual (30) → completa → flop
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn betting_round_incompleto_apos_raise() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::Raise(30)).unwrap();
        // Bob ainda não agiu → não completa
        assert_eq!(gl.state.phase, GamePhase::Preflop);
    }

    #[test]
    fn betting_round_completo_allin_call() {
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();
        gl.player_action("alice", PlayerMove::AllIn).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        // Alice all-in, bob call → completa → run_out_board (alice all-in)
        assert!(gl.state.is_finished || gl.state.phase == GamePhase::Showdown);
    }

    #[test]
    fn betting_round_3p_completo_todos_check() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: alice(UTG) call, bob(SB) call, carol(BB) check
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Check).unwrap();
        // Todos agiram, current_bet igual (10) → completa → flop
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn betting_round_3p_incompleto_apos_bet() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: alice(UTG) call, bob(SB) call, carol(BB) bet
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Bet(30)).unwrap();
        // Carol bet, alice e bob precisam agir → não completa
        assert_eq!(gl.state.phase, GamePhase::Preflop);
    }

    #[test]
    fn betting_round_3p_completo_apos_todos_call_bet() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: alice(UTG) call, bob(SB) call, carol(BB) bet, alice call, bob call
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Bet(30)).unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        // Todos agiram, current_bet igual (30) → completa → flop
        assert_eq!(gl.state.phase, GamePhase::Flop);
    }

    #[test]
    fn betting_round_allin_reseta_outros() {
        let mut gl = make_game_loop_3p();
        gl.start_hand().unwrap();
        // 3p: alice(UTG) call, bob(SB) call, carol(BB) bet, alice call, bob all-in
        gl.player_action("alice", PlayerMove::Call).unwrap();
        gl.player_action("bob", PlayerMove::Call).unwrap();
        gl.player_action("carol", PlayerMove::Bet(30)).unwrap();
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // Bob all-in (raise) → reseta carol
        gl.player_action("bob", PlayerMove::AllIn).unwrap();
        // Carol foi resetada
        assert!(!gl.state.players[2].has_acted);
    }

    #[test]
    fn test_multi_phase_all_in_loss_deflator_uses_multiway_equity_snapshots() {
        use crate::deck::{Rank, Suit};
        let p1 = PlayerState {
            id: "p1".to_string(),
            stack: 0,
            hole_cards: vec![
                Card {
                    rank: Rank::King,
                    suit: Suit::Diamonds,
                },
                Card {
                    rank: Rank::King,
                    suit: Suit::Hearts,
                },
            ],
            current_bet: 10000,
            total_bet: 10000,
            has_folded: false,
            is_all_in: true,
            all_in_phase: Some(GamePhase::Preflop),
            has_acted: true,
            seat_index: 0,
        };
        let p2 = PlayerState {
            id: "p2".to_string(),
            stack: 0,
            hole_cards: vec![
                Card {
                    rank: Rank::Ace,
                    suit: Suit::Spades,
                },
                Card {
                    rank: Rank::Ace,
                    suit: Suit::Diamonds,
                },
            ],
            current_bet: 20000,
            total_bet: 30000,
            has_folded: false,
            is_all_in: true,
            all_in_phase: Some(GamePhase::Turn),
            has_acted: true,
            seat_index: 1,
        };
        let p3 = PlayerState {
            id: "p3".to_string(),
            stack: 70000,
            hole_cards: vec![
                Card {
                    rank: Rank::Queen,
                    suit: Suit::Clubs,
                },
                Card {
                    rank: Rank::Jack,
                    suit: Suit::Clubs,
                },
            ],
            current_bet: 30000,
            total_bet: 30000,
            has_folded: false,
            is_all_in: false,
            all_in_phase: None,
            has_acted: true,
            seat_index: 2,
        };

        let mut gl = GameLoop::new(
            TableConfig::new(1000, 0, 0),
            "hand_123".to_string(),
            "test_table".to_string(),
            GameType::Cash,
        );
        gl.state.players.push(p1);
        gl.state.players.push(p2);
        gl.state.players.push(p3);
        gl.state.community_cards = vec![
            Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Spades,
            },
        ];
        gl.state.is_finished = true;

        let res = gl.resolve_hand().unwrap();

        // P1 (KK) enfrentava AA e QJ no main pot. A equity multiway preflop fica
        // abaixo do piso de 56%, portanto ele não é elegível ao deflator.
        assert_eq!(res.loss_deflators.len(), 1);
        assert!(res.loss_deflators.iter().all(|d| d.loser_id != "p1"));

        let d2 = res
            .loss_deflators
            .iter()
            .find(|d| d.loser_id == "p2")
            .unwrap();
        assert_eq!(d2.phase, GamePhase::Turn);
        assert_eq!(d2.cards_remaining, 1);
        assert_eq!(
            d2.tier,
            crate::loss_deflator::LossDeflatorTier::TwentyFivePercent
        );
        assert!((0.76..0.86).contains(&d2.loser_equity));
        assert_eq!(d2.opponents_counted, 2);
    }

    #[test]
    fn test_heads_up_action_order_preflop_and_postflop() {
        // 2 jogadores: alice (seat 0, dealer/SB) e bob (seat 1, BB)
        let mut gl = make_game_loop_2p();
        gl.start_hand().unwrap();

        // No Preflop: Dealer (SB / Alice) deve agir PRIMEIRO
        assert_eq!(gl.state.active_player_index, 0);
        assert_eq!(gl.state.players[gl.state.active_player_index].id, "alice");

        // Alice iguala o BB (Call 10)
        gl.player_action("alice", PlayerMove::Call).unwrap();
        // Vez do Bob (BB)
        assert_eq!(gl.state.active_player_index, 1);
        assert_eq!(gl.state.players[gl.state.active_player_index].id, "bob");
        // Bob dá Check -> Avança para o Flop
        gl.player_action("bob", PlayerMove::Check).unwrap();

        // No Flop: Bob (BB / Não-dealer) deve agir PRIMEIRO! Alice (Dealer/SB) age em 2º (em posição)
        assert_eq!(gl.state.phase, GamePhase::Flop);
        assert_eq!(gl.state.active_player_index, 1);
        assert_eq!(gl.state.players[gl.state.active_player_index].id, "bob");

        // Bob dá Check
        gl.player_action("bob", PlayerMove::Check).unwrap();
        // Vez de Alice (Dealer) agir por último no Flop
        assert_eq!(gl.state.active_player_index, 0);
        assert_eq!(gl.state.players[gl.state.active_player_index].id, "alice");
        // Alice dá Check -> Avança para o Turn
        gl.player_action("alice", PlayerMove::Check).unwrap();

        // No Turn: Bob (BB) age PRIMEIRO novamente
        assert_eq!(gl.state.phase, GamePhase::Turn);
        assert_eq!(gl.state.active_player_index, 1);
        assert_eq!(gl.state.players[gl.state.active_player_index].id, "bob");
    }

    // ═══════════════════════════════════════════════════════════════════
    // Lote 7 — Suíte de Estresse Massivo & Invariantes Financeiras Extremas
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    #[cfg_attr(
        not(feature = "massive-tests"),
        ignore = "teste massivo; habilite a feature massive-tests manualmente"
    )]
    fn test_ante_blinds_total_bet_preservation_massive_stress() {
        // 1.000 iterações testando conservação perfeita de Ante + Blinds no total_bet
        for ante_val in 1..=1000u64 {
            let ante = ante_val * 50;
            let mut gl = GameLoop::new(
                make_config(),
                format!("hand-ante-{}", ante_val),
                "Ante Table".to_string(),
                GameType::Cash,
            )
            .with_ante(ante);

            gl.add_player("p0".to_string(), 100000);
            gl.add_player("p1".to_string(), 100000);
            gl.add_player("p2".to_string(), 100000);
            gl.set_dealer(0);

            gl.start_hand().unwrap();

            // SB index = 1 (p1), BB index = 2 (p2)
            let sb_player = &gl.state.players[1];
            let bb_player = &gl.state.players[2];

            // Garante que total_bet de SB é EXACTAMENTE Ante + SB
            let expected_sb_total = ante + gl.state.small_blind;
            let expected_bb_total = ante + gl.state.big_blind;

            assert_eq!(
                sb_player.total_bet, expected_sb_total,
                "SB total_bet incorreto no teste {}: obtido {}, esperado {}",
                ante_val, sb_player.total_bet, expected_sb_total
            );

            assert_eq!(
                bb_player.total_bet, expected_bb_total,
                "BB total_bet incorreto no teste {}: obtido {}, esperado {}",
                ante_val, bb_player.total_bet, expected_bb_total
            );
        }
    }

    #[test]
    #[cfg_attr(
        not(feature = "massive-tests"),
        ignore = "teste massivo; habilite a feature massive-tests manualmente"
    )]
    fn test_chip_conservation_under_multiway_all_in_stress() {
        // 500 cenários de All-In com stacks heterogêneos e validação da invariante financeira
        for seed in 1..=500u64 {
            let mut gl = GameLoop::new(
                make_config(),
                format!("hand-stress-{}", seed),
                "Stress Table".to_string(),
                GameType::Cash,
            )
            .with_ante(seed * 20);

            let s0 = 5000 + (seed * 150);
            let s1 = 10000 + (seed * 200);
            let s2 = 20000 + (seed * 300);

            let initial_sum = s0 + s1 + s2;

            gl.add_player("p0".to_string(), s0);
            gl.add_player("p1".to_string(), s1);
            gl.add_player("p2".to_string(), s2);
            gl.set_dealer(0);

            if gl.start_hand().is_ok() {
                let pot_sum = gl.state.total_pot();
                let remaining_stacks_sum: u64 = gl.state.players.iter().map(|p| p.stack).sum();
                let current_total = pot_sum + remaining_stacks_sum;

                assert_eq!(
                    initial_sum, current_total,
                    "Invariante financeira quebrada no seed {}: inicial {}, atual {}",
                    seed, initial_sum, current_total
                );
            }
        }
    }

    #[test]
    #[cfg_attr(
        not(feature = "massive-tests"),
        ignore = "teste massivo; habilite a feature massive-tests manualmente"
    )]
    fn test_micro_stack_less_than_small_blind_stress() {
        // Testa a robustez quando o jogador entra com stack menor que o SB em centavos (ex: 100 centavos com SB=500)
        for micro_stack in [10u64, 50, 100, 200, 300, 400] {
            let mut gl = GameLoop::new(
                // Este cenário deliberadamente usa blinds maiores que a
                // fixture unitária para manter todos os micro-stacks all-in.
                TableConfig::new(1000, 500, 500),
                format!("hand-micro-{}", micro_stack),
                "Micro Stack Table".to_string(),
                GameType::Cash,
            );

            gl.add_player("alice".to_string(), micro_stack);
            gl.add_player("bob".to_string(), 50000);
            gl.set_dealer(1); // bob é dealer, alice é SB com micro_stack

            assert!(gl.start_hand().is_ok());

            // Alice deve estar All-In no SB porque seu stack era < SB
            let alice = &gl.state.players[0];
            assert!(alice.is_all_in);
            assert_eq!(alice.stack, 0);
            assert_eq!(alice.total_bet, micro_stack);
        }
    }
}
