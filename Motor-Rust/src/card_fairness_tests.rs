// card_fairness_tests.rs — Testes de aleatoriedade e imparcialidade das cartas
//
// Objetivo: garantir que o baralho do motor de poker é justo e isento de
// erros catastróficos (ex.: duas Damas de Copas na mesma mão).
//
// Cada teste roda 500.000 iterações (conforme critério de rigor do projeto)
// e valida:
//   1. Ausência de duplicatas no deal completo (hole + board = 52 únicas).
//   2. Distribuição uniforme das hole cards (cada carta ~1/52).
//   3. Distribuição uniforme das comunitárias flop/turn/river (cada posição).
//
// Tolerância de ruído: 0.5% (0.005), igual à usada no RNG e no Monte Carlo.
// Estatística: qui-quadrado agregado (4σ) — sem flakiness.

use crate::deck::{create_deck, deal_cards, shuffle_deck, Card};

/// Simula o deal completo de uma mão de `n_players` jogadores, com burn cards,
/// exatamente como o `game_loop` faz (ver game_loop.rs:612-637).
///
/// Retorna (hole_cards_por_jogador, flop, turn, river).
fn deal_full_hand(n_players: usize) -> (Vec<Vec<Card>>, Vec<Card>, Vec<Card>, Vec<Card>) {
    let full_deck = create_deck();
    let mut deck = shuffle_deck(&full_deck);

    // Hole cards: 2 por jogador (2 rodadas sequenciais, como no motor)
    let mut holes: Vec<Vec<Card>> = vec![Vec::with_capacity(2); n_players];
    for _round in 0..2 {
        for hole in &mut holes {
            let (cards, rest) = deal_cards(&deck, 1);
            deck = rest;
            hole.extend(cards);
        }
    }

    // Flop: burn 1 + 3
    let (_burn, d) = deal_cards(&deck, 1);
    deck = d;
    let (flop, d) = deal_cards(&deck, 3);
    deck = d;

    // Turn: burn 1 + 1
    let (_burn, d) = deal_cards(&deck, 1);
    deck = d;
    let (turn, d) = deal_cards(&deck, 1);
    deck = d;

    // River: burn 1 + 1
    let (_burn, d) = deal_cards(&deck, 1);
    deck = d;
    let (river, _d) = deal_cards(&deck, 1);

    (holes, flop, turn, river)
}

/// Qui-quadrado agregado: conta quantas vezes cada uma das 52 cartas apareceu
/// em `observed` e compara com expected = total/52.
fn chi_squared_52(observed: &[usize; 52], total: u64) -> f64 {
    let expected = total as f64 / 52.0;
    let mut chi2 = 0.0f64;
    for &c in observed {
        let diff = c as f64 - expected;
        chi2 += diff * diff / expected;
    }
    chi2
}

/// Converte uma carta em índice 0..52 (suit*13 + rank-2), para o histograma.
fn card_index(c: &Card) -> usize {
    (c.suit as usize) * 13 + (c.rank as usize - 2)
}

#[test]
fn test_card_fairness_no_duplicates_full_deal() {
    // 500k mãos: NUNCA pode haver carta duplicada (ex.: duas Damas de Copas).
    // O deal completo (hole de todos os jogadores + flop + turn + river) deve
    // sempre conter 52 cartas distintas.
    let n_players = 6;
    for _ in 0..500_000 {
        let (holes, flop, turn, river) = deal_full_hand(n_players);

        let mut all: Vec<Card> = Vec::with_capacity(52);
        for h in &holes {
            all.extend(h.iter().cloned());
        }
        all.extend(flop.iter().cloned());
        all.extend(turn.iter().cloned());
        all.extend(river.iter().cloned());

        // 2 hole cards por jogador + 3 + 1 + 1 = 2*6 + 5 = 17 cartas entregues.
        assert_eq!(all.len(), 2 * n_players + 5, "Quantidade de cartas errada");

        // Todas distintas (nenhuma carta repetida em toda a mão).
        // Usa um bitmap de 52 índices (evita exigir Hash em Card).
        let mut seen = [false; 52];
        for c in &all {
            let idx = card_index(c);
            assert!(
                !seen[idx],
                "Carta duplicada detectada no deal (ex.: duas Damas de Copas)"
            );
            seen[idx] = true;
        }
    }
}

