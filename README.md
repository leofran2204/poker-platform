# Poker Platform - Online Texas Hold'em

A full-stack online poker platform with a real-time table engine for traditional Texas Hold'em.

## Current Rules
- Game mode: traditional Texas Hold'em using a full 52-card deck
- Flow: blinds, betting rounds, showdown, and pot distribution
- Special mechanic: a loss deflator for specific heads-up all-in-call situations
  - Applies only when the losing player was involved in a pre-river all-in call
  - Requires the losing hand equity to be at least 55%
  - Does not apply to coin-flip situations
  - Cashback tiers are 15%, 25%, or 35% of the pot depending on equity

## Tech Stack
- **Backend**: Node.js + Express + Socket.io + PostgreSQL
- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Auth**: JWT + bcrypt
- **Real-time**: WebSocket (Socket.io)

## Getting Started

### Prerequisites
- Node.js 18+
- PostgreSQL 14+

### Setup
```bash
# Install dependencies
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

## Build Verification
```bash
cd server && npm run build
cd ../client && npm run build
```