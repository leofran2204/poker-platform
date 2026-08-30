// deck.rs — Motor de baralho: criação, embaralhamento, avaliação de mãos (Texas Hold'em)
// Migrado de TypeScript (deck.ts) para Rust em 2026-07-02

use crate::rng_crypto::csprng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

// ─── Tipos (enums + structs) ───

/// Naipes do baralho (4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

/// Valores das cartas (13 ranks, A=14, 2=2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

/// Uma carta do baralho
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

/// Hierarquia de mãos (da mais fraca para a mais forte)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HandRank {
    HighCard = 1,
    OnePair = 2,
    TwoPair = 3,
    ThreeOfAKind = 4,
    Straight = 5,
    Flush = 6,
    FullHouse = 7,
    FourOfAKind = 8,
    StraightFlush = 9,
    RoyalFlush = 10,
}

/// Resultado da avaliação de uma mão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandResult {
    pub rank: HandRank,
    pub cards: Vec<Card>,   // cartas que formam a mão principal
    pub kickers: Vec<Card>, // cartas de desempate
    pub value: u8,          // valor numérico para comparação rápida
}

// ─── Constantes ───

/// Todos os ranks em ordem decrescente (A=14 até 2=2)
const ALL_RANKS: [Rank; 13] = [
    Rank::Ace,
    Rank::King,
    Rank::Queen,
    Rank::Jack,
    Rank::Ten,
    Rank::Nine,
    Rank::Eight,
    Rank::Seven,
    Rank::Six,
    Rank::Five,
    Rank::Four,
    Rank::Three,
    Rank::Two,
];

/// Todos os naipes
const ALL_SUITS: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];

// ─── Funções públicas (API do módulo) ───

/// Cria um baralho completo de 52 cartas (4 naipes × 13 ranks)
pub fn create_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    for &suit in &ALL_SUITS {
        for &rank in &ALL_RANKS {
            deck.push(Card { rank, suit });
        }
    }
    deck
}

/// Embaralha o baralho usando Fisher-Yates (via CSPRNG — criptograficamente seguro)
/// Retorna um NOVO Vec — o original não é modificado (imutabilidade Rust)
pub fn shuffle_deck(deck: &[Card]) -> Vec<Card> {
    let mut shuffled = deck.to_vec();
    shuffled.shuffle(&mut csprng());
    shuffled
}

/// Distribui `count` cartas do topo do baralho
/// Retorna as cartas distribuídas + o baralho restante
pub fn deal_cards(deck: &[Card], count: usize) -> (Vec<Card>, Vec<Card>) {
    let cards = deck[..count.min(deck.len())].to_vec();
    let remaining = deck[count.min(deck.len())..].to_vec();
    (cards, remaining)
}

/// Avalia a melhor mão de 5 cartas entre as cartas do jogador + comunitárias
pub fn evaluate_hand(hole_cards: &[Card], community_cards: &[Card]) -> HandResult {
    let mut all_cards = Vec::with_capacity(hole_cards.len() + community_cards.len());
    all_cards.extend_from_slice(hole_cards);
    all_cards.extend_from_slice(community_cards);

    if all_cards.is_empty() {
        return HandResult {
            rank: HandRank::HighCard,
            cards: vec![],
            kickers: vec![],
            value: 0,
        };
    }

    // FASE 1: Integração com Evaluator SIMD (rs-poker)
    /*
    use rs_poker::core::{Hand, Card as RsCard, Suit as RsSuit, Value as RsValue};
    // Transforma as cartas para o formato da rs_poker
    let rs_cards: Vec<RsCard> = all_cards.iter().map(|c| {
        // Mapear c.rank e c.suit para rs_poker format...
        RsCard::new(RsValue::Two, RsSuit::Spades)
    }).collect();
    let hand = Hand::new_with_cards(rs_cards);
    let eval_score = hand.rank();
    // Converter eval_score devolta para HandResult para preservar retrocompatibilidade...
    */

    // Ordenação decrescente: maior rank primeiro. Desempata por naipe.
    all_cards.sort_by(|a, b| b.rank.cmp(&a.rank).then_with(|| b.suit.cmp(&a.suit)));

    let mut suit_counts = [0; 4];
    let mut has_flush_suit = false;
    for card in &all_cards {
        let idx = match card.suit {
            Suit::Hearts => 0,
            Suit::Diamonds => 1,
            Suit::Clubs => 2,
            Suit::Spades => 3,
        };
        suit_counts[idx] += 1;
        if suit_counts[idx] >= 5 {
            has_flush_suit = true;
        }
    }

    // Verifica da mão mais forte para a mais fraca com short-circuit
    if has_flush_suit {
        if let Some(result) = get_straight_flush(&all_cards) {
            return result;
        }
    }
    if let Some(result) = get_four_of_a_kind(&all_cards) {
        return result;
    }
    if let Some(result) = get_full_house(&all_cards) {
        return result;
    }
    if has_flush_suit {
        if let Some(result) = get_flush(&all_cards) {
            return result;
        }
    }
    if let Some(result) = get_straight(&all_cards) {
        return result;
    }
    if let Some(result) = get_three_of_a_kind(&all_cards) {
        return result;
    }
    if let Some(result) = get_two_pair(&all_cards) {
        return result;
    }
    if let Some(result) = get_one_pair(&all_cards) {
        return result;
    }
    if let Some(result) = get_high_card(&all_cards) {
        return result;
    }

    unreachable!("get_high_card always returns Some for any non-empty hand");
}

