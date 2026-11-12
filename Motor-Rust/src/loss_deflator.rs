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
//   86% até < 100%     | 35%   (100% = sem perdedor possível: quem tem 100% não perde)
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
use crate::types::{GamePhase, PokerVariant, Pot};
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Tier do deflator (para serialização/exibição)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    /// [56%, 66%), [66%, 76%), [76%, 86%) e [86%, 100%).
    /// Equity 1.0 (100%) não tem tier: quem tem 100% não perde, logo não
    /// há perdedor elegível; valores fora de [0.0, 1.0] também são None.
    pub fn from_loser_equity(loser_equity: f64) -> Option<Self> {
        if !loser_equity.is_finite() || !(0.0..1.0).contains(&loser_equity) {
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Quantos oponentes entraram no cálculo de equity (1 = heads-up).
    #[serde(default = "default_one_opponent")]
    pub opponents_counted: u8,
}

fn default_one_opponent() -> u8 {
    1
}

/// Entrada individual do rateio: quanto cada pot contribuiu para o cashback (em centavos)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PotCashbackEntry {
    pub pot_index: usize,
    pub amount: u64,
}

/// Resumo auditável do deflator para hand history / API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LossDeflatorAudit {
    pub loser_id: String,
    pub winner_id: String,
    pub cashback: u64,
    /// Equity do perdedor na escala 0.0..=1.0.
    pub loser_equity: f64,
    /// Percentual de cashback aplicado (7, 15, 25 ou 35).
    pub tier_percent: u8,
    pub phase: GamePhase,
    /// Quantos oponentes entraram no cálculo de equity (1 = heads-up).
    pub opponents_counted: u8,
}

impl ProgressiveLossDeflatorResult {
    pub fn to_audit(&self) -> LossDeflatorAudit {
        LossDeflatorAudit {
            loser_id: self.loser_id.clone(),
            winner_id: self.winner_id.clone(),
            cashback: self.cashback,
            loser_equity: self.loser_equity,
            tier_percent: match self.tier {
                LossDeflatorTier::SevenPercent => 7,
                LossDeflatorTier::FifteenPercent => 15,
                LossDeflatorTier::TwentyFivePercent => 25,
                LossDeflatorTier::ThirtyFivePercent => 35,
            },
            phase: self.phase,
            opponents_counted: self.opponents_counted.max(1),
        }
    }
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
        opponents_counted: 1,
        phase,
        cards_remaining,
        eligible_pot_ids: eligible_pot_indices,
        eligible_pot_total,
        per_pot_cashback,
    })
}

/// Número de amostras Monte Carlo por chamada (quando não há board completo).
///
/// Heads-up pré-flop com quatro cartas conhecidas tem C(48,5) = 1.712.304
/// boards possíveis. Nos espaços maiores usamos Monte Carlo e
/// fixamos o custo em no máximo `MC_SAMPLES` avaliações (sem reposição,
/// determinístico via seed), o que mantém a precisão dentro da tolerância dos
/// testes sem custo exponencial. A precisão amostral não garante um tier exato
/// para equities próximas dos limites; `mc_error_bound` é uma aproximação.
const MC_SAMPLES: u32 = 500_000;

/// Calcula equity heads-up (vitória + metade da frequência de empate).
///
/// Quando o board já está completo (river), a avaliação é exata (1 board).
/// Caso contrário embaralha índices a cada tentativa, sorteia um board e
/// rejeita repetições: amostragem uniforme sem reposição entre boards. Quando
/// todos os boards distintos cabem no limite, o resultado é exato.
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

/// Equity multiway do herói contra um ou mais vilões no board conhecido.
///
/// - 0 vilões → 1.0  
/// - 1 vilão → delega para `get_heads_up_win_probability`  
/// - 2+ vilões → Monte Carlo determinístico: herói vence se tiver a melhor mão
///   (empates contam como 1/N entre os empatados).
pub fn get_multiway_win_probability(
    hero_cards: &[Card],
    villain_hands: &[&[Card]],
    board_cards: &[Card],
) -> f64 {
    if villain_hands.is_empty() {
        return 1.0;
    }
    if villain_hands.len() == 1 {
        return get_heads_up_win_probability(hero_cards, villain_hands[0], board_cards);
    }

    let mut known_cards: Vec<Card> = hero_cards.to_vec();
    for hand in villain_hands {
        known_cards.extend_from_slice(hand);
    }
    known_cards.extend_from_slice(board_cards);

    let cards_to_deal = 5 - board_cards.len();
    if cards_to_deal == 0 {
        return multiway_outcome(hero_cards, villain_hands, board_cards);
    }

    let deck = get_remaining_deck(&known_cards);
    let mut rng = monte_carlo_rng(&known_cards);
    let max_boards = combinations_count(deck.len(), cards_to_deal);
    let samples = MC_SAMPLES.min(max_boards as u32) as usize;

    let mut equity_sum = 0.0_f64;
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut indices: Vec<usize> = (0..deck.len()).collect();

    while seen.len() < samples && seen.len() < max_boards {
        indices.shuffle(&mut rng);
        let board_idx: Vec<usize> = indices[..cards_to_deal].to_vec();
        let mut key_idx = board_idx.clone();
        key_idx.sort_unstable();
        let mut key = 0u64;
        for &i in &key_idx {
            key = key.wrapping_mul(67).wrapping_add(i as u64 + 1);
        }
        if !seen.insert(key) {
            continue;
        }

        let mut final_board = board_cards.to_vec();
        for &i in &board_idx {
            final_board.push(deck[i]);
        }
        equity_sum += multiway_outcome(hero_cards, villain_hands, &final_board);
    }

    equity_sum / seen.len() as f64
}

