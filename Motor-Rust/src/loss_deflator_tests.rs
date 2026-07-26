// src/loss_deflator_tests.rs
// Testes edge-case para o módulo loss_deflator
// Cobre: cashback progressivo, rateio proporcional, boundary conditions,
// cenários multi-jogador, pots vazios, valores zero, arredondamento.

use crate::deck::{Card, Rank, Suit};
use crate::loss_deflator::{
    calculate_progressive_loss_deflator, get_heads_up_win_probability, LossDeflatorTier,
    ProgressiveLossDeflatorParams,
};
use crate::types::{GamePhase, Pot};

// ─── Helpers ───

fn card(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

fn pot(amount: u64, eligible: Vec<&str>) -> Pot {
    Pot {
        amount,
        eligible_players: eligible.into_iter().map(|s| s.into()).collect(),
    }
}

fn params(
    pots: Vec<Pot>,
    loser: &str,
    winner: &str,
    phase: GamePhase,
) -> ProgressiveLossDeflatorParams {
    ProgressiveLossDeflatorParams {
        pots,
        loser_id: loser.into(),
        winner_id: winner.into(),
        phase,
    }
}

// ─── Cashback Progressivo ───

#[test]
fn test_preflop_cashback_15_percent() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(100000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Preflop,
    ))
    .unwrap();
    assert_eq!(r.cashback, 15000); // 15% de 1000
    assert_eq!(r.tier, LossDeflatorTier::FifteenPercent);
    assert_eq!(r.cards_remaining, 5);
}

#[test]
fn test_flop_cashback_25_percent() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(100000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Flop,
    ))
    .unwrap();
    assert_eq!(r.cashback, 25000);
    assert_eq!(r.tier, LossDeflatorTier::TwentyFivePercent);
    assert_eq!(r.cards_remaining, 2);
}

#[test]
fn test_turn_cashback_35_percent() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(100000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Turn,
    ))
    .unwrap();
    assert_eq!(r.cashback, 35000);
    assert_eq!(r.tier, LossDeflatorTier::ThirtyFivePercent);
    assert_eq!(r.cards_remaining, 1);
}

#[test]
fn test_river_returns_none() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(100000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::River,
    ));
    assert!(
        r.is_none(),
        "River não deve gerar cashback (showdown direto)"
    );
}

// ─── Elegibilidade de Pots ───

#[test]
fn test_loser_not_in_any_pot_returns_none() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(50000, vec!["x", "y"]), pot(30000, vec!["x"])],
        "z",
        "x",
        GamePhase::Flop,
    ));
    assert!(r.is_none());
}

#[test]
fn test_loser_only_in_side_pot() {
    // main pot: x + y (200), side pot: loser + x (100)
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(20000, vec!["x", "y"]), pot(10000, vec!["loser", "x"])],
        "loser",
        "x",
        GamePhase::Flop,
    ))
    .unwrap();
    // 25% de 100 = 25
    assert_eq!(r.cashback, 2500);
    assert_eq!(r.eligible_pot_total, 10000);
    assert_eq!(r.eligible_pot_ids, vec![1]);
}

#[test]
fn test_loser_in_all_pots() {
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(20000, vec!["loser", "x", "y"]),
            pot(10000, vec!["loser", "x"]),
            pot(5000, vec!["loser", "x", "y", "z"]),
        ],
        "loser",
        "x",
        GamePhase::Turn,
    ))
    .unwrap();
    // 35% de 350 = 122.5 → em f64, 350.0 * 0.35 = 122.4999... truncado para 122.49
    assert_eq!(r.eligible_pot_total, 35000);
    assert_eq!(r.eligible_pot_ids.len(), 3);
    // 350.0 * 0.35 = 122.4999..., truncar_2_casas = 122.49
    assert!(
        r.cashback == 12249,
        "cashback={}, expected ~122.49",
        r.cashback
    );
}

// ─── Rateio Proporcional ───

#[test]
fn test_proportional_distribution_sums_to_total() {
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(30000, vec!["a", "b"]),
            pot(20000, vec!["a", "b"]),
            pot(10000, vec!["a", "b"]),
        ],
        "a",
        "b",
        GamePhase::Flop,
    ))
    .unwrap();
    // 25% de 600 = 150
    let sum: u64 = r.per_pot_cashback.iter().map(|e| e.amount).sum();
    assert_eq!(sum, r.cashback);
    assert_eq!(r.cashback, 15000);
}

