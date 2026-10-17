"""Published release names must not replace authenticated runtime role names."""
import ast
import json
from pathlib import Path
import unittest

D = Path(__file__).resolve().parent


class AssetBindings(unittest.TestCase):
    def test_download_and_measurement_have_identical_closed_roles(self):
        assets = json.loads((D / 'assets.json').read_text())
        workflow = (D / 'workflow-source.yml').read_text()
        start = workflow.index('          assets = [')
        end = workflow.index('          ]', start) + len('          ]')
        assignment = ast.parse('\n'.join(line[10:] for line in workflow[start:end].splitlines()))
        tuples = ast.literal_eval(assignment.body[0].value)
        self.assertEqual(len(tuples), 8)
        self.assertEqual({row['name'] for row in assets}, {row[0] for row in tuples})
        for row in assets:
            self.assertEqual(row['name'], row['role'])
            self.assertIn((row['name'], row['asset_id'], row['bytes'], row['sha256']), tuples)
            expected = 'd7847d84fcfb22a84fd0a2d218aef8c8c1282eb3' if row['role'].startswith('baseline-') else '06fd7a1e7e741961417cd1ee5e6e0a2229f2f837'
            self.assertEqual(row['actual_producer'], expected)
