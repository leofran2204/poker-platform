import { execSync } from "node:child_process";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const WebSocket = require("ws");
process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
const BASE = "https://localhost";
const sh = (c) => execSync(c, { encoding: "utf8" });

const email = process.argv[2];
const pass = "PokerDemo1";
let r = await fetch(`${BASE}/api/auth/login`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ email, password: pass }),
});
let j = await r.json();
const token = j.token;
r = await fetch(`${BASE}/api/lobby/tables?mode=play`);
j = await r.json();
const table = (j.find((x) => x.poker_variant === "holdem") ?? j[0]);
console.log("mesa:", table.name, table.id);
r = await fetch(`${BASE}/api/lobby/tables/${table.id}/ws-ticket`, {
  method: "POST",
  headers: { Authorization: `Bearer ${token}` },
});
j = await r.json();
console.log("ticket:", JSON.stringify(j).slice(0, 80));
const ws = new WebSocket(
  `${BASE.replace(/^http/, "ws")}/ws/game/${table.id}?ticket=${j.ticket}`,
  { rejectUnauthorized: false }
);
ws.on("open", () => {
  console.log("WS OPEN");
  ws.send(JSON.stringify({ type: "get_table_info" }));
});
ws.on("message", (raw) => {
  let m;
  try {
    m = JSON.parse(String(raw));
  } catch {
    console.log("RAW:", String(raw).slice(0, 120));
    return;
  }
  const extra =
    m.type === "your_turn"
      ? ` actions=${JSON.stringify(m.actions)}`
      : m.type === "table_state"
        ? ` stage=${m.stage} avail=${JSON.stringify(m.available_actions)} call=${m.call_amount}`
        : m.type === "error"
          ? ` msg=${m.message}`
          : "";
  console.log(`IN ${m.type}${extra}`);
});
ws.on("error", (e) => console.log("WS ERROR:", e.message));
ws.on("close", (c) => console.log("WS CLOSE:", c));
setTimeout(() => process.exit(0), 90000);
