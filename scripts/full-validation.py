#!/usr/bin/env python3
"""Single owner for bounded, isolated full-validation orchestration (stdlib only)."""
import argparse
import base64
import hmac
import uuid
import urllib.request
import hashlib
import itertools
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
MOTOR_TESTS = ["cash_catalog_10k_hands", "tournament_to_champion", "full_validation_equity",
               "full_validation_situations", "variant_audit_regressions", "loss_deflator_regressions",
               "loss_deflator_seven_percent", "rake_cap_schedule", "rake_regressions"]
API_TESTS = ["api_tests", "payments_tests", "rate_limit_tests", "tournament_actor_tests",
             "concurrency_ws_tests", "actor_disconnect_stress_tests", "red_team_simulation_tests"]
SOURCE_MAP = {
    "deck": "ranking; variant card-selection rules",
    "game_loop": "legal betting; turn order; streets; runout; chip conservation",
    "side_pots": "pot eligibility; ties; odd cents",
    "loss_deflator": "equity tiers; net-pot basis; all-in phase",
    "rake": "rake rounding; caps; uncalled contributions",
    "tournament": "registration; blinds; reentry; elimination; prize accounting",
    "game_actor": "actor isolation; timeout; reconnect; settlement HMAC",
    "cash_seats": "cash-out; wallet separation; durable escrow",
    "payments": "synthetic ledger; idempotency; concurrent reservations",
    "concurrency": "internal actors only, no network evidence",
    "rate_limit": "shared Redis boundary",
    "full_validation_equity": "exact-reference error; 99% CI; threshold ambiguity",
    "full_validation_situations": "directed betting acceptance; uncalled wager",
    "cash_catalog": "catalogue occupancy/position/street/stack/action/outcome pairs",
}
def save(path, value):
    temp = path.with_suffix(path.suffix + ".tmp")
    temp.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    temp.replace(path)

def validate_immediate_retry(rows, retry):
    """Tie the retry to a hand and its paid street contribution, not just price.

    Fixed raises can legitimately produce the same call price twice. Also
    reject every nonblind call above its outstanding amount, detecting an
    immediately duplicated payment even if its before_bet already changed.
    """
    matching = [row for row in rows if row["id"] == retry["hand"]]
    if len(matching) != 1: raise RuntimeError("retry hand missing or duplicated")
    bets = {}
    calls = 0
    for index, action in enumerate(matching[0]["actions_json"]):
        phase = str(action["phase"]).lower()
        key = (phase, action["player_id"])
        before = bets.get(key, 0)
        amount = action["amount"]
        kind = action["action"]
        if kind == "call":
            to_match = max((value for (street, _), value in bets.items() if street == phase), default=0)
            if index >= 2 and not 0 < amount <= to_match - before:
                raise RuntimeError("duplicate or illegal durable call payment")
            if (index >= 2 and action["player_id"] == retry["player"] and phase == retry["phase"].lower()
                    and amount == retry["amount"] and before == retry["before_bet"]):
                calls += 1
            bets[key] = before + amount
        elif kind in ("bet", "raise"):
            bets[key] = amount
        elif kind in ("allin", "all_in"):
            bets[key] = before + amount
    if calls != 1: raise RuntimeError("immediate repeated call changed the durable action count")