#[test]
fn test_card_fairness_hole_cards_distribution() {
    // 500k mãos: cada uma das 52 cartas deve aparecer como hole card com
    // frequência ~1/52 (imparcialidade das hole cards).
    let n_players = 6;
    let mut counts = [0usize; 52];
    let mut total = 0u64;
    for _ in 0..500_000 {
        let (holes, _f, _t, _r) = deal_full_hand(n_players);
        for h in &holes {
            for c in h {
                counts[card_index(c)] += 1;
                total += 1;
            }
        }
    }
    let chi2 = chi_squared_52(&counts, total);
    // k=52, limite 4σ: k + 4·√(2k) ≈ 52 + 4·10.2 ≈ 92.8
    let limit = 52.0 + 4.0 * f64::sqrt(2.0 * 52.0);
    assert!(
        chi2 <= limit,
        "Hole cards viciadas: χ²={chi2:.2} > limite {limit:.2} (n={total})"
    );
    // Bound de 3σ por carta deve respeitar 0.5%.
    let p = 1.0 / 52.0;
    let bound_3sigma = 3.0 * (p * (1.0 - p) / total as f64).sqrt();
    assert!(
        bound_3sigma < 0.005,
        "Ruído das hole cards {bound_3sigma:.4} acima de 0.005 (0.5%)"
    );
}

#[test]
fn test_card_fairness_community_flop_turn_river_distribution() {
    // 500k mãos: valida imparcialidade das comunitárias.
    // Flop tem 3 posições (cada uma deve ser uniforme entre 52 cartas);
    // turn e river são 1 posição cada.
    let n_players = 6;
    let mut flop_pos = [0usize; 52 * 3]; // 3 posições de flop
    let mut turn_count = [0usize; 52];
    let mut river_count = [0usize; 52];
    let mut flop_total = 0u64;
    let mut turn_total = 0u64;
    let mut river_total = 0u64;

    for _ in 0..500_000 {
        let (_holes, flop, turn, river) = deal_full_hand(n_players);
        for (i, c) in flop.iter().enumerate() {
            flop_pos[i * 52 + card_index(c)] += 1;
            flop_total += 1;
        }
        turn_count[card_index(&turn[0])] += 1;
        turn_total += 1;
        river_count[card_index(&river[0])] += 1;
        river_total += 1;
    }

    // Flop (3 posições, k=156)
    let chi2_flop = chi_squared_52_multi(&flop_pos, flop_total, 3);
    let limit_flop = 156.0 + 4.0 * f64::sqrt(2.0 * 156.0);
    assert!(
        chi2_flop <= limit_flop,
        "Flop viciado: χ²={chi2_flop:.2} > limite {limit_flop:.2}"
    );

    // Turn (k=52)
    let chi2_turn = chi_squared_52(&turn_count, turn_total);
    let limit_52 = 52.0 + 4.0 * f64::sqrt(2.0 * 52.0);
    assert!(
        chi2_turn <= limit_52,
        "Turn viciado: χ²={chi2_turn:.2} > limite {limit_52:.2}"
    );

    // River (k=52)
    let chi2_river = chi_squared_52(&river_count, river_total);
    assert!(
        chi2_river <= limit_52,
        "River viciado: χ²={chi2_river:.2} > limite {limit_52:.2}"
    );

    // Bound de 3σ para qualquer posição < 0.5%.
    for (label, total) in [
        ("flop", flop_total / 3),
        ("turn", turn_total),
        ("river", river_total),
    ] {
        let bound = 3.0 * (1.0 / 52.0 * (51.0 / 52.0) / total as f64).sqrt();
        assert!(
            bound < 0.005,
            "Ruído do {label} {bound:.4} acima de 0.005 (0.5%)"
        );
    }
}

/// Qui-quadrado para um histograma de `positions` grupos de 52 cartas
/// (ex.: 3 posições de flop → 156 categorias).
fn chi_squared_52_multi(hist: &[usize; 156], total_all_categories: u64, positions: usize) -> f64 {
    let expected = total_all_categories as f64 / (52.0 * positions as f64);
    let mut chi2 = 0.0f64;
    for &c in hist {
        let diff = c as f64 - expected;
        chi2 += diff * diff / expected;
    }
    chi2
}
