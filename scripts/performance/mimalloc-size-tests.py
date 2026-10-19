"""Saved release inventories must be complete, paired and hash verified."""
import copy
import json
from pathlib import Path
import runpy
import unittest
from unittest.mock import patch

MODULE = runpy.run_path(str(Path(__file__).with_name('mimalloc-size.py')))
FIXTURES = Path(__file__).resolve().parents[2] / 'fixtures/mimalloc-size'
TARGET = 'x86_64-unknown-linux-gnu'


class SizeTests(unittest.TestCase):
    def test_complete_inventory_reports_four_exact_deltas(self):
        rows = MODULE['summarize'](FIXTURES / 'valid')
        self.assertEqual(len(rows), 4)
        for row in rows:
            for field in ['cli', 'addon']:
                sizes = row[field]
                self.assertEqual(sizes['deltaBytes'], sizes['candidateBytes'] - sizes['baselineBytes'])
                self.assertEqual(sizes['percentDelta'], 100 * sizes['deltaBytes'] / sizes['baselineBytes'])

    def test_invalid_artifacts_and_incomplete_matrix_are_rejected(self):
        with self.assertRaises(ValueError):
            MODULE['summarize'](FIXTURES / 'bad-hash')
        for name in ['bad-hash', 'bad-tls', 'bad-flags', 'missing-identity', 'bad-proof', 'missing-flag']:
            with self.subTest(name=name), self.assertRaises(ValueError):
                MODULE['read_variant'](FIXTURES / name, TARGET, 'mimalloc')

    def test_pair_identity_and_identical_binaries_are_rejected(self):
        # Preserve the real saved artifact verifier; vary only a returned proof.
        original = MODULE['summarize'].__globals__['read_variant']
        cases = json.loads((FIXTURES / 'paired-errors.json').read_text())
        for case in cases + [{'field': 'same-binary'}]:
            def changed(folder, target, variant):
                proof, binary = original(folder, target, variant)
                proof = copy.deepcopy(proof)
                if variant == 'mimalloc':
                    if case['field'] == 'same-binary':
                        base, _ = original(folder.with_name(folder.name.replace('-mimalloc', '-system')), target, 'system')
                        proof['artifacts'] = base['artifacts']
                    else:
                        proof[case['field']] = case['value']
                return proof, binary
            with self.subTest(case=case), patch.dict(MODULE['summarize'].__globals__, read_variant=changed):
                with self.assertRaises(ValueError):
                    MODULE['summarize'](FIXTURES / 'valid')


if __name__ == '__main__':
    unittest.main()