class Campaign:
    def __init__(self, args):
        self.args = args
        self.ident = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime()) + "-" + secrets.token_hex(3)
        self.out = ROOT / "artifacts/full-validation" / ("runner-tests" if getattr(args,"self_test",False) else "") / self.ident
        self.out.mkdir(parents=True)
        self.stages, self.binaries, self.containers = [], {}, []
        self.started = None
        self.started_wall = None
        self.deadline = None
        self.api = None
        self.gateway = None
        self.network = "poker-validation-" + self.ident.lower()
        # Do not inherit provider credentials, DATABASE_URL, public origins or dotenv.
        self.env = {key: os.environ[key] for key in ("PATH", "HOME", "USER", "LANG", "RUSTUP_HOME", "CARGO_HOME") if key in os.environ}
        self.env.update(FULL_VALIDATION_APPROVED="1", FULL_VALIDATION_REPORT_DIR=str(self.out),
                        RUST_TEST_THREADS="2", CARGO_BUILD_JOBS="2", TOKIO_WORKER_THREADS="2",
                        PIX_PROVIDER="mock", PIX_MODE="mock", PIX_LIVE_ENABLED="false",
                        DEPIX_API_KEY="", DEPIX_WEBHOOK_SECRET="", RESEND_API_KEY="",
                        EMAIL_PROVIDER="log", ENVIRONMENT="development",
                        REQUIRE_INVITE="false", REQUIRE_EMAIL_VERIFICATION="false",
                        JWT_SECRET="synthetic-full-validation-secret-32-characters",
                        EMAIL_CODE_PEPPER="synthetic-email-pepper", KYC_DATA_PEPPER="synthetic-kyc-pepper")
        self.gaps = []
        self.status = "preparing"
        save(self.out/"source-fingerprints.json", {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()
            for p in [ROOT/"scripts/full-validation.py",ROOT/"scripts/full-validation-network.mjs",
                      ROOT/"Motor-Rust/src/game_loop.rs",ROOT/"Motor-Rust/src/tournament_engine.rs",
                      ROOT/"API-Axum/src/tournament_coordinator.rs",ROOT/"API-Axum/src/handlers/websocket.rs",
                      ROOT/"API-Axum/src/game_actor.rs",ROOT/"API-Axum/src/tournament_actor.rs",
                      ROOT/"API-Axum/src/bots.rs",ROOT/"API-Axum/src/academy.rs",
                      *(ROOT/"Motor-Rust/tests"/(t+".rs") for t in MOTOR_TESTS)]})

    def command(self, cmd, *, cwd=ROOT, input=None, timeout=300, env=None):
        return subprocess.run([str(c) for c in cmd], cwd=cwd, input=input, text=True,
                              capture_output=True, env=env or self.env, timeout=timeout, check=True).stdout

    def checkpoint(self):
        save(self.out / "campaign.json", dict(id=self.ident, status=self.status,
             seconds=None if self.started is None else round(time.monotonic()-self.started, 3),
             wall_seconds=None if self.started_wall is None else round(time.time()-self.started_wall, 3),
             max_seconds=self.args.minutes*60, simulation_workers=2,
             betting_rule_version="brazilian_pineapple_pot_before_call_v2", phase=self.args.phase, stages=self.stages, gaps=self.gaps,
             redundancy_discarded=[
                 "No engine duplication for PM vs Real; isolation belongs to API/database tests.",
                 "Removed frontend legacy no-op and fictional two-million-input counter.",
                 "Legacy 79-massive/100-table scripts retained in source, not repeated by this campaign.",
                 "One identical equity input repeats solely for determinism; not a new sample."],
             approval="User-authorized local campaign; no demo traffic, commit, push or deploy."))
        inventory_path=self.out/"test-inventory.json"
        if inventory_path.exists():
            inventory=json.loads(inventory_path.read_text(encoding="utf-8"))
            for stage in self.stages:
                log=self.out/stage.get("log","absent.log")
                if not log.is_file(): continue
                text=log.read_text(encoding="utf-8",errors="replace")
                outcomes=dict(re.findall(r"^test (\S+) \.\.\. (ok|FAILED|ignored)",text,re.M))
                for item in inventory:
                    if item["binary"]==stage["binary"] and item["test"] in outcomes:
                        item["status"]={"ok":"passed","FAILED":"failed","ignored":"ignored"}[outcomes[item["test"]]]
                        item["evidence"]=stage["log"]
            save(inventory_path,inventory)
        rows = ["stage\tstatus\tduration_seconds\tpassed\tfailed\tignored"]
        for s in self.stages:
            rows.append("\t".join(str(s.get(k, "")) for k in ("name", "status", "seconds", "passed", "failed", "ignored")))
        (self.out/"metrics.tsv").write_text("\n".join(rows)+"\n", encoding="utf-8")

    def build(self):
        for crate, tests, release in [("Motor-Rust", MOTOR_TESTS, True), ("API-Axum", API_TESTS, False)]:
            if crate == "API-Axum" and self.args.phase in ("motor", "pineapple-cash"):
                continue
            target = Path.home()/"poker-build"/("motor-stress-release" if release else "api-target")
            env = dict(self.env, CARGO_TARGET_DIR=str(target))
            cmd = ["cargo", "test", "--locked", "--no-run", "--message-format=json", "--lib"]
            if release: cmd += ["--release"]
            else: cmd += ["--bin", "poker-api", "--features", "full-validation"]
            for name in tests: cmd += ["--test", name]
            print("BUILD", crate, flush=True)
            with (self.out / (crate+"-build.log")).open("w") as log:
                proc = subprocess.Popen(cmd, cwd=ROOT/crate, env=env, text=True,
                                        stdout=subprocess.PIPE, stderr=log)
                for line in proc.stdout:
                    log.write(line)
                    try: message = json.loads(line)
                    except json.JSONDecodeError: continue
                    if message.get("reason") == "compiler-artifact" and message.get("executable") and message.get("profile",{}).get("test"):
                        self.binaries[crate+"/"+message["target"]["name"]] = message["executable"]
                if proc.wait(): raise RuntimeError("compilation failed: "+crate)
            if crate == "API-Axum":
                with (self.out/"api-binary-build.log").open("w") as log:
                    subprocess.run(["cargo","build","--locked","--bin","poker-api"],cwd=ROOT/crate,env=env,
                                   stdout=log,stderr=log,timeout=1800,check=True)
                self.api_binary = str(target/"debug/poker-api")
        save(self.out/"binaries.json", self.binaries)

    def inventory(self):
        inventory = []
        sources = {}
        for crate in ("Motor-Rust","API-Axum"):
            for folder in ("src","tests"):
                for path in (ROOT/crate/folder).rglob("*.rs"):
                    source=path.read_text(encoding="utf-8")
                    for match in re.finditer(r"\bfn\s+(\w+)\s*\(",source):
                        sources.setdefault((crate,match[1]),[]).append({
                            "file":str(path.relative_to(ROOT)),"line":source[:match.start()].count("\n")+1})
        for key,binary in self.binaries.items():
            listing=self.command([binary,"--list","--format","terse"])
            for line in listing.splitlines():
                if not line.endswith(": test"): continue
                name=line[:-6];leaf=name.split("::")[-1]
                text=key+"::"+name
                situations=sorted({v for k,v in SOURCE_MAP.items() if k in text})
                inventory.append(dict(binary=key,test=name,source=sources.get((key.split("/")[0],leaf),[]),
                    situations=situations or ["supporting routine test; no mandatory situation claimed"],
                    status="not_run",mapping="source/module classification; execution result recorded separately"))
        save(self.out/"test-inventory.json",inventory)

    def sql(self, query):
        return self.command(["docker","exec","-i",self.db_name,"psql","-X","-v","ON_ERROR_STOP=1",
                            "-U","validation","-d","validation","-At"], input=query)

    def isolate(self):
        # Docker Desktop does not publish ports on an internal bridge. The API
        # runs on the host, so use a dedicated bridge with loopback-only ports.
        self.command(["docker","network","create",self.network])
        self.db_name = self.network+"-db"
        self.redis_name = self.network+"-redis"
        for name,image,port,envs in [
            (self.db_name,"postgres:15-alpine","127.0.0.1:5549:5432",
             ["-e","POSTGRES_USER=validation","-e","POSTGRES_PASSWORD=validation-only","-e","POSTGRES_DB=validation"]),
            (self.redis_name,"redis:7-alpine","127.0.0.1:6399:6379",[])]:
            self.command(["docker","run","-d","--rm","--name",name,"--network",self.network,"-p",port,*envs,image])
            self.containers.append(name)
        for _ in range(60):
            try: self.sql("SELECT 1;"); break
            except subprocess.CalledProcessError: time.sleep(1)
        else: raise RuntimeError("isolated PostgreSQL did not become ready")
        for port in (5549,6399):
            with socket.create_connection(("127.0.0.1",port),timeout=5): pass
        self.env.update(DATABASE_URL="postgres://validation:validation-only@127.0.0.1:5549/validation",
                        REDIS_URL="redis://127.0.0.1:6399/0", HOST="127.0.0.1", PORT="3189",
                        CORS_ORIGINS="https://localhost:3449")
        self.sql("""CREATE TABLE _sqlx_migrations (
            version BIGINT PRIMARY KEY, description TEXT NOT NULL,
            installed_on TIMESTAMPTZ NOT NULL DEFAULT now(), success BOOLEAN NOT NULL,
            checksum BYTEA NOT NULL, execution_time BIGINT NOT NULL);""")
        migrations = []
        for path in sorted((ROOT/"API-Axum/migrations").glob("*.sql")):
            version = int(path.name.split("_")[0])
            raw = path.read_bytes()
            checksum = hashlib.sha384(raw).hexdigest()
            description = path.stem.split("_",1)[1].replace("_"," ").replace("'","''")
            query = "BEGIN;\n"+raw.decode("utf-8")+(
                f"\nINSERT INTO _sqlx_migrations(version,description,success,checksum,execution_time) "
                f"VALUES ({version},'{description}',true,decode('{checksum}','hex'),0);\nCOMMIT;")
            self.sql(query)
            migrations.append(dict(version=version,sha384=checksum))
        save(self.out/"migrations.json",migrations)
        tables = json.loads(self.sql("SELECT coalesce(json_agg(t),'[]') FROM tables t WHERE game_type='cash' AND status='OPEN' AND visibility='public';"))
        events = json.loads(self.sql("SELECT coalesce(json_agg(t),'[]') FROM tournaments t WHERE status='registering';"))
        canonical = json.loads((ROOT/"Documentacao/STATUS_OPERACIONAL.json").read_text(encoding="utf-8"))
        cash, mtt, catalogue_differences = [], [], []
        for spec in canonical["cash_tables"]:
            for mode in ("play","real"):
                match = [r for r in tables if r["money_mode"]==mode and r["poker_variant"]==spec["variant"]
                         and r["small_blind"]==spec["small_blind_cents"] and r["big_blind"]==spec["big_blind_cents"]
                         and r["min_buy_in"]==r["max_buy_in"]==spec["buy_in_cents"] and r["max_players"]==spec["max_players"]]
                if len(match)!=1: raise RuntimeError("cash catalogue mismatch: "+str(spec))
                if mode=="play": cash.append(match[0])
        for spec in canonical["tournament"]["events"]:
            for mode in ("play","real"):
                match = [r for r in events if r["money_mode"]==mode and r["poker_variant"]==spec["variant"]
                         and r["buy_in"]==spec["buy_in_cents"] and r["guaranteed_prize"]==spec["gtd_cents"]
                         and r["table_max_players"]==spec["table_max"] and r["max_players"]==spec["max_players"]]
                if len(match)!=1: raise RuntimeError("MTT catalogue mismatch: "+str(spec))
                expected_reentries=spec.get("play_reentries",spec["reentries"]) if mode=="play" else spec["reentries"]
                if match[0]["rebuy_max_count"]!=expected_reentries:
                    catalogue_differences.append(dict(event=spec["name"],mode=mode,field="rebuy_max_count",
                        expected=expected_reentries,actual=match[0]["rebuy_max_count"],
                        interpretation="0 means unlimited; PM and Real limits are explicit in the canonical catalogue"))
                if mode=="real": mtt.append(match[0])
        save(self.out/"catalog-db.json",dict(cash=cash,mtt=mtt))
        save(self.out/"catalog-differences.json",catalogue_differences)
        if catalogue_differences: self.gaps.append("Canonical catalogue vs migrated PM reentry policy differs; see catalog-differences.json")
        save(self.out/"isolation.json",dict(network=self.network,containers=self.containers,
             database="127.0.0.1:5549/validation",redis="127.0.0.1:6399/0",
             providers="PIX mock, email log, no external keys",canonical_sha256=hashlib.sha256(
                 (ROOT/"Documentacao/STATUS_OPERACIONAL.json").read_bytes()).hexdigest()))

    def add(self, name, binary, arguments=(), extra=None, required=True):
        self.stages.append(dict(name=name,binary=binary,arguments=list(arguments),extra=extra or {},
                                required=required,status="not_run"))

    def plan(self):
        if self.args.phase in ("pineapple", "pineapple-cash"):
            self.add("directed-pineapple", "Motor-Rust/full_validation_situations", ["pineapple_"])
            self.add("cash-4", "Motor-Rust/cash_catalog_10k_hands", ["--ignored"], {"FULL_VALIDATION_CASH_INDEX":"4"})
        if self.args.phase == "pineapple":
            self.add("mtt-4", "Motor-Rust/tournament_to_champion", ["--ignored"], {"FULL_VALIDATION_MTT_INDEX":"4"})
            self.add("api-rule-contracts", "API-Axum/poker_api", ["pineapple"])
            for tables in (5,20):
                self.add("https-wss-"+str(tables), "network", extra={"FULL_VALIDATION_TABLES":str(tables)})
            self.add("controlled-api-restart", "restart")
        if self.args.phase in ("all","motor"):
            # Finite directed fixtures run before stochastic batches.
            for test in ("invalid_deck_and_under_minimum_raise_are_rejected_without_mutation",
                         "short_allin_does_not_reopen_a_completed_action",
                         "cumulative_short_allins_reopen_only_players_facing_a_full_increment",
                         "checked_player_can_raise_a_short_opening_allin",
                         "capped_raise_uses_the_actual_full_increment",
                         "allin_small_blind_does_not_keep_the_turn",
                         "tournament_prizes_include_eliminated_places_and_conserve_odd_cents",
                         "tournament_two_entrants_redistribute_proportionally_without_float_money",
                         "invalid_prize_distribution_does_not_finish_tournament",
                         "uncalled_flop_wager_is_returned_before_rake",
                         "distinct_side_pot_winners_from_legal_entries",
                         "deflator_timing_with_later_fold_is_auditable"):
                self.add("directed-"+test,"Motor-Rust/full_validation_situations",[test,"--exact"])
            for test in ("poker_engine","variant_audit_regressions","loss_deflator_regressions",
                         "loss_deflator_seven_percent","rake_cap_schedule","rake_regressions"):
                self.add("deterministic-"+test,"Motor-Rust/"+test)
            for index in range(5):
                self.add("cash-"+str(index),"Motor-Rust/cash_catalog_10k_hands",["--ignored"],
                         {"FULL_VALIDATION_CASH_INDEX":str(index)})
                self.add("mtt-"+str(index),"Motor-Rust/tournament_to_champion",["--ignored"],
                         {"FULL_VALIDATION_MTT_INDEX":str(index)})
            self.add("equity","Motor-Rust/full_validation_equity",["--ignored"])
        if self.args.phase in ("all","api"):
            for test in ("poker_api","poker-api","api_tests","payments_tests","red_team_simulation_tests"):
                self.add("api-routine-"+test,"API-Axum/"+test)
            for test in ("api_tests","payments_tests","rate_limit_tests","tournament_actor_tests"):
                self.add("api-db-"+test,"API-Axum/"+test,["--ignored"])
            # The ignored library contracts include durable settlement, HMAC,
            # ledger idempotency, wallet modes and controlled recovery guards.
            self.add("api-db-library","API-Axum/poker_api",["--ignored"])
            for tables in (1,5,20):
                self.add("actors-"+str(tables),"API-Axum/concurrency_ws_tests",extra={"FULL_VALIDATION_TABLES":str(tables)})
            self.add("actors-disconnect","API-Axum/actor_disconnect_stress_tests")
        if self.args.phase in ("all","gateway"):
            for tables in (1,5,20):
                self.add("https-wss-"+str(tables),"network",extra={"FULL_VALIDATION_TABLES":str(tables)})
            self.add("controlled-api-restart","restart")
        self.checkpoint()

    def execute(self, stage, reproduction=False):
        stage["status"]="running"
        self.checkpoint()
        binary=stage["binary"]
        if binary in ("network","restart"):
            self.network_stage(stage)
            return
        cmd=[self.binaries[binary],*stage["arguments"],"--nocapture","--test-threads=2"]
        env=dict(self.env,**stage["extra"])
        logpath=self.out/(stage["name"]+("-reproduction" if reproduction else "")+".log")
        started=time.monotonic()
        limit=max(0.01,self.deadline-started)
        with logpath.open("w") as log:
            process=subprocess.Popen(cmd,cwd=self.out,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
            try: code=process.wait(timeout=limit)
            except (subprocess.TimeoutExpired, TimeoutError):
                os.killpg(process.pid,signal.SIGTERM)
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired: os.killpg(process.pid,signal.SIGKILL); process.wait()
                code=124
            except BaseException:
                os.killpg(process.pid,signal.SIGTERM)
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired: os.killpg(process.pid,signal.SIGKILL); process.wait()
                raise
        output=logpath.read_text(encoding="utf-8",errors="replace")
        summaries=re.findall(r"test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored;",output)
        counts=[sum(int(row[i]) for row in summaries) for i in range(3)]
        stage.update(seconds=round(time.monotonic()-started,3),passed=counts[0],failed=counts[1],
                     ignored=counts[2],log=logpath.name,exit_code=code,
                     command=cmd,environment_overrides=stage["extra"])
        deadline_hit=code==124 or "incomplete campaign: time limit" in output
        stage["status"]="incomplete" if deadline_hit or not summaries or sum(counts[:2])==0 else ("passed" if code==0 else "failed")
        if stage["status"]=="passed":
            if stage["name"]=="equity":
                thresholds=json.loads((self.out/"equity-thresholds.json").read_text())
                if any(t["status"]!="covered" for t in thresholds):
                    self.gaps.append("Some equity threshold neighbourhoods lack a directed witness; see equity-thresholds.json")
        self.checkpoint()
        if deadline_hit:
            raise TimeoutError("campaign deadline reached in "+stage["name"])
        if stage["status"]!="passed":
            if not reproduction and code!=124 and self.deadline-time.monotonic()>5:
                # Repeat ONLY the actual failed test names, not successful cases.
                failed=re.findall(r"^test (\S+) \.\.\. FAILED$",output,re.M)
                if not failed:
                    failed=re.findall(r"^---- (\S+) stdout ----$",output,re.M)
                for name in sorted(set(failed)):
                    repro_dir=self.out/"reproductions"/(stage["name"]+"-"+hashlib.sha256(name.encode()).hexdigest()[:12])
                    repro_dir.mkdir(parents=True,exist_ok=True)
                    if (self.out/"catalog-db.json").exists(): shutil.copy2(self.out/"catalog-db.json",repro_dir/"catalog-db.json")
                    repro=dict(stage,name=stage["name"]+"-minimal",arguments=[name,"--exact"]+
                               (["--ignored"] if "--ignored" in stage["arguments"] else []),
                               extra=dict(stage["extra"],FULL_VALIDATION_REPORT_DIR=str(repro_dir)))
                    try: self.execute(repro,True)
                    except RuntimeError: pass
                    save(self.out/(stage["name"]+"-reproduction.json"),repro)
            raise RuntimeError("stop-on-failure: "+stage["name"])

    def start_api(self):
        handle=(self.out/"api-service.log").open("a")
        # Only this private loopback gateway is trusted. Synthetic clients bind
        # distinct loopback IPs so per-IP limits retain their production values.
        api_env=dict(self.env,TRUST_PROXY_HEADERS="true")
        self.api=subprocess.Popen([self.api_binary],cwd=self.out,env=api_env,stdout=handle,stderr=handle)
        handle.close()
        for _ in range(100):
            if self.api.poll() is not None: raise RuntimeError("isolated API exited; inspect api-service.log")
            try:
                with urllib.request.urlopen("http://127.0.0.1:3189/health",timeout=1) as response:
                    if response.status==200: return
            except (OSError,TimeoutError): pass
            time.sleep(.2)
        raise RuntimeError("API readiness deadline")

    def prepare_gateway(self):
        if self.api: return
        self.start_api()
        cert=self.out/"localhost.pem"
        key=self.out/"localhost-key.pem"
        self.command(["openssl","req","-x509","-newkey","rsa:2048","-nodes","-keyout",key,
                      "-out",cert,"-days","2","-subj","/CN=localhost",
                      "-addext","subjectAltName=DNS:localhost"])
        # Extract the locally installed Caddy binary; no production compose/env.
        temp=self.network+"-caddy-extract"
        self.command(["docker","create","--name",temp,"caddy:2-alpine"])
        try: self.command(["docker","cp",temp+":/usr/bin/caddy",self.out/"caddy"])
        finally: self.command(["docker","rm",temp])
        (self.out/"caddy").chmod(0o700)
        config={"admin":{"disabled":True},"apps":{
            "tls":{"certificates":{"load_files":[{"certificate":str(cert),"key":str(key)}]}},
            "http":{"servers":{"validation":{"listen":["127.0.0.1:3449"],
                "tls_connection_policies":[{}],"automatic_https":{"disable":True},
                "routes":[{"handle":[{"handler":"reverse_proxy","upstreams":[{"dial":"127.0.0.1:3189"}]}]}]}}}}}
        save(self.out/"caddy.json",config)
        handle=(self.out/"gateway.log").open("w")
        self.gateway=subprocess.Popen([str(self.out/"caddy"),"run","--config",str(self.out/"caddy.json")],
                                      cwd=self.out,env=self.env,stdout=handle,stderr=handle)
        handle.close()
        time.sleep(1)
        if self.gateway.poll() is not None: raise RuntimeError("local Caddy failed")

    def synthetic_fixture(self, stage):
        details=json.loads((self.out/"catalog-db.json").read_text())
        count=int(stage["extra"].get("FULL_VALIDATION_TABLES","1"))
        tables=[]
        now=int(time.time())
        def token(uid,username):
            encode=lambda value: base64.urlsafe_b64encode(json.dumps(value,separators=(",",":")).encode()).decode().rstrip("=")
            data=encode({"alg":"HS256","typ":"JWT"})+"."+encode({"sub":uid,"username":username,"role":"player",
                "token_version":0,"iat":now,"exp":now+3600,"type":"access"})
            signature=base64.urlsafe_b64encode(hmac.new(self.env["JWT_SECRET"].encode(),data.encode(),hashlib.sha256).digest()).decode().rstrip("=")
            return data+"."+signature
        for i in range(count):
            queries=[]
            row=dict(details["cash"][4 if self.args.phase == "pineapple" else i%5])
            row.update(id=str(uuid.uuid4()),name="validation-"+stage["name"]+"-"+str(i),current_players=0)
            encoded=json.dumps(row).replace("'","''")
            queries.append("INSERT INTO tables SELECT * FROM json_populate_record(NULL::tables,'"+encoded+"'::json);")
            users=[]
            for seat in range(row["max_players"]+1):
                uid=str(uuid.uuid4()); username="val_"+uid.replace("-","")[:24]
                queries.append(f"""INSERT INTO users (id,username,email,password_hash,role,status,balance,
                    balance_pm_cash,balance_pm_mtt,balance_real,last_pm_reset_date,email_verified_at,created_at)
                    VALUES ('{uid}','{username}','{username}@validation.invalid','synthetic-no-login','player',
                    'active',15000,15000,15000,15000,(timezone('America/Sao_Paulo',now()))::date,
                    {now},{now});""")
                users.append(dict(id=uid,token=token(uid,username),ip=f"127.1.{i}.{seat+1}"))
            self.sql("BEGIN;\n"+"\n".join(queries)+"\nCOMMIT;")
            tables.append(dict(id=row["id"],cap=row["max_players"],buy_in=row["min_buy_in"],variant=row["poker_variant"],big_blind=row["big_blind"],users=users))
        fixture=dict(origin="https://localhost:3449",ca=str(self.out/"localhost.pem"),
                     output=str(self.out/stage["name"]),tables=tables,recovery=stage["binary"]=="restart")
        path=self.out/(stage["name"]+"-fixture.json")
        save(path,fixture)
        return path,tables

    def network_stage(self, stage):
        started=time.monotonic()
        self.prepare_gateway()
        path,tables=self.synthetic_fixture(stage)
        # Windows -> WSL forwarding collapses source IPs, invalidating the
        # multi-client rate-limit experiment. Run clients beside the gateway.
        node=shutil.which("node") or str(ROOT/"artifacts/full-validation/runtime/node")
        if node.endswith(".exe") or not Path(node).is_file():
            raise RuntimeError("Native Linux Node required for independent loopback clients")
        handle=(self.out/(stage["name"]+"-client.log")).open("w")
        client=subprocess.Popen([node,str(ROOT/"scripts/full-validation-network.mjs"),str(path)],
                                cwd=ROOT,env=self.env,stdout=handle,stderr=handle)
        handle.close()
        try:
            if stage["binary"]=="restart":
                ready=self.out/(stage["name"]+"-ready.json")
                until=min(self.deadline,time.monotonic()+60)
                while not ready.exists() and time.monotonic()<until and client.poll() is None: time.sleep(.1)
                if not ready.exists(): raise RuntimeError("no active hand available for controlled restart")
                table_id=tables[0]["id"]
                guards=int(self.sql(f"SELECT count(*) FROM table_hand_recovery_guards WHERE table_id='{table_id}';").strip())
                if guards!=1: raise RuntimeError("missing durable recovery guard")
                before=self.sql(f"SELECT row_to_json(s) FROM cash_game_seats s WHERE table_id='{table_id}' ORDER BY user_id;")
                # SIGKILL is intentional, restricted to the child API of this isolated run.
                self.api.kill();self.api.wait()
                self.start_api()
                if self.sql(f"SELECT status FROM tables WHERE id='{table_id}';").strip()!="PAUSED":
                    raise RuntimeError("guarded table resumed unsafely after restart")
                after=self.sql(f"SELECT row_to_json(s) FROM cash_game_seats s WHERE table_id='{table_id}' ORDER BY user_id;")
                if before!=after: raise RuntimeError("recovery changed escrow")
                self.api.terminate();self.api.wait(timeout=5);self.start_api()
                again=self.sql(f"SELECT row_to_json(s) FROM cash_game_seats s WHERE table_id='{table_id}' ORDER BY user_id;")
                if again!=after: raise RuntimeError("repeated recovery changed escrow")
                save(self.out/"recovery.json",dict(status="passed",table=table_id,guard=guards,
                     contract="paused for review; active hands are not automatically replayed",escrow_unchanged=True))
            else:
                code=client.wait(timeout=max(.01,min(150,self.deadline-time.monotonic())))
                result=json.loads((self.out/(stage["name"]+".json")).read_text())
                if code or result["status"]!="passed": raise RuntimeError("HTTPS/WSS client failed")
                self.gaps.extend(result.get("gaps",[]))
                for table in tables:
                    rows=json.loads(self.sql(f"SELECT coalesce(json_agg(h),'[]') FROM hand_history h WHERE table_id='{table['id']}';"))
                    if not rows: raise RuntimeError("no durable hand settlement")
                    witness=next(w for w in result["witnesses"] if w["table"]==table["id"])
                    actions=[a for row in rows for a in row["actions_json"][2:]]
                    retry=witness["retry"]
                    validate_immediate_retry(rows, retry)
                    if not witness["timeout"]["observed"] or not any(a["player_id"]==witness["timeout"]["player"] and a["action"]=="fold" for a in actions):
                        raise RuntimeError("no durable network timeout witness")
                    ids=set()
                    rake=0
                    for row in rows:
                        settlement=row["settlement_json"]
                        if isinstance(settlement,str): settlement=json.loads(settlement)
                        if table["variant"] == "brazilian_pineapple" and settlement.get("betting_rule_version") != "brazilian_pineapple_pot_before_call_v2":
                            raise RuntimeError("missing Pineapple rule version in durable settlement")
                        payload=json.dumps(settlement,sort_keys=True,separators=(",",":"),ensure_ascii=False).encode()
                        expected=hmac.new(self.env["JWT_SECRET"].encode(),payload,hashlib.sha256).hexdigest()
                        if not hmac.compare_digest(expected,row["settlement_signature"]): raise RuntimeError("settlement HMAC mismatch")
                        if settlement["hand_id"] in ids: raise RuntimeError("duplicate settlement")
                        ids.add(settlement["hand_id"])
                        if sum(p["amount"] for p in settlement["payouts"])+settlement["rake_collected"]!=settlement["pot_total"]:
                            raise RuntimeError("settlement chips diverged")
                        rake+=settlement["rake_collected"]
                    chips=int(self.sql(f"SELECT sum(chips) FROM cash_game_seats WHERE table_id='{table['id']}' AND status='ACTIVE';").strip())
                    if chips+rake!=table["cap"]*table["buy_in"]: raise RuntimeError("durable cash chips diverged")
                    for user in table["users"]:
                        balance=json.loads(self.sql(f"SELECT row_to_json(u) FROM (SELECT balance_pm_cash,balance_pm_mtt,balance_real FROM users WHERE id='{user['id']}') u;"))
                        if balance["balance_real"]!=15000 or balance["balance_pm_mtt"]!=15000:
                            raise RuntimeError("wallet isolation divergence")
                save(self.out/(stage["name"]+"-settlement-checks.json"),dict(status="passed",tables=len(tables)))
            stage.update(status="passed",seconds=round(time.monotonic()-started,3))
            self.checkpoint()
        finally:
            if client.poll() is None: client.terminate();client.wait(timeout=5)
            # Tokens and TLS private keys are synthetic and stay only in ignored artifacts.

    def matrix(self):
        factors = {
            "variant":["holdem","short_deck","omaha","brazilian_pineapple"],
            "occupancy":["hu","partial","full"],"position":["button","sb","bb","other"],
            "street":["Preflop","Flop","Turn","River"],"stack":["short","medium","deep"],
            "sequence":["passive","fold","fold_after_allin","allin","open","reraise","check_raise"],
            "allin":["none","single","multi"],"tie":["false","true"],"pots":["single","multiple"]}
        # Declared pair feasibility: no nonblind "other" position in HU; a fold
        # after all-in requires an all-in. Other structural uncertainty stays a gap.
        if self.args.phase.startswith("pineapple"): factors["variant"] = ["brazilian_pineapple"]
        expected=set()
        exclusions=[]
        for (a,aa),(b,bb) in itertools.combinations(factors.items(),2):
            for av,bv in itertools.product(aa,bb):
                values={a:av,b:bv}
                key="|".join(sorted((a+"="+av,b+"="+bv)))
                if (values.get("occupancy")=="hu" and values.get("position") in ("sb","other")) or (
                    values.get("sequence")=="fold_after_allin" and values.get("allin")=="none") or (
                    values.get("sequence")=="allin" and values.get("allin")=="none") or (
                    values.get("pots")=="multiple" and (values.get("allin")=="none" or values.get("occupancy")=="hu")):
                    exclusions.append(dict(pair=key,reason="structurally incompatible with positional/sequence definition"))
                else: expected.add(key)
        actual=set()
        configurations=[]
        for index in ([4] if self.args.phase.startswith("pineapple") else range(5)):
            path=self.out/f"cash-{index}.json"
            result=json.loads(path.read_text()) if path.exists() else {}
            seen=set(result.get("coverage",[]));actual.update(seen)
            config=json.loads((ROOT/"Documentacao/STATUS_OPERACIONAL.json").read_text())["cash_tables"][index]
            variant=config["variant"]
            required={pair for pair in expected if "variant=" not in pair or ("variant="+variant) in pair.split("|")}
            local_exclusions=[]
            if config["small_blind_cents"] < config["big_blind_cents"]:
                for pair,reason in (
                    ("sequence=check_raise|street=Preflop", "With SB < BB only BB can check preflop; that check closes the round."),
                    ("position=button|sequence=check_raise", "Button faces a bet preflop with SB < BB and acts last postflop; its check closes the round.")):
                    required.discard(pair)
                    local_exclusions.append(dict(pair=pair,reason=reason))
            configurations.append(dict(index=index,hands=result.get("hands",0),observed=len(seen),
                missing_pairs=sorted(required-seen),excluded_pairs=local_exclusions,status=result.get("stop","not_run")))
        missing=sorted(expected-actual)
        save(self.out/"coverage-matrix.json",dict(factors=factors,required_pairs=sorted(expected),
             observed=sorted(actual),missing_pairs=missing,excluded_pairs=exclusions,configurations=configurations,
             critical_cases={name:next((s["status"] for s in self.stages if s["name"]==stage),"not_run")
                 for name,stage in {
                     "short all-in reopening":"directed-short_allin_does_not_reopen_a_completed_action",
                     "uncalled return before rake":"directed-uncalled_flop_wager_is_returned_before_rake",
                     "distinct side-pot winners":"directed-distinct_side_pot_winners_from_legal_entries",
                     "Loss Deflator snapshot audit":"directed-deflator_timing_with_later_fold_is_auditable",
                     "odd cashback cents":"deterministic-loss_deflator_regressions",
                     "single durable settlement":"api-db-library",
                     "controlled API restart":"controlled-api-restart"}.items()},
             limitations=["Pair feasibility is conservative; outstanding unwitnessed pairs require review, never assumed impossible.",
                          "Actor tests do not establish TLS/WSS coverage."]))
        return missing + [f"configuration {c['index']} has missing pairs" for c in configurations if c["missing_pairs"]]

    def close(self):
        for process in (self.api,self.gateway):
            if process and process.poll() is None:
                process.terminate()
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired: process.kill();process.wait()
        if hasattr(self,"db_name") and self.db_name in self.containers:
            try:
                dump=self.command(["docker","exec",self.db_name,"pg_dump","-U","validation","validation"])
                (self.out/"isolated-db.sql").write_text(dump,encoding="utf-8")
            except Exception as error: self.gaps.append("DB snapshot unavailable: "+str(error))
        for container in reversed(self.containers):
            try: self.command(["docker","stop",container],timeout=30)
            except Exception as error: self.gaps.append("cleanup: "+str(error))
        if self.containers:
            try: self.command(["docker","network","rm",self.network])
            except Exception as error: self.gaps.append("cleanup network: "+str(error))

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("phase",nargs="?",choices=["all","motor","api","gateway","frontend","pineapple","pineapple-cash"],default="all")
    parser.add_argument("--minutes",type=int,default=60)
    args=parser.parse_args()
    if os.environ.get("FULL_VALIDATION_APPROVED")!="1": parser.error("explicit approval required")
    if not 1<=args.minutes<=60: parser.error("budget must be 1..60 minutes")
    if args.phase=="frontend":
        parser.error("frontend is outside the gameplay campaign; no test was executed or counted")
    campaign=Campaign(args)
    print("ARTIFACTS",campaign.out,flush=True)
    try:
        campaign.build()
        campaign.inventory()
        campaign.plan()
        # Compilation is excluded; infrastructure setup and all execution count.
        campaign.started=time.monotonic()
        campaign.started_wall=time.time()
        campaign.deadline=campaign.started+args.minutes*60
        campaign.env["FULL_VALIDATION_DEADLINE"]=str(int(time.time()+args.minutes*60))
        campaign.status="running";campaign.checkpoint()
        signal.signal(signal.SIGALRM, lambda *_: (_ for _ in ()).throw(TimeoutError(f"{args.minutes}-minute campaign deadline")))
        signal.alarm(args.minutes*60)
        campaign.isolate()
        for stage in campaign.stages:
            print("RUN",stage["name"],flush=True)
            campaign.execute(stage)
        missing=campaign.matrix()
        campaign.status="incomplete" if missing or campaign.gaps else "passed"
    except (Exception,KeyboardInterrupt) as error:
        campaign.status="incomplete" if isinstance(error,(KeyboardInterrupt,TimeoutError,subprocess.TimeoutExpired)) else "failed"
        campaign.gaps.append(str(error))
        for stage in campaign.stages:
            if stage["status"]=="running": stage["status"]="failed"
        print(type(error).__name__,error,file=sys.stderr,flush=True)
    finally:
        campaign.matrix()
        signal.alarm(0)
        campaign.close()
        campaign.checkpoint()
        print("RESULT",campaign.status,"ARTIFACTS",campaign.out,flush=True)
    return 0 if campaign.status=="passed" else 1

if __name__=="__main__":
    sys.exit(main())
