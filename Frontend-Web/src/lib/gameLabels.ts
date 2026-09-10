type GameDescriptor = {
  poker_variant?: string;
  game_type?: string;
  final_table_variant?: string | null;
  final_table_max_players?: number | null;
};

function isOmahaFourCards(game: GameDescriptor): boolean {
  if (game.poker_variant === "short_deck_omaha") return true;
  return (game.game_type ?? "").toLowerCase().includes("omaha");
}

function isUltimatePineapple(game: GameDescriptor): boolean {
  if (game.poker_variant === "ultimate_pineapple") return true;
  return (game.game_type ?? "").toLowerCase().includes("pineapple");
}

export function gameNameLabel(
  game: GameDescriptor,
  format: "cash" | "tournament",
): string {
  const formatName = format === "cash" ? "Cash Game" : "Torneio";
  if (isUltimatePineapple(game)) return `Ultimate Pineapple — ${formatName}`;
  if (isOmahaFourCards(game)) return `Omaha 4 Cartas — ${formatName}`;
  if (game.poker_variant === "short_deck" || (game.game_type ?? "").toLowerCase().includes("short")) {
    return `Texas Hold’em Short Deck — ${formatName}`;
  }
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
    case "short_deck":
      return "Short Deck · trinca > sequência · flush > full house";
    case "short_deck_omaha":
      return "Omaha 4 cartas · 2 hole + 3 board · ranking Short Deck";
    case "ultimate_pineapple":
      return "Ultimate Pineapple · 3 cartas, sem descarte · 2 hole + 3 board";
    case "holdem":
      return "Texas Hold’em · 2 cartas";
    default:
      return null;
  }
}

export function deckTypeLabel(game: GameDescriptor): string {
  if (game.final_table_variant === "short_deck") {
    return "Tradicional / FT Short Deck";
  }
  const variant = game.poker_variant ?? "";
  const gameType = (game.game_type ?? "").toLowerCase();
  return variant === "short_deck" ||
    variant === "short_deck_omaha" ||
    variant === "ultimate_pineapple" ||
    gameType.includes("short") ||
    gameType.includes("pineapple")
    ? "Short Deck"
    : "Tradicional";
}
