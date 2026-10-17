"""Authenticate the fixed download-only producer before restoring executable modes."""
import hashlib
import json
from pathlib import Path


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def verify(stage, assets, context):
    proof = json.loads((stage / 'transport.json').read_text())
    assert proof['complete'] is True and proof['producer'] == 'native-download'
    for key, env in [('repository', 'GITHUB_REPOSITORY'), ('run_id', 'GITHUB_RUN_ID'),
                     ('run_attempt', 'GITHUB_RUN_ATTEMPT'), ('workflow_sha', 'GITHUB_SHA')]:
        assert proof[key] == context[env]
    assert len(assets) == len(proof['assets']) == 8
    assert len({r['name'] for r in assets}) == 8
    assert len({r['name'] for r in proof['assets']}) == 8
    for expected in assets:
        rows = [r for r in proof['assets'] if r['name'] == expected['name']]
        assert len(rows) == 1
        row = rows[0]
        path = stage / expected['name']
        assert path.parent == stage and not path.is_symlink() and path.is_file()
        assert row['asset_id'] == expected['asset_id'] and row['exit'] == 0
        assert row['bytes'] == path.stat().st_size == expected['bytes']
        assert row['sha256'] == sha(path) == expected['sha256']
    # No mode change until the complete closed inventory is authenticated.
    for expected in assets:
        (stage / expected['name']).chmod(0o700)
    return proof
