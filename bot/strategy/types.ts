// Tipos compartilhados da estrategia do bot Zero Tilt.
// Baselines simplificados e auditaveis. Nao sao saidas exatas de solver.

export type Rank = '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | 'T' | 'J' | 'Q' | 'K' | 'A';
export type Suit = 's' | 'h' | 'd' | 'c';
export type Card = `${Rank}${Suit}`;
export type Hand = [Card, Card];

export type Position =
  | 'UTG' | 'UTG+1' | 'UTG+2' | 'MP1' | 'MP2' | 'MP3'
  | 'LJ' | 'HJ' | 'CO' | 'BTN' | 'SB' | 'BB';

export type Action = 'fold' | 'call' | 'raise' | 'check' | 'bet' | 'allin';
export type Street = 'preflop' | 'flop' | 'turn' | 'river';
export type GameType =
  | 'cash_6max' | 'cash_9max' | 'mtt' | 'sng' | 'pko'
  | 'hu_cash' | 'hu_sng';

export interface ActionRecord {
  street: Street;
  position: Position;
  action: Action;
  sizeBB?: number;
}

export interface Opponent {
  position: Position;
  stackBB: number;
  vpip?: number;
  pfr?: number;
  threeBet?: number;
  foldTo3Bet?: number;
  foldToCBet?: number;
  hands?: number;
}

export interface ICMContext {
  payouts: number[];
  allStacks: number[];
  playersLeft: number;
  paidPlaces: number;
}

export interface GameState {
  gameType: GameType;
  street: Street;
  position: Position;
  stackBB: number;
  effectiveStackBB: number;
  potBB: number;
  betToCallBB: number;
  myHand: Hand;
  board: Card[];
  actionHistory: ActionRecord[];
  opponents: Opponent[];
  icm?: ICMContext;
  minRaiseBB?: number;
}

export interface StrategyOutput {
  action: Action;
  sizeBB?: number;
  reasoning: string;
}

/** Faixa de 0 a 1. Ex: 0.33 = 33%. */
export type Ratio = number;
