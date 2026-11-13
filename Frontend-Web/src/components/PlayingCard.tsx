import { parseCard, suitSymbol } from "@/lib/cards";

interface Props {
  code?: string;
  faceDown?: boolean;
  size?: "xs" | "sm" | "md";
  /** Inline no texto de tips — mesmo visual da mesa, menor. */
  inline?: boolean;
  /** Realce dourado (carta usada no jogo vencedor do showdown). */
  highlight?: boolean;
}

const SUIT_PT: Record<"s" | "h" | "d" | "c", string> = {
  s: "espadas",
  h: "copas",
  d: "ouros",
  c: "paus",
};

const SIZE_CLASS: Record<NonNullable<Props["size"]>, string> = {
  xs: "h-7 w-5 text-[9px] p-0.5",
  sm: "h-[52px] w-[36px] text-[11px]",
  md: "h-[72px] w-[50px] text-sm",
};

export function PlayingCard({ code, faceDown, size = "md", inline = false, highlight = false }: Props) {
  const dim = SIZE_CLASS[size];
  const wrap = inline ? "inline-flex align-middle mx-0.5 shrink-0" : "";
  const glow = highlight ? "zt-card-winner" : "";

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
  const corner = size === "md" ? "text-[11px]" : "text-[9px]";
  const big = size === "md" ? "text-[30px]" : "text-[21px]";

  return (
    <div
      className={`zt-playing-card ${dim} ${wrap} ${glow} ${color}`}
      title={`${card.rank}${suitSymbol(card.suit)}`}
      aria-label={`${card.rank} de ${SUIT_PT[card.suit]}`}
    >
      {size === "xs" ? (
        <>
          <span className="leading-none">{card.rank}</span>
          <span className="text-xs leading-none">{suitSymbol(card.suit)}</span>
        </>
      ) : (
        <>
          <span className="flex flex-col items-start leading-none">
            <span>{card.rank}</span>
            <span className={`${corner} leading-none`}>{suitSymbol(card.suit)}</span>
          </span>
          <span className={`${big} self-end leading-none`} aria-hidden="true">
            {suitSymbol(card.suit)}
          </span>
        </>
      )}
    </div>
  );
}