#[test]
fn test_proportional_distribution_respects_pot_sizes() {
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(60000, vec!["a", "b"]), // 60%
            pot(40000, vec!["a", "b"]), // 40%
        ],
        "a",
        "b",
        GamePhase::Preflop,
    ))
    .unwrap();
    // 15% de 1000 = 150
    // main: 150 * 600/1000 = 90, side: 150 * 400/1000 = 60
    assert!(r.per_pot_cashback[0].amount == 9000);
    assert!(r.per_pot_cashback[1].amount == 6000);
    assert_eq!(
        r.per_pot_cashback[0].amount + r.per_pot_cashback[1].amount,
        15000
    );
}

#[test]
fn test_last_pot_absorbs_rounding_error() {
    // Cenário com divisão que gera erro de arredondamento
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(33300, vec!["a", "b"]),
            pot(33300, vec!["a", "b"]),
            pot(33400, vec!["a", "b"]),
        ],
        "a",
        "b",
        GamePhase::Flop,
    ))
    .unwrap();
    // 25% de 1000 = 250
    let sum: u64 = r.per_pot_cashback.iter().map(|e| e.amount).sum();
    assert!(
        sum == r.cashback,
        "Soma do rateio deve igualar cashback total"
    );
}

// ─── Boundary Conditions ───

#[test]
fn test_zero_amount_pot_ignored() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(0, vec!["a", "b"]), pot(10000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Flop,
    ))
    .unwrap();
    // Pot 0 não contribui, elegível_total = 100
    assert_eq!(r.eligible_pot_total, 10000);
    assert_eq!(r.cashback, 2500);
}

#[test]
fn test_all_pots_zero_returns_none() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(0, vec!["a", "b"]), pot(0, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Flop,
    ));
    assert!(r.is_none());
}

#[test]
fn test_single_chip_pot() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(100, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Turn,
    ))
    .unwrap();
    // 35% de 1 = 0.35 → truncado para 0.35 (não arredondado para 0!)
    assert_eq!(r.cashback, 35);
    assert_eq!(r.eligible_pot_total, 100);
}

#[test]
fn test_minimum_cashback_one_chip() {
    // 3 * 0.35 = 1.05 → em f64, 3.0 * 0.35 = 1.0499... truncado para 1.04
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(300, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Turn,
    ))
    .unwrap();
    assert!(
        r.cashback == 104,
        "cashback={}, expected ~1.04",
        r.cashback
    );
}

#[test]
fn test_large_pot_no_overflow() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(100000000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Preflop,
    ))
    .unwrap();
    assert_eq!(r.cashback, 15000000); // 15% de 1M
}

#[test]
fn test_max_f64_pot_no_panic() {
    // Testa que não há panic com valores extremos
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(u64::MAX / 2, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Preflop,
    ));
    // Pode ou não dar overflow no float, mas não deve panicar
    // Só verificamos que a função retorna (Some ou None)
    assert!(r.is_some() || r.is_none());
}

// ─── Múltiplos Jogadores ───

#[test]
fn test_three_players_loser_in_main_only() {
    // Jogador A all-in 100, B all-in 200, C all-in 300
    // main pot: A+B+C = 300 (100 cada), side1: B+C = 200 (100 cada), side2: C = 100
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(30000, vec!["a", "b", "c"]), // main
            pot(20000, vec!["b", "c"]),      // side 1
            pot(10000, vec!["c"]),           // side 2
        ],
        "a",
        "b",
        GamePhase::Flop,
    ))
    .unwrap();
    // A só está no main pot (300)
    assert_eq!(r.eligible_pot_total, 30000);
    assert_eq!(r.eligible_pot_ids, vec![0]);
    assert_eq!(r.cashback, 7500); // 25% de 300
}

#[test]
fn test_three_players_loser_in_main_and_side1() {
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(30000, vec!["a", "b", "c"]), // main
            pot(20000, vec!["b", "c"]),      // side 1
            pot(10000, vec!["c"]),           // side 2
        ],
        "b",
        "c",
        GamePhase::Flop,
    ))
    .unwrap();
    // B está no main (300) + side1 (200) = 500
    assert_eq!(r.eligible_pot_total, 50000);
    assert_eq!(r.eligible_pot_ids, vec![0, 1]);
    assert_eq!(r.cashback, 12500); // 25% de 500
}

