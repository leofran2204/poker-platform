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

export function gameNameLabel(
  game: GameDescriptor,
  format: "cash" | "tournament",
): string {
  const gameName = isOmahaFourCards(game) ? "Omaha 4 Cartas" : "Hold’em";
  const formatName = format === "cash" ? "Cash Game" : "Torneio";
  return `${gameName} — ${formatName}`;
}

export function deckTypeLabel(game: GameDescriptor): string {
  if (game.final_table_variant === "short_deck") {
    return "Long/Short (mesa final)";
  }
  const variant = game.poker_variant ?? "";
  const gameType = (game.game_type ?? "").toLowerCase();
  return variant === "short_deck" ||
    variant === "short_deck_omaha" ||
    gameType.includes("short")
    ? "Short Deck"
    : "Tradicional";
}
