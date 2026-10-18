#!/usr/bin/env python3
"""Saved-fixture tests for the hosted measurement acceptance guard."""
import json
from pathlib import Path
import unittest

from summarize import nearest_rank, validate_cells, validate_pairs, validate_sample


class AcceptanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixture = Path(__file__).resolve().parents[2] / 'fixtures/performance/branch-state-gate/acceptance-cases.json'
        cls.cases = json.loads(fixture.read_text())

    def test_accepts_only_complete_exact_report_with_expected_status(self):
        validate_sample(self.cases['valid'], ('1', 'off'), 1)

    def test_rejects_timeout_and_report_drift(self):
        for name in ('censored', 'wrong_report'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                validate_sample(self.cases[name], ('1', 'off'), 1)

    def test_nearest_rank_p95_for_small_samples_is_the_observed_maximum(self):
        self.assertEqual(nearest_rank(self.cases['rank_values'], 95), 6)

    def test_rejects_missing_cell_or_pair_evidence(self):
        cells = {tuple(cell) for cell in self.cases['present_cells']}
        with self.assertRaises(ValueError):
            validate_cells(cells)
        pairs = [{'pair': number} for number in self.cases['missing_pairs']]
        with self.assertRaises(ValueError):
            validate_pairs(pairs)


if __name__ == '__main__':
    unittest.main()
