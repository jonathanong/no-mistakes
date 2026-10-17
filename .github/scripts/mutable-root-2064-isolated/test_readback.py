"""Actual saved control labels and locally proven current API matrix."""
import copy
import json
from pathlib import Path
import sys
import unittest

D = Path(__file__).resolve().parent
sys.path.insert(0, str(D))
import readback


class CurrentReader(unittest.TestCase):
    def test_actual_twenty_five_API_records(self):
        api = json.loads((D / 'fixtures/current-API-matrix.json').read_text())
        counts = json.loads((D / 'api-counts.json').read_text())
        self.assertEqual(len(api['records']), 25)
        readback.verify_api(api, counts)
        for broken in [api['records'][:-1], api['records'][:-1] + [api['records'][0]]]:
            changed = copy.deepcopy(api)
            changed['records'] = broken
            with self.assertRaises(AssertionError):
                readback.verify_api(changed, counts)
        changed = copy.deepcopy(api)
        changed['records'][0]['counts']['skipped'] = 1
        with self.assertRaises(AssertionError):
            readback.verify_api(changed, counts)

    def test_actual_two_ordered_fact_controls(self):
        controls = json.loads((D / 'controls.json').read_text())['A26']
        for index in [10, 26]:
            path = D / 'fixtures/fact-order' / (str(index) + '.stdout')
            result = readback.verify_stream(path, index, controls[index])
            self.assertIn('canonical', result)
            self.assertEqual(result['bytes'], controls[index]['bytes'])
            bad = dict(controls[index], label='graph')
            with self.assertRaises(AssertionError):
                readback.verify_stream(path, index, bad)
            with self.assertRaises(AssertionError):
                readback.verify_stream(path, 1, controls[index])
