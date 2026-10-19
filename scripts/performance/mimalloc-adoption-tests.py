#!/usr/bin/env python3
"""Adoption must not pass on absent cells or four noisy observations."""
import json
from pathlib import Path
import runpy
import unittest

HERE = Path(__file__).parent
FIXTURES = HERE / 'fixtures/mimalloc-adoption'
ADOPTION = runpy.run_path(str(HERE / 'mimalloc-adoption.py'))
RULE = runpy.run_path(str(HERE / 'mimalloc-adoption-rule.py'))


def load(name):
    return json.loads((FIXTURES / name).read_text())


class AdoptionTests(unittest.TestCase):
    def test_empty_or_hosted_only_evidence_cannot_pass(self):
        for name in ('empty.json', 'hosted-only.json'):
            with self.subTest(name=name):
                result = ADOPTION['evaluate'](**load(name))
                self.assertFalse(result['eligibleForMaintainerReview'])
                self.assertFalse(result['completeRequiredScopes'])
                self.assertFalse(result['adopt'])

    def test_four_pairs_are_exploratory_even_when_all_ratios_improve(self):
        result = RULE['evaluate_cell']([load('cell-four.json')], False)
        self.assertFalse(result['eligible'])
        self.assertFalse(result['sufficientReplication'])
        self.assertIsNone(result['upper95MedianRatios']['wall_seconds'])

    def test_twelve_distinct_pairs_can_support_maintainer_review(self):
        result = RULE['evaluate_cell'](load('cell-twelve.json'), False)
        self.assertTrue(result['eligible'])
        self.assertEqual(result['upper95MedianRatios']['wall_seconds'], .98)
        result = ADOPTION['evaluate'](**load('complete.json'))
        self.assertTrue(result['completeRequiredScopes'])
        self.assertTrue(result['eligibleForMaintainerReview'])
        self.assertEqual(len(result['cells']), 88)
        self.assertFalse(result['adopt'])

    def test_order_statistic_bound_is_conservative(self):
        for case in load('median-bounds.json'):
            self.assertEqual(RULE['upper_median_bound'](case['values']), case['expected'])

    def test_observed_regressions_uncertainty_and_invalid_evidence_do_not_pass(self):
        cases = load('cell-errors.json')
        for name in ('pair-wall-regression', 'rss-regression', 'uncertain-wall'):
            with self.subTest(name=name):
                self.assertFalse(RULE['evaluate_cell'](cases[name], False)['eligible'])
        for name in ('missing-metric', 'invalid-pair'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                RULE['evaluate_cell'](cases[name], False)

    def test_workload_drift_cannot_be_pooled(self):
        with self.assertRaisesRegex(ValueError, 'workload identity'):
            ADOPTION['evaluate'](**load('wrong-workload.json'))

    def test_renaming_a_saved_summary_does_not_add_replication(self):
        with self.assertRaisesRegex(ValueError, 'duplicated'):
            RULE['evaluate_cell'](load('cell-duplicate.json'), False)


if __name__ == '__main__':
    unittest.main()
