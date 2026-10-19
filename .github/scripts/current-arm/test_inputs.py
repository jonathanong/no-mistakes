"""Protect current-source pins and exclusion of capacity-only observations."""
from pathlib import Path
import unittest

from inputs import CAP_SECONDS, JOB_MINUTES, MEASURE_MINUTES, SOURCES, outcome, validate_parity


class CapacityContract(unittest.TestCase):
    def test_source_pins_are_exact_pr_pair(self):
        self.assertEqual(SOURCES, {
            'base': 'e377c2ba31c3f1a59f39054b67d3eafa7c2a850a',
            'phases': 'b2701498b16bd63cd08993283151c4f169275dbe'})
        workflow = Path(__file__).resolve().parents[2] / 'workflows/current-phase-feasibility-arm.yml'
        text = workflow.read_text()
        for source in SOURCES.values():
            self.assertIn('ref: ' + source, text)
        self.assertNotIn('matched-cli-base', text)
        self.assertNotIn('matched-cli-phases', text)

    def test_collection_fits_measurement_and_cleanup(self):
        self.assertEqual(CAP_SECONDS, 10000)
        self.assertLess(2 * (CAP_SECONDS + 5), MEASURE_MINUTES * 60)
        self.assertLessEqual(MEASURE_MINUTES + 8 + 3, JOB_MINUTES)
        self.assertLess(JOB_MINUTES, 360)
        workflow = Path(__file__).resolve().parents[2] / 'workflows/current-phase-feasibility-arm.yml'
        text = workflow.read_text()
        self.assertIn(f'timeout-minutes: {JOB_MINUTES}', text)
        self.assertIn(f'timeout-minutes: {MEASURE_MINUTES}', text)

    def test_complete_and_censored_capacity_are_never_gate_evidence(self):
        for rows in ([], [{'eligible': True}, {'eligible': True}],
                     [{'eligible': False}, {'eligible': True}]):
            result = outcome(rows)
            self.assertFalse(result['eligibleForGate'])
            self.assertEqual(result['purpose'], 'feasibility-only')
            self.assertNotIn('ratios', result)
            self.assertNotIn('eligible', result)

    def test_saved_report_parity_runs_actual_capacity_path(self):
        fixture = Path(__file__).resolve().parents[3] / 'fixtures/performance/matched-arm/feature-delta'
        rows = [{'eligible': True, 'exit': 1, 'semanticStderr': '',
                 'stdout': 'baseline.json'} for _ in range(2)]
        validate_parity(fixture, rows)
        rows[1] = dict(rows[1], stdout='changed-warning.json')
        with self.assertRaises((AssertionError, ValueError)):
            validate_parity(fixture, rows)

    def test_parity_failure_is_explicitly_excluded(self):
        result = outcome([{'eligible': True}, {'eligible': True}], 'failed')
        self.assertEqual(result['status'], 'failed')
        self.assertFalse(result['parityVerified'])
        self.assertFalse(result['eligibleForGate'])
