import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { Card } from "../components/Card";
import { ChipStack, Pot } from "../components/Chips";
import { Sfx, SpadeMark } from "../components/Frame";
import { Seat, Table } from "../components/Table";
import { Body, Headline, Kicker, Panel, Pill, Stamp } from "../components/Text";
import { CLAMP, glide, ramp, rise } from "../lib/anim";
import { flight } from "../lib/deal";
import { C, DISPLAY, UI } from "../theme";
import { SceneMap, SceneProps } from "../Video";

// Fatos da aula m2l2 (courseContent.json): pote 10 BB, aposta 3, raise para 12;
// faltam 9, pote após o call 34, limiar 9/34 ≈ 26,47%; K♦9♦ em K♣8♠3♥.
const HERO = ["Kd", "9d"];
const FLOP = ["Kc", "8s", "3h"];

/** Centraliza o filho em (x, y); `style` anima um invólucro interno e não desfaz a centralização. */
const Abs: React.FC<{ x: number; y: number; children: React.ReactNode; style?: React.CSSProperties }> = ({ x, y, children, style }) => (
  <div style={{ position: "absolute", left: x, top: y, transform: "translate(-50%, -50%)" }}>
    <div style={style}>{children}</div>
  </div>
);

const Label: React.FC<{ children: React.ReactNode; color?: string; size?: number }> = ({ children, color = C.muted, size = 24 }) => (
  <div style={{ fontFamily: UI, fontWeight: 700, fontSize: size, color, letterSpacing: 0.5 }}>{children}</div>
);

/* 1. Gancho: a mão do check-raise acontece na mesa. */
const Hook: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const aposta = cue("aposta");
  const check = cue("check");
  const aumento = cue("aumento");
  const options = [
    { label: "PAGAR", at: cue("Pagar") },
    { label: "DESISTIR", at: cue("desistir") },
    { label: "AUMENTAR", at: cue("aumentar") },
  ];
  const heroBet = flight(frame, aposta, fps, { x: 900, y: 860 }, { x: 1300, y: 700 }, 0);
  const oponenteBet = flight(frame, aumento, fps, { x: 900, y: 270 }, { x: 1300, y: 390 }, 0);
  const raised = frame >= aumento;
  return (
    <AbsoluteFill>
      <Table x={900} y={560} width={1240} height={600} opacity={ramp(frame, 0, 18)} />
      <Seat
        x={900}
        y={270}
        name="Oponente"
        stack="fora de posição"
        badge={raised ? "RAISE 12" : "CHECK"}
        badgeColor={raised ? C.danger : C.muted}
        badgeOpacity={raised ? ramp(frame, aumento, 8) : ramp(frame, 6, 12) * (1 - 0.4 * Math.max(0, 1 - Math.abs(frame - check) / 20))}
        active={raised ? glide(frame, aumento, fps, 10) : 0}
      />
      <Seat x={900} y={880} name="Você" stack="em posição" hero dealer badge={frame >= aposta ? "APOSTA 3" : undefined} badgeOpacity={ramp(frame, aposta, 8)} />
      {FLOP.map((code, i) => (
        <Abs key={code} x={900 + (i - 1) * 124} y={520} style={rise(frame, 4 + i * 4, 30)}>
          <Card code={code} width={110} />
        </Abs>
      ))}
      {HERO.map((code, i) => (
        <Abs key={code} x={900 + (i - 0.5) * 100} y={690} style={rise(frame, 10 + i * 4, 30)}>
          <Card code={code} width={90} />
        </Abs>
      ))}
      <Abs x={900} y={410}>
        <Pot value="10 BB" opacity={ramp(frame, 8, 14)} />
      </Abs>
      <Abs x={heroBet.x - 60} y={heroBet.y} style={{ opacity: heroBet.opacity }}>
        <ChipStack count={2} color="red" size={52} label="3 BB" labelOpacity={ramp(frame, aposta + 14, 10)} />
      </Abs>
      <Abs x={oponenteBet.x - 60} y={oponenteBet.y} style={{ opacity: oponenteBet.opacity }}>
        <ChipStack count={5} color="black" size={52} label="12 BB" labelOpacity={ramp(frame, aumento + 14, 10)} />
      </Abs>
      <Abs x={1640} y={300}>
        <Stamp text="CHECK-RAISE" at={aumento + 8} size={64} color={C.danger} rotate={-6} />
      </Abs>
      <div style={{ position: "absolute", left: 1540, top: 470, display: "flex", flexDirection: "column", gap: 22 }}>
        {options.map((option) => (
          <Pill key={option.label} tone="muted" at={option.at} size={30} style={{ justifyContent: "center", width: 260 }}>
            {option.label}
          </Pill>
        ))}
      </div>
      <Sfx at={aposta + 18} name="chips" volume={0.4} />
      <Sfx at={aumento + 18} name="chips" volume={0.45} />
      <Sfx at={aumento + 8} name="stamp" volume={0.35} />
      {options.map((option) => (
        <Sfx key={option.label} at={option.at} name="pop" volume={0.25} />
      ))}
    </AbsoluteFill>
  );
};

