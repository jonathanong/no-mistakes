"""Saved protocol rejection tests, no analyzer processes or generated source fixtures."""
import json,unittest
from pathlib import Path
from common import command,parity,schedule
from measure import capacity
class ProtocolTests(unittest.TestCase):
    def test_saved_pair_and_budget(self):
        p=json.loads((Path(__file__).parent/'protocol.json').read_text())
        self.assertEqual(p['runner'],'ubuntu-24.04-arm');self.assertNotEqual(p['baseline'],p['candidate'])
        self.assertEqual(capacity([1,2,1,2],2300),2200)
        with self.assertRaises(AssertionError):capacity([1,2,1,2],2199)
        with self.assertRaises(AssertionError):capacity([1,2,1,120],2500)
    def test_full_parity_rejects_exit_and_changed_bytes(self):
        rows=[{'exit':0,'stdout_sha256':'a','stdout_bytes':2}]*2;parity(rows)
        for change in [{'exit':1},{'stdout_sha256':'b'},{'stdout_bytes':3}]:
            with self.assertRaises(AssertionError):parity([rows[0],dict(rows[0],**change)])
    def test_fixed_balanced_schedule(self):
        s=schedule();self.assertEqual(len(s),8)
        self.assertEqual(''.join(x[3]for x in s),'ABBABAAB')
    def test_command_has_explicit_modes(self):
        off=command('/cli','/fixture','/config',0,'off','default')
        self.assertNotIn('--jobs',off);self.assertNotIn('--timings',off)
        self.assertEqual(off[off.index('--timeout')+1],'0')
        on=command('/cli','/fixture','/config',120,'on','4')
        self.assertIn('--verbose-timings',on);self.assertEqual(on[on.index('--jobs')+1],'4')
    def test_disabled_workflow_and_same_run(self):
        text=(Path(__file__).parent/'workflow-source.yml').read_text()
        self.assertIn("inputs.confirm != 'DRAFT_DISABLED'",text)
        self.assertIn(json.loads((Path(__file__).parent/'protocol.json').read_text())['status'],['DRAFT_DISABLED','REVIEWED_ACTIVE_DEADLINE_DIAGNOSTIC'])
        for name in ['build.py','measure.py']:
            code=(Path(__file__).parent/name).read_text()
            self.assertIn("assert spec['status']=='REVIEWED_ACTIVE_DEADLINE_DIAGNOSTIC'",code);self.assertNotIn('run-id:',text)
        self.assertNotIn('contents: write',text);self.assertNotIn('ubuntu-latest',text)
    def test_producer_identity_rejections(self):
        from common import validate_producer
        import copy
        fixture=json.loads((Path(__file__).parent/'saved-producer-control.json').read_text())
        args=[fixture['producer'],'candidate',fixture['spec'],fixture['machine'],'abc',123,183]
        validate_producer(*args)
        for key,value in [('source','stale'),('bytes',124),('cli_sha256','wrong')]:
            changed=copy.deepcopy(args);changed[0][key]=value
            with self.assertRaises(AssertionError):validate_producer(*changed)
        for key,value in [('run','old'),('attempt','2')]:
            changed=copy.deepcopy(args);changed[0]['machine'][key]=value
            with self.assertRaises(AssertionError):validate_producer(*changed)
        changed=copy.deepcopy(args);changed[-1]=62
        with self.assertRaises(AssertionError):validate_producer(*changed)
        changed=copy.deepcopy(args);changed[0]['cargo_selected']['fresh']=True
        with self.assertRaises(AssertionError):validate_producer(*changed)
    def test_deadline_is_anchored_before_setup(self):
        from common import command_deadline
        import time
        now=1000
        end=command_deadline(900,{'JOB_STARTED_UNIX':'800'},now)
        self.assertAlmostEqual(end-time.monotonic(),2350,delta=.1)
        with self.assertRaises(AssertionError):command_deadline(900,{'JOB_STARTED_UNIX':'901'},now)
    def test_package_commit_and_manifest_reject_drift(self):
        from common import package_guard
        package=Path(__file__).parent
        env={'APPROVED_SOURCE_SHA':'a'*40,'GITHUB_SHA':'a'*40,'GITHUB_WORKFLOW_SHA':'a'*40,'PACKAGE_MANIFEST_SHA256':'wrong'}
        with self.assertRaises(AssertionError):package_guard(package,env)
        from common import sha
        env['PACKAGE_MANIFEST_SHA256']=sha(package/'package-manifest.json')
        package_guard(package,env)
        env['GITHUB_WORKFLOW_SHA']='b'*40
        with self.assertRaises(AssertionError):package_guard(package,env)
    def test_actual_status_guards_reject_draft(self):
        import ast
        for name in ['build.py','measure.py']:
            tree=ast.parse((Path(__file__).parent/name).read_text())
            guards=[n for n in ast.walk(tree)if isinstance(n,ast.Assert)and "spec['status']" in ast.unparse(n.test)]
            self.assertEqual(len(guards),1)
            test=compile(ast.Expression(guards[0].test),name,'eval')
            self.assertFalse(eval(test,{'spec':{'status':'DRAFT_DISABLED'}}))
            self.assertTrue(eval(test,{'spec':{'status':'REVIEWED_ACTIVE_DEADLINE_DIAGNOSTIC'}}))
    def test_fast_owned_child_keeps_wait4_accounting(self):
        import tempfile
        import subreaper
        from common import run
        subreaper.enable()
        with tempfile.TemporaryDirectory(prefix='nm2126-unit-') as directory:
            row=run(['/usr/bin/true'],Path(directory)/'fast',2)
            self.assertEqual(row['exit'],0);self.assertGreaterEqual(row['rss_kib'],0)
            self.assertEqual(subreaper.children(),[])
if __name__=='__main__':unittest.main()
