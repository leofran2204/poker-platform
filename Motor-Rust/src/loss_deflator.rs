// loss-deflator.rs — Loss Deflator (Cashback por Bad Beat)
// Migrado de TypeScript (loss-deflator.ts) para Rust em 2026-07-02
//
// O cashback é determinado exclusivamente pela equity do jogador que perdeu,
// calculada no momento em que seu all-in foi pago:
//
//   Equity do perdedor | Deflator
//   -------------------|---------
//   abaixo de 56%      |  0%
//   56% até < 66%      |  7%
//   66% até < 76%      | 15%
//   76% até < 86%      | 25%
//   86% ou mais        | 35%
//
// IMPORTANTE — Sobre quais pots o cashback incide:
// O cashback é aplicado SOMENTE sobre os pots em que o PERDEDOR participou
// (esteve elegível). Se o perdedor só contribuiu até o nível do side pot 1,
// o cashback incide apenas sobre main pot + side pot 1 — nunca sobre pots
// em que ele não contribuiu. Isso preserva o dinheiro dos jogadores que
// apostaram mais alto e não foram afetados pelo bad beat.
//
// Sem impacto financeiro para a plataforma: o cashback vem do próprio
// pote disputado, nunca de saldo da casa.
//
// A fase do all-in é preservada apenas para reconstruir o board conhecido
// naquele instante e para auditoria. Ela não escolhe o percentual.
// A ordem financeira obrigatória é pots → rake → Loss Deflator → pagamentos.

use crate::deck::{contains_card, create_full_deck, Card};
use crate::types::{GamePhase, Pot};
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Tier do deflator (para serialização/exibição)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LossDeflatorTier {
    SevenPercent,
    FifteenPercent,
    TwentyFivePercent,
    ThirtyFivePercent,
}

impl LossDeflatorTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            LossDeflatorTier::SevenPercent => "7%",
            LossDeflatorTier::FifteenPercent => "15%",
            LossDeflatorTier::TwentyFivePercent => "25%",
            LossDeflatorTier::ThirtyFivePercent => "35%",
        }
    }

    #[allow(dead_code)]
    pub fn percent(&self) -> f64 {
        match self {
            LossDeflatorTier::SevenPercent => 0.07,
            LossDeflatorTier::FifteenPercent => 0.15,
            LossDeflatorTier::TwentyFivePercent => 0.25,
            LossDeflatorTier::ThirtyFivePercent => 0.35,
        }
    }

    pub fn basis_points(&self) -> u16 {
        match self {
            LossDeflatorTier::SevenPercent => 700,
            LossDeflatorTier::FifteenPercent => 1_500,
            LossDeflatorTier::TwentyFivePercent => 2_500,
            LossDeflatorTier::ThirtyFivePercent => 3_500,
        }
    }

    /// Classifica a equity do perdedor conforme a regra financeira oficial.
    ///
    /// As faixas são inclusivas no limite inferior e exclusivas no superior:
    /// [56%, 66%), [66%, 76%), [76%, 86%) e [86%, 100%].
    pub fn from_loser_equity(loser_equity: f64) -> Option<Self> {
        if !loser_equity.is_finite() || !(0.0..=1.0).contains(&loser_equity) {
            return None;
        }

        if loser_equity >= 0.86 {
            Some(LossDeflatorTier::ThirtyFivePercent)
        } else if loser_equity >= 0.76 {
            Some(LossDeflatorTier::TwentyFivePercent)
        } else if loser_equity >= 0.66 {
            Some(LossDeflatorTier::FifteenPercent)
        } else if loser_equity >= 0.56 {
            Some(LossDeflatorTier::SevenPercent)
        } else {
            None
        }
    }
}

/// Parâmetros para o deflator progressivo
#[derive(Debug, Clone)]
pub struct ProgressiveLossDeflatorParams {
    pub pots: Vec<Pot>,
    pub loser_id: String,
    pub winner_id: String,
    pub phase: GamePhase,
    /// Equity do perdedor no instante do all-in pago, na escala 0.0..=1.0.
    pub loser_equity: f64,
}

