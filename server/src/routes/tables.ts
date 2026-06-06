import { Router, Response } from 'express';
import { z } from 'zod';
import { v4 as uuidv4 } from 'uuid';
import { query } from '../db';
import { authMiddleware, AuthRequest } from '../middleware/auth';
import { TableConfig, GameType, GameSpeed } from '@shared/types';

const router = Router();

const createTableSchema = z.object({
  name: z.string().min(3).max(100),
  gameType: z.enum(['cash', 'tournament']),
  smallBlind: z.number().positive(),
  bigBlind: z.number().positive(),
  minBuyIn: z.number().positive(),
  maxBuyIn: z.number().positive(),
  maxPlayers: z.number().int().min(2).max(9),
  speed: z.enum(['normal', 'turbo', 'hyper']),
  ante: z.number().min(0).optional(),
});

// Get all active tables
router.get('/', async (_req: AuthRequest, res: Response) => {
  try {
    const result = await query(
      `SELECT id, name, game_type, small_blind, big_blind,
              min_buy_in, max_buy_in, max_players, speed, ante
       FROM game_tables
       WHERE is_active = true
       ORDER BY created_at DESC`
    );

    const tables = result.rows.map(row => ({
      id: row.id,
      name: row.name,
      gameType: row.game_type,
      smallBlind: parseFloat(row.small_blind),
      bigBlind: parseFloat(row.big_blind),
      minBuyIn: parseFloat(row.min_buy_in),
      maxBuyIn: parseFloat(row.max_buy_in),
      maxPlayers: row.max_players,
      speed: row.speed,
      ante: row.ante ? parseFloat(row.ante) : undefined,
    }));

    res.json(tables);
  } catch (error) {
    console.error('Get tables error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Create table
router.post('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const config = createTableSchema.parse(req.body);

    const result = await query(
      `INSERT INTO game_tables (name, game_type, small_blind, big_blind,
        min_buy_in, max_buy_in, max_players, speed, ante)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
       RETURNING id`,
      [config.name, config.gameType, config.smallBlind, config.bigBlind,
       config.minBuyIn, config.maxBuyIn, config.maxPlayers, config.speed,
       config.ante || 0]
    );

    res.status(201).json({ id: result.rows[0].id });
  } catch (error) {
    if (error instanceof z.ZodError) {
      res.status(400).json({ error: 'Invalid input', details: error.errors });
      return;
    }
    console.error('Create table error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

export default router;