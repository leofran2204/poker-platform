import historyWorld from "@/data/pokerHistoryWorld.json";
import historyBrazil from "@/data/pokerHistoryBrazil.json";
import { TipRichText } from "@/components/TipRichText";

type Variant = "world" | "brazil";

const DATA: Record<Variant, typeof historyWorld> = {
  world: historyWorld,
  brazil: historyBrazil,
};

export function PokerHistory({ variant }: { variant: Variant }) {
  const data = DATA[variant];
  const theme = variant === "world" ? "from-amber-900/30 via-felt-800 to-felt-900" : "from-emerald-900/30 via-felt-800 to-felt-900";

  return (
    <div className="zt-panel overflow-hidden">
      <div className={`border-b border-felt-600 bg-gradient-to-r ${theme} px-4 py-3`}>
        <h2 className="text-sm font-bold uppercase tracking-wide text-gold-bright">{data.title}</h2>
        <p className="text-xs text-felt-300">{data.subtitle}</p>
      </div>
      <div className="max-h-[520px] space-y-0 overflow-y-auto px-4 py-3 scrollbar-thin">
        <div className="relative border-l-2 border-gold/20 pl-6">
          {data.blocks.map((block, idx) => (
            <div key={idx} className="relative pb-5 last:pb-0">
              <span className="absolute -left-[25px] top-1 flex h-3 w-3 items-center justify-center rounded-full border-2 border-gold bg-felt-800" />
              <div className="flex items-baseline gap-2">
                <span className="rounded bg-gold/20 px-1.5 py-0.5 text-xs font-bold text-gold-bright">{block.year}</span>
                <h3 className="text-sm font-semibold text-cream">{block.title}</h3>
              </div>
              <div className="mt-1.5 text-sm leading-relaxed text-felt-200">
                <TipRichText text={block.text} className="text-sm leading-relaxed" />
              </div>
              <div className="mt-2 rounded border border-gold/20 bg-gold/10 px-2.5 py-1.5 text-xs leading-relaxed text-gold-soft">
                <span className="font-bold">Para você:</span> {block.takeaway}
              </div>
            </div>
          ))}
        </div>
      </div>
      <div className="border-t border-felt-600 bg-felt-900/40 px-4 py-2 text-center">
        <p className="text-[11px] text-felt-400">História curta — o essencial para começar com contexto</p>
      </div>
    </div>
  );
}
