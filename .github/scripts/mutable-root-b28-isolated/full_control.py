"""Raw-first verification; only two source-reviewed fact controls permit map order."""
import gzip
import hashlib
import json
from pathlib import Path

D = Path(__file__).resolve().parent
LABELS = {10: 'facts-mutable-pattern-deferred-diamond-values',
          26: 'extra-mutable-projection-wide.ts-values'}
HELPER_SHA = '1d6f6623f5970025983d8094208d58ecced308a841a5e5f8fe7613e4ea83bdaf'


def verify_tokens(raw, arm, index, label, goldens):
    assert arm in ['A28', 'B28'], 'undeclared arm'
    assert index in LABELS and label == LABELS[index], 'undeclared unordered control'
    golden = goldens[arm][str(index)]
    assert golden['label'] == label
    assert hashlib.sha256((D / 'debug_digest.py').read_bytes()).hexdigest() == HELPER_SHA
    from debug_digest import digest_debug
    assert digest_debug(raw) == golden['tokens'], 'full tokens or statistics changed'


def verify_raw(path, arm, index, row):
    if path.exists():
        raw = path.read_bytes()
    else:
        raw = gzip.decompress(path.with_suffix(path.suffix + '.gz').read_bytes())
    identity = {'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
    if identity == {k: row[k] for k in ['bytes', 'sha256']}:
        return {'raw_identity': identity, 'comparison': 'raw exact'}
    assert len(raw) == row['bytes'], 'raw byte length changed'
    goldens = json.loads((D / 'unordered-control-goldens.json').read_text())
    verify_tokens(raw, arm, index, row['label'], goldens)
    golden = goldens[arm][str(index)]
    assert golden['raw_sha256'] == row['sha256'] and golden['raw_bytes'] == row['bytes']
    return {'raw_identity': identity, 'comparison': 'complete tokens and five statistics exact'}
