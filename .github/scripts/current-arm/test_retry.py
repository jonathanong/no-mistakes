"""Saved-fixture tests for the observable capacity retry contract."""
import contextlib
import io
import json
import os
from pathlib import Path
import re
import signal
import threading
import tempfile
import unittest

import retry
import worker

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / 'fixtures/performance/current-arm/retry'


class RetryProvenance(unittest.TestCase):
    def fixture(self, name):
        return json.loads((FIXTURES / name).read_text())

    def test_build_manifest_pins_both_source_trees_and_release_artifacts(self):
        manifest = self.fixture('build-artifact-root-verified.json')
        self.assertEqual(manifest['run'], retry.RUN)
        self.assertEqual(manifest['artifact'], retry.ARTIFACT)
        self.assertEqual(manifest['archiveSha256'], retry.ARCHIVE)
        self.assertEqual(manifest['sources'], retry.SOURCES)
        for role in retry.SOURCES:
            self.assertEqual(manifest['builds'][role]['binarySha256'], retry.BINARIES[role])
            self.assertEqual(manifest['builds'][role]['cpuProbeSha256'], retry.PROBE)

    def test_artifact_origin_and_digest_are_pinned(self):
        row = self.fixture('artifact-valid.json')
        verified = retry.check_artifact_row(row, retry.ARTIFACT, retry.ARCHIVE,
                                             retry.RUN, retry.HARNESS, 'current-phase-clis')
        self.assertEqual(verified['id'], retry.ARTIFACT)
        for name in ('artifact-expired.json', 'artifact-wrong-origin.json'):
            with self.subTest(name=name), self.assertRaises(AssertionError):
                retry.check_artifact_row(self.fixture(name), retry.ARTIFACT, retry.ARCHIVE,
                                         retry.RUN, retry.HARNESS, 'current-phase-clis')

    def test_stable_identity_ignores_hostname_and_volatile_meminfo(self):
        builder = retry.stable_machine(self.fixture('machine-builder.json'))
        measured = retry.stable_machine(self.fixture('machine-measurement.json'))
        self.assertEqual(builder, measured)
        self.assertNotEqual(builder, retry.stable_machine(self.fixture('machine-wrong-image.json')))
        with self.assertRaises(AssertionError):
            retry.stable_machine(self.fixture('machine-missing-identity.json'))


class ObservableWorker(unittest.TestCase):
    def test_heartbeat_is_separate_from_raw_cli_streams(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            out = root / 'results'
            out.mkdir()
            fixture = FIXTURES / 'capacity-cli.py'
            console = io.StringIO()
            prior_cap = worker.CAP_SECONDS
            worker.CAP_SECONDS = retry.CAP_SECONDS
            with contextlib.redirect_stdout(console):
                row = worker.sample(fixture, root, root / 'config.yml', '1', 'on', False,
                                    out, 'base', 0, heartbeat=True)
            worker.CAP_SECONDS = prior_cap
            self.assertTrue(row['eligible'])
            self.assertEqual(row['command'][2], '10000')
            self.assertIn('--progress', row['command'])
            self.assertIn('--verbose-timings', row['command'])
            self.assertEqual(json.loads((out / '0-base.json').read_text()), {'ok': True})
            self.assertEqual((out / '0-base.stderr').read_text(),
                             '[timing] preserved raw diagnostic\n')
            self.assertIn('CAPACITY arm=base status=started', console.getvalue())
            self.assertIn('CAPACITY arm=base status=finished', console.getvalue())
            self.assertIn('cpuSeconds=', console.getvalue())
            self.assertIn('peakRssKiB=', console.getvalue())
            self.assertNotIn('CAPACITY', (out / '0-base.json').read_text())

    def test_interrupt_reaps_child_and_emits_terminal_heartbeat(self):
        with tempfile.TemporaryDirectory() as temp:
            root, out = Path(temp), Path(temp) / 'results'
            out.mkdir()
            pid_path = root / 'child.pid'
            previous_env = os.environ.get('CAPACITY_CHILD_PID')
            previous_handler = signal.getsignal(signal.SIGTERM)
            os.environ['CAPACITY_CHILD_PID'] = str(pid_path)
            def cancel(*_):
                raise InterruptedError('test cancellation')
            signal.signal(signal.SIGTERM, cancel)
            timer = threading.Timer(0.3, os.kill, args=(os.getpid(), signal.SIGTERM))
            console = io.StringIO()
            try:
                timer.start()
                with contextlib.redirect_stdout(console), self.assertRaises(InterruptedError):
                    worker.sample(FIXTURES / 'capacity-sleeper.py', root, root / 'config.yml',
                                  '1', 'on', False, out, 'base', 0, heartbeat=True)
            finally:
                timer.cancel()
                signal.signal(signal.SIGTERM, previous_handler)
                if previous_env is None:
                    os.environ.pop('CAPACITY_CHILD_PID', None)
                else:
                    os.environ['CAPACITY_CHILD_PID'] = previous_env
            pid = int(pid_path.read_text())
            with self.assertRaises(ProcessLookupError):
                os.kill(pid, 0)
            self.assertIn('status=interrupted', console.getvalue())
            self.assertTrue((out / '0-base.json').exists())

    def test_live_heartbeat_reports_elapsed_cpu_and_rss(self):
        source = Path(worker.__file__).read_text()
        self.assertIn('elapsedSeconds={elapsed:.1f}', source)
        self.assertIn('cpuSeconds=', source)
        self.assertIn('rssKiB=', source)
        self.assertIn('lastDiagnostic=', source)
        self.assertEqual(worker.diagnostic_tail(FIXTURES / 'capacity-cli.py')[-1],
                         "print('[timing] preserved raw diagnostic', file=sys.stderr)")
        self.assertIn('elapsed - last_heartbeat >= 60', source)


class WorkflowContract(unittest.TestCase):
    def test_one_job_is_split_around_immediate_base_retention(self):
        workflow = (ROOT / '.github/workflows/current-phase-observable-capacity-arm.yml').read_text()
        self.assertIn('timeout-minutes: 355', workflow)
        self.assertEqual(workflow.count('timeout-minutes: 169'), 2)
        budgets = [int(value) for value in re.findall(r'timeout-minutes: (\d+)', workflow)]
        self.assertEqual(budgets[0], 355)
        self.assertLess(sum(budgets[1:]), 355)
        self.assertIn('artifact-ids: 12204356184', workflow)
        self.assertIn('run-id: 39282454396', workflow)
        self.assertIn('MATCHED_COLLECTION_CAP_SECONDS: \'10000\'', workflow)
        base = workflow.index('retry.py base')
        upload = workflow.index('Retain baseline raw output immediately')
        candidate = workflow.index('retry.py candidate')
        self.assertLess(base, upload)
        self.assertLess(upload, candidate)
        self.assertIn('eligibleForGate', (ROOT / '.github/scripts/current-arm/inputs.py').read_text())
        self.assertIn('capacityOnly', (ROOT / '.github/scripts/current-arm/retry.py').read_text())


if __name__ == '__main__':
    unittest.main()
