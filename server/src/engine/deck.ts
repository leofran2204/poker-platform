import { Card, Rank, Suit, HandRank, HandResult } from '@shared/types';

const RANKS: Rank[] = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2'];
const SUITS: Suit[] = ['hearts', 'diamonds', 'clubs', 'spades'];

const RANK_VALUES: Record<Rank, number> = {
  '2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7, '8': 8, '9': 9,
  'T': 10, 'J': 11, 'Q': 12, 'K': 13, 'A': 14,
};

export function createDeck(): Card[] {
  const deck: Card[] = [];
  for (const suit of SUITS) {
    for (const rank of RANKS) {
      deck.push({ rank, suit });
    }
  }
  return deck;
}

export function shuffleDeck(deck: Card[]): Card[] {
  const shuffled = [...deck];
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  return shuffled;
}

export function dealCards(deck: Card[], count: number): { cards: Card[]; remaining: Card[] } {
  const cards = deck.slice(0, count);
  const remaining = deck.slice(count);
  return { cards, remaining };
}

// Texas Hold'em tradicional — 52 cartas
// Hierarquia: Royal Flush > Straight Flush > Quadra > Full House > Flush > Sequência > Trinca > Dois Pares > Par > Carta Alta

function sortByRank(cards: Card[]): Card[] {
  return [...cards].sort((a, b) => RANK_VALUES[b.rank] - RANK_VALUES[a.rank]);
}

function getRankCounts(cards: Card[]): Map<Rank, number> {
  const counts = new Map<Rank, number>();
  for (const card of cards) {
    counts.set(card.rank, (counts.get(card.rank) || 0) + 1);
  }
  return counts;
}

function getSuits(cards: Card[]): Map<Suit, number> {
  const suits = new Map<Suit, number>();
  for (const card of cards) {
    suits.set(card.suit, (suits.get(card.suit) || 0) + 1);
  }
  return suits;
}

function hasStraight(cards: Card[]): { isStraight: boolean; highCard: Rank | null } {
  const uniqueRanks = [...new Set(cards.map(c => RANK_VALUES[c.rank]))].sort((a, b) => a - b);

  if (uniqueRanks.includes(14) && uniqueRanks.includes(2) && uniqueRanks.includes(3) && uniqueRanks.includes(4) && uniqueRanks.includes(5)) {
    return { isStraight: true, highCard: '5' };
  }

  const sortedDesc = [...uniqueRanks].sort((a, b) => b - a);
  for (let i = 0; i <= sortedDesc.length - 5; i++) {
    if (sortedDesc[i] - sortedDesc[i + 4] === 4) {
      const highRank = Object.entries(RANK_VALUES).find(([, v]) => v === sortedDesc[i])![0] as Rank;
      return { isStraight: true, highCard: highRank };
    }
  }

  return { isStraight: false, highCard: null };
}

function getStraightFlush(cards: Card[]): { rank: HandRank; cards: Card[] } | null {
  const suits = getSuits(cards);
  for (const [suit, count] of suits) {
    if (count >= 5) {
      const suitedCards = cards.filter(c => c.suit === suit);
      const { isStraight, highCard } = hasStraight(suitedCards);
      if (isStraight && highCard) {
        const straightCards = suitedCards
          .filter(c => {
            const val = RANK_VALUES[c.rank];
            const highVal = RANK_VALUES[highCard];
            if (highCard === '5' && c.rank === 'A') return true;
            return val <= highVal && val >= highVal - 4;
          })
          .slice(0, 5);
        if (highCard === 'A') {
          return { rank: 'royal_flush', cards: straightCards };
        }
        return { rank: 'straight_flush', cards: straightCards };
      }
    }
  }
  return null;
}

function getFourOfAKind(cards: Card[]): { rank: HandRank; cards: Card[]; kickers: Card[] } | null {
  const counts = getRankCounts(cards);
  for (const [rank, count] of counts) {
    if (count === 4) {
      const quads = cards.filter(c => c.rank === rank);
      const kickers = sortByRank(cards.filter(c => c.rank !== rank)).slice(0, 1);
      return { rank: 'four_of_a_kind', cards: quads, kickers };
    }
  }
  return null;
}

function getFlush(cards: Card[]): { rank: HandRank; cards: Card[] } | null {
  const suits = getSuits(cards);
  for (const [suit, count] of suits) {
    if (count >= 5) {
      const flushCards = sortByRank(cards.filter(c => c.suit === suit)).slice(0, 5);
      return { rank: 'flush', cards: flushCards };
    }
  }
  return null;
}

function getFullHouse(cards: Card[]): { rank: HandRank; cards: Card[] } | null {
  const counts = getRankCounts(cards);
  let three: Rank | null = null;
  let pair: Rank | null = null;

  for (const [rank, count] of counts) {
    if (count >= 3) {
      if (!three || RANK_VALUES[rank] > RANK_VALUES[three]) {
        pair = three;
        three = rank;
      } else if (!pair || RANK_VALUES[rank] > RANK_VALUES[pair]) {
        pair = rank;
      }
    } else if (count >= 2) {
      if (!pair || RANK_VALUES[rank] > RANK_VALUES[pair]) {
        pair = rank;
      }
    }
  }

  if (three && pair) {
    const threeCards = cards.filter(c => c.rank === three).slice(0, 3);
    const pairCards = cards.filter(c => c.rank === pair).slice(0, 2);
    return { rank: 'full_house', cards: [...threeCards, ...pairCards] };
  }
  return null;
}

