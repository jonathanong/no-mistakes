import ast,copy,json,unittest
from pathlib import Path
from requests import make_request,nonvacuous
from analyze import summary
P=Path(__file__).parent
class SourceTests(unittest.TestCase):
    def test_real_import_payload_required(self):
        c=json.loads((P/'saved-nonvacuous-controls.json').read_text());nonvacuous('2127','representative',c['positive'])
        for n in ['roots_only','empty_imports']:
            with self.assertRaises(AssertionError):nonvacuous('2127','representative',c[n])
    def test_import_identity_kind_and_line(self):
        c=json.loads((P/'saved-nonvacuous-controls.json').read_text())['positive']
        for key,value in [('specifier',''),('kind','invented'),('line',0)]:
            b=copy.deepcopy(c);b['reports'][0]['result']['files'][0]['imports'][0][key]=value
            with self.assertRaises(AssertionError):nonvacuous('2127','representative',b)
        b=copy.deepcopy(c);b['reports'][0]['id']='wrong'
        with self.assertRaises(AssertionError):nonvacuous('2127','representative',b)
    def test_fixed_workload_counts(self):
        self.assertEqual(len(make_request('2127','stress','/fixture')['reports']),128)
        self.assertEqual(len(make_request('2127','representative','/fixture')['reports']),1)
        self.assertEqual(len(make_request('2129','representative','/fixture')['reports']),3)
        self.assertEqual(len(make_request('2129','control','/fixture')['reports']),1)
    def test_fixture_proof_and_distinct_hash_assertions_exist(self):
        text=(P/'measure.py').read_text();self.assertIn("before==frozen['files']",text);self.assertIn("producers['baseline']['addon_sha256']!=producers['candidate']['addon_sha256']",text)
    def test_draft_and_four_producers(self):
        spec=json.loads((P/'protocol.json').read_text());self.assertEqual(spec['status'],'REVIEWED_ASYNC_API_DIAGNOSTIC');self.assertEqual(spec['total_rows'],192)
        workflow=(P/'workflow-source.yml').read_text();self.assertIn("inputs.confirm != 'DRAFT_DISABLED'",workflow);self.assertNotIn('contents: write',workflow)
        self.assertEqual(workflow.count('            arm:'),0);self.assertIn('run-id: 39039054266',workflow);self.assertEqual(workflow.count('            case:'),24)
    def test_helper_identity_and_budget(self):
        from common import command_deadline
        with self.assertRaises(AssertionError):command_deadline(0,{'JOB_STARTED_UNIX':'20'},now=10)
    def test_original_draft_status_rejected_by_actual_build_and_measure_guards(self):
        draft=json.loads((P/'protocol-DRAFT.json').read_text());self.assertEqual(draft['status'],'DRAFT_DISABLED')
        active=json.loads((P/'protocol.json').read_text())
        for file in ['build.py','measure.py']:
            nodes=[n for n in ast.walk(ast.parse((P/file).read_text()))if isinstance(n,ast.Assert) and "spec['status']" in ast.unparse(n.test)]
            self.assertEqual(len(nodes),1)
            code=compile(ast.Expression(nodes[0].test),'<actual-status-guard>','eval')
            from common import sha
            from types import SimpleNamespace
            context={'sha':sha,'P':P,'os':SimpleNamespace(environ={'PROTOCOL_SHA256':sha(P/'protocol.json')})}
            self.assertFalse(eval(code,dict(context,spec=draft)));self.assertTrue(eval(code,dict(context,spec=active)))
    def test_syntax(self):
        for f in P.glob('*.py'):ast.parse(f.read_text())
if __name__=='__main__':unittest.main()
