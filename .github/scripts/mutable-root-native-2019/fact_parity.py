"""Raw-first full-fact comparison, bound to immutable historical snapshots."""
import hashlib
import importlib.util
import json
from pathlib import Path

PINS = {
    'helper': '1d6f6623f5970025983d8094208d58ecced308a841a5e5f8fe7613e4ea83bdaf',
    'sidecar': 'ae61235124e7b4de2c6c2f549eaca63a8b10eb69e1d54ce5acc206bb8d69e6b6',
    'prior': 'f01ae54b3457849fa10226dfde3b1c52b1f6fd6195902fb490aac1bbcea9a6ff',
}
ARMS = {'ARM_BASE': 'A22', 'ARM_CAND': 'B23'}

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

class FullFactParity:
    def __init__(self, directory, prior_manifest):
        directory = Path(directory)
        helper = directory/'debug_digest.py'
        sidecar = directory/'payload/expected-prior/full-fact-canonical-sidecar.json'
        raw_helper, raw_sidecar, raw_prior = helper.read_bytes(), sidecar.read_bytes(), Path(prior_manifest).read_bytes()
        assert sha(raw_helper) == PINS['helper'], 'canonical helper changed'
        assert sha(raw_sidecar) == PINS['sidecar'], 'historical sidecar changed'
        assert sha(raw_prior) == PINS['prior'], 'historical manifest changed'
        self.sidecar = json.loads(raw_sidecar)
        self.prior = json.loads(raw_prior)
        assert self.sidecar['algorithm_sha256'] == PINS['helper']
        assert self.sidecar['prior_manifest']['sha256'] == PINS['prior']
        self.labels = set(self.sidecar['outputs'])
        assert len(self.labels) == 13
        assert self.labels == {name for name in self.prior['outputs'] if name.startswith('facts-')}
        for label in self.labels:
            assert set(self.sidecar['outputs'][label]) == {'A22', 'B23'}
            for arm in ARMS.values():
                row = self.sidecar['outputs'][label][arm]
                assert row['raw'] == self.prior['outputs'][label][arm], 'sidecar raw provenance differs'
                assert set(row['canonical']['stats']) == {'atoms', 'groups', 'unordered_groups', 'entries', 'max_depth'}
        spec = importlib.util.spec_from_file_location('pinned_full_fact_debug_digest', helper)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.digest = module.digest_debug

    def compare(self, label, arm, data, raw_record):
        assert label in self.labels, 'undeclared full-fact label'
        assert arm in ARMS, 'undeclared full-fact arm'
        actual = {'sha256': sha(data), 'bytes': len(data)}
        assert raw_record == actual, 'actual full-fact raw metadata differs'
        expected = self.sidecar['outputs'][label][ARMS[arm]]
        result = {'actual_raw': actual, 'expected_raw': expected['raw'], 'raw_equal': actual == expected['raw'],
                  'helper_sha256': PINS['helper'], 'sidecar_sha256': PINS['sidecar'], 'prior_manifest_sha256': PINS['prior']}
        if result['raw_equal']:
            result['comparison'] = 'exact raw SHA and byte length'
            return result
        canonical = self.digest(data)
        assert canonical == expected['canonical'], 'full-fact tokens or counts changed'
        result.update(comparison='full tokens; only anonymous map/set entry order ignored', canonical=canonical)
        return result
