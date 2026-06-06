import { create } from 'zustand';
import { io, Socket } from 'socket.io-client';
import { useGameStore } from './gameStore';

interface ChatMessage {
  userId: string;
  username: string;
  message: string;
  timestamp: string;
}

interface SocketState {
  socket: Socket | null;
  connected: boolean;
  connect: () => void;
  disconnect: () => void;
}

export const useSocketStore = create<SocketState>((set, get) => ({
  socket: null,
  connected: false,

  connect: () => {
    const token = localStorage.getItem('poker-auth');
    if (!token) return;

    const newSocket = io('http://localhost:3001', {
      auth: { token },
      transports: ['websocket', 'polling'],
    });

    newSocket.on('connect', () => {
      set({ connected: true });
    });

    newSocket.on('disconnect', () => {
      set({ connected: false });
    });

    newSocket.on('connect_error', (err) => {
      console.error('Socket connection error:', err.message);
      set({ connected: false });
    });

    // Game events
    newSocket.on('tableState', (state: any) => {
      useGameStore.getState().setCurrentTable(state);
    });

    newSocket.on('playerJoined', (data: any) => {
      const table = useGameStore.getState().currentTable;
      if (table) {
        useGameStore.getState().setCurrentTable({
          ...table,
          players: [...table.players, data.player],
        });
      }
    });

    newSocket.on('playerLeft', (data: any) => {
      const table = useGameStore.getState().currentTable;
      if (table) {
        useGameStore.getState().setCurrentTable({
          ...table,
          players: table.players.filter((p) => p.userId !== data.userId),
        });
      }
    });

    newSocket.on('handStarted', (data: any) => {
      const table = useGameStore.getState().currentTable;
      if (table) {
        useGameStore.getState().setCurrentTable({
          ...table,
          ...data,
        });
      }
    });

    newSocket.on('actionPerformed', (data: any) => {
      const table = useGameStore.getState().currentTable;
      if (table) {
        useGameStore.getState().setCurrentTable({
          ...table,
          players: table.players.map((p) =>
            p.userId === data.userId
              ? { ...p, stack: data.stack, bet: data.bet, lastAction: data.action }
              : p
          ),
          pot: data.pot ?? table.pot,
          currentBet: data.currentBet ?? table.currentBet,
          currentPlayerIndex: data.nextPlayerIndex ?? table.currentPlayerIndex,
          phase: data.phase ?? table.phase,
        });
      }
    });

    newSocket.on('communityCards', (data: any) => {
      const table = useGameStore.getState().currentTable;
      if (table) {
        useGameStore.getState().setCurrentTable({
          ...table,
          communityCards: data.cards,
          phase: data.phase,
          pot: data.pot,
        });
      }
    });

    newSocket.on('handResult', (data: any) => {
      const table = useGameStore.getState().currentTable;
      if (table) {
        useGameStore.getState().setCurrentTable({
          ...table,
          winners: data.winners,
          communityCards: data.communityCards || table.communityCards,
          phase: 'showdown',
          pot: 0,
        });
      }
    });

    newSocket.on('tableError', (data: any) => {
      console.error('Table error:', data.message);
    });

    newSocket.on('lobbyUpdate', (data: any) => {
      useGameStore.getState().setTables(data.tables);
    });

    set({ socket: newSocket });
  },

  disconnect: () => {
    const { socket } = get();
    if (socket) {
      socket.removeAllListeners();
      socket.disconnect();
    }
    set({ socket: null, connected: false });
  },
}));