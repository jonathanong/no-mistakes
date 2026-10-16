"""Saved actual failed-run Cargo records distinguish host and production serde."""
import copy,json,unittest
from pathlib import Path
from common import HERE
from dependency_producer import select_release_rlib

PACKAGE = 'registry+https://github.com/rust-lang/crates.io-index#serde_json@1.0.151'
class DependencyProducerTests(unittest.TestCase):
    def records(self, arm='baseline'):
        return json.loads((HERE / 'payload/selector-fixtures' / (arm + '-host-and-production.json')).read_text())
    def select(self, rows, arm='baseline'):
        return select_release_rlib('\n'.join(json.dumps(row) for row in rows), 'serde_json', Path('/tmp/nm2019-arm2019') / arm, PACKAGE)
    def test_saved_host_and_production_select_exact_owned_release(self):
        for arm in ['baseline', 'candidate']:
            rows=self.records(arm);self.assertEqual(len(rows), 2)
            row,path=self.select(rows,arm)
            self.assertEqual(row,rows[1]);self.assertFalse(row['fresh'])
            self.assertEqual(path.name,'libserde_json-4220ee3c9bd65d8a.rlib')
            self.assertEqual(row['package_id'],PACKAGE)
    def test_missing_or_host_only_rejected(self):
        for rows in [[],self.records()[:1]]:
            with self.assertRaises(AssertionError):self.select(rows)
    def test_duplicate_release_rejected(self):
        rows=self.records()
        with self.assertRaises(AssertionError):self.select(rows+[copy.deepcopy(rows[1])])
    def test_wrong_profile_rejected(self):
        for key,value in [('opt_level','0'),('debuginfo',2),('debug_assertions',True),('test',True),('overflow_checks',True)]:
            rows=self.records();rows[1]['profile'][key]=value
            with self.assertRaises(AssertionError):self.select(rows)
    def test_foreign_arm_or_parent_path_rejected(self):
        for path in ['/tmp/nm2019-arm2019/candidate/target/release/deps/libserde_json-other.rlib','/outside/libserde_json-other.rlib','/tmp/nm2019-arm2019/baseline/target/release/deps/../../foreign.rlib']:
            rows=self.records();rows[1]['filenames']=[path]
            with self.assertRaises(AssertionError):self.select(rows)
    def test_wrong_package_or_kind_rejected(self):
        for key,value in [('package_id',PACKAGE+'-wrong'),('target',dict(self.records()[1]['target'],kind=['custom-build']))]:
            rows=self.records();rows[1][key]=value
            with self.assertRaises(AssertionError):self.select(rows)
    def test_missing_or_duplicate_rlib_rejected(self):
        for paths in [['only.rmeta'],['/tmp/nm2019-arm2019/baseline/target/release/deps/a.rlib','/tmp/nm2019-arm2019/baseline/target/release/deps/b.rlib']]:
            rows=self.records();rows[1]['filenames']=paths
            with self.assertRaises(AssertionError):self.select(rows)
if __name__ == '__main__':unittest.main()
