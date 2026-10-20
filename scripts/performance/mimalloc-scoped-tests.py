#!/usr/bin/env python3
"""Saved inputs guard scope identity, annotation completion and adoption exclusion."""
import copy
import json
from pathlib import Path
import runpy
import unittest

HERE = Path(__file__).resolve().parent
CONTRACT = runpy.run_path(str(HERE / 'mimalloc-scoped-contract.py'))
REPORT = runpy.run_path(str(HERE / 'mimalloc-scoped-report.py'))
ORIGIN = runpy.run_path(str(HERE / 'mimalloc-scoped-origin.py'))
SAVED = HERE / 'fixtures/mimalloc-scoped'


class ScopedTests(unittest.TestCase):
    def test_saved_annotation_sources_are_exact(self):
        root = CONTRACT['SAVED'] / 'annotation'
        proof = json.loads((root / 'contract.json').read_text())
        for name, expected in proof['files'].items():
            self.assertEqual(CONTRACT['digest'](root / name), expected, name)
        self.assertEqual(proof['expectedFindings'], 32)
        self.assertEqual(len(list((root / 'src').glob('entry-*.mts'))), 32)

    def test_completed_annotation_and_censored_control(self):
        row = {'exit': 1, 'timed_out': False, 'signal': None,
               'metadata': {'workload': 'annotation-workers', 'diagnostics': True}}
        CONTRACT['validate_report'](row, SAVED / 'annotation.stdout', SAVED / 'annotation.stderr')
        row['timed_out'] = True
        with self.assertRaises(AssertionError):
            CONTRACT['validate_report'](row, SAVED / 'annotation.stdout', SAVED / 'annotation.stderr')
        row['timed_out'] = False
        with self.assertRaises(AssertionError):
            CONTRACT['validate_report'](row, SAVED / 'annotation.stdout', HERE / 'fixtures/mimalloc-report/report-empty.stderr')

    def test_exact_archived_consumer_origin(self):
        row = json.loads((SAVED / 'origin.json').read_text())
        ORIGIN['validate'](row)
        for key, value in (('digest', 'changed'), ('expired', True), ('id', 1)):
            changed = dict(row, **{key: value})
            with self.subTest(key=key), self.assertRaises(AssertionError):
                ORIGIN['validate'](changed)

    def test_matrix_is_complete_same_host_and_scope_pinned(self):
        fixture = json.loads((SAVED / 'matrix.json').read_text())
        result = REPORT['summarize'](fixture['rows'], fixture['scopes'])
        self.assertTrue(result['complete'])
        self.assertFalse(result['adopt'])
        self.assertEqual(len(result['cells']), 10)
        for field in ('config_sha256', 'inventory_sha256', 'gitignore_sha256', 'scope'):
            changed = copy.deepcopy(fixture['rows'])
            changed[0]['metadata'][field] = 'changed'
            with self.subTest(field=field), self.assertRaises(AssertionError):
                REPORT['summarize'](changed, fixture['scopes'])
        with self.assertRaises(ValueError):
            REPORT['summarize'](fixture['rows'][:-1], fixture['scopes'])

    def test_scoped_rows_cannot_fill_any_adoption_matrix(self):
        rows = json.loads((SAVED / 'matrix.json').read_text())['rows']
        adoption = runpy.run_path(str(HERE / 'mimalloc-adoption.py'))
        fixture = json.loads((SAVED / 'matrix.json').read_text())
        summary = REPORT['summarize'](rows, fixture['scopes'])
        with self.assertRaises(ValueError):
            adoption['evaluate']({'hosted': [summary['cells']]})
        for matrix in ('hosted', 'napi', 'many-core'):
            with self.subTest(matrix=matrix), self.assertRaises(ValueError):
                REPORT['REPORT']['summarize'](rows, matrix, platform='linux-arm64')


if __name__ == '__main__':
    unittest.main()
