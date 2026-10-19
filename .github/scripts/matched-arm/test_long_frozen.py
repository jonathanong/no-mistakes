"""Saved cross-run identity and deadline checks for the longer evaluation."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

from long_frozen import ARTIFACTS, verify_artifact

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / 'fixtures/performance/matched-arm/long-frozen-artifacts.json'


class LongFrozenTests(unittest.TestCase):
    def test_saved_artifacts_are_exact_run_and_archive(self):
        rows = json.loads(FIXTURE.read_text())
        self.assertEqual(set(ARTIFACTS), {row['name'] for row in rows})
        for row in rows:
            with self.subTest(name=row['name']):
                verify_artifact(row['name'], row)
                for mutation in ('digest', 'id', 'head_sha', 'expired'):
                    changed = copy.deepcopy(row)
                    if mutation == 'head_sha':
                        changed['workflow_run']['head_sha'] = '0' * 40
                    elif mutation == 'expired':
                        changed['expired'] = True
                    elif mutation == 'id':
                        changed['id'] += 1
                    else:
                        changed['digest'] = 'sha256:' + '0' * 64
                    with self.assertRaises(AssertionError):
                        verify_artifact(row['name'], changed)

    def test_long_cap_is_explicit_and_default_cap_is_preserved(self):
        command = [sys.executable, '-c', 'from common import CAP_SECONDS; print(CAP_SECONDS)']
        root = str(Path(__file__).parent)
        env = dict(os.environ, PYTHONPATH=root)
        env.pop('MATCHED_COLLECTION_CAP_SECONDS', None)
        self.assertEqual(subprocess.check_output(command, env=env, text=True).strip(), '600')
        env['MATCHED_COLLECTION_CAP_SECONDS'] = '3600'
        self.assertEqual(subprocess.check_output(command, env=env, text=True).strip(), '3600')
        env['MATCHED_COLLECTION_CAP_SECONDS'] = '86400'
        self.assertNotEqual(subprocess.run(command, env=env, capture_output=True).returncode, 0)


if __name__ == '__main__':
    unittest.main()
