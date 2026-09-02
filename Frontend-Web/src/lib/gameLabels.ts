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
  if (isUltimatePineapple(game)) return `Ultimate Pineapple — ${format === "cash" ? "Cash Game" : "Torneio"}`;
  const gameName = isOmahaFourCards(game) ? "Omaha 4 Cartas" : "Hold’em";
  const formatName = format === "cash" ? "Cash Game" : "Torneio";
  return `${gameName} — ${formatName}`;
}

export function deckTypeLabel(game: GameDescriptor): string {
  if (game.final_table_variant === "short_deck") {
    return "Tradicional (Mesa Final Short Deck)";
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