/* 2. Título da aula. */
const Title: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const como = cue("Como");
  const arte = cue("arte");
  return (
    <AbsoluteFill style={{ padding: "0 160px", justifyContent: "center" }}>
      <Kicker at={2}>Módulo 2 · Aula 2</Kicker>
      <Headline text="Como responder ao check-raise" at={como} size={112} style={{ marginTop: 22 }} />
      <Headline text="e a arte do float" at={arte} size={112} color={C.gold} accent={["float"]} />
      <div style={{ marginTop: 50, maxWidth: 1100, ...rise(frame, arte + 30) }}>
        <Panel at={arte + 30}>
          <Label color={C.mint} size={22}>
            OBJETIVO
          </Label>
          <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 38, color: C.ivory, marginTop: 8 }}>
            Recalcular o preço depois da agressão do oponente.
          </div>
        </Panel>
      </div>
      <Sfx at={como} name="whoosh" volume={0.3} />
    </AbsoluteFill>
  );
};

/* 3. Definição e os quatro fatores. */
const What: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const steps = [
    { text: "Check", at: cue("passar") },
    { text: "Aposta do oponente", at: cue("aposta") },
    { text: "Raise", at: cue("aumentar") },
  ];
  const factors = [
    { title: "Preço", note: "quanto custa continuar", at: cue("custa") },
    { title: "Stacks", note: "quanto ainda há atrás", at: cue("stacks") },
    { title: "Textura", note: "o que o board conecta", at: cue("textura") },
    { title: "Range", note: "o que o oponente pode ter", at: cue("range") },
  ];
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 140, top: 170, width: 760 }}>
        <Kicker at={0}>Check-raise</Kicker>
        <Headline text="Passar, e depois aumentar" at={4} size={70} accent={["aumentar"]} style={{ marginTop: 14 }} />
        <div style={{ display: "flex", alignItems: "center", gap: 16, marginTop: 50 }}>
          {steps.map((step, i) => (
            <React.Fragment key={step.text}>
              {i > 0 && <div style={{ fontFamily: UI, fontSize: 40, color: C.faint, opacity: ramp(frame, step.at - 4, 10) }}>→</div>}
              <Pill tone={i === 2 ? "danger" : "muted"} at={step.at} size={28}>
                {i + 1} · {step.text}
              </Pill>
            </React.Fragment>
          ))}
        </div>
        <div style={{ display: "flex", gap: 18, marginTop: 60 }}>
          <Pill tone="ok" at={cue("forte")} size={28}>
            Pode ser valor
          </Pill>
          <Pill tone="danger" at={cue("blefe")} size={28}>
            Pode ser blefe
          </Pill>
        </div>
      </div>
      <div style={{ position: "absolute", left: 1000, top: 200, width: 780 }}>
        <Body at={cue("quatro")} size={32} color={C.goldSoft}>
          Para responder, olhe:
        </Body>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24, marginTop: 24 }}>
          {factors.map((factor) => (
            <Panel key={factor.title} at={factor.at} accent={C.mint} style={{ padding: "26px 28px" }}>
              <div style={{ fontFamily: DISPLAY, fontWeight: 600, fontSize: 52, color: C.ivory }}>{factor.title}</div>
              <Label>{factor.note}</Label>
            </Panel>
          ))}
        </div>
      </div>
      {[...steps, ...factors].map((item, i) => (
        <Sfx key={i} at={item.at} name="pop" volume={0.22} />
      ))}
    </AbsoluteFill>
  );
};

/* 4. O pote: 10 + 3 + 12 = 25 BB. */
const Price: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const parts = [
    { label: "Pote", value: 10, at: cue("dez"), color: "#2f8a5a" },
    { label: "Sua aposta", value: 3, at: cue("três"), color: C.gold },
    { label: "Raise do oponente", value: 12, at: cue("doze"), color: C.danger },
  ];
  const vinte = cue("vinte");
  const width = 1500;
  const total = parts.reduce((sum, part) => sum + part.value * glide(frame, part.at, fps, 20), 0);
  let x = 0;
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 210, top: 170 }}>
        <Kicker at={0}>Refaça o preço</Kicker>
        <Headline text="Quanto há no meio agora?" at={4} size={76} style={{ marginTop: 12 }} />
      </div>
      <div style={{ position: "absolute", left: 210, top: 560, width, height: 110 }}>
        {parts.map((part) => {
          const w = (part.value / 25) * width;
          const grow = glide(frame, part.at, fps, 22);
          const left = x;
          x += w;
          return (
            <div key={part.label} style={{ position: "absolute", left, top: 0, width: w * grow, height: 110 }}>
              <div style={{ width: "100%", height: "100%", background: part.color, borderRadius: 14, boxShadow: "inset 0 -10px 0 rgba(0,0,0,0.18)" }} />
              <div style={{ position: "absolute", left: 0, top: -88, opacity: ramp(frame, part.at + 6, 12), whiteSpace: "nowrap" }}>
                <Label>{part.label}</Label>
                <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 48, color: C.ivory }}>{part.value} BB</div>
              </div>
            </div>
          );
        })}
      </div>
      <Abs x={960} y={830}>
        <div style={{ display: "flex", alignItems: "baseline", gap: 22, opacity: ramp(frame, parts[0].at, 12) }}>
          <span style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 130, color: C.gold, fontVariantNumeric: "tabular-nums" }}>
            {Math.round(total)}
          </span>
          <span style={{ fontFamily: UI, fontWeight: 700, fontSize: 40, color: C.muted }}>BB no meio</span>
        </div>
      </Abs>
      <Abs x={960} y={950}>
        <Pill tone="gold" at={vinte}>
          10 + 3 + 12 = 25 BB
        </Pill>
      </Abs>
      {parts.map((part) => (
        <Sfx key={part.label} at={part.at} name="chips" volume={0.35} />
      ))}
    </AbsoluteFill>
  );
};