#[test]
fn test_five_players_complex_side_pots() {
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(50000, vec!["a", "b", "c", "d", "e"]), // main
            pot(40000, vec!["b", "c", "d", "e"]),
            pot(30000, vec!["c", "d", "e"]),
            pot(20000, vec!["d", "e"]),
            pot(10000, vec!["e"]),
        ],
        "c",
        "e",
        GamePhase::Turn,
    ))
    .unwrap();
    // C está em main(500) + side1(400) + side2(300) = 1200
    assert_eq!(r.eligible_pot_total, 120000);
    assert_eq!(r.eligible_pot_ids.len(), 3);
    // 35% de 1200 = 420
    assert_eq!(r.cashback, 42000);
}

// ─── Consistência de Resultados ───

#[test]
fn test_result_contains_correct_ids() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(50000, vec!["loser1", "winner1"])],
        "loser1",
        "winner1",
        GamePhase::Preflop,
    ))
    .unwrap();
    assert_eq!(r.loser_id, "loser1");
    assert_eq!(r.winner_id, "winner1");
}

#[test]
fn test_odds_field_is_zero() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(50000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Flop,
    ))
    .unwrap();
    assert_eq!(
        r.odds, 0.0,
        "Campo odds deve ser 0 (não usado nesta versão)"
    );
}

#[test]
fn test_multiplier_is_one() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(50000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Turn,
    ))
    .unwrap();
    assert_eq!(r.multiplier, 1.0);
}

#[test]
fn test_base_cashback_equals_cashback() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(50000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Preflop,
    ))
    .unwrap();
    assert_eq!(r.base_cashback, r.cashback);
}

#[test]
fn test_phase_preserved_in_result() {
    for phase in [GamePhase::Preflop, GamePhase::Flop, GamePhase::Turn] {
        let r = calculate_progressive_loss_deflator(params(
            vec![pot(10000, vec!["a", "b"])],
            "a",
            "b",
            phase,
        ))
        .unwrap();
        assert_eq!(r.phase, phase);
    }
}

// ─── Per-Pot Cashback ───

#[test]
fn test_per_pot_indices_match_eligible() {
    let r = calculate_progressive_loss_deflator(params(
        vec![
            pot(10000, vec!["x"]),          // idx 0 - loser não elegível
            pot(20000, vec!["loser", "x"]), // idx 1
            pot(30000, vec!["loser", "x"]), // idx 2
            pot(40000, vec!["x"]),          // idx 3 - loser não elegível
        ],
        "loser",
        "x",
        GamePhase::Flop,
    ))
    .unwrap();
    assert_eq!(r.eligible_pot_ids, vec![1, 2]);
    assert_eq!(r.per_pot_cashback.len(), 2);
    assert_eq!(r.per_pot_cashback[0].pot_index, 1);
    assert_eq!(r.per_pot_cashback[1].pot_index, 2);
}

#[test]
fn test_per_pot_amounts_non_zero_for_positive_pots() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(100000, vec!["a", "b"]), pot(100000, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Turn,
    ))
    .unwrap();
    // 35% de 2000 = 700, rateio 50/50 = 350 cada
    for entry in &r.per_pot_cashback {
        assert!(
            entry.amount > 0,
            "Cada pot elegível deve contribuir com cashback > 0"
        );
    }
}

// ─── LossDeflatorTier ───

#[test]
fn test_tier_as_str() {
    assert_eq!(LossDeflatorTier::FifteenPercent.as_str(), "15%");
    assert_eq!(LossDeflatorTier::TwentyFivePercent.as_str(), "25%");
    assert_eq!(LossDeflatorTier::ThirtyFivePercent.as_str(), "35%");
}

#[test]
fn test_tier_percent() {
    assert!((LossDeflatorTier::FifteenPercent.percent() - 0.15).abs() < f64::EPSILON);
    assert!((LossDeflatorTier::TwentyFivePercent.percent() - 0.25).abs() < f64::EPSILON);
    assert!((LossDeflatorTier::ThirtyFivePercent.percent() - 0.35).abs() < f64::EPSILON);
}

// ─── Heads-Up Win Probability ───