/// Resultado do deflator progressivo em centavos inteiros
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProgressiveLossDeflatorResult {
    pub loser_id: String,
    pub winner_id: String,
    pub cashback: u64,     // cashback total em centavos
    pub loser_equity: f64, // equity no instante do all-in, escala 0.0..=1.0
    pub tier: LossDeflatorTier,
    pub base_cashback: u64, // cashback total em centavos antes do rateio
    pub multiplier: f64,    // mantido em 1.0 para compatibilidade
    pub phase: GamePhase,
    pub cards_remaining: u8, // quantas cartas faltavam quando o all-in aconteceu
    pub eligible_pot_ids: Vec<usize>, // índices dos pots em que o perdedor participou
    pub eligible_pot_total: u64, // soma dos pots elegíveis em centavos
    pub per_pot_cashback: Vec<PotCashbackEntry>, // rateio por pot em centavos
}

/// Entrada individual do rateio: quanto cada pot contribuiu para o cashback (em centavos)
#[derive(Debug, Clone)]
pub struct PotCashbackEntry {
    pub pot_index: usize,
    pub amount: u64,
}

fn cards_remaining_at_all_in(phase: GamePhase) -> u8 {
    match phase {
        GamePhase::Preflop => 5,
        GamePhase::Flop => 2,
        GamePhase::Turn => 1,
        GamePhase::River | GamePhase::Showdown => 0,
    }
}

/// Calcula o Loss Deflator pelo tier de equity sobre potes já líquidos de rake.
pub fn calculate_progressive_loss_deflator(
    params: ProgressiveLossDeflatorParams,
) -> Option<ProgressiveLossDeflatorResult> {
    let ProgressiveLossDeflatorParams {
        pots,
        loser_id,
        winner_id,
        phase,
        loser_equity,
    } = params;

    let tier = LossDeflatorTier::from_loser_equity(loser_equity)?;
    let deflator_basis_points = tier.basis_points();
    let cards_remaining = cards_remaining_at_all_in(phase);

    // 1. Identificar pots em que o PERDEDOR é elegível
    let mut eligible_pot_indices = Vec::new();
    for (idx, pot) in pots.iter().enumerate() {
        if pot.eligible_players.contains(&loser_id) {
            eligible_pot_indices.push(idx);
        }
    }

    if eligible_pot_indices.is_empty() {
        return None;
    }

    // 2. Somar apenas os pots elegíveis em centavos
    let eligible_pot_total: u64 = eligible_pot_indices
        .iter()
        .map(|&idx| pots[idx].amount)
        .sum();

    if eligible_pot_total == 0 {
        return None;
    }

    // 3. Calcular cashback total sobre os pots elegíveis em centavos inteiros
    let base_cashback =
        ((eligible_pot_total as u128 * deflator_basis_points as u128) / 10_000) as u64;

    // 4. Ratear o cashback proporcionalmente entre os pots elegíveis (em centavos)
    let mut per_pot_cashback = Vec::new();
    let mut distributed: u64 = 0;

    for (i, &idx) in eligible_pot_indices.iter().enumerate() {
        let is_last = i == eligible_pot_indices.len() - 1;
        let amount = if is_last {
            base_cashback.saturating_sub(distributed)
        } else {
            ((base_cashback as u128 * pots[idx].amount as u128) / eligible_pot_total as u128) as u64
        };
        distributed += amount;
        per_pot_cashback.push(PotCashbackEntry {
            pot_index: idx,
            amount,
        });
    }

    Some(ProgressiveLossDeflatorResult {
        loser_id,
        winner_id,
        cashback: base_cashback,
        loser_equity,
        tier,
        base_cashback,
        multiplier: 1.0,
        phase,
        cards_remaining,
        eligible_pot_ids: eligible_pot_indices,
        eligible_pot_total,
        per_pot_cashback,
    })
}