/** Indicador circular de porcentagem. */
const Gauge: React.FC<{ value: number; at: number; size?: number }> = ({ value, at, size = 420 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = glide(frame, at, fps, 40);
  const radius = 160;
  const circumference = 2 * Math.PI * radius;
  const shown = value * t;
  return (
    <div style={{ position: "relative", width: size, height: size, opacity: ramp(frame, at - 10, 12) }}>
      <svg width={size} height={size} viewBox="0 0 400 400">
        <circle cx="200" cy="200" r={radius} fill="none" stroke="rgba(255,255,255,0.08)" strokeWidth="34" />
        <circle
          cx="200"
          cy="200"
          r={radius}
          fill="none"
          stroke={C.gold}
          strokeWidth="34"
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={circumference * (1 - shown / 100)}
          transform="rotate(-90 200 200)"
        />
      </svg>
      <div style={{ position: "absolute", inset: 0, display: "grid", placeItems: "center", textAlign: "center" }}>
        <div>
          <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 96, color: C.ivory }}>{shown.toFixed(1).replace(".", ",")}%</div>
          <Label>equity mínima</Label>
        </div>
      </div>
    </div>
  );
};

/* 5. A conta: faltam 9, pote 34, 9 ÷ 34 ≈ 26,5%. */
const Price2: React.FC<SceneProps> = ({ cue }) => {
  const rows = [
    { label: "Falta pagar", expr: "12 − 3", result: "= 9 BB", at: cue("faltam") },
    { label: "Pote após o call", expr: "25 + 9", result: "= 34 BB", at: cue("trinta") },
    { label: "Equity mínima", expr: "9 ÷ 34", result: "≈ 26,5%", at: cue("sobre") },
  ];
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 170 }}>
        <Kicker at={0}>Refaça o preço</Kicker>
      </div>
      <div style={{ position: "absolute", left: 160, top: 260, width: 1000, display: "flex", flexDirection: "column", gap: 30 }}>
        {rows.map((row, i) => (
          <Panel key={row.label} at={row.at} accent={i === 2 ? C.gold : C.mint}>
            <Label>{row.label}</Label>
            <div style={{ display: "flex", alignItems: "baseline", gap: 26, marginTop: 4 }}>
              <span style={{ fontFamily: DISPLAY, fontWeight: 600, fontSize: 76, color: C.ivory }}>{row.expr}</span>
              <span style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 76, color: i === 2 ? C.gold : C.goldSoft }}>{row.result}</span>
            </div>
          </Panel>
        ))}
      </div>
      <Abs x={1540} y={520}>
        <Gauge value={26.47} at={cue("vinte")} />
      </Abs>
      <Abs x={1540} y={840}>
        <Pill tone="muted" at={cue("taxas")} size={22}>
          sem taxas · supondo que as apostas terminem aqui
        </Pill>
      </Abs>
      {rows.map((row) => (
        <Sfx key={row.label} at={row.at} name="pop" volume={0.25} />
      ))}
    </AbsoluteFill>
  );
};

/* 6. O erro comum: usar 12 como custo. */
const Trap: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const doze = cue("doze");
  const tres = cue("três");
  const pote = cue("pote");
  const strike = glide(frame, doze + 14, fps, 16);
  const chip = flight(frame, pote, fps, { x: 1380, y: 600 }, { x: 960, y: 880 }, 0);
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 170 }}>
        <Kicker at={0} color={C.danger}>
          Erro comum
        </Kicker>
      </div>
      <Abs x={560} y={520} style={rise(frame, doze - 4, 40)}>
        <div style={{ width: 640, padding: 44, borderRadius: 28, background: "rgba(239,106,95,0.12)", border: `2px solid ${C.danger}` }}>
          <Label color={C.danger}>✗ ERRADO</Label>
          <div style={{ position: "relative", display: "inline-block", fontFamily: DISPLAY, fontWeight: 700, fontSize: 100, color: C.ivory }}>
            Custo = 12
            <div style={{ position: "absolute", left: 0, top: "52%", height: 10, width: `${strike * 100}%`, background: C.danger, borderRadius: 6 }} />
          </div>
        </div>
      </Abs>
      <Abs x={1360} y={520} style={rise(frame, tres - 4, 40)}>
        <div style={{ width: 640, padding: 44, borderRadius: 28, background: "rgba(95,211,148,0.12)", border: `2px solid ${C.ok}` }}>
          <Label color={C.ok}>✓ CERTO</Label>
          <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 100, color: C.ivory }}>Custo = 9</div>
          <Label>os 3 que você já pôs fazem parte do pote</Label>
        </div>
      </Abs>
      <Abs x={chip.x} y={chip.y} style={{ opacity: chip.opacity }}>
        <ChipStack count={2} color="red" size={50} />
      </Abs>
      <Abs x={960} y={930}>
        <Pot value="34 BB" label="POTE APÓS O CALL" opacity={ramp(frame, pote + 10, 12)} />
      </Abs>
      <Sfx at={doze + 14} name="stamp" volume={0.25} />
      <Sfx at={pote + 18} name="chips" volume={0.35} />
    </AbsoluteFill>
  );
};

