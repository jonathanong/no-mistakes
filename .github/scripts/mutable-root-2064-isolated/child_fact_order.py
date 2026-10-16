"""Only two observed unordered fact maps use the immutable full-token proof."""
import hashlib
import json
from pathlib import Path

D = Path(__file__).resolve().parent
HELPER = '1d6f6623f5970025983d8094208d58ecced308a841a5e5f8fe7613e4ea83bdaf'
FACT_LABELS = {10: 'facts-mutable-pattern-deferred-diamond-values',
               26: 'extra-mutable-projection-wide.ts-values'}
assert hashlib.sha256((D / 'debug_digest.py').read_bytes()).hexdigest() == HELPER
import debug_digest


def compare(index, label, data, expected):
    actual = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
    assert actual['bytes'] == expected['bytes'], 'Full raw byte length changed'
    if actual['sha256'] == expected['sha256']:
        return {'comparison': 'EXACT raw', **actual}
    assert FACT_LABELS.get(index) == label, 'Undeclared raw difference'
    assert hashlib.sha256((D / 'debug_digest.py').read_bytes()).hexdigest() == HELPER
    proof = json.loads((D / 'parent-child-two-fact-order-proof.json').read_text())
    assert proof['helper_sha256'] == HELPER and proof['all_equal'] is True
    rows = [r for r in proof['rows'] if r['index'] == index]
    assert len(rows) == 1
    row = rows[0]
    assert row['equal'] is True and row['parent'] == row['child']
    assert expected['sha256'] in [row['parent_raw_sha256'], row['child_raw_sha256']]
    canonical = debug_digest.digest_debug(data)
    assert canonical == row['parent'], 'Full tokens or any of five statistics changed'
    return {'comparison': 'Pinned full tokens; only anonymous map/set entry order', **actual,
            'helper_sha256': HELPER, 'canonical': canonical}


def compare_file(index, label, path, expected):
    actual = {'bytes': path.stat().st_size}
    with path.open('rb') as stream:
        actual['sha256'] = hashlib.file_digest(stream, 'sha256').hexdigest()
    assert actual['bytes'] == expected['bytes'], 'Full raw byte length changed'
    if actual['sha256'] == expected['sha256']:
        return {'comparison': 'EXACT raw', **actual}
    assert FACT_LABELS.get(index) == label, 'Undeclared raw difference'
    return compare(index, label, path.read_bytes(), expected)
