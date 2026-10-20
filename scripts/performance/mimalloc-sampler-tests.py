#!/usr/bin/env python3
"""Native rusage must not include a prior large controller heap image."""
import json
import os
from pathlib import Path
import runpy
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
SHARED = runpy.run_path(str(HERE / 'mimalloc-measure.py'))


class SamplerTests(unittest.TestCase):
    @unittest.skipUnless(sys.platform.startswith('linux'), 'Linux fork/exec RSS regression')
    def test_fresh_worker_discards_large_controller_residency(self):
        # This is an intentional controller allocation, not an inline input fixture.
        retained = bytearray(96 * 1024 * 1024)
        for offset in range(0, len(retained), 4096):
            retained[offset] = 1
        fixture = HERE / 'fixtures/mimalloc-scoped/sampler-target.py'
        with tempfile.TemporaryDirectory(prefix='mimalloc-sampler-') as output:
            argv = [sys.executable, str(fixture)]
            metadata = {'host_id': 'sampler-test', 'expected_exit': 0}
            SHARED['measure_child'](argv, HERE, output, 'old', dict(metadata), 10)
            old = json.loads((Path(output) / 'old.json').read_text())
            self.assertGreater(old['peak_rss_bytes'], len(retained) * .8)
            print(json.dumps({'control': 'old-direct-fork', 'peak_rss_bytes': old['peak_rss_bytes']}), flush=True)
            identities = []
            for index in range(2):
                name = 'fresh' + str(index)
                SHARED['measure'](argv, HERE, output, name, dict(metadata), 10)
                row = json.loads((Path(output) / (name + '.json')).read_text())
                child = json.loads((Path(output) / (name + '.stdout')).read_text())
                self.assertEqual(row['exit'], 0)
                self.assertLess(row['peak_rss_bytes'], len(retained) / 2)
                launcher = row['metadata']['sampler_launcher']
                self.assertTrue(launcher['freshInterpreter'])
                self.assertNotEqual(launcher['pid'], os.getpid())
                self.assertNotEqual(launcher['pid'], child['pid'])
                self.assertLess(launcher['VmRSSKiB'] * 1024, len(retained) / 2)
                print(json.dumps({'control': name, 'peak_rss_bytes': row['peak_rss_bytes'],
                                  'launcher': launcher}), flush=True)
                identities.append(launcher['pid'])
            self.assertEqual(len(set(identities)), 2)


if __name__ == '__main__':
    unittest.main()