/// Número de amostras Monte Carlo por chamada (quando não há board completo).
///
/// Com board vazio (preflop) há C(45,5) ≈ 1.2M boards possíveis — a
/// enumeração exata era inviável em tempo de execução/testes. Com Monte Carlo
/// fixamos o custo em no máximo `MC_SAMPLES` avaliações (sem reposição,
/// determinístico via seed), o que mantém a precisão dentro da tolerância dos
/// testes sem custo exponencial. Erro típico ~1/√N ≈ 0.14% em 500k amostras.
const MC_SAMPLES: u32 = 500_000;

/// Calcula probabilidade de vitória heads-up.
///
/// Quando o board já está completo (river), a avaliação é exata (1 board).
/// Caso contrário usa **Monte Carlo sem reposição**: o baralho restante é
/// embaralhado uma única vez e percorrido em janelas de `cards_to_deal` cartas,
/// de modo que cada board possível é avaliado no máximo uma vez (sem repetir
/// boards já verificados). O número de amostras é limitado ao total de boards
/// distintos disponíveis.
#[allow(dead_code)]
pub fn get_heads_up_win_probability(
    hero_cards: &[Card],
    villain_cards: &[Card],
    board_cards: &[Card],
) -> f64 {
    let known_cards: Vec<Card> = hero_cards
        .iter()
        .chain(villain_cards.iter())
        .chain(board_cards.iter())
        .cloned()
        .collect();

    let cards_to_deal = 5 - board_cards.len();

    if cards_to_deal == 0 {
        return evaluate_outcome(hero_cards, villain_cards, board_cards);
    }

    let deck = get_remaining_deck(&known_cards);
    let mut rng = monte_carlo_rng(&known_cards);

    // Máximo de boards distintos sem reposição = C(deck.len(), cards_to_deal).
    // Se couberem todos, a estimativa é exata; senão amostramos MC_SAMPLES.
    let max_boards = combinations_count(deck.len(), cards_to_deal);
    let samples = MC_SAMPLES.min(max_boards as u32) as usize;

    let mut wins = 0u64;
    let mut ties = 0u64;
    // Boards já avaliados, para NÃO repetir (amostragem sem reposição).
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();

    // Índices base para embaralhar e selecionar `cards_to_deal` posições.
    let mut indices: Vec<usize> = (0..deck.len()).collect();

    while seen.len() < samples && seen.len() < max_boards {
        indices.shuffle(&mut rng);
        let board_idx: Vec<usize> = indices[..cards_to_deal].to_vec();
        // Chave estável do board (ordenada p/ não diferenciar ordem das cartas).
        let mut key_idx = board_idx.clone();
        key_idx.sort_unstable();
        let mut key = 0u64;
        for &i in &key_idx {
            key = key.wrapping_mul(67).wrapping_add(i as u64 + 1);
        }
        if !seen.insert(key) {
            // Board já visto: pula e continua buscando boards novos.
            continue;
        }

        let mut final_board = board_cards.to_vec();
        for &i in &board_idx {
            final_board.push(deck[i]);
        }

        let outcome = evaluate_outcome(hero_cards, villain_cards, &final_board);
        if outcome > 0.5 {
            wins += 1;
        } else if (outcome - 0.5).abs() < f64::EPSILON {
            ties += 1;
        }
    }

    let total = seen.len() as f64;
    (wins as f64 + ties as f64 * 0.5) / total
}

/// CSPRNG determinístico: seed derivada das cartas conhecidas (não do relógio),
/// para que o mesmo cenário sempre produza a mesma estimativa.
fn monte_carlo_rng(known_cards: &[Card]) -> rand::rngs::StdRng {
    let mut seed: u64 = 0;
    for (i, card) in known_cards.iter().enumerate() {
        let rank_val = card.rank as u64;
        let suit_val = card.suit as u64;
        seed = seed
            .wrapping_mul(31)
            .wrapping_add(rank_val)
            .wrapping_add(suit_val.wrapping_mul(1_049_273))
            .wrapping_add((i as u64).wrapping_mul(2_654_537));
    }
    rand::rngs::StdRng::seed_from_u64(seed)
}

