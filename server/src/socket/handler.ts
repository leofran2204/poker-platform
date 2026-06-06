import { Server as SocketServer, Socket } from 'socket.io';
import jwt from 'jsonwebtoken';
import { TableManager } from '../engine/table-manager';
import { query } from '../db';
import { PlayerAction } from '@shared/types';

interface AuthenticatedSocket extends Socket {
  userId?: string;
  username?: string;
}

interface TableRoom {
  manager: TableManager;
  players: Map<string, { socketId: string; userId: string; username: string }>;
}

const tableRooms = new Map<string, TableRoom>();

export function setupSocketHandlers(io: SocketServer): void {
  // Auth middleware for socket connections
  io.use((socket: AuthenticatedSocket, next) => {
    const token = socket.handshake.auth.token || socket.handshake.query.token;
    if (!token) {
      next(new Error('Authentication required'));
      return;
    }

    try {
      const decoded = jwt.verify(token as string, process.env.JWT_SECRET || 'secret') as {
        userId: string;
        username: string;
      };
      socket.userId = decoded.userId;
      socket.username = decoded.username;
      next();
    } catch (error) {
      next(new Error('Invalid token'));
    }
  });

  io.on('connection', (socket: AuthenticatedSocket) => {
    console.log(`User connected: ${socket.username} (${socket.id})`);

    // Join lobby
    socket.join('lobby');
    emitLobbyUpdate(io);

    // Create table
    socket.on('createTable', async (config: any) => {
      try {
        const tableId = config.id || require('uuid').v4();
        const tableConfig = {
          id: tableId,
          name: config.name,
          gameType: config.gameType || 'cash',
          smallBlind: config.smallBlind || 1,
          bigBlind: config.bigBlind || 2,
          minBuyIn: config.minBuyIn || 20,
          maxBuyIn: config.maxBuyIn || 200,
          maxPlayers: config.maxPlayers || 6,
          speed: config.speed || 'normal',
          ante: config.ante,
        };

        const manager = new TableManager(tableConfig);
        tableRooms.set(tableId, {
          manager,
          players: new Map(),
        });

        io.to('lobby').emit('lobbyUpdate', getLobbyTables());
        socket.emit('tableCreated', tableId);
      } catch (error) {
        socket.emit('error', 'Failed to create table');
      }
    });

    // Join table
    socket.on('joinTable', async (tableId: string, buyIn: number) => {
      try {
        const room = tableRooms.get(tableId);
        if (!room) {
          socket.emit('error', 'Table not found');
          return;
        }

        // Check if already at table
        if (room.players.has(socket.userId!)) {
          socket.emit('error', 'Already at this table');
          return;
        }

        // Check if table is full
        if (room.players.size >= room.manager.state.config.maxPlayers) {
          socket.emit('error', 'Table is full');
          return;
        }

        // Get user balance
        const userResult = await query('SELECT balance FROM users WHERE id = $1', [socket.userId]);
        if (userResult.rows.length === 0) {
          socket.emit('error', 'User not found');
          return;
        }

        const balance = parseFloat(userResult.rows[0].balance);
        if (balance < buyIn) {
          socket.emit('error', 'Insufficient balance');
          return;
        }

        // Deduct buy-in
        await query(
          'UPDATE users SET balance = balance - $1 WHERE id = $2',
          [buyIn, socket.userId]
        );

        // Add player to table
        room.manager.addPlayer({
          id: socket.userId!,
          username: socket.username!,
          chips: buyIn,
          seatIndex: 0,
          avatar: 'default',
        });

        room.players.set(socket.userId!, {
          socketId: socket.id,
          userId: socket.userId!,
          username: socket.username!,
        });

        socket.join(tableId);
        socket.emit('balanceUpdate', balance - buyIn);

        // Start hand if enough players
        if (room.manager.canStartHand()) {
          room.manager.startHand();
        }

        // Send table state to all players
        io.to(tableId).emit('tableUpdate', room.manager.getPublicState());
        io.to('lobby').emit('lobbyUpdate', getLobbyTables());
      } catch (error) {
        socket.emit('error', 'Failed to join table');
      }
    });

    // Leave table
    socket.on('leaveTable', () => {
      leaveTable(socket, io);
    });

    // Player action
    socket.on('playerAction', (action: PlayerAction, amount?: number) => {
      handlePlayerAction(socket, io, action, amount);
    });

    // Chat message
    socket.on('chatMessage', (message: string) => {
      handleChatMessage(socket, io, message);
    });

    // Sit in
    socket.on('sitIn', (seatIndex: number) => {
      // Handled by joinTable
    });

    // Sit out
    socket.on('sitOut', () => {
      leaveTable(socket, io);
    });

    // Add chips
    socket.on('addChips', async (amount: number) => {
      try {
        const userResult = await query('SELECT balance FROM users WHERE id = $1', [socket.userId]);
        if (userResult.rows.length === 0) return;

        const balance = parseFloat(userResult.rows[0].balance);
        if (balance < amount) {
          socket.emit('error', 'Insufficient balance');
          return;
        }

        await query('UPDATE users SET balance = balance - $1 WHERE id = $2', [amount, socket.userId]);

        // Find which table the player is at
        for (const [tableId, room] of tableRooms) {
          const playerEntry = room.players.get(socket.userId!);
          if (playerEntry) {
            const player = room.manager.state.players.find(p => p.id === socket.userId);
            if (player) {
              player.chips += amount;
              io.to(tableId).emit('tableUpdate', room.manager.getPublicState());
            }
            break;
          }
        }

        socket.emit('balanceUpdate', balance - amount);
      } catch (error) {
        socket.emit('error', 'Failed to add chips');
      }
    });

    // Disconnect
    socket.on('disconnect', () => {
      console.log(`User disconnected: ${socket.username} (${socket.id})`);
      leaveTable(socket, io);
      socket.leave('lobby');
    });
  });
}

