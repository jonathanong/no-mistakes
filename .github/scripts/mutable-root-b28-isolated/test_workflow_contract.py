"""Actual fixed privileged tuples must match every reviewed runtime role."""
import ast
import json
from pathlib import Path
import re
import textwrap
import unittest

D = Path(__file__).resolve().parent


def fixed_assets(workflow):
    match = re.search(r'^          assets = \[\n.*?^          \]', workflow, re.M | re.S)
    assert match, 'fixed asset assignment absent'
    tree = ast.parse(textwrap.dedent(match.group()))
    return ast.literal_eval(tree.body[0].value)


def verify_contract(workflow, assets, spec):
    expected = [(a['name'], a['asset_id'], a['bytes'], a['sha256']) for a in assets]
    assert all(a['name'] == a['role'] for a in assets), 'runtime name differs from role'
    assert fixed_assets(workflow) == expected, 'fixed tuples differ from reviewed assets'
    refs = re.findall(r'^          ref: (\S+)$', workflow, re.M)
    assert refs == [spec['candidate_head']], 'candidate checkout differs'


class WorkflowContract(unittest.TestCase):
    def test_actual_workflow_and_negative_aliases(self):
        workflow = (D.parents[1] / 'workflows/receiver-fact-native.yml').read_text()
        assets = json.loads((D / 'assets.json').read_text())
        spec = json.loads((D / 'hosted-spec.json').read_text())
        verify_contract(workflow, assets, spec)
        old = json.loads(json.dumps(assets))
        old[0]['name'] = old[0]['release_asset_name']
        with self.assertRaisesRegex(AssertionError, 'runtime name differs from role'):
            verify_contract(workflow, old, spec)
        with self.assertRaisesRegex(AssertionError, 'fixed tuples differ'):
            verify_contract(workflow.replace(str(assets[0]['asset_id']), '1', 1), assets, spec)
        with self.assertRaisesRegex(AssertionError, 'candidate checkout differs'):
            verify_contract(workflow.replace(spec['candidate_head'], 'wrong-head'), assets, spec)


if __name__ == '__main__':
    unittest.main()
