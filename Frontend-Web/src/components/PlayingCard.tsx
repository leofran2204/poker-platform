import { parseCard, suitSymbol } from "@/lib/cards";

interface Props {
  code?: string;
  faceDown?: boolean;
  size?: "xs" | "sm" | "md";
  /** Inline no texto de tips — mesmo visual da mesa, menor. */
  inline?: boolean;
}

const SIZE_CLASS: Record<NonNullable<Props["size"]>, string> = {
  xs: "h-7 w-5 text-[9px] p-0.5",
  sm: "h-[52px] w-[36px] text-[11px]",
  md: "h-[72px] w-[50px] text-sm",
};

export function PlayingCard({ code, faceDown, size = "md", inline = false }: Props) {
  const dim = SIZE_CLASS[size];
  const wrap = inline ? "inline-flex align-middle mx-0.5 shrink-0" : "";

  if (faceDown || !code) {
    return (
      <div className={`zt-playing-card back ${dim} ${wrap}`} aria-label="Carta fechada" />
    );
  }

  const card = parseCard(code);
  if (!card) {
    return (
      <div className={`zt-playing-card ${dim} ${wrap} items-center justify-center text-xs text-ink`}>
        {code}
      </div>
    );
  }

  const color = card.red ? "text-red-700" : "text-gray-900";
  const suitSize = size === "xs" ? "text-xs leading-none" : "self-center text-lg leading-none";

  return (
    <div
      className={`zt-playing-card ${dim} ${wrap} ${color}`}
      title={`${card.rank}${suitSymbol(card.suit)}`}
      aria-label={`${card.rank} de ${card.suit}`}
    >
      <span className="font-bold leading-none">{card.rank}</span>
      <span className={suitSize}>{suitSymbol(card.suit)}</span>
      {size !== "xs" && (
        <span className="self-end rotate-180 font-bold leading-none">{card.rank}</span>
      )}
    </div>
  );
}