fn multiway_outcome(hero_cards: &[Card], villain_hands: &[&[Card]], board: &[Card]) -> f64 {
    multiway_outcome_for_variant(hero_cards, villain_hands, board, PokerVariant::Holdem)
}

fn multiway_outcome_for_variant(
    hero_cards: &[Card],
    villain_hands: &[&[Card]],
    board: &[Card],
    variant: PokerVariant,
) -> f64 {
    use crate::deck::{compare_hands, evaluate_hand, evaluate_hand_short_deck};
    use std::cmp::Ordering;

    let evaluate = if variant == PokerVariant::ShortDeck {
        evaluate_hand_short_deck
    } else {
        evaluate_hand
    };
    let hero_hand = evaluate(hero_cards, board);
    let mut best = hero_hand.clone();
    let mut tied = 1u32;

    for hand in villain_hands {
        let villain_hand = evaluate(hand, board);
        match compare_hands(&villain_hand, &best) {
            Ordering::Greater => {
                best = villain_hand;
                tied = 1;
            }
            Ordering::Equal => {
                tied += 1;
            }
            Ordering::Less => {}
        }
    }

    match compare_hands(&hero_hand, &best) {
        Ordering::Equal => 1.0 / f64::from(tied),
        Ordering::Less => 0.0,
        Ordering::Greater => 1.0, // hero sole best
    }
}

/// Equity para liquidação, com validação do baralho e ranking da modalidade.
///
/// - Hold’em: 2 cartas privadas; Monte Carlo determinístico ou exato.
/// - Short Deck: 2 cartas privadas, exato (até C(32, 5) = 201.376 boards HU pré-flop).
/// - Omaha 4: 4 cartas privadas, exatamente 2 + 3 (`two_plus_three_equity`).
/// - Brazilian Pineapple (`pineapple_draw`): 3 cartas privadas até o descarte do
///   river e 2 depois dele; vale a melhor mão de 5 entre a mesa e no máximo 2
///   privadas, com as cartas que cada jogador tinha no pagamento do all-in. Quem
///   está all-in não troca; as trocas que outros ainda possam fazer não entram no
///   cálculo (regra pública e determinística). Só o board é completado.
pub fn get_multiway_win_probability_for_variant(
    hero_cards: &[Card],
    villain_hands: &[&[Card]],
    board_cards: &[Card],
    variant: PokerVariant,
) -> Option<f64> {
    if !matches!(board_cards.len(), 0 | 3 | 4 | 5) {
        return None;
    }
    if matches!(variant, PokerVariant::Omaha | PokerVariant::PineappleDraw) {
        return two_plus_three_equity_for_variant(
            hero_cards,
            villain_hands,
            board_cards,
            variant,
            TWO_PLUS_THREE_MC_SAMPLES,
        );
    }
    if hero_cards.len() != 2 || villain_hands.iter().any(|hand| hand.len() != 2) {
        return None;
    }
    let full_deck = if variant == PokerVariant::ShortDeck {
        crate::deck::create_short_deck()
    } else {
        create_full_deck()
    };
    let mut known = hero_cards.to_vec();
    for hand in villain_hands {
        known.extend_from_slice(hand);
    }
    known.extend_from_slice(board_cards);
    if known
        .iter()
        .enumerate()
        .any(|(i, card)| !full_deck.contains(card) || known[..i].contains(card))
    {
        return None;
    }
    let deck: Vec<Card> = full_deck
        .into_iter()
        .filter(|c| !known.contains(c))
        .collect();
    if deck.len() < 5 - board_cards.len() {
        return None;
    }
    if variant == PokerVariant::Holdem {
        return Some(get_multiway_win_probability(
            hero_cards,
            villain_hands,
            board_cards,
        ));
    }
    if villain_hands.is_empty() {
        return Some(1.0);
    }
    let mut total = 0_u64;
    let mut equity = 0.0;
    visit_runouts(&deck, 0, &mut board_cards.to_vec(), &mut |board| {
        equity += multiway_outcome_for_variant(hero_cards, villain_hands, board, variant);
        total += 1;
    });
    Some(equity / total as f64)
}

