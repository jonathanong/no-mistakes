"""Saved current/frozen bytes exercise the real strict materialization loop."""
import ast
import copy
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest

D = Path(__file__).resolve().parent


class Snapshot(unittest.TestCase):
    def test_historical_snapshot_and_current_git_are_separate(self):
        tree = ast.parse((D / 'hosted.py').read_text())
        calls = [n for n in ast.walk(tree) if isinstance(n, ast.Call)
                 and isinstance(n.func, ast.Name) and n.func.id == 'command']
        git_targets = [ast.unparse(n.args[0]) for n in calls if n.args and 'git' in ast.unparse(n.args[0])]
        self.assertFalse(any('str(G)' in value for value in git_targets))
        self.assertTrue(any('str(CURRENT_GIT)' in value for value in git_targets))
        collector = next(row for row in json.loads((D / 'payload-index.json').read_text())
                         if row['path'].endswith('/collect-current-pair-semantics.py'))
        source = (D / 'payload' / collector['payload']).read_text()
        self.assertIn("GIT = Path('/tmp/no-mistakes-2022-assignment-target-opt')", source)
        self.assertIn('semantics.ROOT = GIT', source)
        self.assertIn('napi_parity.ROOT = GIT', source)

    def test_all_971_historical_snapshot_files_remain_authenticated(self):
        rows = json.loads((D / 'payload-index.json').read_text())
        historical = [row for row in rows if row['path'].startswith('/tmp/no-mistakes-2022-region-production/')]
        self.assertEqual(len(historical), 971)
        for row in historical:
            raw = (D / 'payload' / row['payload']).read_bytes()
            self.assertEqual(len(raw), row['bytes'])
            self.assertEqual(hashlib.sha256(raw).hexdigest(), row['sha256'])

    def test_saved_collision_rejects_and_fresh_snapshot_restores_exact(self):
        row = next(row for row in json.loads((D / 'payload-index.json').read_text())
                   if row['path'] == '/tmp/no-mistakes-2022-region-production/packages/no-mistakes/scripts/query-reached-per-item-api.test.js')
        frozen = D / 'payload' / row['payload']
        self.assertEqual(hashlib.sha256(frozen.read_bytes()).hexdigest(), row['sha256'])
        current = D / 'fixtures/current-query-api.data'
        self.assertNotEqual(current.read_bytes(), frozen.read_bytes())
        tree = ast.parse((D / 'hosted.py').read_text())
        loop = next(n for n in tree.body if isinstance(n, ast.For)
                    and isinstance(n.target, ast.Name) and n.target.id == 'row'
                    and "payload-index.json" in ast.unparse(n.iter))
        loop = copy.deepcopy(loop)
        loop.iter = ast.Name(id='rows', ctx=ast.Load())
        module = ast.fix_missing_locations(ast.Module(body=[loop], type_ignores=[]))
        code = compile(module, 'actual-hosted-materialization', 'exec')
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary) / 'legacy/packages/query.test.js'
            target.parent.mkdir(parents=True)
            shutil.copyfile(current, target)
            namespace = dict(rows=[dict(row, path=str(target))], D=D, Path=Path,
                             shutil=shutil, sha=lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest())
            with self.assertRaises(AssertionError):
                exec(code, namespace)
            self.assertEqual(target.read_bytes(), current.read_bytes())
            target.unlink()
            exec(code, namespace)
            self.assertEqual(target.read_bytes(), frozen.read_bytes())
