"""Saved pilot capacity and unchanged rejection gates; no native sampling."""
import ast
import copy
import hashlib
import json
import math
from pathlib import Path
import sys
import unittest

D = Path(__file__).resolve().parent
sys.path.insert(0, str(D / 'protocol'))
from paired_summary import assess


def sampler_functions():
    module = ast.parse((D / 'protocol/measure.py').read_text())
    selected = [node for node in module.body if isinstance(node, ast.FunctionDef)
                and node.name in ('pilot_iterations', 'long_iterations')]
    assert len(selected) == 2
    scope = {'math': math}
    exec(compile(ast.Module(body=selected, type_ignores=[]), '<actual sampler calibration>', 'exec'), scope)
    return scope


class Capacity(unittest.TestCase):
    def test_saved_cap_limited_pilots_receive_duration_headroom(self):
        observed = json.loads((D / 'fixtures/capacity-pilot-observations.json').read_text())
        count = observed['observed_pilot_count']
        function = sampler_functions()['long_iterations']
        for minimum in observed['observed_minimum_cpu_seconds'].values():
            old = min(10000, max(count, math.ceil(5 / minimum * count)))
            self.assertEqual(old, 10000)
            actual = function(count, minimum, 100000)
            self.assertEqual(actual, math.ceil(6 / minimum * count))
            self.assertGreater(actual, 10000)
            self.assertLessEqual(actual, 100000)

    def test_pilot_formula_and_hard_cap_are_preserved(self):
        functions = sampler_functions()
        for calibration in (16, 128):
            for minimum in (0, 1e-9, .02, 2):
                for cap in (1, 10000, 100000):
                    self.assertEqual(functions['pilot_iterations'](calibration, minimum, cap),
                                     min(cap, max(1, math.ceil(2 * calibration / max(minimum, 1e-6)))))
        self.assertEqual(functions['long_iterations'](100000, .1, 100000), 100000)
        self.assertEqual(functions['long_iterations'](7, 100, 100000), 7)

    def test_unchanged_assessor_still_rejects_below_five_cpu(self):
        rows = json.loads((D / 'fixtures/synthetic-archive/0/public/shared-imports-long-rows.json').read_text())
        short = copy.deepcopy(rows)
        for row in short:
            row['cpu_s'] = 4.99
            row['user_s'] = 4.99 - row['system_s']
        result = assess(short)
        self.assertIn('inconclusive', result['status'])
        original = json.loads((D / 'immutable-source-pins.json').read_text())['paired_summary.py']
        self.assertEqual(original, 'f2670dc7120efd8d9b6aaaa12fd3acb22cf337e0c88873a328726ed5eaa4f755')
        self.assertEqual(hashlib.sha256((D / 'protocol/paired_summary.py').read_bytes()).hexdigest(), original)

    def test_all_original_indices_are_fixed_adjacent_pairs(self):
        plan = json.loads((D / 'plan.json').read_text())
        self.assertEqual([r['indices'] for r in plan['rounds']], [[i, i + 1] for i in range(0, 16, 2)])
        self.assertEqual(sum(r['rows'] for r in plan['rounds']), 704)
        self.assertEqual(json.loads((D / 'protocol/pair.json').read_text())['iterations_cap'], 100000)


class AssetRoles(unittest.TestCase):
    def test_authentic_producer_names_have_explicit_baseline_roles(self):
        from contracts import asset_role
        rows = json.loads((D / 'fixtures/explicit-baseline-asset-roles.json').read_text())
        self.assertEqual([asset_role(r) for r in rows],
                         [('baseline', name) for name in ('graph-probe', 'timing-probe', 'cli', 'addon.node')])
        self.assertNotIn('baseline', rows[2]['name'])

    def test_invalid_or_missing_role_is_rejected(self):
        from contracts import asset_role
        original = json.loads((D / 'fixtures/explicit-baseline-asset-roles.json').read_text())[0]
        for change in ({'arm': 'other'}, {'file': '../cli'}, {'name': '../cli'}):
            with self.assertRaises(AssertionError):
                asset_role({**original, **change})
        missing = {k: v for k, v in original.items() if k != 'arm'}
        with self.assertRaises(KeyError):
            asset_role(missing)

    def test_all_eight_roles_unique_and_missing_ids_require_draft(self):
        from contracts import asset_role
        rows = json.loads((D / 'assets.json').read_text())
        self.assertEqual(len(set(asset_role(r) for r in rows)), 8)
        missing = [r for r in rows if r['id'] is None]
        if missing:
            self.assertTrue(json.loads((D / 'plan.json').read_text())['status'].startswith('DRAFT'))
        else:
            self.assertTrue(all(type(r['id']) is int and r['id'] > 0 for r in rows))
