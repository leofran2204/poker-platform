/**
 * Cria contas de teste play-money para validacoes locais.
 * Uso: node scripts/setup-test-users.mjs   (exige docker local com poker_api/postgres)
 * Saida: linha BOT_USERS="login:senha,..." pronta para strategy-bots.mjs
 * Env: PREFIX (default strat), COUNT (default 3), PASSWORD (default TestBot1!).
 * Local apenas: le o codigo de e-mail via `docker logs poker_api`.
 */
import { execSync } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
const BASE = process.env.BASE_URL ?? 'https://localhost';
const PREFIX = process.env.PREFIX ?? 'strat';
const COUNT = Number(process.env.COUNT ?? 3);
const PASS = process.env.PASSWORD ?? 'TestBot1!';

const sh = (cmd) => execSync(cmd, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
const stripAnsi = (s) => s.replace(/\[[0-9;]*m/g, '');

async function api(path, { method = 'GET', body } = {}) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: { Accept: 'application/json', ...(body ? { 'Content-Type': 'application/json' } : {}) },
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let json = null;
  try { json = text ? JSON.parse(text) : null; } catch { json = text; }
  if (!res.ok) throw new Error(`${method} ${path} -> ${res.status} ${JSON.stringify(json).slice(0, 200)}`);
  return json;
}

function lastCode(email) {
  const logs = stripAnsi(sh('docker logs poker_api 2>&1 | tail -5000'));
  const esc = email.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  let last = null;
  for (const re of [new RegExp(`to="${esc}"[^\\n]*verification_code=(\\d{6})`, 'g'), new RegExp(`to=${esc}[^\\n]*code=(\\d{6})`, 'g')]) {
    let m; re.lastIndex = 0;
    while ((m = re.exec(logs)) !== null) last = m[1];
    if (last) return last;
  }
  throw new Error(`sem codigo para ${email}`);
}

const seed = sh(`docker exec poker_postgres psql -U user -d poker_db -tA -c "SELECT referral_code FROM users WHERE referral_code IS NOT NULL LIMIT 1;"`).trim().split('\n')[0].trim() || null;

const stamp = String(Date.now()).slice(-6);
const out = [];
for (let i = 0; i < COUNT; i++) {
  const username = `${PREFIX}${stamp}${i}`;
  const email = `${username}@test.local`;
  const body = { username, email, password: PASS, password_confirm: PASS };
  try {
    await api('/api/auth/register', { method: 'POST', body });
  } catch (e) {
    if (!/convite/i.test(String(e.message))) throw e;
    if (!seed) throw new Error('servidor exige convite e banco sem referral_code');
    await api('/api/auth/register', { method: 'POST', body: { ...body, invite_code: seed } });
  }
  await sleep(800);
  const code = lastCode(email);
  await api('/api/auth/verify-email', { method: 'POST', body: { email, code } });
  out.push(`${username}:${PASS}`);
  console.log(`ok ${username}`);
}
console.log(`\nBOT_USERS="${out.join(',')}"`);