#[test]
fn test_win_prob_aa_vs_kk_preflop() {
    let hero = vec![card(Rank::Ace, Suit::Hearts), card(Rank::Ace, Suit::Spades)];
    let villain = vec![
        card(Rank::King, Suit::Hearts),
        card(Rank::King, Suit::Spades),
    ];
    let prob = get_heads_up_win_probability(&hero, &villain, &[]);
    assert!(
        prob > 0.80 && prob < 0.85,
        "AA vs KK preflop ~82%, got {prob}"
    );
}

#[test]
fn test_win_prob_ak_vs_qq_preflop() {
    let hero = vec![
        card(Rank::Ace, Suit::Hearts),
        card(Rank::King, Suit::Hearts),
    ];
    let villain = vec![
        card(Rank::Queen, Suit::Spades),
        card(Rank::Queen, Suit::Diamonds),
    ];
    let prob = get_heads_up_win_probability(&hero, &villain, &[]);
    // AK vs QQ ~43%
    assert!(
        prob > 0.40 && prob < 0.47,
        "AK vs QQ preflop ~43%, got {prob}"
    );
}

#[test]
fn test_win_prob_dominated_hands() {
    // AK vs AQ (dominated)
    let hero = vec![
        card(Rank::Ace, Suit::Hearts),
        card(Rank::King, Suit::Hearts),
    ];
    let villain = vec![
        card(Rank::Ace, Suit::Spades),
        card(Rank::Queen, Suit::Spades),
    ];
    let prob = get_heads_up_win_probability(&hero, &villain, &[]);
    // AK vs AQ ~70%
    assert!(prob > 0.65 && prob < 0.75, "AK vs AQ ~70%, got {prob}");
}

#[test]
fn test_win_prob_on_flop() {
    // Hero: top pair, Villain: flush draw
    let hero = vec![
        card(Rank::Ace, Suit::Hearts),
        card(Rank::King, Suit::Diamonds),
    ];
    let villain = vec![
        card(Rank::Queen, Suit::Hearts),
        card(Rank::Jack, Suit::Hearts),
    ];
    let board = vec![
        card(Rank::Ace, Suit::Spades),
        card(Rank::Two, Suit::Hearts),
        card(Rank::Seven, Suit::Hearts),
    ];
    let prob = get_heads_up_win_probability(&hero, &villain, &board);
    // Top pair vs flush draw ~65%
    assert!(
        prob > 0.60 && prob < 0.70,
        "TP vs flush draw ~65%, got {prob}"
    );
}

#[test]
fn test_win_prob_on_turn() {
    // Hero: made straight, Villain: drawing dead
    let hero = vec![
        card(Rank::Ace, Suit::Hearts),
        card(Rank::King, Suit::Spades),
    ];
    let villain = vec![
        card(Rank::Two, Suit::Diamonds),
        card(Rank::Three, Suit::Clubs),
    ];
    let board = vec![
        card(Rank::Queen, Suit::Hearts),
        card(Rank::Jack, Suit::Diamonds),
        card(Rank::Ten, Suit::Spades),
        card(Rank::Nine, Suit::Clubs),
    ];
    let prob = get_heads_up_win_probability(&hero, &villain, &board);
    // Hero já tem straight, villain drawing dead → 100%
    assert!(
        prob > 0.99,
        "Hero made straight, villain drawing dead, got {prob}"
    );
}

#[test]
fn test_win_prob_river_royal_split() {
    let hero = vec![card(Rank::Two, Suit::Clubs), card(Rank::Three, Suit::Clubs)];
    let villain = vec![
        card(Rank::Four, Suit::Diamonds),
        card(Rank::Five, Suit::Diamonds),
    ];
    let board = vec![
        card(Rank::Ace, Suit::Hearts),
        card(Rank::King, Suit::Hearts),
        card(Rank::Queen, Suit::Hearts),
        card(Rank::Jack, Suit::Hearts),
        card(Rank::Ten, Suit::Hearts),
    ];
    let prob = get_heads_up_win_probability(&hero, &villain, &board);
    assert!(
        (prob - 0.5).abs() < f64::EPSILON,
        "Royal no board → split pot"
    );
}

#[test]
fn test_win_prob_empty_board() {
    // Verifica que funciona com board vazio (preflop)
    let hero = vec![
        card(Rank::Seven, Suit::Hearts),
        card(Rank::Two, Suit::Spades),
    ];
    let villain = vec![
        card(Rank::Eight, Suit::Diamonds),
        card(Rank::Three, Suit::Clubs),
    ];
    let prob = get_heads_up_win_probability(&hero, &villain, &[]);
    assert!(prob > 0.0 && prob < 1.0);
}