function handlePlayerAction(
  socket: AuthenticatedSocket,
  io: SocketServer,
  action: PlayerAction,
  amount?: number
): void {
  for (const [tableId, room] of tableRooms) {
    const playerEntry = room.players.get(socket.userId!);
    if (playerEntry && playerEntry.socketId === socket.id) {
      const success = room.manager.handleAction(socket.userId!, action, amount);
      if (success) {
        io.to(tableId).emit('tableUpdate', room.manager.getPublicState());

        // Emit hand result if showdown
        if (room.manager.state.phase === 'showdown') {
          const playersInHand = room.manager.state.players.filter(p => !p.hasFolded);
          const results = playersInHand.map(p => ({
            id: p.id,
            cards: p.cards,
            hand: evaluateHandForResult(p.cards, room.manager.state.communityCards),
          }));

          io.to(tableId).emit('handResult', {
            winners: room.manager.state.pots.flatMap(pot =>
              pot.eligiblePlayers.map(pid => ({
                playerId: pid,
                amount: pot.amount / pot.eligiblePlayers.length,
                hand: results.find(r => r.id === pid)?.hand || { rank: 'high_card', cards: [], kickers: [], value: 1 },
              }))
            ),
            players: results,
          });
        }
      } else {
        socket.emit('error', 'Invalid action');
      }
      break;
    }
  }
}

function handleChatMessage(
  socket: AuthenticatedSocket,
  io: SocketServer,
  message: string
): void {
  for (const [tableId, room] of tableRooms) {
    const playerEntry = room.players.get(socket.userId!);
    if (playerEntry && playerEntry.socketId === socket.id) {
      io.to(tableId).emit('chatMessage', {
        id: require('uuid').v4(),
        userId: socket.userId!,
        username: socket.username!,
        message: message.substring(0, 500),
        timestamp: Date.now(),
        type: 'chat',
      });
      break;
    }
  }
}

function leaveTable(socket: AuthenticatedSocket, io: SocketServer): void {
  for (const [tableId, room] of tableRooms) {
    const playerEntry = room.players.get(socket.userId!);
    if (playerEntry && playerEntry.socketId === socket.id) {
      room.manager.removePlayer(socket.userId!);
      room.players.delete(socket.userId!);
      socket.leave(tableId);

      // Refund chips
      const player = room.manager.state.players.find(p => p.id === socket.userId);
      if (player) {
        query('UPDATE users SET balance = balance + $1 WHERE id = $2',
          [player.chips, socket.userId]);
      }

      io.to(tableId).emit('tableUpdate', room.manager.getPublicState());
      io.to('lobby').emit('lobbyUpdate', getLobbyTables());

      // Clean up empty tables
      if (room.players.size === 0) {
        tableRooms.delete(tableId);
      }
      break;
    }
  }
}

function getLobbyTables(): any[] {
  return Array.from(tableRooms.entries()).map(([id, room]) => ({
    id,
    name: room.manager.state.config.name,
    gameType: room.manager.state.config.gameType,
    smallBlind: room.manager.state.config.smallBlind,
    bigBlind: room.manager.state.config.bigBlind,
    minBuyIn: room.manager.state.config.minBuyIn,
    maxBuyIn: room.manager.state.config.maxBuyIn,
    maxPlayers: room.manager.state.config.maxPlayers,
    speed: room.manager.state.config.speed,
    currentPlayers: room.players.size,
    isRunning: room.manager.state.isRunning,
  }));
}

function emitLobbyUpdate(io: SocketServer): void {
  io.to('lobby').emit('lobbyUpdate', getLobbyTables());
}

function evaluateHandForResult(cards: any[], communityCards: any[]): any {
  const { evaluateHand } = require('../engine/deck');
  return evaluateHand(cards, communityCards);
}