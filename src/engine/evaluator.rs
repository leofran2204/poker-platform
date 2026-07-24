use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandRank {
    HighCard(Vec<u8>),
    OnePair(u8, Vec<u8>),
    TwoPair(u8, u8, u8),
    ThreeOfAKind(u8, Vec<u8>),
    Straight(u8),
    Flush(Vec<u8>),
    FullHouse(u8, u8),
    FourOfAKind(u8, u8),
    StraightFlush(u8),
    RoyalFlush,
}

impl PartialOrd for HandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        fn category(rank: &HandRank) -> u8 {
            match rank {
                HandRank::HighCard(_) => 1,
                HandRank::OnePair(_, _) => 2,
                HandRank::TwoPair(_, _, _) => 3,
                HandRank::ThreeOfAKind(_, _) => 4,
                HandRank::Straight(_) => 5,
                HandRank::Flush(_) => 6,
                HandRank::FullHouse(_, _) => 7,
                HandRank::FourOfAKind(_, _) => 8,
                HandRank::StraightFlush(_) => 9,
                HandRank::RoyalFlush => 10,
            }
        }

        let cat_a = category(self);
        let cat_b = category(other);

        if cat_a != cat_b {
            return cat_a.cmp(&cat_b);
        }

        match (self, other) {
            (HandRank::HighCard(k1), HandRank::HighCard(k2)) => k1.cmp(k2),
            (HandRank::OnePair(p1, k1), HandRank::OnePair(p2, k2)) => p1.cmp(p2).then_with(|| k1.cmp(k2)),
            (HandRank::TwoPair(h1, l1, k1), HandRank::TwoPair(h2, l2, k2)) => {
                h1.cmp(h2).then_with(|| l1.cmp(l2)).then_with(|| k1.cmp(k2))
            }
            (HandRank::ThreeOfAKind(t1, k1), HandRank::ThreeOfAKind(t2, k2)) => t1.cmp(t2).then_with(|| k1.cmp(k2)),
            (HandRank::Straight(s1), HandRank::Straight(s2)) => s1.cmp(s2),
            (HandRank::Flush(k1), HandRank::Flush(k2)) => k1.cmp(k2),
            (HandRank::FullHouse(t1, p1), HandRank::FullHouse(t2, p2)) => t1.cmp(t2).then_with(|| p1.cmp(p2)),
            (HandRank::FourOfAKind(f1, k1), HandRank::FourOfAKind(f2, k2)) => f1.cmp(f2).then_with(|| k1.cmp(k2)),
            (HandRank::StraightFlush(s1), HandRank::StraightFlush(s2)) => s1.cmp(s2),
            (HandRank::RoyalFlush, HandRank::RoyalFlush) => Ordering::Equal,
            _ => Ordering::Equal,
        }
    }
}

/// Avalia exatamente 5 cartas e determina o HandRank.
pub fn evaluate_5_card_hand(cards: &[Card]) -> HandRank {
    if cards.len() != 5 {
        return HandRank::HighCard(vec![]);
    }

    let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank as u8).collect();
    ranks.sort_by(|a, b| b.cmp(a));

    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);

    // Checar Straight
    let mut is_straight = false;
    let mut straight_high = 0u8;

    if ranks[0] - ranks[4] == 4 && ranks.windows(2).all(|w| w[0] - w[1] == 1) {
        is_straight = true;
        straight_high = ranks[0];
    } else if ranks == vec![14, 5, 4, 3, 2] {
        // A-2-3-4-5 (Wheel Straight)
        is_straight = true;
        straight_high = 5;
    }

    if is_flush && is_straight {
        if straight_high == 14 {
            return HandRank::RoyalFlush;
        } else {
            return HandRank::StraightFlush(straight_high);
        }
    }

    // Contagem de frequências
    let mut counts = std::collections::HashMap::new();
    for &r in &ranks {
        *counts.entry(r).or_insert(0) += 1;
    }

    let mut groups: Vec<(u8, u8)> = counts.into_iter().map(|(r, c)| (c, r)).collect();
    groups.sort_by(|a, b| b.cmp(a));

    if groups[0].0 == 4 {
        let kicker = groups[1].1;
        return HandRank::FourOfAKind(groups[0].1, kicker);
    }

    if groups[0].0 == 3 && groups[1].0 == 2 {
        return HandRank::FullHouse(groups[0].1, groups[1].1);
    }

    if is_flush {
        return HandRank::Flush(ranks);
    }

    if is_straight {
        return HandRank::Straight(straight_high);
    }

    if groups[0].0 == 3 {
        let kickers = vec![groups[1].1, groups[2].1];
        return HandRank::ThreeOfAKind(groups[0].1, kickers);
    }

    if groups[0].0 == 2 && groups[1].0 == 2 {
        let kicker = groups[2].1;
        let p1 = groups[0].1.max(groups[1].1);
        let p2 = groups[0].1.min(groups[1].1);
        return HandRank::TwoPair(p1, p2, kicker);
    }

    if groups[0].0 == 2 {
        let kickers = vec![groups[1].1, groups[2].1, groups[3].1];
        return HandRank::OnePair(groups[0].1, kickers);
    }

    HandRank::HighCard(ranks)
}

/// Avalia a melhor mão de 5 cartas selecionada a partir de 5 a 7 cartas disponíveis.
pub fn evaluate_hand(cards: &[Card]) -> HandRank {
    let n = cards.len();
    if n < 5 {
        return HandRank::HighCard(vec![]);
    }
    if n == 5 {
        return evaluate_5_card_hand(cards);
    }

    let mut best_rank: Option<HandRank> = None;

    // Gerar combinações de 5 cartas a partir de N cartas (máx 7)
    fn combination_indices(n: usize, k: usize) -> Vec<Vec<usize>> {
        let mut res = Vec::new();
        let mut combo = (0..k).collect::<Vec<_>>();
        loop {
            res.push(combo.clone());
            let mut i = k;
            while i > 0 && combo[i - 1] == n - k + i - 1 {
                i -= 1;
            }
            if i == 0 {
                break;
            }
            combo[i - 1] += 1;
            for j in i..k {
                combo[j] = combo[j - 1] + 1;
            }
        }
        res
    }

    let combinations = combination_indices(n, 5);
    for combo in combinations {
        let selected: Vec<Card> = combo.iter().map(|&idx| cards[idx]).collect();
        let rank = evaluate_5_card_hand(&selected);

        match &best_rank {
            None => best_rank = Some(rank),
            Some(current_best) => {
                if &rank > current_best {
                    best_rank = Some(rank);
                }
            }
        }
    }

    best_rank.unwrap_or(HandRank::HighCard(vec![]))
}