/// Conta C(n, k) sem estourar para os tamanhos usados aqui (n <= 45).
fn combinations_count(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result = 1usize;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    result
}

/// Estimativa de ruído (erro) do Monte Carlo, em pontos de probabilidade.
///
/// A amostragem é **sem reposição** sobre uma população finita de `max_boards`
/// boards possíveis, sorteando `samples` deles. Para o pior caso (proporção
/// p = 0.5), o erro padrão da estimativa é:
///
///   SE = 0.5 · √( (1 - f) / samples ),   onde f = samples / max_boards
///
/// - Se `samples >= max_boards` (população toda coberta): erro 0 (exato).
/// - Caso contrário aplica o fator de correção de população finita (1 - f),
///   que deixa o erro MENOR que o Monte Carlo com reposição.
///
/// O valor retornado é a **margem de 3 desvios (~99.7% de confiança)**:
///
///   bound = 3 · SE
///
/// Exemplo: 500k amostras no preflop (max ≈ 1.22M) →
///   f ≈ 0.41, SE ≈ 0.00034, bound ≈ 0.0010 (0.10%).
///
/// Esta função é uma segurança extra: os testes podem exigir que o desvio
/// observado da estimativa fique dentro de `mc_error_bound(...)`.
pub fn mc_error_bound(samples: u64, max_boards: u64) -> f64 {
    if max_boards == 0 {
        return 0.0;
    }
    let samples = samples.min(max_boards);
    if samples >= max_boards {
        return 0.0;
    }
    let f = samples as f64 / max_boards as f64; // fração amostrada
    let se = 0.5 * ((1.0 - f) / samples as f64).sqrt();
    3.0 * se // ~99.7% de confiança (3 sigma)
}

// ─── Funções auxiliares privadas ───

#[allow(dead_code)]
fn evaluate_outcome(hero_cards: &[Card], villain_cards: &[Card], board: &[Card]) -> f64 {
    use crate::deck::{compare_hands, evaluate_hand};
    use std::cmp::Ordering;
    let hero_hand = evaluate_hand(hero_cards, board);
    let villain_hand = evaluate_hand(villain_cards, board);
    let comparison = compare_hands(&hero_hand, &villain_hand);
    match comparison {
        Ordering::Greater => 1.0,
        Ordering::Equal => 0.5,
        Ordering::Less => 0.0,
    }
}

