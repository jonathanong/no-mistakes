#!/usr/bin/env python3
"""Build identity is part of the experiment, including allocator separation."""
import json
from pathlib import Path
import runpy
import unittest

HERE = Path(__file__).parent
ARTIFACT = runpy.run_path(str(HERE / 'mimalloc-artifact.py'))
VERIFY = ARTIFACT['verify_pair']
MEASURE = runpy.run_path(str(HERE / 'mimalloc-measure.py'))
FIXTURES = HERE / 'fixtures/mimalloc-artifact'


class ArtifactTests(unittest.TestCase):
    def test_paired_build_identity(self):
        VERIFY(json.loads((FIXTURES / 'valid.json').read_text()))
        for name in ('compiler', 'flags', 'features', 'same-cli', 'same-addon'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                VERIFY(json.loads((FIXTURES / f'{name}.json').read_text()))

    def test_supplied_probe_cannot_override_verified_cpu_capacity(self):
        proof = json.loads((FIXTURES / 'probe-proof.json').read_text())
        ARTIFACT['verify_probe'](FIXTURES / 'probe-expected.txt', proof)
        with self.assertRaisesRegex(ValueError, 'probe does not match'):
            ARTIFACT['verify_probe'](FIXTURES / 'probe-swapped.txt', proof)

    def test_consumer_and_hosted_share_the_outer_collection_deadline(self):
        for workload in ('consumer', 'hosted-acceptance', 'hosted-passing-failing'):
            args = MEASURE['command']('/artifact', workload, '/consumer', '1', False)
            self.assertEqual(args[:6], ['/artifact', '--timeout', '0', '--lock-timeout', '0', 'check'])
            self.assertIn('--jobs', args)


if __name__ == '__main__':
    unittest.main()
