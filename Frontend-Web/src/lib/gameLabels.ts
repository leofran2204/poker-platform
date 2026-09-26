type GameDescriptor = {
  poker_variant?: string;
  game_type?: string;
  final_table_variant?: string | null;
  final_table_max_players?: number | null;
};

function isOmaha(game: GameDescriptor): boolean {
  const variant = game.poker_variant ?? "";
  if (variant === "omaha" || variant === "short_deck_omaha") return true;
  return (game.game_type ?? "").toLowerCase().includes("omaha");
}

function isBrazilianPineapple(game: GameDescriptor): boolean {
  const variant = game.poker_variant ?? "";
  if (
    variant === "brazilian_pineapple" ||
    variant === "ultimate_pineapple"
  ) {
    return true;
  }
  return (game.game_type ?? "").toLowerCase().includes("pineapple");
}

export function gameNameLabel(
  game: GameDescriptor,
  format: "cash" | "tournament",
): string {
  const formatName = format === "cash" ? "Cash Game" : "Torneio";
  if (isBrazilianPineapple(game)) return `Brazilian Pineapple — ${formatName}`;
  if (isOmaha(game)) return `Omaha 4 Cartas — ${formatName}`;
  if (game.poker_variant === "short_deck") return `Texas Hold’em Short Deck — ${formatName}`;
  return `Texas Hold’em — ${formatName}`;
}

export function streetLabel(stage: string): string {
  const key = (stage || "").toLowerCase();
  const map: Record<string, string> = {
    waiting: "Aguardando",
    preflop: "Pré-flop",
    flop: "Flop",
    turn: "Turn",
    river: "River",
    showdown: "Showdown",
    finished: "Fim da mão",
  };
  return map[key] ?? stage ?? "—";
}

export function tournamentStatusLabel(status: string): string {
  const key = (status || "").toLowerCase();
  const map: Record<string, string> = {
    registering: "Inscrições abertas",
    running: "Em andamento",
    paused: "Pausado",
    finished: "Encerrado",
    cancelled: "Cancelado",
    canceled: "Cancelado",
  };
  return map[key] ?? status;
}

/** Dica curta da variante para a mesa ao vivo. */
export const HAND_NAME_PT: Record<string, string> = {
  "High Card": "Carta Alta",
  "One Pair": "Um Par",
  "Two Pair": "Dois Pares",
  "Three of a Kind": "Trinca",
  Straight: "Sequência",
  Flush: "Flush",
  "Full House": "Full House",
  "Four of a Kind": "Quadra",
  "Straight Flush": "Straight Flush",
  "Royal Flush": "Royal Flush",
};

export function handNamePt(name: string | null | undefined): string {
  if (!name) return "a melhor mão";
  return HAND_NAME_PT[name] ?? name;
}

export function variantHint(variant?: string | null): string | null {
  switch (variant) {
    case "omaha":
    case "short_deck_omaha":
      return "Omaha 4 cartas · baralho 52 · exatamente 2 hole + 3 board";
    case "brazilian_pineapple":
    case "ultimate_pineapple":
      return "Brazilian Pineapple · baralho 52 · 2+1+1+1 · 2 da mão + 3 da mesa · ranking clássico";
    case "short_deck":
      return "Texas Hold’em Short Deck · baralho 36 (6 a A) · flush vence full house";
    case "holdem":
      return "Texas Hold’em · 2 cartas · ranking clássico";
    default:
      return null;
  }
}

export function deckTypeLabel(game: GameDescriptor): string {
  const variant = game.poker_variant ?? "";
  const gameType = (game.game_type ?? "").toLowerCase();
  return variant === "short_deck" || gameType.includes("short deck")
    ? "Short Deck"
    : "Tradicional";
}