/// Amostras Monte Carlo das modalidades 2 + 3 quando a enumeração exata não cabe.
///
/// Com amostras independentes, o erro padrão no pior caso (p = 0,5) é
/// 0,5 / √N: 120.000 amostras → margem de três desvios ≈ 0,43 ponto percentual.
/// A semente vem das cartas conhecidas, então a mesma mão sempre gera o mesmo
/// resultado auditável.
pub const TWO_PLUS_THREE_MC_SAMPLES: u64 = 120_000;

/// Até este número de desfechos equiprováveis, a equity 2 + 3 é exata.
pub const TWO_PLUS_THREE_EXACT_LIMIT: u64 = 250_000;

/// Valida cartas e calcula a equity de Omaha 4 ou Brazilian Pineapple.
/// `mc_samples` só é usado quando a enumeração exata não cabe no limite.
fn two_plus_three_equity_for_variant(
    hero_cards: &[Card],
    villain_hands: &[&[Card]],
    board_cards: &[Card],
    variant: PokerVariant,
    mc_samples: u64,
) -> Option<f64> {
    let hole_count = hero_cards.len();
    let extras = match variant {
        PokerVariant::Omaha => {
            if hole_count != 4 {
                return None;
            }
            0
        }
        PokerVariant::PineappleDraw => {
            if !matches!(hole_count, 2 | 3) {
                return None;
            }
            0
        }
        _ => return None,
    };
    let same_shape = |hand: &&[Card]| match variant {
        PokerVariant::PineappleDraw => matches!(hand.len(), 2 | 3),
        _ => hand.len() == hole_count,
    };
    if !villain_hands.iter().all(same_shape) {
        return None;
    }
    let score: HandScore = if variant == PokerVariant::PineappleDraw {
        best_with_up_to_two_hole
    } else {
        best_two_plus_three_on_board
    };

    let full_deck = create_full_deck();
    let mut known = hero_cards.to_vec();
    for hand in villain_hands {
        known.extend_from_slice(hand);
    }
    known.extend_from_slice(board_cards);
    if known
        .iter()
        .enumerate()
        .any(|(i, card)| !full_deck.contains(card) || known[..i].contains(card))
    {
        return None;
    }
    if villain_hands.is_empty() {
        return Some(1.0);
    }
    let deck: Vec<Card> = full_deck
        .into_iter()
        .filter(|c| !known.contains(c))
        .collect();
    let players = 1 + villain_hands.len();
    let board_needed = 5 - board_cards.len();
    if deck.len() < board_needed + extras * players {
        return None;
    }

    let mut hands: Vec<Vec<Card>> = Vec::with_capacity(players);
    hands.push(hero_cards.to_vec());
    hands.extend(villain_hands.iter().map(|hand| hand.to_vec()));
    Some(two_plus_three_equity(
        &hands,
        board_cards,
        extras,
        &deck,
        &known,
        mc_samples.max(1),
        score,
    ))
}

/// Número de desfechos equiprováveis: completar o board e, depois, entregar
/// `extras` cartas a cada jogador, sem reposição. `None` se exceder `u64`.
pub fn two_plus_three_outcome_count(
    deck_len: usize,
    board_needed: usize,
    extras: usize,
    players: usize,
) -> Option<u64> {
    let mut remaining = deck_len;
    let mut total = combinations_count(remaining, board_needed) as u64;
    remaining = remaining.checked_sub(board_needed)?;
    for _ in 0..players {
        total = total.checked_mul(combinations_count(remaining, extras) as u64)?;
        remaining = remaining.checked_sub(extras)?;
    }
    Some(total)
}

/// Equity do jogador `hands[0]` (vitória + fração dos empates) em uma
/// modalidade de exatamente 2 privadas + 3 comunitárias com ranking clássico.
fn two_plus_three_equity(
    hands: &[Vec<Card>],
    board_cards: &[Card],
    extras: usize,
    deck: &[Card],
    known: &[Card],
    mc_samples: u64,
    score: HandScore,
) -> f64 {
    let players = hands.len();
    let board_needed = 5 - board_cards.len();
    let outcomes = two_plus_three_outcome_count(deck.len(), board_needed, extras, players);

    if outcomes.is_some_and(|count| count <= TWO_PLUS_THREE_EXACT_LIMIT) {
        let mut equity_sum = 0.0_f64;
        let mut total = 0_u64;
        let mut board = board_cards.to_vec();
        let mut finals: Vec<Vec<Card>> = hands.to_vec();
        let mut used = vec![false; deck.len()];
        enumerate_board(
            deck,
            0,
            board_needed,
            &mut used,
            &mut board,
            &mut |board, used| {
                let mut walk = ExtrasWalk { deck, used, extras };
                walk.visit(0, 0, extras, &mut finals, &mut |finals| {
                    equity_sum += two_plus_three_share(finals, board, score);
                    total += 1;
                });
            },
        );
        return equity_sum / total as f64;
    }

    let mut rng = monte_carlo_rng(known);
    let draw = board_needed + extras * players;
    let mut pool = deck.to_vec();
    let mut equity_sum = 0.0_f64;
    let mut finals: Vec<Vec<Card>> = hands.to_vec();
    let mut board = Vec::with_capacity(5);
    for _ in 0..mc_samples {
        // Fisher-Yates parcial: as `draw` primeiras posições formam uma
        // amostra uniforme sem reposição do baralho restante.
        for slot in 0..draw {
            let pick = rand::Rng::gen_range(&mut rng, slot..pool.len());
            pool.swap(slot, pick);
        }
        board.clear();
        board.extend_from_slice(board_cards);
        board.extend_from_slice(&pool[..board_needed]);
        let mut cursor = board_needed;
        for (final_hand, original) in finals.iter_mut().zip(hands) {
            final_hand.truncate(original.len());
            final_hand.extend_from_slice(&pool[cursor..cursor + extras]);
            cursor += extras;
        }
        equity_sum += two_plus_three_share(&finals, &board, score);
    }
    equity_sum / mc_samples as f64
}

