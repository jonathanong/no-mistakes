"""Unmodified facade checker against a saved mock Git catalog lacking c896."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

D = Path(__file__).resolve().parent
# In the final package this is the unchanged sibling protocol directory.
PROTOCOL = D / 'protocol'
if not PROTOCOL.exists():
    PROTOCOL = Path('/tmp/nm2022-isolated-five-cohort-source-draft/protocol')
sys.path.insert(0, str(PROTOCOL))
import napi_parity
from facade_lookup import BASELINE_PRODUCER, PUBLIC_BASELINE, with_public_baseline


class FacadeLookup(unittest.TestCase):
    def setup_case(self):
        fixture = D / 'fixtures/public-facade-object'
        objects = json.loads((fixture / 'objects.json').read_text())['objects']
        proof = json.loads((D / 'baseline-public-facade-carry.json').read_text())
        spec = {'baseline': {'revision': BASELINE_PRODUCER}, 'candidate': {'revision': '4c24e9718081e36afd061389fadc465913b223d7'}, 'semantic_policy': {'napi_facade_sha256': {n: hashlib.sha256((fixture / 'packages/no-mistakes' / n).read_bytes()).hexdigest() for n in ['index.js', 'index.mjs']}}}
        seen = []
        def show(argv, cwd):
            assert argv[:2] == ['git', 'show']
            rev, path = argv[2].split(':', 1)
            seen.append(rev)
            assert rev in objects, 'Unpublished baseline Git object requested'
            return (fixture / path).read_bytes()
        return fixture, proof, spec, seen, show

    def test_unpublished_object_not_needed_and_producer_restored(self):
        fixture, proof, spec, seen, show = self.setup_case()
        with patch.object(napi_parity, 'ROOT', fixture), patch.object(napi_parity, 'SPEC', spec), patch.object(napi_parity.subprocess, 'check_output', show):
            with_public_baseline(napi_parity.facade_identity, spec, proof)
        self.assertEqual(seen.count(PUBLIC_BASELINE), 2)
        self.assertNotIn(BASELINE_PRODUCER, seen)
        self.assertEqual(spec['baseline']['revision'], BASELINE_PRODUCER)

    def test_wrong_public_blob_rejected_and_producer_restored(self):
        fixture, proof, spec, seen, show = self.setup_case()
        def wrong(argv, cwd):
            return (fixture / 'wrong-facade.txt').read_bytes()
        with patch.object(napi_parity, 'ROOT', fixture), patch.object(napi_parity, 'SPEC', spec), patch.object(napi_parity.subprocess, 'check_output', wrong), self.assertRaises(Exception):
            with_public_baseline(napi_parity.facade_identity, spec, proof)
        self.assertEqual(spec['baseline']['revision'], BASELINE_PRODUCER)

    def test_missing_tree_equivalence_proof_rejected(self):
        fixture, proof, spec, seen, show = self.setup_case()
        proof['baseline_prior_tree_equal'] = False
        with self.assertRaises(AssertionError):
            with_public_baseline(lambda: None, spec, proof)