/** Faixa de range: segmentos do mais fraco ao mais forte, com marcador do herói. */
const RangeRow: React.FC<{
  title: string;
  segments: { label: string; weight: number; strong?: boolean }[];
  heroAt: number;
  at: number;
  verdict: string;
  verdictTone: "ok" | "danger";
  verdictAt: number;
  y: number;
}> = ({ title, segments, heroAt, at, verdict, verdictTone, verdictAt, y }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const total = segments.reduce((sum, s) => sum + s.weight, 0);
  const width = 1180;
  let x = 0;
  return (
    <div style={{ position: "absolute", left: 160, top: y, width: 1600 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 20, ...rise(frame, at) }}>
        <Label color={C.goldSoft} size={30}>
          {title}
        </Label>
        <Pill tone={verdictTone} at={verdictAt} size={24}>
          {verdict}
        </Pill>
      </div>
      <div style={{ position: "relative", height: 86, marginTop: 20, width }}>
        {segments.map((segment, i) => {
          const w = (segment.weight / total) * width;
          const left = x;
          x += w;
          const t = glide(frame, at + i * 4, fps, 18);
          return (
            <div
              key={segment.label}
              style={{
                position: "absolute",
                left: left + 3,
                width: w - 6,
                height: 86,
                borderRadius: 12,
                background: segment.strong ? "rgba(239,106,95,0.75)" : "rgba(111,208,174,0.35)",
                opacity: t,
                transform: `scaleY(${0.4 + 0.6 * t})`,
                display: "grid",
                placeItems: "center",
                fontFamily: UI,
                fontWeight: 700,
                fontSize: 21,
                color: C.ivory,
                textAlign: "center",
                padding: "0 8px",
                boxSizing: "border-box",
              }}
            >
              {segment.label}
            </div>
          );
        })}
        {[-24, 90].map((top) => (
          <div
            key={top}
            style={{
              position: "absolute",
              left: heroAt * width - 3,
              top,
              width: 6,
              height: 20,
              borderRadius: 3,
              background: C.gold,
              opacity: ramp(frame, at + 16, 10),
              boxShadow: "0 0 18px rgba(212,168,67,0.8)",
            }}
          />
        ))}
        <div style={{ position: "absolute", left: heroAt * width, top: 112, transform: "translateX(-50%)", whiteSpace: "nowrap", opacity: ramp(frame, at + 18, 10) }}>
          <Pill tone="gold" at={at + 18} size={22}>
            Top pair · você
          </Pill>
        </div>
      </div>
    </div>
  );
};

/* 7. Força absoluta x relativa. */
const Relative: React.FC<SceneProps> = ({ cue }) => (
  <AbsoluteFill>
    <div style={{ position: "absolute", left: 160, top: 160 }}>
      <Kicker at={0}>Força absoluta e relativa</Kicker>
      <Headline text="Forte contra quem?" at={4} size={76} style={{ marginTop: 12 }} />
    </div>
    <RangeRow
      title="Range amplo"
      segments={[
        { label: "ar", weight: 3 },
        { label: "projetos", weight: 3 },
        { label: "pares médios", weight: 3 },
        { label: "top pair", weight: 2 },
        { label: "dois pares +", weight: 1, strong: true },
      ]}
      heroAt={0.83}
      at={cue("amplo")}
      verdict="FORTE"
      verdictTone="ok"
      verdictAt={cue("piores")}
      y={400}
    />
    <RangeRow
      title="Só dois pares ou melhor"
      segments={[
        { label: "dois pares", weight: 3, strong: true },
        { label: "trincas", weight: 2, strong: true },
        { label: "melhores", weight: 1, strong: true },
      ]}
      heroAt={0.02}
      at={cue("dois")}
      verdict="FRACA"
      verdictTone="danger"
      verdictAt={cue("fraca")}
      y={700}
    />
    <div style={{ position: "absolute", left: 1390, top: 380, width: 400, display: "flex", flexDirection: "column", gap: 18 }}>
      <Panel at={cue("treino")} accent={C.mint} style={{ padding: "22px 24px" }}>
        <Label color={C.mint} size={20}>
          NO TREINO
        </Label>
        <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 26, color: C.ivory, marginTop: 6 }}>
          O bot de valor só aumenta quando a própria mão cumpre a política declarada.
        </div>
      </Panel>
      <Panel at={cue("blefa")} accent={C.gold} style={{ padding: "22px 24px" }}>
        <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 26, color: C.ivory }}>
          Contra quem também blefa e joga projetos: reconstrua o range.
        </div>
      </Panel>
    </div>
  </AbsoluteFill>
);

