"""Saved semantic mutations and immutable historical provenance controls."""
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
import fact_parity
from debug_digest import digest_debug

HERE = Path(__file__).parent
FIXTURES = HERE/'test-fixtures'
class FullFactParityTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        for name in ['debug_digest.py', 'payload/expected-prior/full-fact-canonical-sidecar.json',
                     'payload/expected-prior/semantic-manifest.json']:
            path = self.root/name;path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(HERE/name, path)
        self.prior = self.root/'payload/expected-prior/semantic-manifest.json'
        self.raw = (FIXTURES/'full-fact-jsx-values.stdout').read_bytes()
    def tearDown(self):
        self.tmp.cleanup()
    def load(self):
        return fact_parity.FullFactParity(self.root, self.prior)
    def record(self, raw):
        return {'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}
    def test_raw_first_and_complete_token_fallback(self):
        comparator = self.load()
        comparator.digest = lambda _: self.fail('raw-equal fast path must not parse again')
        exact = comparator.compare('facts-jsx-values', 'ARM_BASE', self.raw, self.record(self.raw))
        self.assertTrue(exact['raw_equal'])
        comparator = self.load();formatted = self.raw+b' \n'
        result = comparator.compare('facts-jsx-values', 'ARM_CAND', formatted, self.record(formatted))
        self.assertFalse(result['raw_equal'])
        self.assertEqual(result['canonical'], comparator.sidecar['outputs']['facts-jsx-values']['B23']['canonical'])
    def test_immutable_helper_sidecar_and_prior_pins(self):
        for name in ['debug_digest.py', 'payload/expected-prior/full-fact-canonical-sidecar.json',
                     'payload/expected-prior/semantic-manifest.json']:
            with self.subTest(name=name):
                path=self.root/name;saved=path.read_bytes();path.write_bytes(saved+b' ')
                with self.assertRaises(AssertionError):self.load()
                path.write_bytes(saved)
    def test_undeclared_label_arm_and_raw_metadata_failclosed(self):
        comparator=self.load()
        for label,arm in [('graph-off','ARM_BASE'),('facts-undeclared','ARM_BASE'),('facts-jsx-values','A22'),('facts-jsx-values','OTHER')]:
            with self.subTest(label=label,arm=arm),self.assertRaises(AssertionError):
                comparator.compare(label,arm,self.raw,self.record(self.raw))
        with self.assertRaises(AssertionError):
            comparator.compare('facts-jsx-values','ARM_BASE',self.raw,{'sha256':'0'*64,'bytes':len(self.raw)})
    def test_internal_sidecar_provenance_and_inventory(self):
        path=self.root/'payload/expected-prior/full-fact-canonical-sidecar.json';saved=path.read_bytes()
        for mutation in ['raw','arm','label','stats','algorithm','manifest']:
            data=json.loads(saved)
            if mutation=='raw':data['outputs']['facts-jsx-values']['A22']['raw']['bytes']+=1
            elif mutation=='arm':data['outputs']['facts-jsx-values']['OTHER']=data['outputs']['facts-jsx-values']['A22']
            elif mutation=='label':data['outputs']['graph-off']=data['outputs'].pop('facts-jsx-values')
            elif mutation=='stats':del data['outputs']['facts-jsx-values']['A22']['canonical']['stats']['atoms']
            elif mutation=='algorithm':data['algorithm_sha256']='0'*64
            else:data['prior_manifest']['sha256']='0'*64
            raw=json.dumps(data).encode();path.write_bytes(raw)
            # Test deeper provenance checks even after a trusted pin transition.
            with self.subTest(mutation=mutation),patch.dict(fact_parity.PINS,sidecar=hashlib.sha256(raw).hexdigest()),self.assertRaises(AssertionError):self.load()
        path.write_bytes(saved)
    def test_saved_semantic_order_field_and_value_controls(self):
        cases=json.loads((FIXTURES/'full-fact-debug-cases.json').read_text())
        for row in cases['pairs']:
            with self.subTest(name=row['name']):
                a=(FIXTURES/'full-fact-debug'/row['before']).read_bytes()
                b=(FIXTURES/'full-fact-debug'/row['after']).read_bytes()
                first,second=digest_debug(a),digest_debug(b)
                self.assertEqual(first['sha256']==second['sha256'],row['equal'])
                if row['equal']:self.assertEqual(first['stats'],second['stats'])
        for name in cases['invalid']:
            with self.subTest(name=name),self.assertRaises(ValueError):
                digest_debug((FIXTURES/'full-fact-debug'/name).read_bytes())
    def test_actual_changed_field_is_rejected_by_fallback(self):
        changed=self.raw.replace(b'fatal_parse_error: false',b'fatal_parse_error: true',1)
        self.assertNotEqual(changed,self.raw)
        with self.assertRaises(AssertionError):
            self.load().compare('facts-jsx-values','ARM_BASE',changed,self.record(changed))

if __name__=='__main__':unittest.main()
