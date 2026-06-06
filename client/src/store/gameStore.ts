import { create } from 'zustand';

export interface Card {
  suit: 'h' | 'd' | 'c' | 's';
  rank: string;
}

export interface Player {
  id: string;
  userId: string;
  username: string;
  avatar: string;
  stack: number;
  bet: number;
  holeCards: Card[];
  folded: boolean;
  isDealer: boolean;
  isCurrentTurn: boolean;
  isAllIn: boolean;
  isSittingOut: boolean;
  lastAction: string | null;
  seatIndex: number;
}

export interface TableState {
  id: string;
  name: string;
  players: Player[];
  communityCards: Card[];
  pot: number;
  currentBet: number;
  minRaise: number;
  phase: 'waiting' | 'preflop' | 'flop' | 'turn' | 'river' | 'showdown';
  dealerIndex: number;
  currentPlayerIndex: number;
  smallBlind: number;
  bigBlind: number;
  minBuyIn: number;
  maxBuyIn: number;
  maxPlayers: number;
  handId: string | null;
  winners: Array<{ playerId: string; username: string; handName: string; winAmount: number }> | null;
  lastHandResults: Array<{ playerId: string; username: string; handName: string; winAmount: number }> | null;
}

interface GameState {
  currentTable: TableState | null;
  tables: any[];
  loading: boolean;
  error: string | null;
  setCurrentTable: (table: TableState | null) => void;
  setTables: (tables: any[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  updatePlayer: (playerId: string, updates: Partial<Player>) => void;
  fetchTables: (token: string) => Promise<void>;
}

export const useGameStore = create<GameState>((set, get) => ({
  currentTable: null,
  tables: [],
  loading: false,
  error: null,

  setCurrentTable: (table) => set({ currentTable: table }),
  setTables: (tables) => set({ tables }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),

  updatePlayer: (playerId, updates) => {
    const table = get().currentTable;
    if (!table) return;
    set({
      currentTable: {
        ...table,
        players: table.players.map((p) =>
          p.id === playerId ? { ...p, ...updates } : p
        ),
      },
    });
  },

  fetchTables: async (token) => {
    set({ loading: true });
    try {
      const res = await fetch('http://localhost:3001/api/tables', {
        headers: { Authorization: `Bearer ${token}` },
      });
      const data = await res.json();
      if (res.ok) set({ tables: data.tables, loading: false });
      else set({ error: data.error, loading: false });
    } catch (err: any) {
      set({ error: err.message, loading: false });
    }
  },
}));