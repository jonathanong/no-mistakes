import ast
import copy
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest

D = Path(__file__).resolve().parent


def verifier():
    module = ast.parse((D / 'hosted.py').read_text())
    function = next(n for n in module.body if isinstance(n, ast.FunctionDef) and n.name == 'verify_staged_assets')
    scope = {'json': json, 'sha': lambda p: hashlib.sha256(p.read_bytes()).hexdigest()}
    exec(compile(ast.Module(body=[function], type_ignores=[]), 'hosted-verifier', 'exec'), scope)
    return scope['verify_staged_assets']


class TransferTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.stage = Path(self.temporary.name) / 'stage'
        shutil.copytree(D / 'fixtures/native-transfer', self.stage)
        self.proof = json.loads((self.stage / 'transport.json').read_text())
        self.spec = {'assets': copy.deepcopy(self.proof['assets'])}
        self.context = {'GITHUB_REPOSITORY': 'jonathanong/no-mistakes', 'GITHUB_RUN_ID': '123',
                        'GITHUB_RUN_ATTEMPT': '1', 'GITHUB_SHA': 'a' * 40}
        self.verify = verifier()

    def tearDown(self):
        self.temporary.cleanup()

    def test_exact_stage_and_foreign_context_rejection(self):
        self.assertEqual(self.verify(self.stage, self.spec, self.context), self.proof)
        for name in self.context:
            foreign = dict(self.context)
            foreign[name] = 'foreign'
            with self.subTest(name=name), self.assertRaises(AssertionError):
                self.verify(self.stage, self.spec, foreign)

    def test_payload_and_symlink_rejection(self):
        path = self.stage / 'baseline-cli'
        path.write_bytes(path.read_bytes() + b'tampered')
        with self.assertRaises(AssertionError):
            self.verify(self.stage, self.spec, self.context)
        path.unlink()
        path.symlink_to(D / 'fixtures/native-transfer/baseline-cli')
        with self.assertRaises(AssertionError):
            self.verify(self.stage, self.spec, self.context)

    def test_partial_and_duplicate_proof_rejection(self):
        for change in ['partial', 'duplicate']:
            proof = copy.deepcopy(self.proof)
            if change == 'partial':
                proof['complete'] = False
            else:
                proof['assets'][-1] = proof['assets'][0]
            (self.stage / 'transport.json').write_text(json.dumps(proof))
            with self.subTest(change=change), self.assertRaises(AssertionError):
                self.verify(self.stage, self.spec, self.context)

    def test_credential_and_job_boundary(self):
        workflow = (D.parents[1] / 'workflows/receiver-fact-native.yml').read_text()
        privileged, measurement = workflow.split('  measure:\n', 1)
        privileged = privileged.split('  native-download:\n', 1)[1]
        self.assertIn('contents: write', privileged)
        self.assertNotIn('checkout@', privileged)
        self.assertNotIn('hosted.py', privileged)
        self.assertIn("'request = \"GET\"'", privileged)
        self.assertNotIn('contents: write', measurement)
        self.assertIn('needs: native-download', measurement)
        self.assertIn('actions/download-artifact@', measurement)
        self.assertNotIn('run-id:', measurement)
        self.assertNotIn('github-token:', measurement)
        self.assertIn('parameter-cache-native-transfer-${{ github.run_id }}-${{ github.run_attempt }}', measurement)


if __name__ == '__main__':
    unittest.main()
