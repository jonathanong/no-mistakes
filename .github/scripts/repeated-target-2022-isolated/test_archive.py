"""Saved synthetic archive: all 704 rows and 910 raw semantic file hashes."""
import copy
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from union import verify

D = Path(__file__).resolve().parent


class Archive(unittest.TestCase):
    def test_complete_saved_archive_reparses_all_rows(self):
        plan = json.loads((D / 'plan.json').read_text())
        result = verify([D / 'fixtures/synthetic-archive' / str(i) for i in range(5)], plan)
        self.assertEqual(result['rows'], 704)
        self.assertEqual(len(result['workloads']), 16)
        self.assertEqual(sum(len(w['metrics']) for w in result['workloads']), 48)
        self.assertFalse(result['cross_vm_pooling'])

    def test_duplicate_cohort_rejected(self):
        plan = json.loads((D / 'plan.json').read_text())
        roots = [D / 'fixtures/synthetic-archive' / str(i) for i in range(5)]
        roots[4] = roots[0]
        with self.assertRaises(AssertionError):
            verify(roots, plan)

    def test_missing_raw_or_row_not_masked(self):
        plan = json.loads((D / 'plan.json').read_text())
        roots = [D / 'fixtures/synthetic-archive' / str(i) for i in range(5)]
        original = Path.read_bytes
        def changed(path):
            raw = original(path)
            if path.name == 'SYNTHETIC-0.stdout':
                return raw + b'changed'
            return raw
        with patch.object(Path, 'read_bytes', changed), self.assertRaises(AssertionError):
            verify(roots, plan)
        read = Path.read_bytes
        def missing(path, *args, **kwargs):
            text = read(path, *args, **kwargs)
            if path.name == 'shared-imports-long-rows.json':
                return json.dumps(json.loads(text)[:-1]).encode()
            return text
        with patch.object(Path, 'read_bytes', missing), self.assertRaises(AssertionError):
            verify(roots, plan)