/* 8. Modelo explícito do exercício. */
const Model: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const rei = cue("rei");
  const flop = cue("flop");
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 170, width: 780 }}>
        <Kicker at={0}>Exercício · modelo explícito</Kicker>
        <div style={{ marginTop: 26 }}>
          <Panel at={cue("política")}>
            <Label color={C.mint} size={22}>
              POLÍTICA DECLARADA DO OPONENTE
            </Label>
            <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 36, color: C.ivory, marginTop: 18, ...rise(frame, cue("dois")) }}>
              • Check-raise grande só com <span style={{ color: C.gold }}>dois pares ou melhor</span>
            </div>
            <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 36, color: C.ivory, marginTop: 14, ...rise(frame, cue("blefa")) }}>
              • Quase nunca blefa
            </div>
          </Panel>
        </div>
      </div>
      <div style={{ position: "absolute", left: 1080, top: 230 }}>
        <Label color={C.faint}>VOCÊ</Label>
        <div style={{ display: "flex", gap: 20, marginTop: 12 }}>
          {HERO.map((code, i) => (
            <div key={code} style={rise(frame, rei + i * 5, 40)}>
              <Card code={code} width={150} face={ramp(frame, rei + 8 + i * 5, 14)} />
            </div>
          ))}
        </div>
        <div style={{ marginTop: 40 }}>
          <Label color={C.faint}>FLOP</Label>
        </div>
        <div style={{ display: "flex", gap: 20, marginTop: 12 }}>
          {FLOP.map((code, i) => (
            <div key={code} style={rise(frame, flop + i * 5, 40)}>
              <Card code={code} width={130} face={ramp(frame, flop + 8 + i * 5, 14)} />
            </div>
          ))}
        </div>
      </div>
      <Sfx at={rei + 8} name="flip" volume={0.35} />
      <Sfx at={flop + 8} name="flip" volume={0.35} />
    </AbsoluteFill>
  );
};

/* 9. Contra esse range, fold — sem virar regra geral. */
const Fold: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const vencem = cue("vencem");
  const hands = [
    { cards: ["K", "8"], what: "dois pares" },
    { cards: ["K", "3"], what: "dois pares" },
    { cards: ["8", "3"], what: "dois pares" },
    { cards: ["K", "K"], what: "trinca" },
    { cards: ["8", "8"], what: "trinca" },
    { cards: ["3", "3"], what: "trinca" },
  ];
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 170 }}>
        <Kicker at={0}>Contra esse modelo</Kicker>
        <Body at={4} size={30} style={{ marginTop: 8 }}>
          Mãos do oponente que dão check-raise grande em K♣ 8♠ 3♥
        </Body>
      </div>
      <div style={{ position: "absolute", left: 160, top: 320, display: "grid", gridTemplateColumns: "repeat(3, 280px)", gap: 24 }}>
        {hands.map((hand, i) => {
          const beat = ramp(frame, vencem + i * 5, 10);
          return (
            <div
              key={hand.cards.join("") + i}
              style={{
                padding: "22px 24px",
                borderRadius: 20,
                background: C.panel,
                border: `2px solid ${beat > 0.5 ? C.danger : C.line}`,
                ...rise(frame, 8 + i * 4, 30),
              }}
            >
              <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 64, color: C.ivory }}>{hand.cards.join(" ")}</div>
              <Label>{hand.what}</Label>
              <div style={{ marginTop: 10, fontFamily: UI, fontWeight: 800, fontSize: 22, color: C.danger, opacity: beat }}>VENCE K♦ 9♦</div>
            </div>
          );
        })}
      </div>
      <Abs x={1460} y={420}>
        <Stamp text="FOLD" at={cue("fold")} size={150} color={C.gold} rotate={-7} />
      </Abs>
      <div style={{ position: "absolute", left: 1150, top: 620, width: 640, display: "flex", flexDirection: "column", gap: 22 }}>
        <Panel at={cue("atenção")} accent={C.danger}>
          <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 32, color: C.ivory }}>Não é regra contra todo check-raise.</div>
        </Panel>
        <Panel at={cue("refaça")} accent={C.mint}>
          <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 32, color: C.ivory }}>Mudou a frequência do oponente? Refaça a análise.</div>
        </Panel>
      </div>
      <Sfx at={cue("fold")} name="stamp" volume={0.4} />
    </AbsoluteFill>
  );
};

