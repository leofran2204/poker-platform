#!/usr/bin/env node
// VPS version - 100 contas play money: 72 cash (4 mesas 8/5/6/9) 2h + 28 MTT até campeão
import { randomBytes } from "node:crypto";
import { setTimeout as sleep } from "node:timers/promises";
process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
const BASE_URL = process.env.BASE_URL ?? "https://zerotiltpoker.net";
const CASH_DURATION_MS = Number(process.env.CASH_DURATION_MS ?? 2*60*60*1000);
const TOTAL=100, CASH_TOTAL=72, TOURNEY_TOTAL=28;
const USER_PREFIX="loadtest";
const DOMAIN="load.test";
const PASSWORD="Test1234!A1";
const JWT_SECRET="bKCpK84luXuuIjwm+mTAM5FehOnywPAmTeaPGUom9tQsdKdB5jLADOPbnDyt1yj5";
const BCRYPT_HASH="$2b$10$ApzxxrdmmEsHzMq0jpyJou/CBS2vNqSL66oix1Gpk.o9fiAY3YoB6";
async function api(path, {method="GET", body, token}={}) {
  const headers={"Content-Type":"application/json"};
  if(token) headers.Authorization=`Bearer ${token}`;
  const res=await fetch(`${BASE_URL}${path}`,{method, headers, body:body?JSON.stringify(body):undefined});
  const text=await res.text(); let data=null; try{data=text?JSON.parse(text):null;}catch{data=text;}
  if(!res.ok) throw new Error(`${method} ${path} -> ${res.status} ${JSON.stringify(data).slice(0,400)}`);
  return data;
}
function wsUrl(tableId, ticket){ const u=new URL(BASE_URL); u.protocol=u.protocol==="https:"?"wss:":"ws:"; u.pathname=`/ws/game/${encodeURIComponent(tableId)}`; u.search=new URLSearchParams({ticket}).toString(); return u.toString(); }
import { execSync } from "node:child_process";
import fs from "node:fs";
import crypto from "node:crypto";
function b64url(s){return Buffer.from(s).toString("base64url");}
function signJwt(p,s){const h=b64url(JSON.stringify({alg:"HS256",typ:"JWT"})); const b=b64url(JSON.stringify(p)); const sig=crypto.createHmac("sha256",s).update(`${h}.${b}`).digest("base64url"); return `${h}.${b}.${sig}`;}
async function ensureUsers(n){
  console.log(`[1/4] Criando ${n} contas ${USER_PREFIX}@${DOMAIN} via DB...`);
  const users=[];
  for(let i=0;i<n;i++){ const username=`${USER_PREFIX}_${String(i).padStart(3,"0")}_${randomBytes(2).toString("hex")}`; users.push({username,email:`${username}@${DOMAIN}`,password:PASSWORD});}
  const batch=users.map(u=>`INSERT INTO users (id, username, email, password_hash, role, status, balance, mfa_enabled, created_at, email_verified_at, balance_pm_cash, balance_pm_mtt, balance_real, last_pm_reset_date, preferred_wallet_mode) VALUES (gen_random_uuid(), '${u.username}', '${u.email}', '${BCRYPT_HASH}', 'player', 'active', 0, false, EXTRACT(EPOCH FROM NOW())::BIGINT, EXTRACT(EPOCH FROM NOW())::BIGINT, 15000, 15000, 0, (timezone('America/Sao_Paulo', now()))::date, 'play') ON CONFLICT (username) DO NOTHING;`).join("\n");
  fs.writeFileSync("/tmp/batch_users.sql", batch);
  execSync(`docker exec -i poker_postgres psql -U poker_user -d poker_db < /tmp/batch_users.sql`,{stdio:"ignore"});
  execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -c "UPDATE users SET status='active', email_verified_at=EXTRACT(EPOCH FROM NOW())::BIGINT WHERE email LIKE '%@${DOMAIN}';"`,{stdio:"ignore"});
  console.log(`  Inseridos ${n}`);
  const raw=execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -t -A -F"," -c "SELECT email, id, username, role, token_version FROM users WHERE email LIKE '%@${DOMAIN}'"`,{encoding:"utf8"}).trim();
  const map=new Map();
  for(const line of raw.split("\n")){ if(!line.trim()) continue; const [email,id,username,role,tv]=line.split(",").map(s=>s.trim()); map.set(email,{id,username,role,tv});}
  for(const u of users){ const info=map.get(u.email); if(!info) continue; u.user_id=info.id; u.username=info.username||u.username; const now=Math.floor(Date.now()/1000); const payload={sub:info.id, username:u.username, role:info.role||"player", token_version:parseInt(info.tv||"0",10), iat:now, exp:now+900, type:"access"}; u.token=signJwt(payload,JWT_SECRET); }
  console.log(`  Tokens ${users.filter(u=>u.token).length}/${n}`);
  return users;
}
async function prepareTables(users){
  console.log(`[2/4] Mesas cash 4 + torneios...`);
  const tables=await api("/api/lobby/tables?mode=play",{token:users[0].token});
  const want=["PM · NL 0,25","PM · SD 0,25/0,50","PM · SD Omaha 0,50/0,50","PM · Pineapple 0,50"];
  let cashTables=tables.filter(t=>want.includes(t.name) && t.money_mode==="play");
  if(cashTables.length!==4) cashTables=tables.filter(t=>t.money_mode==="play").slice(0,4);
  console.log(`  Cash: ${cashTables.map(t=>`${t.name} ${t.max_players}-max`).join(" | ")}`);
  const tourneys=await api("/api/lobby/tournaments?mode=play",{token:users[0].token});
  const playTourneys=tourneys.filter(t=>t.money_mode==="play" && t.status==="registering").slice(0,4);
  console.log(`  Torneios: ${playTourneys.map(t=>t.name).join(" | ")}`);
  return {cashTables, playTourneys};
}
async function runCash(cashTables, cashUsers){
  console.log(`[3/4] Cash 2h: ${cashTables.length} mesas, ${cashUsers.length} contas, 2 espera/mesa`);
  const tablesState=cashTables.map(t=>({table:t, seated:[], waiting:[]}));
  let cur=0; for(const ts of tablesState){ const need=ts.table.max_players; ts.seated=cashUsers.slice(cur,cur+need); cur+=need; ts.waiting=cashUsers.slice(cur,cur+2); cur+=2; }
  const extraPool=cashUsers.slice(cur);
  console.log(`  Seated ${tablesState.reduce((s,ts)=>s+ts.seated.length,0)} + waiting ${tablesState.reduce((s,ts)=>s+ts.waiting.length,0)} + extra ${extraPool.length}`);
  for(const ts of tablesState){ for(const u of ts.seated){ try{ const r=await api("/api/lobby/join",{method:"POST", body:{table_id:ts.table.id, buy_in:ts.table.min_buy_in, wallet_mode:"play"}, token:u.token}); u.seat=r.seat; }catch(e){ console.warn(`  join ${u.username} ${e.message.slice(0,80)}`);} } }
  const start=Date.now(), end=start+CASH_DURATION_MS;
  let hands=0, rots=0;
  const sockets=[];
  async function open(u, ts){
    try{
      const ticket=(await api(`/api/lobby/tables/${ts.table.id}/ws-ticket`,{method:"POST", token:u.token})).ticket;
      const ws=new WebSocket(wsUrl(ts.table.id, ticket));
      sockets.push(ws); u.ws=ws;
      ws.addEventListener("open",()=>ws.send(JSON.stringify({type:"get_table_info"})));
      ws.addEventListener("message",ev=>{
        try{
          const msg=JSON.parse(String(ev.data));
          if(msg.type==="welcome") u.playerId=msg.player_id;
          if(msg.type!=="table_state") return;
          if(msg.is_finished) hands++;
          if(!u.playerId) return;
          const me=(msg.players||[]).find(p=>p.id===u.playerId);
          if(!me?.is_active) return;
          const acts=(msg.available_actions||[]).map(a=>a.toLowerCase());
          if(!acts.length) return;
          let act="fold"; const r=Math.random();
          if(r<0.7) act=acts.includes("check")?"check":"call";
          else if(r<0.9) act="fold";
          else act=acts.includes("raise")?"raise":(acts.includes("allin")?"allin":"call");
          const amt=act==="raise"?(msg.minimum_wager||0):0;
          if(ws.readyState===1) ws.send(JSON.stringify({type:"action", action:act, amount:amt}));
        }catch{}
      });
      await new Promise(r=>setTimeout(r,600));
    }catch{}
  }
  for(const ts of tablesState) for(const u of ts.seated) await open(u, ts);
  const rot=setInterval(async()=>{
    if(Date.now()>end) return;
    for(const ts of tablesState){
      if(Math.random()<0.08 && ts.waiting.length>0 && extraPool.length>0){
        const out=ts.seated.shift(); if(!out) continue;
        try{ await api("/api/lobby/leave",{method:"POST", body:{table_id:ts.table.id}, token:out.token}); }catch{}
        try{ out.ws?.close(); }catch{}
        const inc=ts.waiting.shift(); ts.waiting.push(extraPool.shift()??out); ts.seated.push(inc);
        try{ const r=await api("/api/lobby/join",{method:"POST", body:{table_id:ts.table.id, buy_in:ts.table.min_buy_in, wallet_mode:"play"}, token:inc.token}); inc.seat=r.seat; await open(inc, ts); rots++; }catch{}
      }
    }
  },30000);
  while(Date.now()<end){
    await sleep(5000);
    const min=Math.round((Date.now()-start)/60000);
    if(min%10===0) console.log(`  cash ${min}min hands~${hands} rot ${rots}`);
  }
  clearInterval(rot);
  for(const ws of sockets) try{ws.close();}catch{}
  for(const ts of tablesState) for(const u of ts.seated) try{await api("/api/lobby/leave",{method:"POST", body:{table_id:ts.table.id}, token:u.token});}catch{}
  console.log(`  Cash fim hands~${hands} rots ${rots}`);
  return {hands, rots};
}
async function runTournaments(playTourneys, tourneyUsers){
  console.log(`[4/4] MTT até campeão ${playTourneys.length} torneios ${tourneyUsers.length} contas`);
  const per=Math.floor(tourneyUsers.length/playTourneys.length);
  let cur=0;
  for(const t of playTourneys){
    const slice=tourneyUsers.slice(cur,cur+per); cur+=per;
    console.log(`  Insc ${slice.length} em ${t.name}`);
    for(const u of slice) try{ await api("/api/tournament/register",{method:"POST", body:{tournament_id:t.id, wallet_mode:"play"}, token:u.token}); }catch(e){ console.warn(e.message.slice(0,80));}
    t._slice=slice;
  }
  for(const t of playTourneys){
    console.log(`  Sim ${t.name} até campeão...`);
    let players=t._slice.slice();
    while(players.length>1){
      const out=players.splice(Math.floor(Math.random()*players.length),1)[0];
      try{ execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -c "UPDATE tournament_players SET stack=0 WHERE tournament_id='${t.id}'::uuid AND player_id='${out.user_id}';"`,{stdio:"ignore"});}catch{}
      try{ execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -c "UPDATE tournaments SET players_remaining=${players.length} WHERE id='${t.id}'::uuid;"`,{stdio:"ignore"});}catch{}
      await sleep(30);
    }
    try{ execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -c "UPDATE tournaments SET status='finished', finished_at=EXTRACT(EPOCH FROM NOW())::BIGINT, players_remaining=1 WHERE id='${t.id}'::uuid;"`,{stdio:"ignore"});}catch{}
    console.log(`  Campeão ${t.name}: ${players[0]?.username}`);
  }
  return {n:playTourneys.length};
}
async function verify(){
  console.log(`[Verificação]`);
  try{ const r=execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -t -c "SELECT COALESCE(SUM(rake_collected),0) FROM hand_history;"`,{encoding:"utf8"}).trim(); console.log(`  Rake total: ${r} cents`);}catch{}
  try{ const r=execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -t -c "SELECT COUNT(*) FROM hand_history WHERE loss_deflators_json IS NOT NULL AND loss_deflators_json != '[]'::jsonb;"`,{encoding:"utf8"}).trim(); console.log(`  Loss Deflator mãos: ${r}`);}catch{}
  try{ const r=execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -t -c "SELECT COUNT(*) FROM hand_history;"`,{encoding:"utf8"}).trim(); console.log(`  Total mãos: ${r}`);}catch{}
}
async function main(){
  console.log(`=== Full Catalog VPS 100 2h ===`);
  const users=await ensureUsers(100);
  const {cashTables, playTourneys}=await prepareTables(users);
  const cashUsers=users.slice(0,72);
  const tourneyUsers=users.slice(72);
  for(const u of tourneyUsers){ if(!u.user_id){ try{ const out=execSync(`docker exec poker_postgres psql -U poker_user -d poker_db -t -c "SELECT id FROM users WHERE email='${u.email}'"`,{encoding:"utf8"}).trim(); u.user_id=out; }catch{}}}
  const cashRes=await runCash(cashTables, cashUsers);
  const tourRes=await runTournaments(playTourneys, tourneyUsers);
  await verify();
  console.log(`=== FIM VPS 100 2h cash ${cashRes.hands} tour ${tourRes.n} ===`);
}
main().catch(e=>{console.error(e); process.exit(1);});
