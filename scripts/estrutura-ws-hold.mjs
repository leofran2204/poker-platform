/**
 * Holder WS da Fase D: mantém N contas com WebSocket aberto jogando
 * check/fold (check grátis, fold no resto), com reconnect automático.
 * Sem WS, a conta tem assento no banco mas nunca entra na mão do ator.
 *
 *   BASE_URL=https://zerotiltpoker.net DURATION_MIN=150 node scripts/estrutura-ws-hold.mjs
 *
 * Roster Fase D (conta@test.local, senha PokerDemo1, mesa cash play):
 *   NL b2000001-...-31: raizA, n1a1, n2a1, lone1
 *   SD b2000001-...-02: n1a2, n2a2, raizB, n1b1
 *   OM b2000001-...-41: n2b1, flip1, ghost1
 *   PIN b2000001-...-50: deep1, n2b2, cyc1, cyc2
 */
import { setTimeout as sleep } from "node:timers/promises";

const BASE = process.env.BASE_URL ?? "https://zerotiltpoker.net";
const PASS = process.env.HOLD_PASS ?? "PokerDemo1";
const DURATION_MS = Number(process.env.DURATION_MIN ?? 150) * 60 * 1000;
const T = {
  NL: "b2000001-0001-4000-8000-000000000031",
  SD: "b2000001-0001-4000-8000-000000000002",
  OM: "b2000001-0001-4000-8000-000000000041",
  PIN: "b2000001-0001-4000-8000-000000000050",
};
const BUYIN = { NL: 2500, SD: 7500, OM: 10000, PIN: 7500 };
// MODE=loose (check/call 35% p/ rake real) | fest (allin sempre p/ potes máximos)
// TABLES=NL,SD filtra mesas (padrão: todas).
const MODE = process.env.MODE ?? "loose";
const ONLY_TABLES = (process.env.TABLES ?? "").split(",").map((s) => s.trim()).filter(Boolean);
const ROSTER_ALL = [
  ["t18_raizA", "NL"], ["t18_n1a1", "NL"], ["t18_n2a1", "NL"], ["t18_lone1", "NL"],
  ["t18_n1a2", "SD"], ["t18_n2a2", "SD"], ["t18_raizB", "SD"], ["t18_n1b1", "SD"],
  ["t18_n2b1", "OM"], ["t18_flip1", "OM"], ["t18_ghost1", "OM"],
  ["t18_deep1", "PIN"], ["t18_n2b2", "PIN"], ["t18_cyc1", "PIN"], ["t18_cyc2", "PIN"],
];
const ROSTER = ONLY_TABLES.length ? ROSTER_ALL.filter(([, t]) => ONLY_TABLES.includes(t)) : ROSTER_ALL;

let WebSocket;
try {
  const { createRequire } = await import("node:module");
  WebSocket = createRequire(import.meta.url)("ws");
} catch {
  console.error("pacote 'ws' ausente: rode a partir da raiz (scripts/node_modules) ou npm i ws em scripts/");
  process.exit(2);
}

async function api(path, { method = "GET", body, token } = {}) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: {
      Accept: "application/json",
      ...(body ? { "Content-Type": "application/json" } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let json = null;
  try { json = text ? JSON.parse(text) : null; } catch { json = text; }
  return { status: res.status, json };
}

function holdOne(name, tableKey, deadline) {
  const tableId = T[tableKey];
  const stats = { name, acted: 0, states: 0, reconnects: 0 };
  let stop = false;
  (async () => {
    let token = null;
    for (let attempt = 0; !token && Date.now() < deadline; attempt++) {
      try {
        const login = await api("/api/auth/login", {
          method: "POST",
          body: { email: `${name}@test.local`, password: PASS },
        });
        if (login.status === 200 && login.json?.token) token = login.json.token;
        else await sleep(15000);
      } catch {
        await sleep(15000);
      }
    }
    if (!token) {
      console.error(`[${name}] login falhou (deadline)`);
      return;
    }
    for (let attempt = 0; attempt < 10; attempt++) {
      try {
        await api("/api/lobby/join", {
          method: "POST",
          token,
          body: { table_id: tableId, buy_in: BUYIN[tableKey], wallet_mode: "play" },
        });
        break;
      } catch {
        await sleep(10000);
      }
    }
    while (!stop && Date.now() < deadline) {
      try {
        await new Promise(async (resolve) => {
          const tk = await api(`/api/lobby/tables/${tableId}/ws-ticket`, { method: "POST", token });
          if (tk.status !== 200 || !tk.json?.ticket) {
            await sleep(5000);
            return resolve();
          }
          const ws = new WebSocket(
            `${BASE.replace(/^http/, "ws")}/ws/game/${tableId}?ticket=${tk.json.ticket}`,
          );
          const idleKiller = setTimeout(() => { try { ws.close(); } catch {} }, 10 * 60 * 1000);
          ws.on("message", (raw) => {
            let msg;
            try { msg = JSON.parse(raw.toString()); } catch { return; }
            if (msg.type !== "table_state") return;
            stats.states++;
            const a = Array.isArray(msg.available_actions) ? msg.available_actions : [];
            if (!a.length) return;
            // Loose-passive p/ gerar potes/rake reais: check grátis; senão
            // paga ~35% das vezes e folda o resto (semeia flops sem blefar).
            // Fest: allin sempre (potes máximos p/ validar comissões reais).
            let pick = "fold";
            if (MODE === "fest" && a.includes("allin")) pick = "allin";
            else if (a.includes("check")) pick = "check";
            else if (a.includes("call") && Math.random() < 0.35) pick = "call";
            try {
              ws.send(JSON.stringify({ type: "action", action: pick, amount: 0 }));
              stats.acted++;
            } catch {}
          });
          ws.on("close", () => { clearTimeout(idleKiller); stats.reconnects++; resolve(); });
          ws.on("error", () => { try { ws.close(); } catch {} });
        });
      } catch {
        await sleep(5000);
      }
      await sleep(2000);
    }
  })();
  return { stats, stop: () => { stop = true; } };
}

async function main() {
  process.on("unhandledRejection", (e) => console.error("unhandledRejection:", e?.message ?? e));
  process.on("uncaughtException", (e) => console.error("uncaughtException:", e?.message ?? e));
  const deadline = Date.now() + DURATION_MS;
  console.log(`hold WS: ${ROSTER.length} contas por ${Math.round(DURATION_MS / 60000)}min em ${BASE}`);
  const holders = ROSTER.map(([n, t]) => holdOne(n, t, deadline));
  const t0 = Date.now();
  while (Date.now() < deadline) {
    await sleep(10 * 60 * 1000);
    const el = Math.round((Date.now() - t0) / 60000);
    const agg = holders.map((h) => h.stats).reduce((a, s) => ({ acted: a.acted + s.acted, states: a.states + s.states }), { acted: 0, states: 0 });
    console.log(`HEARTBEAT ${el}min acted=${agg.acted} states=${agg.states}`);
  }
  holders.forEach((h) => h.stop());
  await sleep(3000);
  for (const h of holders) console.log(JSON.stringify(h.stats));
  console.log("HOLD_DONE");
}

main().catch((e) => { console.error(e); process.exit(1); });
