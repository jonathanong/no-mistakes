"""Saved real pre-native failure: workflow checkout must bind genuine candidate."""
import json
from pathlib import Path
import re
import unittest

D = Path(__file__).resolve().parent
MARKER = '      - name: Checkout exact published candidate API source\n'


def verify_checkout(workflow, expected):
    assert workflow.count(MARKER) == 1, 'one candidate source checkout required'
    block = workflow.split(MARKER)[1].split('      - name:')[0]
    refs = re.findall(r'^          ref: ([0-9a-f]{40})\s*$', block, re.MULTILINE)
    assert refs == [expected], 'candidate checkout must equal spec/native source'
    assert '          path: candidate-source\n' in block
    assert '          persist-credentials: false\n' in block


class CheckoutBindingTests(unittest.TestCase):
    def test_workflow_matches_exact_frozen_candidate_artifacts(self):
        spec = json.loads((D / 'hosted-spec.json').read_text())
        assets = json.loads((D / 'assets.json').read_text())
        candidates = {a['actual_producer'] for a in assets if a['role'].startswith('candidate-')}
        self.assertEqual(candidates, {spec['candidate_head']})
        verify_checkout((D / 'workflow-source.yml').read_text(), spec['candidate_head'])

    def test_saved_actual_old_checkout_is_rejected(self):
        observed = json.loads((D / 'fixtures/checkout-ref-39004092627.json').read_text())
        self.assertEqual(observed['observed_source_head'], 'bea5a183c663410b61fed6e7619b6268103adbe6')
        self.assertNotEqual(observed['observed_source_head'], observed['required_candidate_head'])
        with self.assertRaisesRegex(AssertionError, 'candidate checkout must equal'):
            verify_checkout(observed['workflow_step_excerpt'], observed['required_candidate_head'])
