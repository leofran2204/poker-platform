"""Contract tests for truthful counts, failure stops and timeout cleanup; no poker load."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import sys
import time
import unittest

spec=importlib.util.spec_from_file_location("campaign",Path(__file__).with_name("full-validation.py"))
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class ReportingContracts(unittest.TestCase):
    def setUp(self):
        self.c=module.Campaign(argparse.Namespace(phase="motor",minutes=1,self_test=True))
        self.c.started=time.monotonic()
        self.c.deadline=time.monotonic()+10
        self.c.binaries={"fixture":sys.executable}
    def stage(self,program):
        self.c.add("fixture","fixture",["-c",program])
        return self.c.stages[0]
    def test_fixed_limit_repeat_price_in_a_later_turn_is_legal(self):
        actions = [
            dict(player_id="sb",action="call",amount=50,phase="Preflop"),
            dict(player_id="bb",action="raise",amount=50,phase="Preflop"),
            dict(player_id="hero",action="call",amount=50,phase="Preflop"),
            dict(player_id="bb",action="raise",amount=100,phase="Preflop"),
            dict(player_id="hero",action="call",amount=50,phase="Preflop"),
        ]
        retry=dict(hand="h1",player="hero",phase="preflop",amount=50,before_bet=0)
        module.validate_immediate_retry([dict(id="h1",actions_json=actions)],retry)

    def test_retry_still_rejects_a_duplicated_payment(self):
        actions = [
            dict(player_id="sb",action="call",amount=50,phase="Preflop"),
            dict(player_id="bb",action="raise",amount=50,phase="Preflop"),
            dict(player_id="hero",action="call",amount=50,phase="Preflop"),
            dict(player_id="hero",action="call",amount=50,phase="Preflop"),
        ]
        retry=dict(hand="h1",player="hero",phase="preflop",amount=50,before_bet=0)
        with self.assertRaisesRegex(RuntimeError,"duplicate or illegal"):
            module.validate_immediate_retry([dict(id="h1",actions_json=actions)],retry)

    def test_unexecuted_stage_is_never_passed(self):
        self.c.add("pending","fixture")
        self.c.checkpoint()
        result=json.loads((self.c.out/"campaign.json").read_text())
        self.assertEqual(result["stages"][0]["status"],"not_run")
        self.assertNotIn("passed",result["stages"][0])
    def test_actual_counts_not_planned_counts(self):
        module.save(self.c.out/"test-inventory.json",[dict(binary="fixture",test="real_case",status="not_run")])
        s=self.stage('print("test real_case ... ok"); print("test result: ok. 2 passed; 0 failed; 3 ignored; 9 filtered out;")')
        self.c.execute(s)
        self.assertEqual((s["passed"],s["failed"],s["ignored"]),(2,0,3))
        self.assertEqual(s["status"],"passed")
        inventory=json.loads((self.c.out/"test-inventory.json").read_text())
        self.assertEqual(inventory[0]["status"],"passed")
    def test_noop_and_all_ignored_are_incomplete(self):
        for output in ("nothing executed","test result: ok. 0 passed; 0 failed; 3 ignored;"):
            self.c.stages=[]
            s=self.stage("print("+repr(output)+")")
            with self.assertRaises(RuntimeError): self.c.execute(s,reproduction=True)
            self.assertEqual(s["status"],"incomplete")
    def test_failure_does_not_execute_next_stage(self):
        s=self.stage('print("test result: FAILED. 0 passed; 1 failed; 0 ignored;"); raise SystemExit(1)')
        self.c.add("next","fixture",["-c",'raise Exception("must not run")'])
        with self.assertRaises(RuntimeError): self.c.execute(s,reproduction=True)
        self.assertEqual(s["failed"],1)
        self.assertEqual(self.c.stages[1]["status"],"not_run")
    def test_timeout_kills_child_and_keeps_partial_log(self):
        pidfile=self.c.out/"child.pid"
        s=self.stage(f'import os,time; open({str(pidfile)!r},"w").write(str(os.getpid())); print("checkpoint",flush=True); time.sleep(10)')
        self.c.deadline=time.monotonic()+.2
        with self.assertRaises(TimeoutError): self.c.execute(s,reproduction=True)
        self.assertEqual(s["status"],"incomplete")
        self.assertIn("checkpoint",(self.c.out/s["log"]).read_text())
        with self.assertRaises(ProcessLookupError): os.kill(int(pidfile.read_text()),0)
    def test_test_reported_deadline_is_incomplete_without_reproduction(self):
        s=self.stage('print("incomplete campaign: time limit"); print("test result: FAILED. 0 passed; 1 failed; 0 ignored;"); raise SystemExit(1)')
        with self.assertRaises(TimeoutError): self.c.execute(s)
        self.assertEqual(s["status"],"incomplete")
        self.assertFalse((self.c.out/"reproductions").exists())
    def test_reproduction_cannot_overwrite_primary_artifacts(self):
        executable=self.c.out/"fixture-executable"
        executable.write_text('#!'+sys.executable+'\nimport os,pathlib,sys\np=pathlib.Path(os.environ["FULL_VALIDATION_REPORT_DIR"])\n(p/"trace.json").write_text("reproduction" if "--exact" in sys.argv else "primary")\nprint("test broken ... FAILED")\nprint("test result: FAILED. 0 passed; 1 failed; 0 ignored;")\nraise SystemExit(1)\n')
        executable.chmod(0o700)
        self.c.binaries["fixture"]=str(executable)
        self.c.add("fixture","fixture")
        with self.assertRaises(RuntimeError): self.c.execute(self.c.stages[0])
        self.assertEqual((self.c.out/"trace.json").read_text(),"primary")
        traces=list((self.c.out/"reproductions").glob("*/trace.json"))
        self.assertEqual(len(traces),1)
        self.assertEqual(traces[0].read_text(),"reproduction")
    def test_external_environment_is_not_inherited(self):
        os.environ["DATABASE_URL"]="postgres://remote.example/prod"
        os.environ["DEPIX_API_KEY"]="must-not-inherit"
        c=module.Campaign(argparse.Namespace(phase="motor",minutes=1,self_test=True))
        self.assertNotIn("DATABASE_URL",c.env)
        self.assertEqual(c.env["DEPIX_API_KEY"],"")
        self.assertEqual(c.env["PIX_PROVIDER"],"mock")

if __name__=="__main__":
    unittest.main()
