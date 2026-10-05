// Real local HTTPS/WSS clients. Synthetic fixture is created by full-validation.py.
import https from "node:https";
import fs from "node:fs";
import assert from "node:assert/strict";
import WebSocket from "ws";
import {performance} from "node:perf_hooks";
const fixture=JSON.parse(fs.readFileSync(process.argv[2],"utf8"));
const base=new URL(fixture.origin);
assert.equal(base.hostname,"localhost");
assert.equal(base.port,"3449");
assert.equal(base.protocol,"https:");
const ca=fs.readFileSync(fixture.ca);
const out=fixture.output;
const log=fs.createWriteStream(out+"-events.jsonl",{flags:"w"});
const sockets=[];
const errors=[];
const latencies=[];
const witnesses=[];
const counters={requests:0,expected_rejections:0,unexpected_errors:0,messages:0,actions:0,
  completed_tables:0,reconnections:0,private_states:0};
const agent=new https.Agent({ca,keepAlive:true});
function event(value){log.write(JSON.stringify({at:Date.now(),...value})+"\n");}
function request(path,user,body){
  const start=performance.now();counters.requests++;
  return new Promise((resolve,reject)=>{
    const data=body===undefined?null:JSON.stringify(body);
    const req=https.request(new URL(path,base),{agent,localAddress:user.ip,family:4,method:data?"POST":"GET",
      headers:{Authorization:"Bearer "+user.token,"Content-Type":"application/json",
        ...(data?{"Content-Length":Buffer.byteLength(data)}:{})}},res=>{
      let raw="";res.on("data",chunk=>raw+=chunk);
      res.on("end",()=>{latencies.push(performance.now()-start);
        let value;try{value=JSON.parse(raw);}catch{value=raw;}
        event({type:"http",path,status:res.statusCode,user:user.id});
        resolve({status:res.statusCode,value});});
    });
    req.on("error",reject);req.setTimeout(10000,()=>req.destroy(new Error("HTTPS timeout")));
    req.end(data);
  });
}
async function open(table,user,onState){
  const ticket=await request("/api/lobby/tables/"+table.id+"/ws-ticket",user,{});
  assert.equal(ticket.status,200,JSON.stringify(ticket.value));
  return await new Promise((resolve,reject)=>{
    const ws=new WebSocket("wss://localhost:3449/ws/game/"+table.id+"?ticket="+encodeURIComponent(ticket.value.ticket),{ca,localAddress:user.ip,family:4});
    sockets.push(ws);
    const timer=setTimeout(()=>{ws.terminate();reject(new Error("WSS handshake deadline"));},10000);
    ws.once("open",()=>{clearTimeout(timer);ws.send(JSON.stringify({type:"get_table_info"}));resolve(ws);});
    ws.on("error",error=>{clearTimeout(timer);reject(error);errors.push(error.message);});
    ws.on("message",bytes=>{
      try{
        const state=JSON.parse(bytes.toString());counters.messages++;
        event({type:"wss",recipient:user.id,table:table.id,payload:state});
        if(state.type==="error"){counters.unexpected_errors++;errors.push(state.message);return;}
        if(state.type!=="table_state")return;
        assert.equal(state.table_id,table.id,"cross-table state leak");
        const allowed=new Set(table.users.map(u=>u.id));
        const visible=[...(state.community_cards||[])];
        for(const p of state.players||[]){
          assert(allowed.has(p.id),"foreign table participant");
          if(p.id!==user.id&&!state.is_finished)assert.equal(p.cards?.length||0,0,"private card leak");
          if(p.id===user.id)visible.push(...(p.cards||[]));
        }
        assert.equal(new Set(visible).size,visible.length,"impossible visible cards");
        if(!state.is_finished)counters.private_states++;
        if(table.variant==="brazilian_pineapple" && !state.is_finished){
          assert.equal(state.betting_structure,"brazilian_pineapple_hybrid_v1");
          const me=state.players.find(p=>p.id===user.id);
          const choices=state.available_actions||[];
          assert(state.players.every(p=>!("_legal_actions" in p)),"internal action contract leaked");
          if(me?.is_active && choices.length){
            const call=Math.max(0,state.current_bet_to_match-me.bet);
            assert.equal(state.call_amount,Math.min(call,me.chips));
            const next=(Math.floor(state.current_bet_to_match/table.big_blind)+1)*table.big_blind;
            const pot=state.pots.reduce((sum,p)=>sum+p.amount,0);
            const cap=state.stage==="preflop"?Math.min(4*table.big_blind,next):me.bet+call+pot;
            if(choices.includes("raise")||choices.includes("bet")){
              assert.equal(state.maximum_wager,Math.min(me.bet+me.chips,cap));
              assert.equal(state.minimum_wager,state.stage==="preflop"?next:state.current_bet_to_match+state.min_raise);
            }
            if(choices.includes("allin"))assert(me.bet+me.chips<=cap,"all-in exceeds cap");
          }
        }
        onState(ws,state,user);
      }catch(error){errors.push(error.message);counters.unexpected_errors++;}
    });
  });
}
async function play(table){
  const users=table.users;
  // Fixed buy-in, mismatched mode and duplicated join are separate API checks.
  const bad=await request("/api/lobby/join",users[0],{table_id:table.id,buy_in:table.buy_in-1,wallet_mode:"play"});
  assert(bad.status>=400&&bad.status<500);counters.expected_rejections++;
  const wrong=await request("/api/lobby/join",users[0],{table_id:table.id,buy_in:table.buy_in,wallet_mode:"real"});
  assert(wrong.status>=400&&wrong.status<500);counters.expected_rejections++;
  const joined=await Promise.all(users.map(user=>request("/api/lobby/join",user,
    {table_id:table.id,buy_in:table.buy_in,wallet_mode:"play"})));
  const accepted=joined.map((r,i)=>({...r,user:users[i]})).filter(r=>r.status===200);
  const refused=joined.filter(r=>r.status!==200);
  assert.equal(accepted.length,table.cap,"seat race capacity");
  assert.equal(refused.length,1,"one contender must lose the seat race");
  assert(refused.every(r=>r.status>=400&&r.status<500&&r.status!==429));
  counters.expected_rejections+=refused.length;
  assert.equal(new Set(accepted.map(r=>r.value.seat)).size,table.cap,"duplicate seat");
  const active=accepted.map(r=>r.user);
  const replay=await request("/api/lobby/join",active[0],{table_id:table.id,buy_in:table.buy_in,wallet_mode:"play"});
  assert.equal(replay.status,200,"repeat join contract");
  assert.equal(replay.value.seat,accepted[0].value.seat);
  let playing=false,started=false,finished=false,timeoutWitness=null,retryWitness=null;
  const acted=new Map();
  const raised=new Set();
  const client=[];
  const handler=(ws,state,user)=>{
    if(!playing)return;
    if(timeoutWitness&&!timeoutWitness.observed&&state.players?.find(p=>p.id===timeoutWitness.player)?.folded){
      timeoutWitness.observed=true;
      timeoutWitness.elapsed_ms=Date.now()-timeoutWitness.started;
      assert(timeoutWitness.elapsed_ms>=Math.max(0,timeoutWitness.time_bank-1)*1000,"fold before timeout");
    }
    if(!state.is_finished)started=true;
    if(fixture.recovery&&started&&!state.is_finished){fs.writeFileSync(out+"-ready.json",JSON.stringify({table:table.id,active:true}));return;}
    if(started&&state.is_finished){finished=Boolean(timeoutWitness?.observed&&retryWitness);return;}
    if(state.is_finished)return;
    const p=state.players?.find(p=>p.id===user.id);
    if(!p?.is_active)return;
    if(!timeoutWitness){
      timeoutWitness={player:user.id,started:Date.now(),time_bank:state.time_bank,observed:false};
      event({type:"timeout_wait",table:table.id,...timeoutWitness});
      return;
    }
    if(!timeoutWitness.observed&&timeoutWitness.player===user.id)return;
    const key=JSON.stringify([state.stage,state.community_cards,state.players.map(p=>[p.id,p.chips,p.bet,p.folded])]);
    if(acted.get(user.id)===key)return;
    acted.set(user.id,key);
    const choices=state.available_actions||[];
    let action=choices.includes("check")?"check":choices.includes("call")?"call":"fold";
    let amount=0;
    if(table.variant==="brazilian_pineapple" && retryWitness && !raised.has(state.stage) && (choices.includes("raise")||choices.includes("bet"))){
      action=choices.includes("raise")?"raise":"bet";
      amount=state.maximum_wager;
      raised.add(state.stage);
    }
    counters.actions++;
    const message=JSON.stringify({type:"action",action,amount});
    ws.send(message);
    if(action==="call"&&!retryWitness){
      retryWitness={hand:state.hand_id,player:user.id,phase:state.stage,amount:state.call_amount,before_bet:p.bet};
      ws.send(message);
      event({type:"immediate_action_retry",table:table.id,...retryWitness});
    }
  };
  for(const user of active)client.push(await open(table,user,handler));
  // Reconnect without explicit leave: the durable seat must remain.
  client[0].close();await new Promise(r=>setTimeout(r,100));
  client[0]=await open(table,active[0],handler);counters.reconnections++;
  playing=true;
  for(const ws of client)ws.send(JSON.stringify({type:"get_table_info"}));
  const until=Date.now()+90000;
  while(!finished&&Date.now()<until&&errors.length===0)await new Promise(r=>setTimeout(r,50));
  assert.equal(errors.length,0,errors.join(";"));
  assert(finished,"table failed to complete an observed hand");
  counters.completed_tables++;
  witnesses.push({table:table.id,timeout:timeoutWitness,retry:retryWitness});
  playing=false;
  for(const ws of client)ws.send(JSON.stringify({type:"sit_out"}));
  await new Promise(r=>setTimeout(r,100));
  for(const ws of client)ws.close();
}
let status="failed";
try{
  await Promise.all(fixture.tables.map(play));
  assert(counters.private_states>0);
  assert.equal(counters.completed_tables,fixture.tables.length);
  status="passed";
}catch(error){errors.push(error.stack);process.exitCode=1;}
finally{
  for(const ws of sockets)ws.terminate();
  agent.destroy();log.end();
  latencies.sort((a,b)=>a-b);
  const percentile=p=>latencies[Math.min(latencies.length-1,Math.floor(latencies.length*p))]??null;
  fs.writeFileSync(out+".json",JSON.stringify({status,counters,errors,witnesses,
    latency_ms:{p50:percentile(.50),p95:percentile(.95),p99:percentile(.99)},
    scope:"real local TLS/WSS; not a VPS capacity promise",
    retry_scope:"Immediate duplicate call on the same socket; delayed retries across turns are not an idempotency guarantee.",
    gaps:[]},null,2));
}