/// Compara duas mãos. Retorna Ordering::Greater se A ganha, Less se B ganha, Equal se empate.
pub fn compare_hands(a: &HandResult, b: &HandResult) -> Ordering {
    // Compara valor base primeiro
    match a.value.cmp(&b.value) {
        Ordering::Equal => {}
        other => return other,
    }

    // Desempate pelas cartas principais
    for (card_a, card_b) in a.cards.iter().zip(b.cards.iter()) {
        match card_a.rank.cmp(&card_b.rank) {
            Ordering::Equal => {}
            other => return other,
        }
    }

    // Desempate pelos kickers
    for (card_a, card_b) in a.kickers.iter().zip(b.kickers.iter()) {
        match card_a.rank.cmp(&card_b.rank) {
            Ordering::Equal => {}
            other => return other,
        }
    }

    Ordering::Equal
}

/// Retorna o nome legível da mão
pub fn get_hand_name(rank: HandRank) -> &'static str {
    match rank {
        HandRank::HighCard => "High Card",
        HandRank::OnePair => "One Pair",
        HandRank::TwoPair => "Two Pair",
        HandRank::ThreeOfAKind => "Three of a Kind",
        HandRank::Straight => "Straight",
        HandRank::Flush => "Flush",
        HandRank::FullHouse => "Full House",
        HandRank::FourOfAKind => "Four of a Kind",
        HandRank::StraightFlush => "Straight Flush",
        HandRank::RoyalFlush => "Royal Flush",
    }
}

// ─── Funções auxiliares públicas (usadas por outros módulos) ───

/// Cria um baralho completo de 52 cartas (nome legado para compatibilidade)
pub fn create_full_deck() -> Vec<Card> {
    create_deck()
}

/// Ranks Short Deck (Six Plus): 6–A (36 cartas)
const SHORT_DECK_RANKS: [Rank; 9] = [
    Rank::Ace,
    Rank::King,
    Rank::Queen,
    Rank::Jack,
    Rank::Ten,
    Rank::Nine,
    Rank::Eight,
    Rank::Seven,
    Rank::Six,
];

/// Baralho Short Deck (36 cartas: remove 2–5)
pub fn create_short_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(36);
    for &suit in &ALL_SUITS {
        for &rank in &SHORT_DECK_RANKS {
            deck.push(Card { rank, suit });
        }
    }
    deck
}

/// Força numérica Short Deck (Flush > Full House)
fn short_deck_rank_value(rank: HandRank) -> u8 {
    match rank {
        HandRank::HighCard => 1,
        HandRank::OnePair => 2,
        HandRank::TwoPair => 3,
        HandRank::ThreeOfAKind => 4,
        HandRank::Straight => 5,
        HandRank::FullHouse => 6, // below flush
        HandRank::Flush => 7,
        HandRank::FourOfAKind => 8,
        HandRank::StraightFlush => 9,
        HandRank::RoyalFlush => 10,
    }
}

fn has_straight_short(cards: &[Card]) -> Option<Rank> {
    let mut values: Vec<u8> = cards.iter().map(|c| c.rank as u8).collect();
    values.sort_unstable();
    values.dedup();

    let mut best_high: Option<Rank> = None;

    // Wheel Short Deck: A-6-7-8-9 (high = 9)
    if values.contains(&14)
        && values.contains(&6)
        && values.contains(&7)
        && values.contains(&8)
        && values.contains(&9)
    {
        best_high = Some(Rank::Nine);
    }

    for window in values.windows(5) {
        if window[4] - window[0] == 4 {
            let high = Rank::try_from(window[4]).ok();
            match (best_high, high) {
                (None, h) => best_high = h,
                (Some(current), Some(h)) if h as u8 > current as u8 => best_high = Some(h),
                _ => {}
            }
        }
    }
    best_high
}

fn build_straight_cards_short(cards: &[Card], high: Rank) -> Vec<Card> {
    if high == Rank::Nine {
        // Prefer wheel A-6-7-8-9 when high is Nine and Ace present with 6-9
        let mut vals: Vec<u8> = cards.iter().map(|c| c.rank as u8).collect();
        vals.sort_unstable();
        vals.dedup();
        if vals.contains(&14) && vals.contains(&6) && vals.contains(&7) && vals.contains(&8) {
            let needed = [Rank::Ace, Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six];
            let mut result = Vec::new();
            for &r in &needed {
                if let Some(c) = cards.iter().find(|c| c.rank == r) {
                    result.push(*c);
                }
            }
            if result.len() == 5 {
                return result;
            }
        }
    }
    build_straight_cards(cards, high)
}

fn get_straight_flush_short(cards: &[Card]) -> Option<HandResult> {
    let suit_counts = get_suit_counts(cards);
    for (&suit, &count) in &suit_counts {
        if count >= 5 {
            let suited: Vec<Card> = cards.iter().filter(|c| c.suit == suit).copied().collect();
            if let Some(high) = has_straight_short(&suited) {
                let is_royal = high == Rank::Ace;
                let rank = if is_royal {
                    HandRank::RoyalFlush
                } else {
                    HandRank::StraightFlush
                };
                let straight_cards = build_straight_cards_short(&suited, high);
                return Some(HandResult {
                    rank,
                    cards: straight_cards,
                    kickers: vec![],
                    value: short_deck_rank_value(rank),
                });
            }
        }
    }
    None
}

