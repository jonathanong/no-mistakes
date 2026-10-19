"""Saved synthetic fixtures verify fail-closed gates, never performance claims."""
import json
import os
from pathlib import Path
import shutil
import select
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from worker import sample
from common import compile_cpu_probe
from frozen import verify_manifest
from report import compare_reports

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / 'fixtures/performance/matched-arm'


class HarnessTests(unittest.TestCase):
    def test_cpu_probe_links_release_lto_rlib(self):
        fixture = FIXTURES / 'cpu-probe-lto'
        with tempfile.TemporaryDirectory(prefix='matched-arm-lto-test-') as directory:
            executable = Path(directory) / 'cpu-probe'
            proof = compile_cpu_probe(fixture, ROOT / 'Cargo.lock', executable)
            count = int(subprocess.check_output([executable], text=True).strip())
            self.assertGreater(count, 0)
            self.assertEqual(proof['dependencyVersions']['num_cpus'], '1.17.0')

    def test_frozen_artifact_provenance_fails_closed(self):
        for path in sorted((FIXTURES / 'frozen-provenance').glob('*.json')):
            row = json.loads(path.read_text())
            with self.subTest(fixture=path.name):
                if row['accepted']:
                    verify_manifest(row['proof'], row['expected'],
                                    row['binarySha256'], row['pinnedSha256'])
                else:
                    with self.assertRaises(AssertionError):
                        verify_manifest(row['proof'], row['expected'],
                                        row['binarySha256'], row['pinnedSha256'])

    def test_only_expected_feature_findings_are_accepted(self):
        folder = FIXTURES / 'feature-delta'
        for path in sorted(folder.glob('*.json')):
            if path.name == 'baseline.json':
                continue
            with self.subTest(fixture=path.name):
                if path.name == 'valid.json':
                    proof = compare_reports(folder / 'baseline.json', path,
                                            with_ledger=False, feature_delta=True)
                    self.assertEqual(len(proof['added_policy_findings']), 2)
                else:
                    with self.assertRaises(AssertionError):
                        compare_reports(folder / 'baseline.json', path,
                                        with_ledger=False, feature_delta=True)

    def worker(self, name):
        # Temporary directories contain outputs; every input is a saved fixture.
        with tempfile.TemporaryDirectory(prefix='matched-arm-test-') as directory:
            return sample(FIXTURES / 'worker' / (name + '.py'), ROOT,
                          ROOT / '.no-mistakes.yml', '1', 'off', False,
                          Path(directory), 'b', 0)

    def test_completed_json_has_resources(self):
        row = self.worker('ok')
        self.assertTrue(row['eligible'])
        self.assertIsNotNone(row['peakRssKiB'])
        times = [point[0] for point in row['cpuTimeline']]
        self.assertEqual(times, sorted(times))
        self.assertEqual(times[-1], row['wallSeconds'])

    def test_invalid_json_has_no_eligible_resources(self):
        row = self.worker('invalid')
        self.assertFalse(row['eligible'])
        self.assertIsNone(row['peakRssKiB'])

    def test_orphan_is_cleaned_and_excluded(self):
        row = self.worker('orphan')
        self.assertFalse(row['eligible'])
        self.assertGreater(row['orphanedDescendants'], 0)
        self.assertIsNone(row['userSeconds'])

    def test_cancellation_cleans_owned_cli(self):
        with tempfile.TemporaryDirectory(prefix='matched-arm-cancel-test-') as directory:
            process = subprocess.Popen([sys.executable, str(FIXTURES / 'worker/cancel.py'),
                                        str(Path(__file__).parent), directory],
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            pidfd = None
            try:
                log = Path(directory) / '0-b.stderr'
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    if log.exists() and log.read_text().startswith('fixture-leader:'):
                        pidfd = os.pidfd_open(int(log.read_text().split(':')[1]))
                        break
                    time.sleep(0.01)
                self.assertIsNotNone(pidfd)
                process.terminate()
                process.wait(timeout=10)
                self.assertTrue(select.select([pidfd], [], [], 10)[0])
            finally:
                if pidfd is not None:
                    try:
                        signal.pidfd_send_signal(pidfd, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    os.close(pidfd)
                if process.poll() is None:
                    process.kill()
                    process.wait()

    def feasibility(self, case):
        with tempfile.TemporaryDirectory(prefix='matched-arm-event-test-') as directory:
            root = Path(directory)
            shutil.copytree(FIXTURES / 'feasibility' / case, root, dirs_exist_ok=True)
            env = dict(os.environ, GITHUB_SHA='1' * 40,
                       GITHUB_OUTPUT=str(root / 'outputs'),
                       GITHUB_STEP_SUMMARY=str(root / 'summary'))
            result = subprocess.run([sys.executable, str(Path(__file__).with_name('feasibility.py'))],
                                    cwd=root, env=env, capture_output=True)
            path = root / 'feasibility.json'
            return result.returncode, json.loads(path.read_text()) if path.exists() else None

    def test_complete_events_allow_matrix(self):
        status, result = self.feasibility('accepted')
        self.assertEqual(status, 0)
        self.assertTrue(result['feasible'])

    def test_censored_events_block_matrix(self):
        status, result = self.feasibility('rejected')
        self.assertEqual(status, 0)
        self.assertFalse(result['feasible'])

    def test_duplicate_events_fail(self):
        self.assertNotEqual(self.feasibility('duplicate')[0], 0)

    def test_stale_events_fail(self):
        self.assertNotEqual(self.feasibility('stale')[0], 0)


if __name__ == '__main__':
    unittest.main()
