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

    def test_authenticated_modes_survive_safe_tar_filter(self):
        import tarfile,state
        for mode,symlink in [(0o664,False),(0o775,False),(0o777,True)]:
            item=tarfile.TarInfo('proof/producer.json');item.mode=mode;item.mtime=123
            if symlink:item.type=tarfile.SYMTYPE;item.linkname='producer-target.json'
            if not symlink:self.assertNotEqual(tarfile.data_filter(item,'/tmp/nm2019-arm2019/baseline').mode,mode)
            row={'mode':mode,'mtime':123,'sha256':'raw','bytes':1,'kind':'symlink'if symlink else'file'}
            manifest={'run_id':'mock','archive_sha256':'archive','files':{item.name:row}}
            archive=mock.MagicMock();archive.getmembers.return_value=[item]
            with mock.patch.object(state,'identity',return_value={'run_id':'mock'}),mock.patch.object(state,'sha',return_value='archive'),mock.patch.object(state,'git',return_value=''),mock.patch.object(Path,'read_text',return_value=json.dumps(manifest)),mock.patch.object(Path,'exists',return_value=False),mock.patch.object(Path,'is_symlink',return_value=False),mock.patch.object(Path,'mkdir'),mock.patch.object(state.tarfile,'open')as opened,mock.patch.object(state,'metadata',return_value=row),mock.patch.object(state.os,'chmod')as chmod,mock.patch.object(state.os,'utime')as utime:
                opened.return_value.__enter__.return_value=archive
                state.restore('baseline','core')
                target=common.WORK/'baseline/proof/producer.json'
                if symlink:
                    chmod.assert_not_called();utime.assert_called_once_with(target,(123,123),follow_symlinks=False)
                else:
                    chmod.assert_called_once_with(target,mode,follow_symlinks=False);utime.assert_not_called()
                archive.extract.assert_called_once_with(item,path=common.WORK/'baseline',filter='data')

if __name__=='__main__':unittest.main()