fn get_straight_short(cards: &[Card]) -> Option<HandResult> {
    let high = has_straight_short(cards)?;
    let straight_cards = build_straight_cards_short(cards, high);
    Some(HandResult {
        rank: HandRank::Straight,
        cards: straight_cards,
        kickers: vec![],
        value: short_deck_rank_value(HandRank::Straight),
    })
}

fn with_sd_value(mut result: HandResult) -> HandResult {
    result.value = short_deck_rank_value(result.rank);
    result
}

/// Avalia mão sob regras Short Deck / Six Plus (Flush > Full House; wheel A6789).
pub fn evaluate_hand_short_deck(hole_cards: &[Card], community_cards: &[Card]) -> HandResult {
    let mut all_cards = Vec::with_capacity(hole_cards.len() + community_cards.len());
    all_cards.extend_from_slice(hole_cards);
    all_cards.extend_from_slice(community_cards);

    if all_cards.is_empty() {
        return HandResult {
            rank: HandRank::HighCard,
            cards: vec![],
            kickers: vec![],
            value: 0,
        };
    }

    all_cards.sort_by(|a, b| b.rank.cmp(&a.rank).then_with(|| b.suit.cmp(&a.suit)));

    let mut suit_counts = [0; 4];
    let mut has_flush_suit = false;
    for card in &all_cards {
        let idx = match card.suit {
            Suit::Hearts => 0,
            Suit::Diamonds => 1,
            Suit::Clubs => 2,
            Suit::Spades => 3,
        };
        suit_counts[idx] += 1;
        if suit_counts[idx] >= 5 {
            has_flush_suit = true;
        }
    }

    if has_flush_suit {
        if let Some(result) = get_straight_flush_short(&all_cards) {
            return result;
        }
    }
    if let Some(result) = get_four_of_a_kind(&all_cards) {
        return with_sd_value(result);
    }
    // Short Deck: Flush before Full House
    if has_flush_suit {
        if let Some(result) = get_flush(&all_cards) {
            return with_sd_value(result);
        }
    }
    if let Some(result) = get_full_house(&all_cards) {
        return with_sd_value(result);
    }
    if let Some(result) = get_straight_short(&all_cards) {
        return result;
    }
    if let Some(result) = get_three_of_a_kind(&all_cards) {
        return with_sd_value(result);
    }
    if let Some(result) = get_two_pair(&all_cards) {
        return with_sd_value(result);
    }
    if let Some(result) = get_one_pair(&all_cards) {
        return with_sd_value(result);
    }
    if let Some(result) = get_high_card(&all_cards) {
        return with_sd_value(result);
    }

    unreachable!("get_high_card always returns Some for any non-empty hand");
}

