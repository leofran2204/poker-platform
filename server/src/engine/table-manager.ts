import { v4 as uuidv4 } from 'uuid';
import {
  Card, Player, PlayerAction, TableConfig, TableState,
  GamePhase, Pot, HandResult
} from '@shared/types';
import { createDeck, shuffleDeck, dealCards, evaluateHand, compareHands } from './deck';
import { calculateLossDeflator, getHeadsupWinProbability } from './loss-deflator';

const ACTION_TIMEOUTS = {
  normal: 30000,
  turbo: 15000,
  hyper: 8000,
};

export class TableManager {
  public state: TableState;
  private timer: NodeJS.Timeout | null = null;
  private onTimeout: ((tableId: string) => void) | null = null;

  constructor(config: TableConfig) {
    this.state = {
      config,
      players: [],
      communityCards: [],
      pots: [],
      currentPot: 0,
      deck: [],
      phase: 'waiting',
      currentPlayerIndex: -1,
      dealerIndex: -1,
      smallBlindAmount: config.smallBlind,
      bigBlindAmount: config.bigBlind,
      currentBet: 0,
      minRaise: config.bigBlind,
      handNumber: 0,
      isRunning: false,
      timeLeft: 0,
      allInCallContext: undefined,
    };
  }

  addPlayer(player: Omit<Player, 'bet' | 'totalBet' | 'cards' | 'isActive' | 'isAllIn' | 'hasFolded' | 'isDealer' | 'isSmallBlind' | 'isBigBlind' | 'lastAction'>): boolean {
    if (this.state.players.length >= this.state.config.maxPlayers) return false;
    if (this.state.players.find(p => p.id === player.id)) return false;

    const takenSeats = new Set(this.state.players.map(p => p.seatIndex));
    let seatIndex = 0;
    while (takenSeats.has(seatIndex)) seatIndex++;

    this.state.players.push({
      ...player,
      bet: 0,
      totalBet: 0,
      cards: [],
      isActive: true,
      isAllIn: false,
      hasFolded: false,
      isDealer: false,
      isSmallBlind: false,
      isBigBlind: false,
      seatIndex,
      lastAction: undefined,
    });

    return true;
  }

  removePlayer(playerId: string): boolean {
    const idx = this.state.players.findIndex(p => p.id === playerId);
    if (idx === -1) return false;
    this.state.players.splice(idx, 1);
    return true;
  }

  getActivePlayers(): Player[] {
    return this.state.players.filter(p => p.isActive && !p.hasFolded && !p.isAllIn);
  }

  getPlayersInHand(): Player[] {
    return this.state.players.filter(p => !p.hasFolded);
  }

  private recordAllInCall(action: PlayerAction, player: Player, becameAllIn: boolean): void {
    if (action !== 'call' || !becameAllIn) return;

    const playersInHand = this.getPlayersInHand();
    if (playersInHand.length !== 2) return;
    if (this.state.communityCards.length >= 5) return;
    if (this.state.phase === 'showdown') return;

    const opponent = playersInHand.find(p => p.id !== player.id);
    if (!opponent) return;

    this.state.allInCallContext = {
      board: [...this.state.communityCards],
      heroId: player.id,
      heroCards: [...player.cards],
      villainId: opponent.id,
      villainCards: [...opponent.cards],
      phase: this.state.phase,
      pot: this.state.currentPot,
    };
  }

  canStartHand(): boolean {
    const activePlayers = this.state.players.filter(p => p.isActive);
    return activePlayers.length >= 2;
  }

  startHand(): void {
    if (!this.canStartHand()) return;

    this.state.handNumber++;
    this.state.communityCards = [];
    this.state.pots = [];
    this.state.currentPot = 0;
    this.state.currentBet = 0;
    this.state.minRaise = this.state.config.bigBlind;
    this.state.allInCallContext = undefined;

    // Reset player states
    const activePlayers = this.state.players.filter(p => p.isActive);
    for (const player of activePlayers) {
      player.cards = [];
      player.bet = 0;
      player.totalBet = 0;
      player.hasFolded = false;
      player.isAllIn = false;
      player.isDealer = false;
      player.isSmallBlind = false;
      player.isBigBlind = false;
      player.lastAction = undefined;
    }

    // Set dealer
    if (this.state.dealerIndex === -1) {
      this.state.dealerIndex = activePlayers[0].seatIndex;
    } else {
      // Move dealer to next active player
      const currentDealerIdx = activePlayers.findIndex(p => p.seatIndex === this.state.dealerIndex);
      this.state.dealerIndex = activePlayers[(currentDealerIdx + 1) % activePlayers.length].seatIndex;
    }

    const dealer = activePlayers.find(p => p.seatIndex === this.state.dealerIndex)!;
    dealer.isDealer = true;

    // Set blinds
    const dealerIdx = activePlayers.indexOf(dealer);
    const sbIdx = (dealerIdx + 1) % activePlayers.length;
    const bbIdx = (dealerIdx + 2) % activePlayers.length;

    const sb = activePlayers[sbIdx];
    const bb = activePlayers[bbIdx];
    sb.isSmallBlind = true;
    bb.isBigBlind = true;

    // Post blinds
    const sbAmount = Math.min(this.state.config.smallBlind, sb.chips);
    const bbAmount = Math.min(this.state.config.bigBlind, bb.chips);
    sb.chips -= sbAmount;
    sb.bet = sbAmount;
    sb.totalBet = sbAmount;
    bb.chips -= bbAmount;
    bb.bet = bbAmount;
    bb.totalBet = bbAmount;
    this.state.currentPot = sbAmount + bbAmount;
    this.state.currentBet = bbAmount;

    // Deal cards
    let deck = shuffleDeck(createDeck());
    for (const player of activePlayers) {
      const result = dealCards(deck, 2);
      player.cards = result.cards;
      deck = result.remaining;
    }
    this.state.deck = deck;

    // Start preflop - action starts after big blind
    this.state.phase = 'preflop';
    this.state.currentPlayerIndex = (bbIdx + 1) % activePlayers.length;
    this.state.isRunning = true;

    this.startTimer();
  }