#[test]
fn test_win_prob_known_cards_dont_overlap() {
    // Verifica que cartas conhecidas são removidas do baralho
    let hero = vec![card(Rank::Ace, Suit::Hearts), card(Rank::Ace, Suit::Spades)];
    let villain = vec![
        card(Rank::Ace, Suit::Diamonds),
        card(Rank::Ace, Suit::Clubs),
    ];
    // Todos os 4 Ases estão nas mãos → 0 Ases no board possível
    let prob = get_heads_up_win_probability(&hero, &villain, &[]);
    // 4 ases empatados → ~split (estimativa Monte Carlo, tolerância de ruído)
    assert!(
        (prob - 0.5).abs() < 0.005,
        "4 Aces dealt → ~split (~0.5), got {prob}"
    );
}

// ─── GamePhase ───

#[test]
fn test_game_phase_debug() {
    // Verifica que GamePhase implementa Debug/Clone/Copy
    let phases = [GamePhase::Preflop,
        GamePhase::Flop,
        GamePhase::Turn,
        GamePhase::River];
    let cloned: Vec<_> = phases.to_vec();
    assert_eq!(format!("{:?}", cloned[0]), "Preflop");
    assert_eq!(cloned.len(), 4);
}

#[test]
fn test_game_phase_eq() {
    assert_eq!(GamePhase::Preflop, GamePhase::Preflop);
    assert_ne!(GamePhase::Preflop, GamePhase::Flop);
    assert_ne!(GamePhase::Turn, GamePhase::River);
}

// ─── Cenários de Integração ───

#[test]
fn test_empty_pots_vec_returns_none() {
    let r = calculate_progressive_loss_deflator(params(vec![], "a", "b", GamePhase::Flop));
    assert!(r.is_none());
}

#[test]
fn test_loser_equals_winner_still_works() {
    // Cenário incomum mas não deve panicar
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(50000, vec!["p", "q"])],
        "p",
        "p",
        GamePhase::Flop,
    ));
    // "p" está elegível, então cashback é calculado
    assert!(r.is_some());
}

#[test]
fn test_special_char_player_ids() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(20000, vec!["player-1", "player_2"])],
        "player-1",
        "player_2",
        GamePhase::Flop,
    ))
    .unwrap();
    assert_eq!(r.loser_id, "player-1");
    assert_eq!(r.winner_id, "player_2");
    assert_eq!(r.cashback, 5000);
}

#[test]
fn test_unicode_player_ids() {
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(20000, vec!["jogador", "café"])],
        "jogador",
        "café",
        GamePhase::Flop,
    ))
    .unwrap();
    assert_eq!(r.loser_id, "jogador");
    assert_eq!(r.winner_id, "café");
}

// ─── Arredondamento ───

#[test]
fn test_rounding_rounds_to_centavos() {
    // 15% de 333 = 49.95 → truncado para 49.95
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(33300, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Preflop,
    ))
    .unwrap();
    // 333 * 0.15 = 49.95 → truncado = 49.95
    assert_eq!(r.cashback, 4995);
}

#[test]
fn test_rounding_down() {
    // 25% de 199 = 49.75 → truncado para 49.75
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(19900, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Flop,
    ))
    .unwrap();
    assert_eq!(r.cashback, 4975);
}

#[test]
fn test_rounding_edge_499() {
    // 15% de 3300 centavos = 495 centavos
    let r = calculate_progressive_loss_deflator(params(
        vec![pot(3300, vec!["a", "b"])],
        "a",
        "b",
        GamePhase::Preflop,
    ))
    .unwrap();
    assert_eq!(r.cashback, 495);
}

// ─── Consistência com múltiplas chamadas ───

#[test]
fn test_idempotent_same_params() {
    let p = params(vec![pot(50000, vec!["a", "b"])], "a", "b", GamePhase::Flop);
    let r1 = calculate_progressive_loss_deflator(p.clone()).unwrap();
    let r2 = calculate_progressive_loss_deflator(p).unwrap();
    assert_eq!(r1.cashback, r2.cashback);
    assert_eq!(r1.eligible_pot_total, r2.eligible_pot_total);
    assert_eq!(r1.tier, r2.tier);
}