/* 10. Float: pagar com um plano. */
const Float: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const title = cue("float");
  const seguinte = cue("seguinte");
  const arrow = glide(frame, seguinte, fps, 30);
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 210 }}>
        <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontStyle: "italic", fontSize: 220, color: C.gold, lineHeight: 1, ...rise(frame, title, 60, 26) }}>
          Float
        </div>
        <Headline text="é pagar com um plano." at={cue("plano") - 6} size={72} accent={["plano."]} style={{ marginTop: 10 }} />
      </div>
      <div style={{ position: "absolute", left: 160, top: 700, display: "flex", alignItems: "center", gap: 30 }}>
        <Pill tone="muted" at={seguinte - 8} size={32}>
          Flop: você paga
        </Pill>
        <div style={{ width: 260, height: 6, borderRadius: 3, background: "rgba(255,255,255,0.1)", position: "relative" }}>
          <div style={{ width: `${arrow * 100}%`, height: "100%", borderRadius: 3, background: C.gold }} />
        </div>
        <Pill tone="gold" at={seguinte + 10} size={32}>
          Street seguinte: disputa o pote
        </Pill>
      </div>
      <div style={{ position: "absolute", left: 160, top: 830 }}>
        <Pill tone="mint" at={cue("posição")} size={28}>
          Em posição: você age por último
        </Pill>
      </div>
      <div style={{ position: "absolute", left: 1180, top: 300, width: 600 }}>
        <Panel at={cue("sabe")} accent={C.danger}>
          <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 30, color: C.ivory }}>
            Preserva opções quando sua mão tem equity, mas o oponente pode não dar check no turn.
          </div>
        </Panel>
      </div>
      <Sfx at={title} name="whoosh" volume={0.3} />
    </AbsoluteFill>
  );
};

/* 11. O plano do float: o que continua e o que encerra. */
const FloatPlan: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const check = cue("check");
  const acabou = cue("acabou");
  const items = [
    { text: "Quais cartas do turn permitem continuar", at: cue("cartas") },
    { text: "Que aposta do oponente encerra o plano", at: cue("encerra") },
  ];
  const branch = (at: number) => glide(frame, at, fps, 26);
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 140, top: 180, width: 700 }}>
        <Kicker at={0}>Antes de pagar</Kicker>
        <div style={{ display: "flex", flexDirection: "column", gap: 22, marginTop: 26 }}>
          {items.map((item) => (
            <Panel key={item.text} at={item.at} accent={C.mint}>
              <div style={{ display: "flex", alignItems: "center", gap: 18 }}>
                <div
                  style={{
                    width: 42,
                    height: 42,
                    borderRadius: 10,
                    border: `3px solid ${C.mint}`,
                    display: "grid",
                    placeItems: "center",
                    color: C.mint,
                    fontFamily: UI,
                    fontWeight: 800,
                    fontSize: 28,
                    flexShrink: 0,
                  }}
                >
                  <span style={{ opacity: ramp(frame, item.at + 14, 8) }}>✓</span>
                </div>
                <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 32, color: C.ivory }}>{item.text}</div>
              </div>
            </Panel>
          ))}
        </div>
      </div>
      <svg width={1920} height={1080} style={{ position: "absolute", inset: 0 }}>
        <path d={`M 1110 560 C 1220 560, 1220 360, 1320 360`} fill="none" stroke={C.ok} strokeWidth={6} strokeDasharray={420} strokeDashoffset={420 * (1 - branch(check))} />
        <path d={`M 1110 560 C 1220 560, 1220 760, 1320 760`} fill="none" stroke={C.danger} strokeWidth={6} strokeDasharray={420} strokeDashoffset={420 * (1 - branch(acabou - 30))} />
      </svg>
      <Abs x={975} y={560}>
        <Pill tone="muted" at={items[1].at + 20} size={26}>
          Flop: você paga
        </Pill>
      </Abs>
      <div style={{ position: "absolute", left: 1330, top: 300 }}>
        <Panel at={check} accent={C.ok} style={{ width: 460 }}>
          <Label color={C.ok}>TURN · OPONENTE DÁ CHECK</Label>
          <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 32, color: C.ivory, marginTop: 6 }}>Aposte e dispute o pote</div>
        </Panel>
      </div>
      <div style={{ position: "absolute", left: 1330, top: 700 }}>
        <Panel at={acabou - 30} accent={C.danger} style={{ width: 460 }}>
          <Label color={C.danger}>TURN · OPONENTE APOSTA FORTE</Label>
          <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 32, color: C.ivory, marginTop: 6 }}>Plano encerrado</div>
        </Panel>
      </div>
      <Abs x={490} y={880}>
        <Pill tone="danger" at={cue("curiosidade")} size={30}>
          Call por curiosidade não é float
        </Pill>
      </Abs>
      <Sfx at={check} name="pop" volume={0.3} />
      <Sfx at={acabou - 30} name="pop" volume={0.3} />
    </AbsoluteFill>
  );
};

type Option = { key: string; text: string };

const FLOAT_OPTIONS: Option[] = [
  { key: "A", text: "Dar fold no pré-flop" },
  { key: "B", text: "Pagar uma aposta no flop, em posição, para tomar o pote no turn quando o oponente checar" },
  { key: "C", text: "Pedir tempo extra" },
];

const EX2_OPTIONS: Option[] = [
  { key: "A", text: "Fold" },
  { key: "B", text: "Call" },
  { key: "C", text: "Raise" },
];

