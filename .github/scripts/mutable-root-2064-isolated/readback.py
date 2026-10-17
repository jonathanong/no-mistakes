"""Independent full raw/gzip re-read at fan-in; never trust a PASS label alone."""
import gzip
import hashlib
import json
from pathlib import Path
from dependency_guard import verify as dependency_guard
from child_fact_order import compare

D = Path(__file__).resolve().parent


def raw_identity(path):
    if path.exists():
        stream = path.open('rb')
    else:
        stream = gzip.open(path.with_suffix(path.suffix + '.gz'), 'rb')
    digest, size = hashlib.sha256(), 0
    with stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
            size += len(chunk)
    return {'bytes': size, 'sha256': digest.hexdigest()}


def verify_stream(path, index, row):
    actual = raw_identity(path)
    expected = {k: row[k] for k in ['bytes', 'sha256']}
    if actual == expected:
        return {'comparison': 'EXACT complete raw', **actual}
    # Same two declared full-fact controls as the pre-sampling verifier.
    # Every token/statistic remains checked; no report field is omitted.
    if path.exists():
        data = path.read_bytes()
    else:
        with gzip.open(path.with_suffix(path.suffix + '.gz'), 'rb') as stream:
            data = stream.read()
    return compare(index, row['label'], data, expected)


def verify_api(api, counts):
    expected = {(arm, name) for arm in ['baseline', 'candidate'] for name in counts
                if not (arm == 'baseline' and name == 'assignment-receiver-api.test.js')}
    assert api['complete'] is True
    assert len(api['records']) == len(expected)
    assert {(r['arm'], r['script']) for r in api['records']} == expected
    for row in api['records']:
        count = counts[row['script']]
        assert row['counts'] == {'tests': count, 'pass': count, 'fail': 0, 'skipped': 0}


def verify(root):
    spec = json.loads((D / 'hosted-spec.json').read_text())
    dependency_guard(json.loads((root / 'JS-dependency-setup/closure.json').read_text()), spec['candidate_head'], spec['candidate_lock_sha256'])
    controls = json.loads((D / 'controls.json').read_text())
    observations = []
    for arm in ['A26', 'B26']:
        for index, row in enumerate(controls[arm]):
            path = root / 'full-controls' / f'{arm}-{index}.stdout'
            observations.append({'arm': arm, 'index': index,
                                 **verify_stream(path, index, row)})
            assert not path.with_suffix('.stderr').read_bytes()
        sql = root / 'full-controls' / (arm + '-SQL.stdout')
        assert raw_identity(sql) == {'bytes':19221,'sha256':'9e4528c15224dd7fb4396c3396b4021e43855f37012c8bf2cc836bde0f1ab3f8'}
    api = json.loads((root / 'api-controls.json').read_text())
    counts = json.loads((D / 'api-counts.json').read_text())
    verify_api(api, counts)
    return {'full_raw_controls': observations, 'API_records': len(api['records'])}