/// Enumera todas as combinações de cartas que completam o board.
fn enumerate_board(
    deck: &[Card],
    start: usize,
    needed: usize,
    used: &mut [bool],
    board: &mut Vec<Card>,
    visit: &mut impl FnMut(&[Card], &mut [bool]),
) {
    if needed == 0 {
        visit(board, used);
        return;
    }
    for i in start..=deck.len() - needed {
        used[i] = true;
        board.push(deck[i]);
        enumerate_board(deck, i + 1, needed - 1, used, board, visit);
        board.pop();
        used[i] = false;
    }
}

/// Enumera, jogador a jogador, todas as combinações de `extras` cartas privadas
/// ainda não usadas pelo board nem pelos jogadores anteriores.
struct ExtrasWalk<'a> {
    deck: &'a [Card],
    used: &'a mut [bool],
    extras: usize,
}

impl ExtrasWalk<'_> {
    fn visit(
        &mut self,
        player: usize,
        start: usize,
        remaining: usize,
        finals: &mut Vec<Vec<Card>>,
        on_outcome: &mut impl FnMut(&[Vec<Card>]),
    ) {
        if player == finals.len() {
            on_outcome(finals);
            return;
        }
        if remaining == 0 {
            // Próximo jogador recebe a mesma quantidade de extras.
            self.visit(player + 1, 0, self.extras, finals, on_outcome);
            return;
        }
        for i in start..self.deck.len() {
            if self.used[i] {
                continue;
            }
            self.used[i] = true;
            finals[player].push(self.deck[i]);
            self.visit(player, i + 1, remaining - 1, finals, on_outcome);
            finals[player].pop();
            self.used[i] = false;
        }
    }
}

/// Fração do pote do jogador `finals[0]`: 1 se for o único melhor, 1/k em empate
/// entre k jogadores, 0 se perder.
fn two_plus_three_share(finals: &[Vec<Card>], board: &[Card], score: HandScore) -> f64 {
    let hero = score(&finals[0], board);
    let mut tied = 1_u32;
    for hand in &finals[1..] {
        let rival = score(hand, board);
        if rival > hero {
            return 0.0;
        }
        if rival == hero {
            tied += 1;
        }
    }
    1.0 / f64::from(tied)
}

/// Pontuação da mão de um jogador com a mesa completa (5 cartas).
type HandScore = fn(&[Card], &[Card]) -> u32;

/// Omaha 4: exatamente 2 privadas + 3 comunitárias.
fn best_two_plus_three_on_board(hole: &[Card], board: &[Card]) -> u32 {
    best_two_plus_three(hole, &board_triples(board))
}

/// Brazilian Pineapple: melhor mão de 5 entre a mesa e no máximo 2 privadas.
/// Com 3 privadas, vale o melhor par (o descarte ótimo do river).
fn best_with_up_to_two_hole(hole: &[Card], board: &[Card]) -> u32 {
    let mut best = 0;
    let mut seven = [board[0]; 7];
    seven[..5].copy_from_slice(&board[..5]);
    let mut consider = |a: Card, b: Card| {
        seven[5] = a;
        seven[6] = b;
        for skip_a in 0..7 {
            for skip_b in (skip_a + 1)..7 {
                let mut five = [seven[0]; 5];
                let mut n = 0;
                for (i, card) in seven.iter().enumerate() {
                    if i != skip_a && i != skip_b {
                        five[n] = *card;
                        n += 1;
                    }
                }
                let score = classic_five_card_score(&five);
                if score > best {
                    best = score;
                }
            }
        }
    };
    for i in 0..hole.len() {
        for j in (i + 1)..hole.len() {
            consider(hole[i], hole[j]);
        }
    }
    best
}

fn board_triples(board: &[Card]) -> [[Card; 3]; 10] {
    let mut triples = [[board[0]; 3]; 10];
    let mut n = 0;
    for a in 0..5 {
        for b in (a + 1)..5 {
            for c in (b + 1)..5 {
                triples[n] = [board[a], board[b], board[c]];
                n += 1;
            }
        }
    }
    triples
}

