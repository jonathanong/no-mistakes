#!/usr/bin/env python3
"""Saved counterexamples for misleading allocator evidence."""
import json
from pathlib import Path
import runpy
import unittest

HERE = Path(__file__).parent
FIXTURES = HERE / 'fixtures/mimalloc-report'
REPORT = runpy.run_path(str(HERE / 'mimalloc-report.py'))


def rows(name):
    return json.loads((FIXTURES / name).read_text())


class EvidenceTests(unittest.TestCase):
    def test_worker_override_cannot_turn_four_cpus_into_many_core(self):
        for name in ('matrix-many-core-four-cpu.json', 'matrix-many-core-override.json',
                     'matrix-many-core-multiple-hosts.json'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                REPORT['summarize'](rows(name), 'many-core')

    def test_compiler_mismatch_invalidates_pair(self):
        self.assertFalse(REPORT['summarize_group'](rows('paired-wrong-compiler.json'))['valid'])

    def test_same_allocator_artifact_cannot_compare_against_itself(self):
        with self.assertRaisesRegex(ValueError, 'distinct'):
            REPORT['summarize_group'](rows('paired-identical-artifact.json'))

    def test_claimed_allocator_hash_must_match_actual_cli(self):
        self.assertFalse(REPORT['summarize_group'](rows('paired-wrong-artifact-claim.json'))['valid'])

    def test_cpu_uses_user_plus_system_within_each_physical_pair(self):
        group = REPORT['summarize_group'](rows('paired-success.json'))
        self.assertEqual(group['pairs'][0]['ratio']['cpu_seconds'], 1.0)
        self.assertEqual(group['paired_ratio']['cpu_seconds']['n'], 4)

    def test_napi_uses_distinct_addons_and_the_same_node_binary(self):
        summary = REPORT['summarize'](rows('matrix-napi.json'), 'napi')
        self.assertEqual(len(summary), 4)
        self.assertTrue(all(group['valid'] for group in summary))


if __name__ == '__main__':
    unittest.main()
