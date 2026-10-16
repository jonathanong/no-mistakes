"""Synthetic bytes only; no native execution or network in source tests."""
import hashlib
import copy
import json
import os
from pathlib import Path
import unittest
from unittest.mock import patch
import contracts

D = Path(__file__).resolve().parent


class Boundaries(unittest.TestCase):
    def test_draft_never_admits_runtime(self):
        with self.assertRaises(AssertionError):
            contracts.approved({**json.loads((D / 'plan.json').read_text()), 'status': 'DRAFT'}, {})

    def test_exact_dispatch_identity(self):
        plan = {'status': 'REVIEWED_ISOLATED_FIVE_COHORTS', 'execution_approval': {'source': 'SYNTHETIC'}}
        env = {'GITHUB_REPOSITORY': 'jonathanong/no-mistakes', 'APPROVED_SHA': 'SYNTHETIC', 'GITHUB_SHA': 'SYNTHETIC'}
        contracts.approved(plan, env)
        for key in env:
            wrong = {**env, key: 'wrong'}
            with self.assertRaises(AssertionError):
                contracts.approved(plan, wrong)

    def test_partition_exact_704(self):
        plan = json.loads((D / 'plan.json').read_text())
        self.assertEqual(sorted(i for c in plan['rounds'] for i in c['indices']), list(range(16)))
        self.assertEqual(sum(c['rows'] for c in plan['rounds']), 704)
        self.assertTrue(all(c['rows'] == len(c['indices']) * 44 for c in plan['rounds']))

    def test_original_assessor_and_sampler_unchanged(self):
        for name, expected in json.loads((D / 'immutable-source-pins.json').read_text()).items():
            self.assertEqual(hashlib.sha256((D / 'protocol' / name).read_bytes()).hexdigest(), expected)

    def test_current_24_layouts_not_invented(self):
        self.assertEqual(contracts.layouts(json.loads((D / 'plan.json').read_text())), 24)

    def test_no_write_token_in_measurement_job(self):
        workflow = (D / 'workflow.yml').read_text()
        bootstrap, measurement = workflow.split('\n  measure:\n')
        self.assertIn('contents: write', bootstrap)
        self.assertNotIn('actions/checkout', bootstrap)
        self.assertNotIn('contents: write', measurement)
        self.assertIn('fail-fast: false', measurement)
        self.assertIn('cohort: [0, 1, 2, 3, 4]', measurement)

    def test_sample_budget_and_frozen_long_target(self):
        cohort = (D / 'cohort.py').read_text()
        measure = (D / 'protocol/measure.py').read_text()
        self.assertIn('started + 600', cohort)
        self.assertIn('deadline - 65', measure)
        self.assertIn('math.ceil(5 / min', measure)
        self.assertIn("'long', count, 4", measure)

    def test_upload_never_contains_native_tree(self):
        workflow = (D / 'workflow.yml').read_text()
        paths = workflow.split('name: Retain all completed and partial evidence')[1]
        self.assertNotIn('/transfer/', paths)
        self.assertNotIn('candidate-source', paths)
        self.assertNotIn('.git', paths)