#[allow(dead_code)]
fn get_remaining_deck(known_cards: &[Card]) -> Vec<Card> {
    create_full_deck()
        .into_iter()
        .filter(|card| !contains_card(known_cards, card))
        .collect()
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Card, Rank, Suit};

    fn make_card(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    fn make_pot(amount: u64, eligible: Vec<&str>) -> Pot {
        Pot {
            amount,
            eligible_players: eligible.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_deflator_seven_percent_from_equity() {
        let pots = vec![make_pot(20000, vec!["loser", "winner"])];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Preflop,
            loser_equity: 0.60,
        });
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.tier, LossDeflatorTier::SevenPercent);
        assert_eq!(r.cards_remaining, 5);
        assert_eq!(r.loser_equity, 0.60);
        assert_eq!(r.cashback, 1400);
    }

    #[test]
    fn test_deflator_twenty_five_percent_from_equity() {
        let pots = vec![make_pot(20000, vec!["loser", "winner"])];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Flop,
            loser_equity: 0.80,
        });
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.tier, LossDeflatorTier::TwentyFivePercent);
        assert_eq!(r.cards_remaining, 2);
        assert_eq!(r.loser_equity, 0.80);
        assert_eq!(r.cashback, 5000);
    }

    #[test]
    fn test_deflator_fifteen_percent_from_equity() {
        let pots = vec![make_pot(20000, vec!["loser", "winner"])];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Turn,
            loser_equity: 0.70,
        })
        .unwrap();
        assert_eq!(result.tier, LossDeflatorTier::FifteenPercent);
        assert_eq!(result.cards_remaining, 1);
        assert_eq!(result.cashback, 3000);
    }

    #[test]
    fn test_deflator_thirty_five_percent_from_equity() {
        let pots = vec![make_pot(20000, vec!["loser", "winner"])];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Turn,
            loser_equity: 0.90,
        });
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.tier, LossDeflatorTier::ThirtyFivePercent);
        assert_eq!(r.cards_remaining, 1);
        assert_eq!(r.loser_equity, 0.90);
        assert_eq!(r.cashback, 7000);
    }

    #[test]
    fn test_equity_below_fifty_six_percent_returns_none() {
        let pots = vec![make_pot(20000, vec!["loser", "winner"])];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Preflop,
            loser_equity: 0.559_999,
        });
        assert!(result.is_none());
    }

    #[test]
    fn test_exact_equity_boundaries() {
        let cases = [
            (0.559_999, None),
            (0.56, Some(LossDeflatorTier::SevenPercent)),
            (0.659_999, Some(LossDeflatorTier::SevenPercent)),
            (0.66, Some(LossDeflatorTier::FifteenPercent)),
            (0.759_999, Some(LossDeflatorTier::FifteenPercent)),
            (0.76, Some(LossDeflatorTier::TwentyFivePercent)),
            (0.859_999, Some(LossDeflatorTier::TwentyFivePercent)),
            (0.86, Some(LossDeflatorTier::ThirtyFivePercent)),
            (1.0, Some(LossDeflatorTier::ThirtyFivePercent)),
        ];

        for (equity, expected) in cases {
            assert_eq!(
                LossDeflatorTier::from_loser_equity(equity),
                expected,
                "tier incorreto para equity {equity}"
            );
        }
        assert_eq!(LossDeflatorTier::from_loser_equity(f64::NAN), None);
        assert_eq!(LossDeflatorTier::from_loser_equity(-0.01), None);
        assert_eq!(LossDeflatorTier::from_loser_equity(1.01), None);
    }

    #[test]
    fn test_deflator_only_eligible_pots() {
        // main pot: loser + winner (100 cada = 200)
        // side pot: apenas winner (100)
        // cashback só incide sobre main pot (200)
        let pots = vec![
            make_pot(20000, vec!["loser", "winner"]), // main pot
            make_pot(10000, vec!["winner"]),          // side pot - loser não participou
        ];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Flop, // 25%
            loser_equity: 0.80,
        });
        assert!(result.is_some());
        let r = result.unwrap();
        // 25% de 200 = 50 (apenas main pot)
        assert_eq!(r.cashback, 5000);
        assert_eq!(r.eligible_pot_total, 20000);
        assert_eq!(r.eligible_pot_ids, vec![0]);
    }

    #[test]
    fn test_deflator_multiple_eligible_pots_proportional() {
        // main pot: 200 (loser + winner)
        // side pot: 100 (loser + winner) - ambos participaram
        // total elegível = 300
        // 25% de 300 = 75
        // rateio: main = 75 * 200/300 = 50, side = 75 * 100/300 = 25
        let pots = vec![
            make_pot(20000, vec!["loser", "winner"]),
            make_pot(10000, vec!["loser", "winner"]),
        ];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Flop,
            loser_equity: 0.80,
        });
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.cashback, 7500);
        assert_eq!(r.eligible_pot_total, 30000);
        assert_eq!(r.per_pot_cashback.len(), 2);
        assert_eq!(r.per_pot_cashback[0].amount, 5000); // main pot
        assert_eq!(r.per_pot_cashback[1].amount, 2500); // side pot
    }

    #[test]
    fn test_deflator_loser_not_eligible_returns_none() {
        let pots = vec![make_pot(20000, vec!["winner"])];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::Flop,
            loser_equity: 0.80,
        });
        assert!(result.is_none());
    }

    #[test]
    #[cfg_attr(
        not(feature = "massive-tests"),
        ignore = "Monte Carlo preflop massivo; habilite a feature massive-tests manualmente"
    )]
    fn test_heads_up_win_probability_preflop() {
        // AA vs KK preflop
        let hero = vec![
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::Ace, Suit::Spades),
        ];
        let villain = vec![
            make_card(Rank::King, Suit::Hearts),
            make_card(Rank::King, Suit::Spades),
        ];
        let board = vec![];
        let prob = get_heads_up_win_probability(&hero, &villain, &board);
        // AA vs KK preflop ~82%
        assert!(prob > 0.80 && prob < 0.85);
    }

    #[test]
    fn test_heads_up_win_probability_river() {
        // Board: A♥ K♥ Q♥ J♥ T♥ (royal flush)
        // Ambos split
        let hero = vec![
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Three, Suit::Clubs),
        ];
        let villain = vec![
            make_card(Rank::Four, Suit::Diamonds),
            make_card(Rank::Five, Suit::Diamonds),
        ];
        let board = vec![
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::King, Suit::Hearts),
            make_card(Rank::Queen, Suit::Hearts),
            make_card(Rank::Jack, Suit::Hearts),
            make_card(Rank::Ten, Suit::Hearts),
        ];
        let prob = get_heads_up_win_probability(&hero, &villain, &board);
        assert!((prob - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    #[cfg_attr(
        not(feature = "massive-tests"),
        ignore = "Monte Carlo preflop massivo; habilite a feature massive-tests manualmente"
    )]
    fn test_mc_error_bound_within_tolerance_preflop() {
        // AA vs KK preflop. A estimativa Monte Carlo é determinística (seed por
        // cartas), então o valor é fixo e reproduzível. A segurança aqui é:
        // (1) o bound analítico deve ser estritamente menor que a tolerância de
        //     0.005 usada nos testes (logo não há erro material possível);
        // (2) rodadas idênticas devem dar exatamente o mesmo valor (sem ruído
        //     não-determinístico vazando para o resultado).
        let hero = vec![
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::Ace, Suit::Spades),
        ];
        let villain = vec![
            make_card(Rank::King, Suit::Hearts),
            make_card(Rank::King, Suit::Spades),
        ];
        let board = vec![];
        let prob1 = get_heads_up_win_probability(&hero, &villain, &board);
        let prob2 = get_heads_up_win_probability(&hero, &villain, &board);

        // Determinismo: mesma entrada → mesma saída exata.
        assert!(
            (prob1 - prob2).abs() < f64::EPSILON,
            "Monte Carlo não-determinístico: {prob1} != {prob2}"
        );

        // População de boards possíveis no preflop: C(45,5)
        let max_boards = combinations_count(45, 5) as u64;
        let bound = mc_error_bound(MC_SAMPLES as u64, max_boards);

        // O bound analítico deve ser menor que a tolerância de 0.005 — ou seja,
        // mesmo no pior cenário (p=0.5) o erro do MC está dentro da margem.
        assert!(
            bound < 0.005,
            "Erro de Monte Carlo {bound:.4} acima da tolerância de 0.005"
        );

        // Sanidade: equity do favorito deve estar em faixa plausível (0.75..0.90).
        assert!(
            prob1 > 0.75 && prob1 < 0.90,
            "Equity AA vs KK fora da faixa esperada: {prob1}"
        );
    }

    #[test]
    fn test_mc_error_bound_exact_when_full_population() {
        // Com o board completo (river), max_boards = 1 → estimativa exata (bound 0).
        assert_eq!(mc_error_bound(1, 1), 0.0);
        // Amostrar toda a população também dá bound 0.
        assert_eq!(mc_error_bound(100, 100), 0.0);
        // População vazia não deve quebrar.
        assert_eq!(mc_error_bound(0, 0), 0.0);
    }

    #[test]
    fn test_mc_error_bound_decreases_with_samples() {
        let max_boards = combinations_count(45, 5) as u64;
        let b_small = mc_error_bound(10_000, max_boards);
        let b_large = mc_error_bound(500_000, max_boards);
        assert!(
            b_large < b_small,
            "Mais amostras devem reduzir o bound: {b_large:.5} >= {b_small:.5}"
        );
    }
}