/// Short Deck Omaha: exatamente 2 hole + 3 board; ranking Short Deck (Flush > Full House).
pub fn evaluate_hand_short_deck_omaha(hole_cards: &[Card], community_cards: &[Card]) -> HandResult {
    if hole_cards.len() < 2 || community_cards.len() < 3 {
        // Sem board/hole suficientes: avalia o que houver sob regras SD (pré-flop etc.)
        return evaluate_hand_short_deck(hole_cards, community_cards);
    }

    let hole_n = hole_cards.len().min(4);
    let board_n = community_cards.len().min(5);
    let hole = &hole_cards[..hole_n];
    let board = &community_cards[..board_n];

    let mut best: Option<HandResult> = None;

    // C(hole, 2)
    for i in 0..hole.len() {
        for j in (i + 1)..hole.len() {
            let h2 = [hole[i], hole[j]];
            // C(board, 3)
            for a in 0..board.len() {
                for b in (a + 1)..board.len() {
                    for c_idx in (b + 1)..board.len() {
                        let b3 = [board[a], board[b], board[c_idx]];
                        let cand = evaluate_hand_short_deck(&h2, &b3);
                        best = Some(match best {
                            None => cand,
                            Some(ref cur) => {
                                if compare_hands(&cand, cur) == Ordering::Greater {
                                    cand
                                } else {
                                    cur.clone()
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    best.unwrap_or_else(|| evaluate_hand_short_deck(hole_cards, community_cards))
}

/// Verifica se uma carta está no slice
pub fn contains_card(cards: &[Card], target: &Card) -> bool {
    cards
        .iter()
        .any(|c| c.rank == target.rank && c.suit == target.suit)
}

// ─── Funções auxiliares privadas ───

/// Ordena cartas por rank decrescente (A=14 primeiro)
fn sort_by_rank(cards: &[Card]) -> Vec<Card> {
    let mut sorted = cards.to_vec();
    sorted.sort_by_key(|c| std::cmp::Reverse(c.rank));
    sorted
}

/// Conta quantas cartas de cada rank existem
fn get_rank_counts(cards: &[Card]) -> HashMap<Rank, usize> {
    let mut counts = HashMap::new();
    for card in cards {
        *counts.entry(card.rank).or_insert(0) += 1;
    }
    counts
}

/// Conta quantas cartas de cada naipe existem
fn get_suit_counts(cards: &[Card]) -> HashMap<Suit, usize> {
    let mut counts = HashMap::new();
    for card in cards {
        *counts.entry(card.suit).or_insert(0) += 1;
    }
    counts
}

/// Verifica se existe uma sequência (straight) nas cartas
/// Retorna o straight MAIS ALTO possível (não o primeiro encontrado)
fn has_straight(cards: &[Card]) -> Option<Rank> {
    let mut values: Vec<u8> = cards.iter().map(|c| c.rank as u8).collect();
    values.sort_unstable();
    values.dedup(); // remove duplicatas

    let mut best_high: Option<Rank> = None;

    // Straight especial: A-2-3-4-5 (wheel)
    if values.contains(&14)
        && values.contains(&2)
        && values.contains(&3)
        && values.contains(&4)
        && values.contains(&5)
    {
        best_high = Some(Rank::Five); // high card do wheel é 5
    }

    // Verifica TODAS as sequências de 5 cartas consecutivas e pega a mais alta
    for window in values.windows(5) {
        if window[4] - window[0] == 4 {
            let high = Rank::try_from(window[4]).ok();
            match (best_high, high) {
                (None, h) => best_high = h,
                (Some(current), Some(h)) if h as u8 > current as u8 => best_high = Some(h),
                _ => {}
            }
        }
    }

    best_high
}

/// Verifica Straight Flush ou Royal Flush
fn get_straight_flush(cards: &[Card]) -> Option<HandResult> {
    let suit_counts = get_suit_counts(cards);
    for (&suit, &count) in &suit_counts {
        if count >= 5 {
            let suited: Vec<Card> = cards.iter().filter(|c| c.suit == suit).copied().collect();
            if let Some(high) = has_straight(&suited) {
                let is_royal = high == Rank::Ace;
                let rank = if is_royal {
                    HandRank::RoyalFlush
                } else {
                    HandRank::StraightFlush
                };
                // Pega as 5 cartas da sequência
                let straight_cards = build_straight_cards(&suited, high);
                return Some(HandResult {
                    rank,
                    cards: straight_cards,
                    kickers: vec![],
                    value: rank as u8,
                });
            }
        }
    }
    None
}

/// Constrói o Vec das 5 cartas que formam a sequência
fn build_straight_cards(cards: &[Card], high: Rank) -> Vec<Card> {
    let high_val = high as u8;
    let low_val = if high == Rank::Five { 14 } else { high_val - 4 }; // wheel: A conta como 1

    let mut result: Vec<Card> = cards
        .iter()
        .filter(|c| {
            let v = c.rank as u8;
            if high == Rank::Five && c.rank == Rank::Ace {
                return true; // Ás no wheel
            }
            v <= high_val && v >= low_val
        })
        .copied()
        .collect();

    result.sort_by_key(|c| std::cmp::Reverse(c.rank));
    result.dedup_by(|a, b| a.rank == b.rank);
    result.truncate(5);
    result
}

/// Verifica Quadra (Four of a Kind)
fn get_four_of_a_kind(cards: &[Card]) -> Option<HandResult> {
    let counts = get_rank_counts(cards);
    for (&rank, &count) in &counts {
        if count == 4 {
            let quads: Vec<Card> = cards.iter().filter(|c| c.rank == rank).copied().collect();
            let kickers: Vec<Card> = sort_by_rank(cards)
                .into_iter()
                .filter(|c| c.rank != rank)
                .take(1)
                .collect();
            return Some(HandResult {
                rank: HandRank::FourOfAKind,
                cards: quads,
                kickers,
                value: HandRank::FourOfAKind as u8,
            });
        }
    }
    None
}

/// Verifica Full House
fn get_full_house(cards: &[Card]) -> Option<HandResult> {
    let counts = get_rank_counts(cards);
    let mut three: Option<Rank> = None;
    let mut pair: Option<Rank> = None;

    for (&rank, &count) in &counts {
        if count >= 3 {
            match three {
                None => three = Some(rank),
                Some(current) if rank as u8 > current as u8 => {
                    pair = three;
                    three = Some(rank);
                }
                Some(_) => {
                    if pair.is_none() || rank as u8 > pair.unwrap() as u8 {
                        pair = Some(rank);
                    }
                }
            }
        } else if count >= 2 && (pair.is_none() || rank as u8 > pair.unwrap() as u8) {
            pair = Some(rank);
        }
    }

    if let (Some(t), Some(p)) = (three, pair) {
        let three_cards: Vec<Card> = cards
            .iter()
            .filter(|c| c.rank == t)
            .take(3)
            .copied()
            .collect();
        let pair_cards: Vec<Card> = cards
            .iter()
            .filter(|c| c.rank == p)
            .take(2)
            .copied()
            .collect();
        let all_cards: Vec<Card> = three_cards.into_iter().chain(pair_cards).collect();
        return Some(HandResult {
            rank: HandRank::FullHouse,
            cards: all_cards,
            kickers: vec![],
            value: HandRank::FullHouse as u8,
        });
    }
    None
}

/// Verifica Flush
fn get_flush(cards: &[Card]) -> Option<HandResult> {
    let suit_counts = get_suit_counts(cards);
    for (&suit, &count) in &suit_counts {
        if count >= 5 {
            let flush_cards: Vec<Card> = sort_by_rank(cards)
                .into_iter()
                .filter(|c| c.suit == suit)
                .take(5)
                .collect();
            return Some(HandResult {
                rank: HandRank::Flush,
                cards: flush_cards,
                kickers: vec![],
                value: HandRank::Flush as u8,
            });
        }
    }
    None
}

/// Verifica Sequência (Straight)
fn get_straight(cards: &[Card]) -> Option<HandResult> {
    let high = has_straight(cards)?;
    let straight_cards = build_straight_cards(cards, high);
    Some(HandResult {
        rank: HandRank::Straight,
        cards: straight_cards,
        kickers: vec![],
        value: HandRank::Straight as u8,
    })
}

/// Verifica Trinca (Three of a Kind)
fn get_three_of_a_kind(cards: &[Card]) -> Option<HandResult> {
    let counts = get_rank_counts(cards);
    for (&rank, &count) in &counts {
        if count == 3 {
            let three: Vec<Card> = cards.iter().filter(|c| c.rank == rank).copied().collect();
            let kickers: Vec<Card> = sort_by_rank(cards)
                .into_iter()
                .filter(|c| c.rank != rank)
                .take(2)
                .collect();
            return Some(HandResult {
                rank: HandRank::ThreeOfAKind,
                cards: three,
                kickers,
                value: HandRank::ThreeOfAKind as u8,
            });
        }
    }
    None
}

/// Verifica Dois Pares
fn get_two_pair(cards: &[Card]) -> Option<HandResult> {
    let counts = get_rank_counts(cards);
    let mut pairs: Vec<Rank> = counts
        .iter()
        .filter(|(_, &c)| c >= 2)
        .map(|(&r, _)| r)
        .collect();
    pairs.sort_by(|a, b| b.cmp(a)); // rank mais alto primeiro

    if pairs.len() >= 2 {
        let top_two = &pairs[..2];
        let pair_cards: Vec<Card> = top_two
            .iter()
            .flat_map(|&r| cards.iter().filter(move |c| c.rank == r).take(2).copied())
            .collect();
        let kickers: Vec<Card> = sort_by_rank(cards)
            .into_iter()
            .filter(|c| !top_two.contains(&c.rank))
            .take(1)
            .collect();
        return Some(HandResult {
            rank: HandRank::TwoPair,
            cards: pair_cards,
            kickers,
            value: HandRank::TwoPair as u8,
        });
    }
    None
}

/// Verifica Um Par
fn get_one_pair(cards: &[Card]) -> Option<HandResult> {
    let counts = get_rank_counts(cards);
    for (&rank, &count) in &counts {
        if count == 2 {
            let pair_cards: Vec<Card> = cards.iter().filter(|c| c.rank == rank).copied().collect();
            let kickers: Vec<Card> = sort_by_rank(cards)
                .into_iter()
                .filter(|c| c.rank != rank)
                .take(3)
                .collect();
            return Some(HandResult {
                rank: HandRank::OnePair,
                cards: pair_cards,
                kickers,
                value: HandRank::OnePair as u8,
            });
        }
    }
    None
}

/// Verifica Carta Alta (High Card) — sempre retorna Some para qualquer mão com cartas
fn get_high_card(cards: &[Card]) -> Option<HandResult> {
    if cards.is_empty() {
        return None;
    }
    let sorted = sort_by_rank(cards);
    let high_cards: Vec<Card> = sorted.into_iter().take(5).collect();
    Some(HandResult {
        rank: HandRank::HighCard,
        cards: vec![high_cards[0]],
        kickers: high_cards[1..].to_vec(),
        value: HandRank::HighCard as u8,
    })
}

// ─── Implementação de TryFrom para converter u8 → Rank ───

impl TryFrom<u8> for Rank {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            2 => Ok(Rank::Two),
            3 => Ok(Rank::Three),
            4 => Ok(Rank::Four),
            5 => Ok(Rank::Five),
            6 => Ok(Rank::Six),
            7 => Ok(Rank::Seven),
            8 => Ok(Rank::Eight),
            9 => Ok(Rank::Nine),
            10 => Ok(Rank::Ten),
            11 => Ok(Rank::Jack),
            12 => Ok(Rank::Queen),
            13 => Ok(Rank::King),
            14 => Ok(Rank::Ace),
            _ => Err(()),
        }
    }
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: cria uma carta rapidamente
    fn c(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    #[test]
    fn test_create_deck_has_52_cards() {
        let deck = create_deck();
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn test_create_deck_no_duplicates() {
        let deck = create_deck();
        for i in 0..deck.len() {
            for j in (i + 1)..deck.len() {
                assert!(
                    deck[i].rank != deck[j].rank || deck[i].suit != deck[j].suit,
                    "Duplicata encontrada: {:?} e {:?}",
                    deck[i],
                    deck[j]
                );
            }
        }
    }

    #[test]
    fn test_shuffle_preserves_all_cards() {
        let deck = create_deck();
        let shuffled = shuffle_deck(&deck);
        assert_eq!(shuffled.len(), 52);
        // Verifica que todas as cartas originais estão presentes
        for card in &deck {
            assert!(
                shuffled.contains(card),
                "Carta {:?} sumiu após embaralhar",
                card
            );
        }
    }

    #[test]
    fn test_deal_cards_splits_correctly() {
        let deck = create_deck();
        let (dealt, remaining) = deal_cards(&deck, 5);
        assert_eq!(dealt.len(), 5);
        assert_eq!(remaining.len(), 47);
        assert_eq!(dealt.len() + remaining.len(), 52);
    }

    #[test]
    fn test_evaluate_royal_flush() {
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::King, Suit::Spades)];
        let community = vec![
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Ten, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::RoyalFlush);
        assert_eq!(result.value, 10);
    }

    #[test]
    fn test_evaluate_straight_flush() {
        let hole = vec![c(Rank::Nine, Suit::Hearts), c(Rank::Eight, Suit::Hearts)];
        let community = vec![
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Six, Suit::Hearts),
            c(Rank::Five, Suit::Hearts),
            c(Rank::Ace, Suit::Clubs),
            c(Rank::King, Suit::Diamonds),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::StraightFlush);
        assert_eq!(result.value, 9);
    }

    #[test]
    fn test_evaluate_four_of_a_kind() {
        let hole = vec![c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)];
        let community = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::FourOfAKind);
        assert_eq!(result.value, 8);
    }

    #[test]
    fn test_evaluate_full_house() {
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)];
        let community = vec![
            c(Rank::Ace, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::King, Suit::Hearts),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::FullHouse);
        assert_eq!(result.value, 7);
    }

    #[test]
    fn test_evaluate_flush() {
        let hole = vec![c(Rank::Ace, Suit::Diamonds), c(Rank::Five, Suit::Diamonds)];
        let community = vec![
            c(Rank::Ten, Suit::Diamonds),
            c(Rank::Seven, Suit::Diamonds),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::King, Suit::Hearts),
            c(Rank::Queen, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Flush);
        assert_eq!(result.value, 6);
    }

    #[test]
    fn test_evaluate_flush_seven_suited_cards_picks_top_5() {
        // 7 cartas de Copas (Hearts): A, K, J, 9, 7, 5, 3
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Three, Suit::Hearts)];
        let community = vec![
            c(Rank::King, Suit::Hearts),
            c(Rank::Jack, Suit::Hearts),
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Five, Suit::Hearts),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Flush);
        assert_eq!(result.cards.len(), 5);
        // Garante que as 5 cartas selecionadas são exatamente as 5 maiores
        let expected_ranks = vec![Rank::Ace, Rank::King, Rank::Jack, Rank::Nine, Rank::Seven];
        let actual_ranks: Vec<Rank> = result.cards.iter().map(|card| card.rank).collect();
        assert_eq!(actual_ranks, expected_ranks);
    }

    #[test]
    fn test_evaluate_flush_six_suited_cards_picks_top_5() {
        // 6 cartas de Paus (Clubs): K, Q, 10, 8, 6, 2 + 1 Ás de Ouros
        let hole = vec![c(Rank::King, Suit::Clubs), c(Rank::Two, Suit::Clubs)];
        let community = vec![
            c(Rank::Queen, Suit::Clubs),
            c(Rank::Ten, Suit::Clubs),
            c(Rank::Eight, Suit::Clubs),
            c(Rank::Six, Suit::Clubs),
            c(Rank::Ace, Suit::Diamonds),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Flush);
        assert_eq!(result.cards.len(), 5);
        let expected_ranks = vec![Rank::King, Rank::Queen, Rank::Ten, Rank::Eight, Rank::Six];
        let actual_ranks: Vec<Rank> = result.cards.iter().map(|card| card.rank).collect();
        assert_eq!(actual_ranks, expected_ranks);
    }

    #[test]
    fn test_compare_flushes_with_board_4_suited() {
        // Board com 4 de Ouros: A♦ K♦ 10♦ 4♦ 2♣
        let community = vec![
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Ten, Suit::Diamonds),
            c(Rank::Four, Suit::Diamonds),
            c(Rank::Two, Suit::Clubs),
        ];
        // Jogador 1 tem Q♦ 3♠ -> Flush: A♦ K♦ Q♦ 10♦ 4♦
        let p1_hole = vec![c(Rank::Queen, Suit::Diamonds), c(Rank::Three, Suit::Spades)];
        // Jogador 2 tem J♦ 9♠ -> Flush: A♦ K♦ J♦ 10♦ 4♦
        let p2_hole = vec![c(Rank::Jack, Suit::Diamonds), c(Rank::Nine, Suit::Spades)];

        let res1 = evaluate_hand(&p1_hole, &community);
        let res2 = evaluate_hand(&p2_hole, &community);

        assert_eq!(res1.rank, HandRank::Flush);
        assert_eq!(res2.rank, HandRank::Flush);
        // Jogador 1 vence pois Q♦ > J♦ na 3ª carta do Flush
        assert_eq!(compare_hands(&res1, &res2), Ordering::Greater);
    }

    #[test]
    fn test_compare_flushes_full_community_flush_tie() {
        // Board de 5 Espadas: A♠ K♠ Q♠ J♠ 7♠
        let community = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::King, Suit::Spades),
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Seven, Suit::Spades),
        ];
        // Jogador 1 tem 5♠ 2♥ (6ª espada 5♠ é menor que as 5 do board)
        let p1_hole = vec![c(Rank::Five, Suit::Spades), c(Rank::Two, Suit::Hearts)];
        // Jogador 2 tem 3♠ 4♦ (6ª espada 3♠ é menor que as 5 do board)
        let p2_hole = vec![c(Rank::Three, Suit::Spades), c(Rank::Four, Suit::Diamonds)];

        let res1 = evaluate_hand(&p1_hole, &community);
        let res2 = evaluate_hand(&p2_hole, &community);

        // Ambos avaliam para as mesmas 5 cartas do board
        assert_eq!(compare_hands(&res1, &res2), Ordering::Equal);
    }

    #[test]
    fn test_flush_vs_straight_flush_7_suited() {
        // 7 Espadas: A♠ K♠ 7♠ 6♠ 5♠ 4♠ 3♠
        // As 5 maiores cartas formariam Flush (A-K-7-6-5), mas 7-6-5-4-3 é Straight Flush
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::King, Suit::Spades)];
        let community = vec![
            c(Rank::Seven, Suit::Spades),
            c(Rank::Six, Suit::Spades),
            c(Rank::Five, Suit::Spades),
            c(Rank::Four, Suit::Spades),
            c(Rank::Three, Suit::Spades),
        ];
        let result = evaluate_hand(&hole, &community);
        // Deve priorizar o Straight Flush mesmo que o Flush simples tenha cartas (A, K) maiores
        assert_eq!(result.rank, HandRank::StraightFlush);
        let expected_ranks = vec![Rank::Seven, Rank::Six, Rank::Five, Rank::Four, Rank::Three];
        let actual_ranks: Vec<Rank> = result.cards.iter().map(|card| card.rank).collect();
        assert_eq!(actual_ranks, expected_ranks);
    }

    #[test]
    fn test_compare_flushes_equal_top_four_different_fifth_card() {
        // Jogador 1: A♣ K♣ Q♣ J♣ 9♣
        let p1_hole = vec![c(Rank::Ace, Suit::Clubs), c(Rank::Nine, Suit::Clubs)];
        // Jogador 2: A♣ K♣ Q♣ J♣ 8♣
        let p2_hole = vec![c(Rank::Ace, Suit::Clubs), c(Rank::Eight, Suit::Clubs)];
        let community = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::Queen, Suit::Clubs),
            c(Rank::Jack, Suit::Clubs),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
        ];

        let res1 = evaluate_hand(&p1_hole, &community);
        let res2 = evaluate_hand(&p2_hole, &community);

        assert_eq!(compare_hands(&res1, &res2), Ordering::Greater);
    }

    #[test]
    fn test_evaluate_straight() {
        let hole = vec![c(Rank::Nine, Suit::Hearts), c(Rank::Eight, Suit::Diamonds)];
        let community = vec![
            c(Rank::Seven, Suit::Clubs),
            c(Rank::Six, Suit::Spades),
            c(Rank::Five, Suit::Hearts),
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::King, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Straight);
        assert_eq!(result.value, 5);
    }

    #[test]
    fn test_evaluate_wheel_straight() {
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Two, Suit::Diamonds)];
        let community = vec![
            c(Rank::Three, Suit::Clubs),
            c(Rank::Four, Suit::Spades),
            c(Rank::Five, Suit::Hearts),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Queen, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Straight);
        assert_eq!(result.value, 5);
    }

    #[test]
    fn test_evaluate_three_of_a_kind() {
        let hole = vec![c(Rank::Queen, Suit::Hearts), c(Rank::Queen, Suit::Diamonds)];
        let community = vec![
            c(Rank::Queen, Suit::Clubs),
            c(Rank::Ace, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::ThreeOfAKind);
        assert_eq!(result.value, 4);
    }

    #[test]
    fn test_evaluate_two_pair() {
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)];
        let community = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::TwoPair);
        assert_eq!(result.value, 3);
    }

    #[test]
    fn test_evaluate_one_pair() {
        let hole = vec![c(Rank::Jack, Suit::Hearts), c(Rank::Jack, Suit::Diamonds)];
        let community = vec![
            c(Rank::Ace, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::OnePair);
        assert_eq!(result.value, 2);
    }

    #[test]
    fn test_evaluate_high_card() {
        // Cartas sem par, sem flush, sem sequência: A♠ K♦ 9♥ 5♣ 2♠
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::King, Suit::Diamonds)];
        let community = vec![
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Five, Suit::Clubs),
            c(Rank::Two, Suit::Spades),
            c(Rank::Three, Suit::Diamonds), // não conecta com 2-5-9-K-A
            c(Rank::Seven, Suit::Hearts),   // não conecta
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::HighCard);
        assert_eq!(result.value, 1);
    }

    #[test]
    fn test_compare_hands_different_rank() {
        let flush = HandResult {
            rank: HandRank::Flush,
            cards: vec![],
            kickers: vec![],
            value: 6,
        };
        let pair = HandResult {
            rank: HandRank::OnePair,
            cards: vec![],
            kickers: vec![],
            value: 2,
        };
        assert_eq!(compare_hands(&flush, &pair), Ordering::Greater);
        assert_eq!(compare_hands(&pair, &flush), Ordering::Less);
    }

    #[test]
    fn test_compare_hands_equal() {
        let a = HandResult {
            rank: HandRank::HighCard,
            cards: vec![c(Rank::Ace, Suit::Hearts)],
            kickers: vec![],
            value: 1,
        };
        let b = HandResult {
            rank: HandRank::HighCard,
            cards: vec![c(Rank::Ace, Suit::Diamonds)],
            kickers: vec![],
            value: 1,
        };
        assert_eq!(compare_hands(&a, &b), Ordering::Equal);
    }

    #[test]
    fn test_get_hand_name_all_ranks() {
        assert_eq!(get_hand_name(HandRank::RoyalFlush), "Royal Flush");
        assert_eq!(get_hand_name(HandRank::HighCard), "High Card");
        assert_eq!(get_hand_name(HandRank::FullHouse), "Full House");
    }

    #[test]
    fn test_create_full_deck_matches_create_deck() {
        let deck = create_full_deck();
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn test_contains_card() {
        let cards = vec![
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Spades,
            },
        ];
        let ace_hearts = Card {
            rank: Rank::Ace,
            suit: Suit::Hearts,
        };
        let ace_spades = Card {
            rank: Rank::Ace,
            suit: Suit::Spades,
        };

        assert!(contains_card(&cards, &ace_hearts));
        assert!(!contains_card(&cards, &ace_spades));
    }

    #[test]
    fn test_short_deck_size_and_ranks() {
        let deck = create_short_deck();
        assert_eq!(deck.len(), 36);
        assert!(deck.iter().all(|c| (c.rank as u8) >= 6));
        assert!(!deck
            .iter()
            .any(|c| matches!(c.rank, Rank::Two | Rank::Three | Rank::Four | Rank::Five)));
    }

    #[test]
    fn test_short_deck_wheel_a6789() {
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Six, Suit::Diamonds)];
        let community = vec![
            c(Rank::Seven, Suit::Clubs),
            c(Rank::Eight, Suit::Spades),
            c(Rank::Nine, Suit::Hearts),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Jack, Suit::Clubs),
        ];
        let result = evaluate_hand_short_deck(&hole, &community);
        assert_eq!(result.rank, HandRank::Straight);
    }

    #[test]
    fn test_short_deck_flush_beats_full_house() {
        let flush = evaluate_hand_short_deck(
            &[c(Rank::Ace, Suit::Hearts), c(Rank::King, Suit::Hearts)],
            &[
                c(Rank::Nine, Suit::Hearts),
                c(Rank::Eight, Suit::Hearts),
                c(Rank::Seven, Suit::Hearts),
                c(Rank::Six, Suit::Clubs),
                c(Rank::Jack, Suit::Diamonds),
            ],
        );
        let boat = evaluate_hand_short_deck(
            &[c(Rank::Ace, Suit::Clubs), c(Rank::Ace, Suit::Diamonds)],
            &[
                c(Rank::Ace, Suit::Spades),
                c(Rank::King, Suit::Clubs),
                c(Rank::King, Suit::Diamonds),
                c(Rank::Nine, Suit::Spades),
                c(Rank::Eight, Suit::Clubs),
            ],
        );
        assert_eq!(flush.rank, HandRank::Flush);
        assert_eq!(boat.rank, HandRank::FullHouse);
        assert_eq!(compare_hands(&flush, &boat), Ordering::Greater);
    }

    #[test]
    fn test_short_deck_omaha_uses_exactly_two_hole() {
        // Hole: Ah Kh Qd Jd — board: 9h 8h 7h 6c Tc
        // Com 2 hole AhKh + 3 board 9h8h7h = flush A-high
        // Se usasse 3+ hole não seria Omaha válido; garantimos flush via 2+3
        let hole = vec![
            c(Rank::Ace, Suit::Hearts),
            c(Rank::King, Suit::Hearts),
            c(Rank::Queen, Suit::Diamonds),
            c(Rank::Jack, Suit::Diamonds),
        ];
        let board = vec![
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Eight, Suit::Hearts),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Six, Suit::Clubs),
            c(Rank::Ten, Suit::Clubs),
        ];
        let r = evaluate_hand_short_deck_omaha(&hole, &board);
        assert_eq!(r.rank, HandRank::Flush);
    }

    #[test]
    fn test_short_deck_omaha_cannot_use_three_hole_for_trips() {
        // Três ases no hole + um ás no board NÃO formam quads via 3 hole —
        // Omaha só usa 2 hole → no máximo trips (AA + Axx)
        let hole = vec![
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::Ace, Suit::Clubs),
            c(Rank::King, Suit::Spades),
        ];
        let board = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Eight, Suit::Diamonds),
            c(Rank::Seven, Suit::Clubs),
            c(Rank::Six, Suit::Hearts),
        ];
        let r = evaluate_hand_short_deck_omaha(&hole, &board);
        // Omaha: no máximo 2 ases do hole + 1 ás do board → trips (nunca quads com 1 ás no board)
        assert_eq!(r.rank, HandRank::ThreeOfAKind);
    }

    #[test]
    fn test_short_deck_omaha_flush_beats_full_house() {
        let flush_hole = vec![
            c(Rank::Ace, Suit::Hearts),
            c(Rank::King, Suit::Hearts),
            c(Rank::Six, Suit::Clubs),
            c(Rank::Seven, Suit::Diamonds),
        ];
        let flush_board = vec![
            c(Rank::Queen, Suit::Hearts),
            c(Rank::Jack, Suit::Hearts),
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Eight, Suit::Clubs),
            c(Rank::Six, Suit::Spades),
        ];
        // Omaha boat: 2 hole KK + board K A A → full house kings full of aces
        let boat_hole = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Nine, Suit::Clubs),
            c(Rank::Eight, Suit::Diamonds),
        ];
        let boat_board = vec![
            c(Rank::King, Suit::Spades),
            c(Rank::Ace, Suit::Spades),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Seven, Suit::Clubs),
            c(Rank::Six, Suit::Diamonds),
        ];
        let flush = evaluate_hand_short_deck_omaha(&flush_hole, &flush_board);
        let boat = evaluate_hand_short_deck_omaha(&boat_hole, &boat_board);
        assert_eq!(flush.rank, HandRank::Flush);
        assert_eq!(boat.rank, HandRank::FullHouse);
        assert_eq!(compare_hands(&flush, &boat), Ordering::Greater);
    }
}
