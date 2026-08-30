type GameDescriptor = {
  poker_variant?: string;
  game_type?: string;
};

function isOmahaFourCards(game: GameDescriptor): boolean {
  if (game.poker_variant === "short_deck_omaha") return true;
  return (game.game_type ?? "").toLowerCase().includes("omaha");
}

export function gameNameLabel(
  game: GameDescriptor,
  format: "cash" | "tournament",
): string {
  const gameName = isOmahaFourCards(game) ? "Omaha 4 Cartas" : "Texas Hold’em";
  const formatName = format === "cash" ? "Cash Game" : "Torneio";
  return `${gameName} — ${formatName}`;
}

export function deckTypeLabel(game: GameDescriptor): string {
  const variant = game.poker_variant ?? "";
  const gameType = (game.game_type ?? "").toLowerCase();
  return variant === "short_deck" ||
    variant === "short_deck_omaha" ||
    gameType.includes("short")
    ? "Short Deck"
    : "Baralho Tradicional";
}
