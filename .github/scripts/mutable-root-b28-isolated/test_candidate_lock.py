"""Exercise the actual hosted assertion against saved public candidate bytes."""
import ast
import hashlib
import json
from pathlib import Path
import unittest

D = Path(__file__).resolve().parent

class CandidateLock(unittest.TestCase):
    def test_actual_pnpm_guard_rejects_original_cargo_pin(self):
        spec = json.loads((D / 'hosted-spec.json').read_text())
        proof = json.loads((D / 'fixtures/d784-lock-source-proof.json').read_text())
        self.assertEqual(spec['candidate_head'], proof['candidate_head'])
        self.assertEqual(spec['candidate_tree'], proof['candidate_tree'])
        nodes = [node for node in ast.walk(ast.parse((D / 'hosted.py').read_text()))
                 if isinstance(node, ast.Assert) and 'candidate_lock_sha256' in ast.unparse(node)]
        self.assertEqual(len(nodes), 1)
        code = compile(ast.fix_missing_locations(ast.Module(body=nodes, type_ignores=[])), '<actual-hosted-lock-guard>', 'exec')
        def candidate_path(name):
            self.assertEqual(name, 'candidate-source/pnpm-lock.yaml')
            return D / 'fixtures/d784-pnpm-lock.yaml.data'
        namespace = {'Path': candidate_path, 'spec': spec,
                     'sha': lambda p: hashlib.sha256(p.read_bytes()).hexdigest()}
        exec(code, namespace)
        old = dict(spec, candidate_lock_sha256=proof['old_wrong_Cargo_sha256'])
        with self.assertRaises(AssertionError):
            exec(code, dict(namespace, spec=old))