function getThreeOfAKind(cards: Card[]): { rank: HandRank; cards: Card[]; kickers: Card[] } | null {
  const counts = getRankCounts(cards);
  for (const [rank, count] of counts) {
    if (count === 3) {
      const three = cards.filter(c => c.rank === rank);
      const kickers = sortByRank(cards.filter(c => c.rank !== rank)).slice(0, 2);
      return { rank: 'three_of_a_kind', cards: three, kickers };
    }
  }
  return null;
}

function getTwoPair(cards: Card[]): { rank: HandRank; cards: Card[]; kickers: Card[] } | null {
  const counts = getRankCounts(cards);
  const pairs: Rank[] = [];
  for (const [rank, count] of counts) {
    if (count === 2) pairs.push(rank);
  }
  if (pairs.length >= 2) {
    pairs.sort((a, b) => RANK_VALUES[b] - RANK_VALUES[a]);
    const pairCards = pairs.slice(0, 2).flatMap(r => cards.filter(c => c.rank === r).slice(0, 2));
    const kickers = sortByRank(cards.filter(c => !pairs.slice(0, 2).includes(c.rank))).slice(0, 1);
    return { rank: 'two_pair', cards: pairCards, kickers };
  }
  return null;
}

function getOnePair(cards: Card[]): { rank: HandRank; cards: Card[]; kickers: Card[] } | null {
  const counts = getRankCounts(cards);
  for (const [rank, count] of counts) {
    if (count === 2) {
      const pairCards = cards.filter(c => c.rank === rank);
      const kickers = sortByRank(cards.filter(c => c.rank !== rank)).slice(0, 3);
      return { rank: 'one_pair', cards: pairCards, kickers };
    }
  }
  return null;
}

export function evaluateHand(holeCards: Card[], communityCards: Card[]): HandResult {
  const allCards = [...holeCards, ...communityCards];

  const straightFlush = getStraightFlush(allCards);
  if (straightFlush) {
    const value = straightFlush.rank === 'royal_flush' ? 10 : 9;
    return { rank: straightFlush.rank, cards: straightFlush.cards, kickers: [], value };
  }

  const fourOfAKind = getFourOfAKind(allCards);
  if (fourOfAKind) {
    return { ...fourOfAKind, value: 8 };
  }

  const fullHouse = getFullHouse(allCards);
  if (fullHouse) {
    return { ...fullHouse, kickers: [], value: 7 };
  }

  const flush = getFlush(allCards);
  if (flush) {
    return { ...flush, kickers: [], value: 6 };
  }

  const { isStraight, highCard } = hasStraight(allCards);
  if (isStraight && highCard) {
    let straightCards = sortByRank(allCards).filter(c => {
      const val = RANK_VALUES[c.rank];
      const highVal = RANK_VALUES[highCard];
      if (highCard === '5' && c.rank === 'A') return true;
      return val <= highVal && val >= highVal - 4;
    }).slice(0, 5);
    if (highCard === '5' && !straightCards.some(c => c.rank === 'A')) {
      const aceCard = allCards.find(c => c.rank === 'A');
      if (aceCard) straightCards = [aceCard, ...straightCards.slice(0, 4)];
    }
    return { rank: 'straight', cards: straightCards, kickers: [], value: 5 };
  }

  const threeOfAKind = getThreeOfAKind(allCards);
  if (threeOfAKind) {
    return { ...threeOfAKind, value: 4 };
  }

  const twoPair = getTwoPair(allCards);
  if (twoPair) {
    return { ...twoPair, value: 3 };
  }

  const onePair = getOnePair(allCards);
  if (onePair) {
    return { ...onePair, value: 2 };
  }

  const highCards = sortByRank(allCards).slice(0, 5);
  return { rank: 'high_card', cards: [highCards[0]], kickers: highCards.slice(1), value: 1 };
}

export function compareHands(a: HandResult, b: HandResult): number {
  if (a.value !== b.value) return a.value - b.value;

  for (let i = 0; i < Math.min(a.cards.length, b.cards.length); i++) {
    const aVal = RANK_VALUES[a.cards[i].rank];
    const bVal = RANK_VALUES[b.cards[i].rank];
    if (aVal !== bVal) return aVal - bVal;
  }

  for (let i = 0; i < Math.min(a.kickers.length, b.kickers.length); i++) {
    const aVal = RANK_VALUES[a.kickers[i].rank];
    const bVal = RANK_VALUES[b.kickers[i].rank];
    if (aVal !== bVal) return aVal - bVal;
  }

  return 0;
}

export function getHandName(rank: HandRank): string {
  const names: Record<HandRank, string> = {
    high_card: 'High Card',
    one_pair: 'One Pair',
    two_pair: 'Two Pair',
    three_of_a_kind: 'Three of a Kind',
    straight: 'Straight',
    flush: 'Flush',
    full_house: 'Full House',
    four_of_a_kind: 'Four of a Kind',
    straight_flush: 'Straight Flush',
    royal_flush: 'Royal Flush',
  };
  return names[rank];
}
