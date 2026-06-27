import { Card, HandResult, LossDeflatorResult } from '@shared/types';
import { evaluateHand, compareHands } from './deck';

const RANKS: Card['rank'][] = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2'];
const SUITS: Card['suit'][] = ['hearts', 'diamonds', 'clubs', 'spades'];

function createFullDeck(): Card[] {
  const deck: Card[] = [];
  for (const suit of SUITS) {
    for (const rank of RANKS) {
      deck.push({ rank, suit });
    }
  }
  return deck;
}

function cardEquals(a: Card, b: Card): boolean {
  return a.rank === b.rank && a.suit === b.suit;
}

function containsCard(cards: Card[], card: Card): boolean {
  return cards.some(c => cardEquals(c, card));
}

function getRemainingDeck(knownCards: Card[]): Card[] {
  const fullDeck = createFullDeck();
  return fullDeck.filter(card => !containsCard(knownCards, card));
}

function combinations<T>(items: T[], k: number): T[][] {
  const result: T[][] = [];
  const combination: T[] = [];

  function backtrack(start: number): void {
    if (combination.length === k) {
      result.push([...combination]);
      return;
    }
    for (let i = start; i < items.length; i++) {
      combination.push(items[i]);
      backtrack(i + 1);
      combination.pop();
    }
  }

  backtrack(0);
  return result;
}

function evaluateOutcome(heroCards: Card[], villainCards: Card[], board: Card[]): number {
  const heroHand = evaluateHand(heroCards, board);
  const villainHand = evaluateHand(villainCards, board);
  const comparison = compareHands(heroHand, villainHand);
  if (comparison > 0) return 1;
  if (comparison === 0) return 0.5;
  return 0;
}

export function getHeadsupWinProbability(heroCards: Card[], villainCards: Card[], boardCards: Card[]): number {
  const knownCards = [...heroCards, ...villainCards, ...boardCards];
  const remainingDeck = getRemainingDeck(knownCards);
  const cardsToDeal = 5 - boardCards.length;

  if (cardsToDeal === 0) {
    return evaluateOutcome(heroCards, villainCards, boardCards);
  }

  const boardCombos = combinations(remainingDeck, cardsToDeal);
  let wins = 0;
  let ties = 0;

  for (const combo of boardCombos) {
    const finalBoard = [...boardCards, ...combo];
    const outcome = evaluateOutcome(heroCards, villainCards, finalBoard);
    if (outcome === 1) wins += 1;
    else if (outcome === 0.5) ties += 1;
  }

  const total = boardCombos.length;
  return (wins + ties * 0.5) / total;
}

export function calculateLossDeflator(
  params: {
    pot: number;
    loserId: string;
    winnerId: string;
    loserOdds: number;
  }
): LossDeflatorResult | null {
  const { pot, loserId, winnerId, loserOdds } = params;

  if (loserOdds < 0.55) return null;
  if (loserOdds >= 0.9) {
    return {
      loserId,
      winnerId,
      cashback: Math.round(pot * 0.35 * 100) / 100,
      odds: loserOdds,
      tier: '35%',
    };
  }
  if (loserOdds >= 0.7) {
    return {
      loserId,
      winnerId,
      cashback: Math.round(pot * 0.25 * 100) / 100,
      odds: loserOdds,
      tier: '25%',
    };
  }
  if (loserOdds >= 0.55) {
    return {
      loserId,
      winnerId,
      cashback: Math.round(pot * 0.15 * 100) / 100,
      odds: loserOdds,
      tier: '15%',
    };
  }

  return null;
}
