"""The saved successful lookup must not leave owned adopted descendants."""
import importlib.util
import importlib.machinery
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from process_owner import OwnedProcess
import subreaper

D = Path(__file__).resolve().parent
REAL_POPEN = subprocess.Popen
EVENTS = []


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path, loader=importlib.machinery.SourceFileLoader(name, str(path)))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class LookupOwnership(unittest.TestCase):
    def exercise(self, filename, corrected):
        subreaper.enable()
        subreaper.require_empty()
        module = load('clock_under_test', D / filename)
        pairs = []
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)
            pidfile = output / 'child.pid'

            def spawn(argv, **kwargs):
                self.assertEqual(argv[:2], ['gh', 'api'])
                # Isolate the saved fixture for safe test teardown even when
                # exercising the old unsupervised subprocess.run call.
                kwargs.setdefault('start_new_session', True)
                process = REAL_POPEN([sys.executable, str(D / 'fixtures/clock/clock_lookup.py'), str(pidfile)], **kwargs)
                pairs.append((process, OwnedProcess(process)))
                return process

            pages = json.loads((D / 'fixtures/clock/actual-A-job-clock.json').read_text())
            job = next(j for p in pages for j in p['jobs'] if j['name'] == 'Child2064 complete workload cohort (A)')
            start = module.datetime.datetime.fromisoformat(job['started_at'].replace('Z', '+00:00')).timestamp()
            env = {'GITHUB_REPOSITORY': 'jonathanong/no-mistakes', 'GITHUB_RUN_ID': '38972147818', 'GITHUB_RUN_ATTEMPT': '1'}
            try:
                with patch.dict(os.environ, env), patch.object(subprocess, 'Popen', spawn), patch.object(module.time, 'time', return_value=start + 1):
                    module.install(output, job['name'])
                observed = subreaper.children()
                event = {'source': filename, 'corrected': corrected, 'owned_leader': pairs[0][1].initial,
                         'observed_children': observed, 'recorded_child_pid': int(pidfile.read_text()),
                         'clock_receipt': json.loads((output / 'job-clock-command.json').read_text())}
                EVENTS.append(event)
                if corrected:
                    self.assertEqual(observed, [])
                    subreaper.require_empty()
                    self.assertTrue(event['clock_receipt']['owned_cleanup'])
                else:
                    self.assertEqual([c['pid'] for c in observed], [event['recorded_child_pid']])
                    self.assertTrue(all(c['ppid'] == os.getpid() for c in observed))
                    with self.assertRaises(AssertionError):
                        subreaper.require_empty()
            finally:
                for process, owner in pairs:
                    cleanup = subreaper.finish(process, owner)
                    if EVENTS:
                        EVENTS[-1]['final_test_cleanup'] = cleanup
                subreaper.require_empty()
                if os.environ.get('CLOCK_FIXTURE_EVIDENCE_PATH'):
                    Path(os.environ['CLOCK_FIXTURE_EVIDENCE_PATH']).write_text(json.dumps(EVENTS, indent=2) + '\n')

    def test_original_plain_lookup_leaves_adopted_child(self):
        self.exercise('fixtures/clock/deadline.original.py.txt', False)

    def test_corrected_lookup_reaps_before_next_command(self):
        self.exercise('deadline.py', True)