/// Melhor pontuação usando exatamente 2 privadas e 3 comunitárias.
fn best_two_plus_three(hole: &[Card], triples: &[[Card; 3]; 10]) -> u32 {
    let mut best = 0;
    for i in 0..hole.len() {
        for j in (i + 1)..hole.len() {
            for triple in triples {
                let score =
                    classic_five_card_score(&[hole[i], hole[j], triple[0], triple[1], triple[2]]);
                if score > best {
                    best = score;
                }
            }
        }
    }
    best
}

/// Pontuação total de exatamente 5 cartas no ranking clássico: maior vence,
/// igual empata. Coerente com `compare_hands(evaluate_hand(..), ..)` para 5
/// cartas (teste `classic_five_card_score_matches_evaluator`).
pub fn classic_five_card_score(cards: &[Card; 5]) -> u32 {
    let mut ranks = [0_u8; 5];
    for (slot, card) in ranks.iter_mut().zip(cards) {
        *slot = card.rank as u8;
    }
    ranks.sort_unstable_by(|a, b| b.cmp(a));
    let flush = cards.iter().all(|card| card.suit == cards[0].suit);
    let distinct = ranks.windows(2).all(|pair| pair[0] != pair[1]);
    let straight_high = if distinct && ranks[0] - ranks[4] == 4 {
        Some(ranks[0])
    } else if distinct && ranks == [14, 5, 4, 3, 2] {
        Some(5)
    } else {
        None
    };

    // Grupos (quantidade, rank) sem alocação: os ranks já estão em ordem
    // decrescente; a ordenação estável por quantidade preserva o rank maior
    // primeiro entre grupos do mesmo tamanho.
    let mut groups = [(0_u8, 0_u8); 5];
    let mut group_count = 0;
    for &rank in &ranks {
        if group_count > 0 && groups[group_count - 1].1 == rank {
            groups[group_count - 1].0 += 1;
        } else {
            groups[group_count] = (1, rank);
            group_count += 1;
        }
    }
    for i in 1..group_count {
        let mut j = i;
        while j > 0 && groups[j].0 > groups[j - 1].0 {
            groups.swap(j, j - 1);
            j -= 1;
        }
    }
    let shape = (groups[0].0, groups[1].0);

    let (category, by_group) = match (straight_high, flush) {
        (Some(14), true) => return (10 << 20) | (14 << 16),
        (Some(high), true) => return (9 << 20) | (u32::from(high) << 16),
        _ if shape == (4, 1) => (8, true),
        _ if shape == (3, 2) => (7, true),
        (_, true) => (6, false),
        (Some(high), false) => return (5 << 20) | (u32::from(high) << 16),
        _ if shape == (3, 1) => (4, true),
        _ if shape == (2, 2) => (3, true),
        _ if shape == (2, 1) => (2, true),
        _ => (1, false),
    };
    let mut score = category << 20;
    if by_group {
        for (position, (_, rank)) in groups[..group_count].iter().enumerate() {
            score |= u32::from(*rank) << (16 - 4 * position as u32);
        }
    } else {
        for (position, rank) in ranks.iter().enumerate() {
            score |= u32::from(*rank) << (16 - 4 * position as u32);
        }
    }
    score
}

fn visit_runouts(
    deck: &[Card],
    start: usize,
    board: &mut Vec<Card>,
    visit: &mut impl FnMut(&[Card]),
) {
    let needed = 5 - board.len();
    if needed == 0 {
        visit(board);
        return;
    }
    for i in start..=deck.len() - needed {
        board.push(deck[i]);
        visit_runouts(deck, i + 1, board, visit);
        board.pop();
    }
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

/// Conta C(n, k) sem estourar para os tamanhos usados aqui (n <= 52, k <= 5).
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
/// p = 0.5), a aproximação do erro padrão para população grande é:
///
///   SE = 0.5 · √( (1 - f) / samples ),   onde f = samples / max_boards
///
/// - Se `samples >= max_boards` (população toda coberta): erro 0 (exato).
/// - Caso contrário aplica o fator de correção de população finita (1 - f),
///   que deixa o erro MENOR que o Monte Carlo com reposição.
///
/// O valor retornado é três vezes esse erro padrão. A interpretação de
/// ~99,7% depende da aproximação normal; não é um limite rigoroso por mão:
///
///   bound = 3 · SE
///
/// Exemplo: 500k amostras entre C(48,5) = 1.712.304 boards →
///   f ≈ 0,292, SE ≈ 0,000595, margem ≈ 0,001785 (0,1785 ponto percentual).
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
            (0.999_999, Some(LossDeflatorTier::ThirtyFivePercent)),
            (1.0, None),
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
    fn test_equity_one_hundred_percent_returns_none() {
        // 100% = sem perdedor possível: quem tem 100% não perde o runout.
        assert_eq!(LossDeflatorTier::from_loser_equity(1.0), None);
        let pots = vec![make_pot(20000, vec!["loser", "winner"])];
        let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots,
            loser_id: "loser".into(),
            winner_id: "winner".into(),
            phase: GamePhase::River,
            loser_equity: 1.0,
        });
        assert!(result.is_none());
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