  dealCommunityCards(count: number): void {
    // Burn one card
    this.state.deck = this.state.deck.slice(1);
    const result = dealCards(this.state.deck, count);
    this.state.communityCards.push(...result.cards);
    this.state.deck = result.remaining;
  }

  handleAction(playerId: string, action: PlayerAction, amount?: number): boolean {
    const player = this.state.players.find(p => p.id === playerId);
    if (!player) return false;

    const activePlayers = this.getActivePlayers();
    const currentPlayer = activePlayers[this.state.currentPlayerIndex];
    if (!currentPlayer || currentPlayer.id !== playerId) return false;

    player.lastAction = action;

    switch (action) {
      case 'fold':
        player.hasFolded = true;
        break;

      case 'check':
        // Valid only if no bet to call
        if (this.state.currentBet > player.bet) {
          return false; // Jogador tentou dar check com aposta pendente
        }
        break;

      case 'call':
        const callAmount = Math.min(this.state.currentBet - player.bet, player.chips);
        const becameAllIn = callAmount === player.chips;
        player.chips -= callAmount;
        player.bet += callAmount;
        player.totalBet += callAmount;
        this.state.currentPot += callAmount;
        if (player.chips === 0) player.isAllIn = true;
        this.recordAllInCall(action, player, becameAllIn);
        break;

      case 'bet':
        if (amount && amount >= this.state.config.bigBlind) {
          const betAmount = Math.min(amount, player.chips);
          player.chips -= betAmount;
          player.bet += betAmount;
          player.totalBet += betAmount;
          this.state.currentPot += betAmount;
          this.state.currentBet = betAmount;
          this.state.minRaise = betAmount;
          if (player.chips === 0) player.isAllIn = true;
        }
        break;

      case 'raise':
        if (amount && amount >= this.state.currentBet + this.state.minRaise) {
          const raiseAmount = Math.min(amount, player.chips);
          player.chips -= raiseAmount;
          player.bet += raiseAmount;
          player.totalBet += raiseAmount;
          this.state.currentPot += raiseAmount;
          this.state.currentBet = raiseAmount;
          this.state.minRaise = amount - this.state.currentBet;
          if (player.chips === 0) player.isAllIn = true;
        }
        break;

      case 'all_in':
        const allInAmount = player.chips;
        player.chips = 0;
        player.bet += allInAmount;
        player.totalBet += allInAmount;
        this.state.currentPot += allInAmount;
        if (allInAmount > this.state.currentBet) {
          this.state.currentBet = allInAmount;
        }
        player.isAllIn = true;
        break;
    }

    this.advanceGame();
    return true;
  }

  private advanceGame(): void {
    this.stopTimer();

    const activePlayers = this.getActivePlayers();
    const playersInHand = this.getPlayersInHand();

    // Check if only one player remains
    if (playersInHand.length === 1) {
      this.endHand();
      return;
    }

    // Check if all active players are all-in or have acted
    const allActed = this.haveAllActed();

    if (allActed) {
      switch (this.state.phase) {
        case 'preflop':
          this.state.phase = 'flop';
          this.dealCommunityCards(3);
          break;
        case 'flop':
          this.state.phase = 'turn';
          this.dealCommunityCards(1);
          break;
        case 'turn':
          this.state.phase = 'river';
          this.dealCommunityCards(1);
          break;
        case 'river':
          this.endHand();
          return;
      }

      // Reset bets for next street
      for (const player of this.state.players) {
        player.bet = 0;
      }
      this.state.currentBet = 0;
      this.state.minRaise = this.state.config.bigBlind;

      // Set first active player after dealer
      const activePlayersList = this.state.players.filter(p => p.isActive && !p.hasFolded);
      const dealerIdx = activePlayersList.findIndex(p => p.isDealer);
      this.state.currentPlayerIndex = (dealerIdx + 1) % activePlayersList.length;
    } else {
      // Move to next player
      const activePlayersList = this.state.players.filter(p => p.isActive && !p.hasFolded);
      this.state.currentPlayerIndex = (this.state.currentPlayerIndex + 1) % activePlayersList.length;
    }

    this.startTimer();
  }

