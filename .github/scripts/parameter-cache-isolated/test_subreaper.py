from pathlib import Path
import json
import subprocess
import sys
import tempfile
import time
import unittest

from process_owner import OwnedProcess
import subreaper

D = Path(__file__).resolve().parent


class AdoptionTests(unittest.TestCase):
    def check_case(self, mode):
        subreaper.enable()
        subreaper.require_empty()
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary) / 'child-pid'
            process = subprocess.Popen([sys.executable, str(D / 'fixtures/orphan_helper.py'), str(marker), mode], start_new_session=True)
            owner = OwnedProcess(process)
            deadline = time.monotonic() + 4
            while not marker.exists():
                self.assertLess(time.monotonic(), deadline)
                time.sleep(.01)
            child = int(marker.read_text())
            identity = subreaper.record(child)
            self.assertEqual(identity['sid'], child)
            if mode == 'natural':
                self.assertEqual(process.wait(timeout=4), 0)
            else:
                with self.assertRaises(subprocess.TimeoutExpired):
                    process.wait(timeout=.05)
            events = subreaper.finish(process, owner)
            self.assertTrue(any(e.get('reaped_status') is not None and e['adopted']['pid'] == child for e in events))
            self.assertFalse((Path('/proc') / str(child)).exists())
            self.assertEqual(subreaper.children(), [])
            self.assertIsNotNone(process.returncode)
            print('ADOPTION_CONTROL ' + json.dumps({'mode': mode, 'leader_exit': process.returncode, 'events': events, 'remaining_children': subreaper.children()}), flush=True)
            return events

    def test_natural_helper_exit_reaps_separate_sid(self):
        self.check_case('natural')

    def test_outer_timeout_reaps_separate_sid(self):
        self.check_case('timeout')


if __name__ == '__main__':
    unittest.main()
