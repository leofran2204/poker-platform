import type { PlayerWsData, PotWsData, ShowdownEntry } from "@/api/types";
import { cashbackFor } from "@/lib/deflator";
import { parseBrlToCents } from "@/lib/money";

// Mãos de demonstração da home (`homeContent.json`) traduzidas para o estado
// que a mesa do jogo (`PokerTable`) desenha. Assim as demonstrações usam a
// mesma mesa oval, as mesmas placas e as mesmas animações da mesa de verdade.

export interface AnimatedStreet {
  label: string;
  board: string[];
  /** Cartas visíveis de cada assento neste passo (no Brazilian Pineapple, a troca muda uma carta). */
  holes?: string[][];
  /** Narração deste passo (MP3). */
  audio?: string;
}

export interface AnimatedSeat {
  name: string;
  cards?: string[];
  showdownHand?: string;
  folded?: boolean;
  isHero?: boolean;
  isWinner?: boolean;
  stack?: string;
}

export interface WinningFive {
  /** Cartas da mão do vencedor usadas no melhor jogo de 5. */
  hole: string[];
  /** Cartas do board usadas no melhor jogo de 5. */
  board: string[];
}

export interface AnimatedHandData {
  id: string;
  title: string;
  heroLabel: string;
  heroCards: string[];
  villainLabel: string;
  villainCards: string[];
  streets: AnimatedStreet[];
  equity: number;
  potCents: number;
  result: string;
  /** Índice da street em que o all-in foi pago. Ausente = exemplo sem all-in (variantes). */
  allInStreetIndex?: number;
  /** Assentos da mesa. Ausente = derivado de herói/vilão + 1 fictício foldado. */
  seats?: AnimatedSeat[];
  /** As 5 cartas do jogo vencedor. Ausente = destaca a mão do vencedor (legado). */
  winningFive?: WinningFive;
}

/** Fichas de quem não tem valor no roteiro. Como todo o resto, é ilustrativo. */
const DEFAULT_STACK_CENTS = 10_000;
/** Mesa de seis lugares: herói embaixo e os outros dois no alto, à esquerda e à direita. */
export const DEMO_TABLE_CAP = 6;
const DEMO_SEATS = [0, 2, 4];

/** Até três assentos; sem `seats` no roteiro, deriva de herói e vilão. */
export function demoSeats(hand: AnimatedHandData): AnimatedSeat[] {
  if (hand.seats && hand.seats.length > 0) return hand.seats.slice(0, DEMO_SEATS.length);
  if (hand.villainCards.length === 0) {
    return [
      { name: hand.heroLabel, cards: hand.heroCards, isHero: true, isWinner: true },
      { name: "Leo", folded: true },
      { name: "Mari", folded: true },
    ];
  }
  return [
    { name: hand.heroLabel, cards: hand.heroCards, isHero: true },
    { name: hand.villainLabel, cards: hand.villainCards, isWinner: true },
    { name: "Ana", folded: true },
  ];
}

export const demoSeatId = (index: number) => `demo-seat-${index}`;

function stageOf(boardLength: number): string {
  if (boardLength === 0) return "preflop";
  if (boardLength === 3) return "flop";
  if (boardLength === 4) return "turn";
  return "river";
}

/**
 * Troca ou descarte deste passo, deduzido das cartas: 1 se a mão do assento
 * mudou em relação ao passo anterior, 0 se outro assento mexeu e este não,
 * `null` fora das rodadas de troca e descarte.
 */
function drawCounts(hand: AnimatedHandData, step: number, seatCount: number): (number | null)[] {
  const now = hand.streets[step]?.holes;
  const before = step > 0 ? hand.streets[step - 1]?.holes : undefined;
  if (!now || !before) return Array.from({ length: seatCount }, () => null);
  const changed = Array.from({ length: seatCount }, (_, seat) => {
    const current = now[seat] ?? [];
    const previous = before[seat] ?? [];
    return current.length !== previous.length || current.some((card) => !previous.includes(card));
  });
  if (!changed.some(Boolean)) return changed.map(() => null);
  return changed.map((didChange) => (didChange ? 1 : 0));
}

