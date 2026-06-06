# Poker Platform - Online Texas Hold'em Short Deck

A full-stack online poker platform inspired by Full Tilt Poker, featuring Texas Hold'em Short Deck (6+ Hold'em).

## Tech Stack
- **Backend**: Node.js + Express + Socket.io + PostgreSQL
- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Auth**: JWT + bcrypt
- **Real-time**: WebSocket (Socket.io)

## Short Deck Rules
- Deck: 36 cards (A, K, Q, J, T, 9, 8, 7, 6 — no 2-5)
- Hand rankings: Flush beats Full House
- A-6-7-8-9 makes a straight (Ace plays low)

## Getting Started

### Prerequisites
- Node.js 18+
- PostgreSQL 14+

### Setup
```bash
# Install all dependencies
cd server && npm install
cd ../client && npm install

# Setup database
createdb poker_platform
psql poker_platform < server/src/db/schema.sql

# Environment
cp server/.env.example server/.env
# Edit .env with your database credentials

# Run
cd server && npm run dev
cd client && npm run dev
```