const Options: React.FC<{ options: Option[]; correctKey: string; at: (key: string) => number; correct?: number; dim?: number; top?: number }> = ({
  options,
  correctKey,
  at,
  correct = 0,
  dim = 0,
  top = 330,
}) => {
  const frame = useCurrentFrame();
  return (
    <div style={{ position: "absolute", left: 160, top, width: 1300, display: "flex", flexDirection: "column", gap: 22 }}>
      {options.map((option) => {
        const right = option.key === correctKey;
        const enter = rise(frame, at(option.key), 30);
        return (
          <div
            key={option.key}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 26,
              padding: "24px 30px",
              borderRadius: 22,
              background: right && correct > 0 ? `rgba(95,211,148,${0.16 * correct})` : C.panel,
              border: `2px solid ${right && correct > 0 ? C.ok : C.line}`,
              transform: enter.transform,
              opacity: enter.opacity * (right ? 1 : 1 - 0.6 * dim),
            }}
          >
            <div
              style={{
                width: 70,
                height: 70,
                borderRadius: 999,
                flexShrink: 0,
                display: "grid",
                placeItems: "center",
                background: right && correct > 0 ? C.ok : "rgba(236,217,154,0.12)",
                color: right && correct > 0 ? C.ink : C.goldSoft,
                fontFamily: UI,
                fontWeight: 800,
                fontSize: 34,
              }}
            >
              {right && correct > 0.5 ? "✓" : option.key}
            </div>
            <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 34, color: C.ivory }}>{option.text}</div>
          </div>
        );
      })}
    </div>
  );
};

/** Relógio de pausa: a narração para e o anel corre até o fim da cena. */
const PauseClock: React.FC<{ from: number; to: number }> = ({ from, to }) => {
  const frame = useCurrentFrame();
  const countdown = interpolate(frame, [from, to], [0, 1], CLAMP);
  return (
    <Abs x={1650} y={560} style={{ opacity: ramp(frame, from, 12) }}>
      <div style={{ position: "relative", width: 220, height: 220 }}>
        <svg width={220} height={220} viewBox="0 0 220 220">
          <circle cx="110" cy="110" r="94" fill="none" stroke="rgba(255,255,255,0.1)" strokeWidth="12" />
          <circle
            cx="110"
            cy="110"
            r="94"
            fill="none"
            stroke={C.gold}
            strokeWidth="12"
            strokeLinecap="round"
            strokeDasharray={2 * Math.PI * 94}
            strokeDashoffset={2 * Math.PI * 94 * countdown}
            transform="rotate(-90 110 110)"
          />
        </svg>
        <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", gap: 18 }}>
          <div style={{ width: 22, height: 80, borderRadius: 6, background: C.ivory }} />
          <div style={{ width: 22, height: 80, borderRadius: 6, background: C.ivory }} />
        </div>
      </div>
      <div style={{ textAlign: "center", marginTop: 16 }}>
        <Label color={C.goldSoft}>Pause e responda</Label>
      </div>
    </Abs>
  );
};

/* 12. Pergunta do float, com pausa. */
const Quiz: React.FC<SceneProps> = ({ cue, duration }) => (
  <AbsoluteFill>
    <div style={{ position: "absolute", left: 160, top: 170 }}>
      <Kicker at={0}>Pergunta</Kicker>
      <Headline text="O que é float?" at={cue("float") - 4} size={84} accent={["float?"]} style={{ marginTop: 10 }} />
    </div>
    <Options options={FLOAT_OPTIONS} correctKey="B" at={(key) => cue(key)} />
    <PauseClock from={cue("Pause")} to={duration} />
    {FLOAT_OPTIONS.map((option) => (
      <Sfx key={option.key} at={cue(option.key)} name="pop" volume={0.22} />
    ))}
  </AbsoluteFill>
);

/* 13. Resposta do float. */
const Answer: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const b = cue("B");
  const reveal = glide(frame, b, fps, 18);
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 170 }}>
        <Kicker at={0} color={C.ok}>
          Resposta
        </Kicker>
        <Headline text="Letra B" at={b - 6} size={84} color={C.ok} style={{ marginTop: 10 }} />
      </div>
      <Options options={FLOAT_OPTIONS} correctKey="B" at={() => -30} correct={reveal} dim={reveal} />
      <div style={{ position: "absolute", left: 160, top: 860, width: 1300 }}>
        <Panel at={cue("planejado")} accent={C.ok}>
          <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 32, color: C.ivory }}>
            Float é um call planejado em posição, para explorar o check do agressor no turn.
          </div>
        </Panel>
      </div>
      <Sfx at={b} name="pop" volume={0.35} />
    </AbsoluteFill>
  );
};