export interface DemoTableState {
  players: PlayerWsData[];
  communityCards: string[];
  stage: string;
  pots: PotWsData[];
  winners: string[];
  showdown: ShowdownEntry[];
  heroId: string;
}

/**
 * Estado da mesa num passo da demonstração. `revealed` = showdown aberto: as
 * cartas aparecem, o pote é pago e, nas mãos com all-in elegível, a parte do
 * Loss Deflator volta para quem perdeu.
 */
export function demoTableState(hand: AnimatedHandData, step: number, revealed: boolean): DemoTableState {
  const seats = demoSeats(hand);
  const street = hand.streets[Math.min(Math.max(step, 0), hand.streets.length - 1)];
  const heroIndex = Math.max(0, seats.findIndex((seat) => seat.isHero));
  const winnerIndex = Math.max(0, seats.findIndex((seat) => seat.isWinner));
  const draws = drawCounts(hand, step, seats.length);

  const allInAt = hand.allInStreetIndex;
  const hasAllIn = typeof allInAt === "number";
  const allInPaid = hasAllIn && step >= allInAt;
  // Antes do all-in só há o pote pequeno das primeiras apostas.
  const openingPot = hasAllIn ? Math.trunc(hand.potCents / 10) : hand.potCents;
  // `equity` é a do herói: a devolução só existe quando é ele quem perde o all-in.
  const cashback =
    hasAllIn && winnerIndex !== heroIndex && hand.equity >= 56
      ? cashbackFor(hand.equity, hand.potCents).cashbackCents
      : 0;
  const baseStack = (seat: AnimatedSeat) =>
    (seat.stack ? parseBrlToCents(seat.stack) : null) ?? DEFAULT_STACK_CENTS;

  const chipsOf = (seat: AnimatedSeat, index: number): number => {
    if (seat.folded) return baseStack(seat);
    if (!hasAllIn) return baseStack(seat) + (revealed && index === winnerIndex ? hand.potCents : 0);
    if (revealed) {
      if (index === winnerIndex) return hand.potCents - cashback;
      return index === heroIndex ? cashback : 0;
    }
    return allInPaid ? 0 : Math.trunc((hand.potCents - openingPot) / 2);
  };

  const players: PlayerWsData[] = seats.map((seat, index) => {
    const shown = street.holes?.[index] ?? seat.cards ?? [];
    const faceUp = !seat.folded && (index === heroIndex || revealed);
    return {
      id: demoSeatId(index),
      name: seat.name,
      chips: chipsOf(seat, index),
      bet: 0,
      cards: faceUp ? shown : [],
      folded: seat.folded === true,
      is_active: false,
      is_dealer: index === seats.length - 1,
      seat: DEMO_SEATS[index],
      is_sitting: true,
      draw_count: seat.folded || revealed ? null : draws[index],
    };
  });

  const potNow = revealed ? 0 : hasAllIn && !allInPaid ? openingPot : hand.potCents;
  const winnerSeat = seats[winnerIndex];
  const winnerHole = street.holes?.[winnerIndex] ?? winnerSeat.cards ?? [];
  const five = hand.winningFive;
  const winningCards =
    five && five.hole.length + five.board.length === 5 ? [...five.hole, ...five.board] : winnerHole;

  return {
    players,
    communityCards: street.board,
    stage: revealed ? "showdown" : stageOf(street.board.length),
    pots: potNow > 0 ? [{ name: "Pote", amount: potNow, eligible_players: [] }] : [],
    winners: revealed ? [demoSeatId(winnerIndex)] : [],
    showdown: revealed
      ? [
          {
            player_id: demoSeatId(winnerIndex),
            player_name: winnerSeat.name,
            hand_name: null,
            cards: winningCards,
          },
        ]
      : [],
    heroId: demoSeatId(heroIndex),
  };
}

/** Fichas na mesa (pote + pilhas) num passo: constante do começo ao fim da mão. */
export function demoChipsInPlay(state: DemoTableState): number {
  return (
    state.pots.reduce((sum, pot) => sum + pot.amount, 0) +
    state.players.reduce((sum, player) => sum + player.chips, 0)
  );
}
