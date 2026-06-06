// Shared types between client and server

export type Suit = 'hearts' | 'diamonds' | 'clubs' | 'spades';
export type Rank = 'A' | 'K' | 'Q' | 'J' | 'T' | '9' | '8' | '7' | '6';

export interface Card {
  rank: Rank;
  suit: Suit;
}

export type HandRank =
  | 'high_card'
  | 'one_pair'
  | 'two_pair'
  | 'three_of_a_kind'
  | 'straight'
  | 'flush'
  | 'full_house'
  | 'four_of_a_kind'
  | 'straight_flush'
  | 'royal_flush';

export interface HandResult {
  rank: HandRank;
  cards: Card[];
  kickers: Card[];
  value: number;
}

export type PlayerAction = 'fold' | 'check' | 'call' | 'bet' | 'raise' | 'all_in';
export type GamePhase = 'waiting' | 'preflop' | 'flop' | 'turn' | 'river' | 'showdown';
export type GameType = 'cash' | 'tournament';
export type GameSpeed = 'normal' | 'turbo' | 'hyper';

export interface Player {
  id: string;
  username: string;
  chips: number;
  bet: number;
  totalBet: number;
  cards: Card[];
  isActive: boolean;
  isAllIn: boolean;
  hasFolded: boolean;
  isDealer: boolean;
  isSmallBlind: boolean;
  isBigBlind: boolean;
  seatIndex: number;
  avatar: string;
  lastAction?: PlayerAction;
}

export interface Pot {
  amount: number;
  eligiblePlayers: string[];
}

export interface TableConfig {
  id: string;
  name: string;
  gameType: GameType;
  smallBlind: number;
  bigBlind: number;
  minBuyIn: number;
  maxBuyIn: number;
  maxPlayers: number;
  speed: GameSpeed;
  ante?: number;
}

export interface TableState {
  config: TableConfig;
  players: Player[];
  communityCards: Card[];
  pots: Pot[];
  currentPot: number;
  deck: Card[];
  phase: GamePhase;
  currentPlayerIndex: number;
  dealerIndex: number;
  smallBlindAmount: number;
  bigBlindAmount: number;
  currentBet: number;
  minRaise: number;
  handNumber: number;
  isRunning: boolean;
  timeLeft: number;
}

export interface ChatMessage {
  id: string;
  userId: string;
  username: string;
  message: string;
  timestamp: number;
  type: 'chat' | 'system' | 'action';
}

export interface User {
  id: string;
  username: string;
  email: string;
  balance: number;
  avatar: string;
  createdAt: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

// Socket events
export interface ServerToClientEvents {
  tableUpdate: (state: TableState) => void;
  gameStarted: (state: TableState) => void;
  playerAction: (data: { playerId: string; action: PlayerAction; amount?: number }) => void;
  chatMessage: (message: ChatMessage) => void;
  handResult: (data: { winners: { playerId: string; amount: number; hand: HandResult }[]; players: { id: string; cards: Card[]; hand: HandResult }[] }) => void;
  error: (message: string) => void;
  lobbyUpdate: (tables: TableConfig[]) => void;
  balanceUpdate: (balance: number) => void;
}

export interface ClientToServerEvents {
  joinTable: (tableId: string, buyIn: number) => void;
  leaveTable: () => void;
  playerAction: (action: PlayerAction, amount?: number) => void;
  chatMessage: (message: string) => void;
  sitIn: (seatIndex: number) => void;
  sitOut: () => void;
  addChips: (amount: number) => void;
  createTable: (config: Omit<TableConfig, 'id'>) => void;
}