import copy,json,os,subprocess,sys,tempfile,unittest
from pathlib import Path
from contracts import CASES,ORDERS,fit,parity,parsed,seal
from kernel_assess import summary
P=Path(__file__).resolve().parent
class ProtocolTests(unittest.TestCase):
    def row(self):return json.loads((P/'fixtures/parity.json').read_text())
    def test_exact_work_values_presence_and_unrelated_metrics(self):
        a=self.row();b=copy.deepcopy(a);b['arm']='candidate';parity([a,b])
        for key in ['ast.walks','postgres.annotation.modules','postgres.annotation.captured_reader_rebuilds']:
            c=copy.deepcopy(b);c['parsed']['work'][key]+=1
            with self.assertRaises(AssertionError):parity([a,c])
        del b['parsed']['work']['postgres.annotation.captured_reader_rebuilds']
        with self.assertRaises(AssertionError):parity([a,b])
    def test_full_report_exit_parity(self):
        a=self.row();b=copy.deepcopy(a);b['stdout_sha256']='changed'
        with self.assertRaises(AssertionError):parity([a,b])
        b=copy.deepcopy(a);b['exit']=0
        with self.assertRaises(AssertionError):parity([a,b])
    def test_saved_structured_projection_and_full_ledger(self):
        x=parsed((P/'fixtures/report.json').read_text(),(P/'fixtures/progress.stderr').read_text(),'consumer-on')
        self.assertEqual(x['projection_wall_ms'],1);self.assertEqual(x['work'],self.row()['parsed']['work'])
        with self.assertRaises(AssertionError):parsed((P/'fixtures/report.json').read_text(),(P/'fixtures/progress.stderr').read_text(),'consumer-off')
    def test_duration_only_seal_and_reject_overcapacity(self):
        rows=[]
        for case in CASES:
            for arm in ['baseline','candidate']:
                row=self.row();row.update(case=case,arm=arm);rows.append(row)
        self.assertEqual(seal(rows)['child_seconds'],80)
        for row in rows:row['wall']=180
        with self.assertRaises(AssertionError):seal(rows)
        fit(180,1380)
        with self.assertRaises(AssertionError):fit(180,1379)
    def test_same_fixed_balanced_schedule_and_original_estimator(self):
        self.assertEqual(ORDERS,('ABBA','BAAB','BAAB','ABBA'))
        x=summary([(o,[1,1],[1,1])for o in ORDERS]);self.assertEqual(x['status'],'FLAT')
    def test_draft_fails_before_output_or_native(self):
        with tempfile.TemporaryDirectory()as tmp:
            out=Path(tmp)/'not-created'
            env={k:v for k,v in os.environ.items()if k not in['APPROVED_SOURCE_SHA','PROTOCOL_SHA256','PACKAGE_MANIFEST_SHA256']}
            p=subprocess.run([sys.executable,str(P/'collect.py'),'preflight','--artifacts',tmp,'--root',tmp,'--output',str(out)],capture_output=True,env=env)
            self.assertNotEqual(p.returncode,0);self.assertFalse(out.exists());self.assertTrue(b'DRAFT cannot execute'in p.stderr or b'approved source SHA required'in p.stderr)

class CrossScopeAndOwnedProcessTests(unittest.TestCase):
    def test_only_annotation_findings_can_differ(self):
        from contracts import cross_scope
        x=json.loads((P/'fixtures/crossscope.json').read_text())
        self.assertEqual(cross_scope(x['consumer'],x['control']),1)
        for field in ['advisories','warnings','rules']:
            changed=copy.deepcopy(x['control']);changed['parsed']['report'][field].append({'rule':'unrelated','message':'changed'})
            with self.assertRaises(AssertionError):cross_scope(x['consumer'],changed)
    def test_empty_removed_set_cannot_hide_exit_change(self):
        from contracts import cross_scope
        x=json.loads((P/'fixtures/crossscope.json').read_text());x['consumer']['parsed']['report']=copy.deepcopy(x['control']['parsed']['report']);x['control']['exit']=0
        with self.assertRaises(AssertionError):cross_scope(x['consumer'],x['control'])
    def test_real_fast_success_and_failure_are_reaped(self):
        from common import run
        import subreaper
        subreaper.enable()
        with tempfile.TemporaryDirectory()as tmp:
            for command,expected in [('/bin/true',0),('/bin/false',1)]:
                result=run([command],Path(tmp)/('exit'+str(expected)),5)
                self.assertEqual(result['exit'],expected);self.assertFalse(subreaper.children())
    def test_real_gnu_time_child_rss_and_failure_exit(self):
        from common import run
        import subreaper
        subreaper.enable()
        with tempfile.TemporaryDirectory()as tmp:
            tmp=Path(tmp);rss=tmp/'rss.txt'
            result=run(['/usr/bin/time','--quiet','--format=%M','--output='+str(rss),'--',sys.executable,str(P/'fixtures/exit_report.py')],tmp/'wrapped',5)
            self.assertEqual(result['exit'],1);self.assertGreater(int(rss.read_text()),0)
            self.assertEqual(json.loads((tmp/'wrapped.stdout').read_text()),json.loads((P/'fixtures/report.json').read_text()));self.assertFalse(subreaper.children())

    def test_complete_transport_manifest_rejects_changed_raw_file(self):
        from transport import verify
        self.assertIn('item.json',verify(P/'fixtures/transport-good'))
        with self.assertRaises(AssertionError):verify(P/'fixtures/transport-wrong')

class WorkflowTests(unittest.TestCase):
    def test_bounded_manual_disabled_dag_and_exact_pair(self):
        import yaml,re
        text=(P/'workflow-source.yml').read_text();workflow=yaml.safe_load(text)
        spec=json.loads((P/'protocol.json').read_text())
        self.assertEqual(workflow['permissions'],{'contents':'read'})
        self.assertEqual(set(workflow['jobs']),{'build','preflight','blocks','aggregate'})
        for job in workflow['jobs'].values():
            self.assertLessEqual(job['timeout-minutes'],30)
            self.assertIn('inputs.approved_source_sha == github.sha',job['if'])
            if spec['status'].startswith('DRAFT'):self.assertIn('false &&',job['if'])
            for step in job['steps']:
                if 'uses'in step:
                    self.assertRegex(step['uses'],r'^[^@]+@[0-9a-f]{40}$')
                    if step['uses'].startswith('actions/checkout@'):self.assertIs(step['with']['persist-credentials'],False)
        pairs=workflow['jobs']['build']['strategy']['matrix']['include']
        self.assertEqual({r['arm']:r['source']for r in pairs},{a:spec[a]for a in ['baseline','candidate']})
        self.assertEqual(workflow['jobs']['blocks']['strategy']['matrix']['block'],[0,1,2,3])
        self.assertNotIn('secrets.',text)

if __name__=='__main__':unittest.main()
