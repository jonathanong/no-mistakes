"""Bounded source/mock tests only; no compiler, network, analyzer or workflow run."""
import unittest,json,unittest.mock as mock,ast
from pathlib import Path
import common,guard

class DraftTests(unittest.TestCase):
    def test_draft_rejects_before_process_or_network(self):
        with mock.patch.dict(common.SPEC,{'status':'DRAFT'}),mock.patch.object(guard,'git',side_effect=AssertionError('process touched')),mock.patch.object(guard,'api',side_effect=AssertionError('network touched')):
            with self.assertRaisesRegex(AssertionError,'DRAFT'):guard.reviewed()
    def test_fixed_controls_exact(self):
        self.assertEqual(set(common.SPEC['controls']),{'workloads.json','batches-1972.json','calibration-batches.json'})
        for name,digest in common.SPEC['controls'].items():self.assertEqual(common.sha(common.HERE/'payload/protocol'/name),digest)
        rows=json.loads((common.HERE/'payload/protocol/workloads.json').read_text());self.assertEqual(len(rows),22)
        self.assertEqual(len(['ABBA','BAAB']*4)*len(rows)*4,704)
    def test_input_inventory_complete(self):
        root=common.HERE/'payload/original/inputs';rows=json.loads((common.HERE/'payload/original/input-manifest.json').read_text())['files']
        self.assertEqual(len(rows),211)
        self.assertEqual({str(p.relative_to(root))for p in root.rglob('*')if p.is_file()},{r['path']for r in rows})
        for row in rows:self.assertEqual(common.sha(root/row['path']),row['sha256'])
    def test_arm_identities_are_distinct_and_no_artifact_reuse(self):
        self.assertNotEqual(common.SPEC['baseline']['revision'],common.SPEC['candidate']['revision'])
        self.assertEqual(common.SPEC['target'],'aarch64-unknown-linux-gnu')
        self.assertEqual(common.SPEC['measurement_budget_seconds'],1500)
        self.assertEqual(common.SPEC['features'],['default','mermaid-validation'])
    def test_controller_math_unchanged(self):
        source=(common.HERE/'payload/protocol/analyze.py').read_text()
        for text in ['random.Random(1972)','range(10000)',"metrics=['cpu_s','user_s','system_s','wall_s','rss_kib']",'rng.randrange(8)','==704']:
            self.assertIn(text,source)
    def test_every_python_parses(self):
        for path in common.HERE.rglob('*.py'):ast.parse(path.read_text(),filename=str(path))

if __name__=='__main__':unittest.main()
