"""Saved actual child raw files against independently retained parent proofs."""
import json
from pathlib import Path
import unittest
from child_fact_order import compare

D = Path(__file__).resolve().parent


class ChildFactOrder(unittest.TestCase):
    def test_both_known_order_differences_keep_all_tokens(self):
        proof = json.loads((D / 'parent-child-two-fact-order-proof.json').read_text())
        controls = json.loads((D / 'controls.json').read_text())['A26']
        for row in proof['rows']:
            data = (D / 'fixtures/fact-order' / (str(row['index']) + '.stdout')).read_bytes()
            expected = {'bytes': len(data), 'sha256': row['parent_raw_sha256']}
            label = controls[row['index']]['label']
            actual = compare(row['index'], label, data, expected)
            self.assertEqual(actual['canonical'], row['parent'])
            # Same length, different scalar spelling must not be treated as order.
            changed = data.replace(b'None', b'Some', 1)
            self.assertNotEqual(changed, data)
            with self.assertRaises(AssertionError):
                compare(row['index'], label, changed, expected)
            with self.assertRaises(AssertionError):
                compare(row['index'], 'graph', data, expected)
            with self.assertRaises(AssertionError):
                compare(1, label, data, expected)
