"""Synthetic bytes only; no native execution or network in source tests."""
import hashlib
import copy
import json
import os
from pathlib import Path
import unittest
from unittest.mock import patch
import contracts
import ast
import textwrap

D = Path(__file__).resolve().parent


class Boundaries(unittest.TestCase):
    def test_privileged_fixed_assets_match_reviewed_roles(self):
        workflow = (D / 'workflow.yml').read_text()
        source = workflow.split("python3 - <<'PY'\n", 1)[1].split('\n          PY', 1)[0]
        tree = ast.parse(textwrap.dedent(source))
        values = [node.value for node in tree.body if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == 'assets' for target in node.targets)]
        self.assertEqual(len(values), 1)
        actual = ast.literal_eval(values[0])
        expected = [(row['name'], row['id'], row['size'], row['sha256']) for row in json.loads((D / 'assets.json').read_text())]
        self.assertEqual(actual, expected)
        self.assertIn('ref: ' + json.loads((D / 'plan.json').read_text())['candidate_actual_compiler'], workflow)

    def test_draft_never_admits_runtime(self):
        with self.assertRaises(AssertionError):
            contracts.approved({**json.loads((D / 'plan.json').read_text()), 'status': 'DRAFT'}, {})

    def test_exact_dispatch_identity(self):
        plan = {'status': 'REVIEWED_ISOLATED_EIGHT_COHORTS', 'execution_approval': {'source': 'SYNTHETIC'}}
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

    def test_explicit_sampler_revision_and_original_assessor_pinned(self):
        for name, expected in json.loads((D / 'immutable-source-pins.json').read_text()).items():
            self.assertEqual(hashlib.sha256((D / 'protocol' / name).read_bytes()).hexdigest(), expected)

    def test_historical_24_layout_proof_not_invented(self):
        plan = json.loads((D / 'fixtures/historical-five-plan.json').read_text())
        # Historical byte pins remain authentic; source units must be portable.
        for key in ['proof', 'root_review']:
            plan['layout_gate'][key]['path'] = str(D / 'fixtures' / ('historical-layout-' + key + '.json'))
        self.assertEqual(contracts.layouts(plan), 24)

    def test_no_write_token_in_measurement_job(self):
        workflow = (D / 'workflow.yml').read_text()
        bootstrap, measurement = workflow.split('\n  measure:\n')
        self.assertIn('contents: write', bootstrap)
        self.assertNotIn('actions/checkout', bootstrap)
        self.assertNotIn('contents: write', measurement)
        self.assertIn('fail-fast: false', measurement)
        self.assertIn('cohort: [0, 1, 2, 3, 4, 5, 6, 7]', measurement)

    def test_sample_budget_and_frozen_long_target(self):
        cohort = (D / 'cohort.py').read_text()
        measure = (D / 'protocol/measure.py').read_text()
        self.assertIn('started + 600', cohort)
        self.assertIn('deadline - 65', measure)
        self.assertIn('math.ceil(6 / minimum', measure)
        self.assertIn("'long', count, 4", measure)

    def test_upload_never_contains_native_tree(self):
        workflow = (D / 'workflow.yml').read_text()
        paths = workflow.split('name: Retain authenticated portable evidence carrier')[1]
        self.assertNotIn('/transfer/', paths)
        self.assertNotIn('candidate-source', paths)
        self.assertNotIn('.git', paths)
