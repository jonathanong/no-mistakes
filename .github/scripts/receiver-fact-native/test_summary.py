"""Saved records exercise evidence validity and prevent false flat conclusions."""
import json
from pathlib import Path
import unittest
from unittest.mock import patch

from budgets import effective_memory
from outputs import curate
import pmu

from summary import assess, stable, validate
from common import ExperimentFailure, failure_record

FIXTURES = Path(__file__).parent / 'fixtures'


class EvidenceTests(unittest.TestCase):
    def test_saved_evidence_classifications(self):
        for path in sorted(FIXTURES.glob('*.json')):
            if 'rows' not in json.loads(path.read_text()):
                continue
            with self.subTest(case=path.stem):
                case = json.loads(path.read_text())
                self.assertEqual(assess(case['rows'])['status'], case['expected_status'])

    def test_material_cost_requires_consistent_order_and_uncertainty(self):
        rows = json.loads((FIXTURES / 'positive.json').read_text())['rows']
        result = assess(rows)
        for demand in result['demands'].values():
            self.assertEqual(demand['status'], 'material regression observed')
            self.assertGreater(demand['descriptive_bootstrap_95_percent'][0], 3)
            self.assertTrue(all(change > 0 for change in demand['orders_percent'].values()))

    def test_mixed_demands_do_not_hide_positive_cost(self):
        for name in ('mixed-flat', 'mixed-noisy'):
            rows = json.loads((FIXTURES / (name + '.json')).read_text())['rows']
            result = assess(rows)
            self.assertEqual(result['status'], 'material regression observed')
            self.assertEqual(result['demands']['values']['status'], 'material regression observed')
            self.assertEqual(result['demands']['imports']['status'], 'inconclusive')

    def test_curated_failures_keep_invariant_and_context(self):
        cases = json.loads((FIXTURES / 'failures.json').read_text())['cases']
        for case in cases:
            error = ExperimentFailure(case['code'], **case['details'])
            result = failure_record(error, case['stage'], arm=case['arm'], profile=case['profile'])
            self.assertEqual(result['status'], 'incomplete')
            for key in ('stage', 'arm', 'profile', 'details'):
                self.assertEqual(result[key], case[key])
            self.assertEqual(result['error_code'], case['code'])
        result = failure_record(RuntimeError('raw diagnostic withheld'), 'opt3', arm='baseline', profile='opt3')
        self.assertNotIn('raw diagnostic withheld', json.dumps(result))


    def test_unexpected_output_is_private_even_when_not_utf8(self):
        cases = json.loads((FIXTURES / 'output-policy.json').read_text())['cases']
        for case in cases:
            stdout, stderr = bytes(case['stdout_bytes']), bytes(case['stderr_bytes'])
            public, private = curate(stdout, stderr, case['demand'])
            self.assertEqual(public['signature'], case['expected_signature'])
            self.assertEqual(public['stdout_sha256'], case['expected_stdout_sha256'])
            self.assertEqual(public['stdout_bytes'], len(stdout))
            self.assertEqual('stdout' in private, case['private_stdout'])
            self.assertEqual('stderr' in private, case['private_stderr'])
            self.assertNotIn('unpublished detail', json.dumps(public))
            self.assertNotIn('private error', json.dumps(public))

    def test_visible_ancestor_headroom_constrains_resource_budget(self):
        cases = json.loads((FIXTURES / 'memory-budgets.json').read_text())['cases']
        for case in cases:
            self.assertEqual(effective_memory(case['proc_available_bytes'], case['limits']),
                             case['expected_effective_bytes'])

    def test_optional_pmu_metadata_failure_is_not_a_preflight_failure(self):
        with patch.object(pmu, 'metadata', side_effect=FileNotFoundError(2, 'private diagnostic')):
            with patch.object(pmu, 'counter_smoke', side_effect=PermissionError(1, 'private diagnostic')):
                result = pmu.capability()
        self.assertFalse(result['available'])
        self.assertTrue(result['metadata_unavailable'])
        self.assertEqual(result['metadata_errno'], 2)
        self.assertEqual(result['errno'], 1)
        self.assertNotIn('private diagnostic', json.dumps(result))


    def test_short_batches_do_not_qualify_as_stable(self):
        # Reuse a saved complete zero-effect case: stability is separate from effect.
        rows = json.loads((FIXTURES / 'zero-effect.json').read_text())['rows']
        validate(rows, 4)
        self.assertTrue(stable(rows))
        self.assertFalse(stable(rows, minimum_cpu=10))


if __name__ == '__main__':
    unittest.main()