#[cfg(test)]
mod two_plus_three_tests {
    use super::*;
    use crate::deck::{
        compare_hands, evaluate_hand, evaluate_hand_omaha, evaluate_hand_pineapple_draw, Rank, Suit,
    };
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};
    use std::cmp::Ordering;

    fn c(text: &str) -> Card {
        let mut chars = text.chars();
        let rank = match chars.next().unwrap() {
            'A' => Rank::Ace,
            'K' => Rank::King,
            'Q' => Rank::Queen,
            'J' => Rank::Jack,
            'T' => Rank::Ten,
            n => Rank::try_from(n.to_digit(10).unwrap() as u8).unwrap(),
        };
        let suit = match chars.next().unwrap() {
            'h' => Suit::Hearts,
            'd' => Suit::Diamonds,
            'c' => Suit::Clubs,
            _ => Suit::Spades,
        };
        Card { rank, suit }
    }

    fn cards(text: &str) -> Vec<Card> {
        text.split_whitespace().map(c).collect()
    }

    fn five(text: &str) -> [Card; 5] {
        cards(text).try_into().unwrap()
    }

    fn assert_same_order(a: &[Card; 5], b: &[Card; 5]) {
        let fast = classic_five_card_score(a).cmp(&classic_five_card_score(b));
        let slow = compare_hands(&evaluate_hand(a, &[]), &evaluate_hand(b, &[]));
        assert_eq!(fast, slow, "{a:?} x {b:?}");
    }

    #[test]
    fn classic_five_card_score_matches_evaluator_on_edge_cases() {
        let samples = [
            "As Ks Qs Js Ts", // royal
            "Kh Qh Jh Th 9h", // straight flush
            "6d 5d 4d 3d 2d", // straight flush 6-high
            "5c 4c 3c 2c Ac", // steel wheel
            "9s 9h 9d 9c Ah", // quadra com kicker
            "9s 9h 9d 9c Kh",
            "Ks Kh Kd 2c 2h", // full house
            "Qs Qh Qd Ac Ah",
            "Ah Jh 8h 6h 3h", // flush
            "Ah Jh 8h 6h 2h",
            "Ts 9h 8d 7c 6s", // sequência
            "6s 5h 4d 3c 2s",
            "5s 4h 3d 2c Ah", // wheel
            "7s 7h 7d Ac 2h", // trinca
            "Js Jh 4d 4c Ah", // dois pares
            "Js Jh 4d 4c Kh",
            "Js Jh 3d 3c Ah",
            "As Ah Kd Qc 2h", // par
            "As Ah Kd Qc Jh",
            "Ks Qh 9d 5c 3h", // carta alta
            "Ks Qh 9d 5c 2h",
        ];
        let hands: Vec<[Card; 5]> = samples.iter().map(|text| five(text)).collect();
        for a in &hands {
            for b in &hands {
                assert_same_order(a, b);
            }
        }
        // Naipes diferentes, mesmos ranks: empate.
        assert_same_order(&five("As Kd Qh Jc 9s"), &five("Ad Ks Qc Jh 9d"));
    }

    #[test]
    fn classic_five_card_score_matches_evaluator_on_random_hands() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(20_261_108);
        let mut pool = create_full_deck();
        let mut sample: Vec<[Card; 5]> = Vec::with_capacity(4_000);
        for _ in 0..4_000 {
            pool.shuffle(&mut rng);
            sample.push(pool[..5].try_into().unwrap());
        }
        sample.sort_by_key(classic_five_card_score);
        for pair in sample.windows(2) {
            assert_same_order(&pair[0], &pair[1]);
        }
        for _ in 0..4_000 {
            let a = &sample[rng.gen_range(0..sample.len())];
            let b = &sample[rng.gen_range(0..sample.len())];
            assert_same_order(a, b);
        }
    }

    /// Conferência exaustiva das 2.598.960 mãos de 5 cartas (lenta em debug).
    #[test]
    #[ignore = "exaustivo: rodar com --release -- --ignored"]
    fn classic_five_card_score_matches_evaluator_exhaustively() {
        let deck = create_full_deck();
        let mut all = Vec::with_capacity(2_598_960);
        for a in 0..48 {
            for b in (a + 1)..49 {
                for x in (b + 1)..50 {
                    for d in (x + 1)..51 {
                        for e in (d + 1)..52 {
                            all.push([deck[a], deck[b], deck[x], deck[d], deck[e]]);
                        }
                    }
                }
            }
        }
        assert_eq!(all.len(), 2_598_960);
        all.sort_by_key(classic_five_card_score);
        for pair in all.windows(2) {
            assert_same_order(&pair[0], &pair[1]);
        }
    }

    #[test]
    fn best_two_plus_three_matches_variant_evaluators() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(7_031);
        let mut pool = create_full_deck();
        for round in 0..400 {
            pool.shuffle(&mut rng);
            // Omaha com 4 privadas; Brazilian Pineapple com 3 (antes do
            // descarte) e com 2 (depois dele).
            let holes = [4, 3, 2][round % 3];
            let a = &pool[..holes];
            let b = &pool[holes..2 * holes];
            let board = &pool[2 * holes..2 * holes + 5];
            let (fast, slow) = if holes == 4 {
                let triples = board_triples(board);
                (
                    best_two_plus_three(a, &triples).cmp(&best_two_plus_three(b, &triples)),
                    compare_hands(
                        &evaluate_hand_omaha(a, board),
                        &evaluate_hand_omaha(b, board),
                    ),
                )
            } else {
                (
                    best_with_up_to_two_hole(a, board).cmp(&best_with_up_to_two_hole(b, board)),
                    compare_hands(
                        &evaluate_hand_pineapple_draw(a, board),
                        &evaluate_hand_pineapple_draw(b, board),
                    ),
                )
            };
            assert_eq!(fast, slow, "rodada {round}");
        }
    }

    #[test]
    fn omaha_flop_equity_is_exact() {
        let hero = cards("Ah Ad Kc Qc");
        let villain = cards("Js Ts 9h 8h");
        let board = cards("7h 6s 2d");
        let known: Vec<Card> = hero.iter().chain(&villain).chain(&board).copied().collect();
        let deck: Vec<Card> = create_full_deck()
            .into_iter()
            .filter(|card| !known.contains(card))
            .collect();
        let mut expected = 0.0;
        let mut total = 0;
        for i in 0..deck.len() {
            for j in (i + 1)..deck.len() {
                let mut full = board.clone();
                full.push(deck[i]);
                full.push(deck[j]);
                expected += match compare_hands(
                    &evaluate_hand_omaha(&hero, &full),
                    &evaluate_hand_omaha(&villain, &full),
                ) {
                    Ordering::Greater => 1.0,
                    Ordering::Equal => 0.5,
                    Ordering::Less => 0.0,
                };
                total += 1;
            }
        }
        // 52 − 11 cartas conhecidas = 41 restantes → C(41, 2) turn/river.
        assert_eq!(total, 820);
        let got = get_multiway_win_probability_for_variant(
            &hero,
            &[&villain],
            &board,
            PokerVariant::Omaha,
        )
        .unwrap();
        assert!((got - expected / f64::from(total)).abs() < 1e-12);
    }

    #[test]
    fn pineapple_draw_turn_equity_enumerates_only_the_river() {
        let hero = cards("Ah Kh Qd");
        let villain = cards("Js Jd 9c");
        let board = cards("Jh 7h 3s 4d");
        let known: Vec<Card> = hero.iter().chain(&villain).chain(&board).copied().collect();
        let deck: Vec<Card> = create_full_deck()
            .into_iter()
            .filter(|card| !known.contains(card))
            .collect();
        // Sem extras: só o river completa o board (52 − 10 = 42 desfechos).
        assert_eq!(two_plus_three_outcome_count(deck.len(), 1, 0, 2), Some(42));
        let mut expected = 0.0;
        for river in &deck {
            let mut full = board.clone();
            full.push(*river);
            expected += match compare_hands(
                &evaluate_hand_pineapple_draw(&hero, &full),
                &evaluate_hand_pineapple_draw(&villain, &full),
            ) {
                Ordering::Greater => 1.0,
                Ordering::Equal => 0.5,
                Ordering::Less => 0.0,
            };
        }
        let got = get_multiway_win_probability_for_variant(
            &hero,
            &[&villain],
            &board,
            PokerVariant::PineappleDraw,
        )
        .unwrap();
        assert!((got - expected / 42.0).abs() < 1e-12);
    }

    #[test]
    fn pineapple_draw_flop_equity_is_exact() {
        let hero = cards("As Ad 7c");
        let villain = cards("Ks Qs Js");
        let board = cards("Ts 4h 2d");
        let known: Vec<Card> = hero.iter().chain(&villain).chain(&board).copied().collect();
        let deck: Vec<Card> = create_full_deck()
            .into_iter()
            .filter(|card| !known.contains(card))
            .collect();
        let mut expected = 0.0;
        let mut total = 0;
        for i in 0..deck.len() {
            for j in (i + 1)..deck.len() {
                let mut full = board.clone();
                full.push(deck[i]);
                full.push(deck[j]);
                expected += match compare_hands(
                    &evaluate_hand_pineapple_draw(&hero, &full),
                    &evaluate_hand_pineapple_draw(&villain, &full),
                ) {
                    Ordering::Greater => 1.0,
                    Ordering::Equal => 0.5,
                    Ordering::Less => 0.0,
                };
                total += 1;
            }
        }
        // 52 − 9 cartas conhecidas = 43 restantes → C(43, 2) turn/river.
        assert_eq!(total, 903);
        let got = get_multiway_win_probability_for_variant(
            &hero,
            &[&villain],
            &board,
            PokerVariant::PineappleDraw,
        )
        .unwrap();
        assert!((got - expected / f64::from(total)).abs() < 1e-12);
    }

    /// Amostra reduzida para a rotina da CI (debug); a amostra cheia de
    /// produção fica nos testes da feature `massive-tests` abaixo.
    const QUICK_SAMPLES: u64 = 4_000;

    fn quick_equity(hero: &[Card], villain: &[Card], variant: PokerVariant) -> f64 {
        two_plus_three_equity_for_variant(hero, &[villain], &[], variant, QUICK_SAMPLES).unwrap()
    }

    #[test]
    fn pineapple_draw_preflop_monte_carlo_is_deterministic_and_consistent() {
        let hero = cards("As Ks 2c");
        let villain = cards("Ad Kd 2h");
        let pineapple = PokerVariant::PineappleDraw;
        let first = quick_equity(&hero, &villain, pineapple);
        assert_eq!(
            first,
            quick_equity(&hero, &villain, pineapple),
            "mesma mão, mesma estimativa"
        );
        // Mãos espelhadas: cada lado perto de 50%, e os dois lados somam ~100%
        // (margem de três desvios com 4.000 amostras ≈ 2,4 pontos).
        assert!((first - 0.5).abs() < 0.03, "{first}");
        assert!((first + quick_equity(&villain, &hero, pineapple) - 1.0).abs() < 0.05);
    }

    #[test]
    fn omaha_preflop_favourite_is_clear() {
        // AAKK de dois naipes contra mão baixa: favorito, mas longe do Hold’em
        // (no Omaha as equities pré-flop são bem mais próximas).
        let equity = quick_equity(
            &cards("As Ah Ks Kh"),
            &cards("7c 2d 8s 3c"),
            PokerVariant::Omaha,
        );
        assert!(equity > 0.6 && equity < 0.8, "{equity}");
    }

    /// Amostra cheia (120.000), igual à liquidação. Lento em debug.
    #[test]
    #[cfg_attr(
        not(feature = "massive-tests"),
        ignore = "Monte Carlo pré-flop com amostra cheia; habilite a feature massive-tests"
    )]
    fn full_sample_preflop_equity_matches_quick_estimate() {
        for (hero, villain, variant) in [
            ("As Ah Ks Kh", "7c 2d 8s 3c", PokerVariant::Omaha),
            ("As Ks 2c", "Ad Kd 2h", PokerVariant::PineappleDraw),
        ] {
            let (hero, villain) = (cards(hero), cards(villain));
            let full =
                get_multiway_win_probability_for_variant(&hero, &[&villain], &[], variant).unwrap();
            let quick = quick_equity(&hero, &villain, variant);
            assert!((full - quick).abs() < 0.03, "{variant:?}: {full} x {quick}");
            if variant == PokerVariant::PineappleDraw {
                assert!((full - 0.5).abs() < 0.01, "{full}");
            }
        }
    }

    #[test]
    fn two_plus_three_rejects_invalid_shapes() {
        let omaha = cards("As Ah Ks Kh");
        let other = cards("7c 2d 8s 3c");
        let flop = cards("Qd Jd Td");
        let equity = |hero: &[Card], villain: &[Card], board: &[Card], variant| {
            get_multiway_win_probability_for_variant(hero, &[villain], board, variant)
        };
        assert!(equity(&cards("As Ah"), &other, &[], PokerVariant::Omaha).is_none());
        assert!(equity(&omaha, &omaha, &flop, PokerVariant::Omaha).is_none());
        assert!(equity(&omaha, &other, &flop, PokerVariant::Omaha).is_some());
        // Brazilian Pineapple: 3 privadas até o descarte do river, 2 depois.
        let pineapple = PokerVariant::PineappleDraw;
        assert!(equity(&cards("As"), &cards("7c 2d 9h"), &flop, pineapple).is_none());
        assert!(equity(&cards("As Ah 5c"), &cards("7c"), &flop, pineapple).is_none());
        assert!(equity(
            &cards("As Ah 5c 6c"),
            &cards("7c 2d 9h 8d"),
            &flop,
            pineapple
        )
        .is_none());
        assert!(equity(&cards("As Ah 5c"), &cards("7c 2d 9h"), &flop, pineapple).is_some());
        let river = cards("Qd Jd Td 4s 3s");
        let at_river = equity(&cards("As Ah 5c"), &cards("Kc 9h Kh"), &river, pineapple);
        // Vilão faz K-Q-J-T-9 (Kc 9h + Q J T); o herói fica com par de ases.
        assert_eq!(at_river, Some(0.0));
        // Depois do descarte: quatro ouros na mesa e um só na mão já é flush.
        let four_diamonds = cards("Qd Jd Td 4d 3s");
        let after_discard = equity(&cards("2d 9c"), &cards("As Ah"), &four_diamonds, pineapple);
        assert_eq!(after_discard, Some(1.0));
    }
}
