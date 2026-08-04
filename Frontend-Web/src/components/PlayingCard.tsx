import { parseCard, suitSymbol } from "@/lib/cards";

interface Props {
  code?: string;
  faceDown?: boolean;
  size?: "sm" | "md";
}

export function PlayingCard({ code, faceDown, size = "md" }: Props) {
  const dim = size === "sm" ? "h-[52px] w-[36px] text-[11px]" : "h-[72px] w-[50px] text-sm";

  if (faceDown || !code) {
    return (
      <div className={`zt-playing-card back ${dim}`} aria-label="Carta fechada" />
    );
  }

  const card = parseCard(code);
  if (!card) {
    return (
      <div className={`zt-playing-card ${dim} items-center justify-center text-xs text-ink`}>
        {code}
      </div>
    );
  }

  const color = card.red ? "text-red-700" : "text-gray-900";

  return (
    <div className={`zt-playing-card ${dim} ${color}`}>
      <span className="font-bold leading-none">{card.rank}</span>
      <span className="self-center text-lg leading-none">{suitSymbol(card.suit)}</span>
      <span className="self-end rotate-180 font-bold leading-none">{card.rank}</span>
    </div>
  );
}