/* Exercício do K♦9♦: pergunta com pausa, antes da resposta (cena Fold). */
const Exercise: React.FC<SceneProps> = ({ cue, duration }) => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 150 }}>
        <Kicker at={0}>Exercício</Kicker>
        <Headline text="Qual decisão conservadora treinamos?" at={4} size={68} style={{ marginTop: 10 }} />
      </div>
      <div style={{ position: "absolute", left: 160, top: 290, display: "flex", alignItems: "center", gap: 12, ...rise(frame, 8, 30) }}>
        <Label color={C.faint} size={22}>
          VOCÊ
        </Label>
        {HERO.map((code) => (
          <Card key={code} code={code} width={86} />
        ))}
        <div style={{ width: 36 }} />
        <Label color={C.faint} size={22}>
          FLOP
        </Label>
        {FLOP.map((code) => (
          <Card key={code} code={code} width={86} />
        ))}
        <div style={{ width: 36 }} />
        <Pill tone="danger" at={8} size={24}>
          Check-raise grande
        </Pill>
      </div>
      <Options options={EX2_OPTIONS} correctKey="A" at={(key) => cue(key)} top={470} />
      <PauseClock from={cue("Pause")} to={duration} />
      {EX2_OPTIONS.map((option) => (
        <Sfx key={option.key} at={cue(option.key)} name="pop" volume={0.22} />
      ))}
    </AbsoluteFill>
  );
};

/* 14. Laboratório na mesa do módulo. */
const Lab: React.FC<SceneProps> = ({ cue }) => {
  const steps = [
    { title: "Abra a mesa do módulo 2", at: cue("mesa") },
    { title: "Cenário: Responder ao check-raise", at: cue("cenário") },
    { title: "Treino guiado: o bot de valor dá check-raise com dois pares ou melhor", at: cue("guiado") },
    { title: "Modo desafio: responda à pergunta antes de cada ação", at: cue("desafio") },
    { title: "Replay: compare uma alternativa na mesma distribuição", at: cue("replay") },
    { title: "Nova distribuição: mantém as cartas do cenário, varia as demais", at: cue("Uma nova") },
  ];
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 160 }}>
        <Kicker at={0}>Laboratório</Kicker>
        <Headline text="Leve a aula para a mesa" at={4} size={76} accent={["mesa"]} style={{ marginTop: 10 }} />
      </div>
      <div style={{ position: "absolute", left: 160, top: 390, width: 1600, display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 24 }}>
        {steps.map((step, i) => (
          <Panel key={step.title} at={step.at} accent={i < 2 ? C.gold : C.mint} style={{ padding: "24px 28px", minHeight: 170 }}>
            <Label color={C.faint} size={20}>
              PASSO {i + 1}
            </Label>
            <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 29, color: C.ivory, marginTop: 6 }}>{step.title}</div>
          </Panel>
        ))}
      </div>
      {steps.map((step) => (
        <Sfx key={step.title} at={step.at} name="pop" volume={0.22} />
      ))}
    </AbsoluteFill>
  );
};

/* 15. Fechamento: caderno, avaliação honesta e limites. */
const Practice: React.FC<SceneProps> = ({ cue }) => {
  const frame = useCurrentFrame();
  const fichas = cue("fichas");
  const items = [
    { title: "Registre a justificativa no caderno", note: "Ler bem e respeitar as hipóteses vale mais que ganhar a mão.", at: cue("caderno") },
    { title: "Avalie o raciocínio", note: "Uma vitória isolada não prova uma boa estratégia.", at: cue("vitória") },
    { title: "Fontes no material de apoio", note: "Confira as referências da aula.", at: cue("fontes") },
  ];
  return (
    <AbsoluteFill>
      <div style={{ position: "absolute", left: 160, top: 160 }}>
        <Kicker at={0}>Antes de sair</Kicker>
        <Headline text="Estude o raciocínio, não o resultado" at={4} size={70} accent={["raciocínio,"]} style={{ marginTop: 10 }} />
      </div>
      <div style={{ position: "absolute", left: 160, top: 400, width: 1600, display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 24 }}>
        {items.map((item) => (
          <Panel key={item.title} at={item.at} accent={C.mint} style={{ padding: "26px 28px", minHeight: 200 }}>
            <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 32, color: C.ivory }}>{item.title}</div>
            <div style={{ fontFamily: UI, fontWeight: 500, fontSize: 26, color: C.muted, marginTop: 10 }}>{item.note}</div>
          </Panel>
        ))}
      </div>
      <Abs x={960} y={800}>
        <Pill tone="muted" at={fichas} size={28}>
          Treino com fichas sem valor · este curso não promete lucro
        </Pill>
      </Abs>
      <Abs x={1760} y={190} style={{ opacity: ramp(frame, fichas + 20, 30) }}>
        <SpadeMark size={110} draw={ramp(frame, fichas + 20, 50)} />
      </Abs>
      {items.map((item) => (
        <Sfx key={item.title} at={item.at} name="pop" volume={0.22} />
      ))}
    </AbsoluteFill>
  );
};

export const M2L2_SCENES: SceneMap = {
  hook: Hook,
  title: Title,
  what: What,
  price: Price,
  price2: Price2,
  trap: Trap,
  relative: Relative,
  model: Model,
  ex2: Exercise,
  fold: Fold,
  float: Float,
  floatplan: FloatPlan,
  quiz: Quiz,
  answer: Answer,
  lab: Lab,
  practice: Practice,
};