  private haveAllActed(): boolean {
    const activePlayers = this.state.players.filter(p => p.isActive && !p.hasFolded && !p.isAllIn);
    if (activePlayers.length === 0) return true;

    // Check if all active players have equal bets
    const bets = activePlayers.map(p => p.bet);
    const allEqual = bets.every(b => b === bets[0]);
    return allEqual && bets[0] === this.state.currentBet;
  }

  private endHand(): void {
    this.state.phase = 'showdown';
    this.state.isRunning = false;
    this.stopTimer();

    // Evaluate hands for all players
    const results = this.state.players
      .filter(p => !p.hasFolded)
      .map(p => ({
        player: p,
        hand: evaluateHand(p.cards, this.state.communityCards),
      }));

    // Sort by hand strength
    results.sort((a, b) => compareHands(b.hand, a.hand));

    // Find all winners (split pot on tie)
    const topHand = results[0].hand;
    const winners = results.filter(r => compareHands(r.hand, topHand) === 0);
    const splitAmount = this.state.currentPot / winners.length;

    let lossDeflatorResult = null;
    let payoutByWinner = splitAmount;

    if (winners.length === 1 && results.length === 2 && this.state.allInCallContext) {
      const loserResult = results.find(r => r.player.id !== winners[0].player.id);
      if (loserResult) {
        const heroOdds = getHeadsupWinProbability(
          this.state.allInCallContext.heroCards,
          this.state.allInCallContext.villainCards,
          this.state.allInCallContext.board
        );
        const loserOdds = loserResult.player.id === this.state.allInCallContext.heroId
          ? heroOdds
          : 1 - heroOdds;

        const deflator = calculateLossDeflator({
          pot: this.state.currentPot,
          loserId: loserResult.player.id,
          winnerId: winners[0].player.id,
          loserOdds,
        });

        if (deflator) {
          lossDeflatorResult = deflator;
          payoutByWinner = Math.max(0, splitAmount - deflator.cashback);
        }
      }
    }

    // Award pot to winner(s)
    for (const winner of winners) {
      const winnerPlayer = this.state.players.find(p => p.id === winner.player.id)!;
      winnerPlayer.chips += payoutByWinner;
    }

    if (lossDeflatorResult) {
      const loserPlayer = this.state.players.find(p => p.id === lossDeflatorResult.loserId)!;
      loserPlayer.chips += lossDeflatorResult.cashback;
    }

    // Prepare hand result data
    const handResult = {
      winners: winners.map(w => ({
        playerId: w.player.id,
        amount: payoutByWinner,
        hand: w.hand,
      })),
      players: results.map(r => ({
        id: r.player.id,
        cards: r.player.cards,
        hand: r.hand,
      })),
      lossDeflator: lossDeflatorResult,
    };

    // Emit hand result (will be sent via socket)
    this.state.pots = [{ amount: this.state.currentPot, eligiblePlayers: winners.map(w => w.player.id) }];

    // Auto-start next hand after delay
    setTimeout(() => {
      if (this.state.players.filter(p => p.isActive).length >= 2) {
        this.startHand();
      }
    }, 5000);
  }

  private startTimer(): void {
    this.stopTimer();
    const timeout = ACTION_TIMEOUTS[this.state.config.speed];
    this.state.timeLeft = timeout / 1000;

    this.timer = setInterval(() => {
      this.state.timeLeft--;
      if (this.state.timeLeft <= 0) {
        // Auto-fold on timeout
        const activePlayers = this.getActivePlayers();
        const currentPlayer = activePlayers[this.state.currentPlayerIndex];
        if (currentPlayer) {
          this.handleAction(currentPlayer.id, 'fold');
        }
      }
    }, 1000);
  }

  private stopTimer(): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  setOnTimeout(callback: (tableId: string) => void): void {
    this.onTimeout = callback;
  }

  getState(): TableState {
    return { ...this.state };
  }

  getPublicState(): any {
    return {
      config: this.state.config,
      players: this.state.players.map(p => ({
        ...p,
        cards: p.id === 'current' ? p.cards : [], // Only show own cards
      })),
      communityCards: this.state.communityCards,
      pots: this.state.pots,
      currentPot: this.state.currentPot,
      phase: this.state.phase,
      currentPlayerIndex: this.state.currentPlayerIndex,
      dealerIndex: this.state.dealerIndex,
      smallBlindAmount: this.state.smallBlindAmount,
      bigBlindAmount: this.state.bigBlindAmount,
      currentBet: this.state.currentBet,
      minRaise: this.state.minRaise,
      handNumber: this.state.handNumber,
      isRunning: this.state.isRunning,
      timeLeft: this.state.timeLeft,
    };
  }
}