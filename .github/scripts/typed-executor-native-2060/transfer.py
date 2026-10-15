"""Digest and payload allowlist for one-run private ARM binary transfer.

The producer emits a manifest for its own files. The consumer verifies the
downloaded bytes, exact arm/source/profile metadata, and rejects extra files.
"""
import hashlib
import json
from pathlib import Path

ALLOWED = {'cli', 'timing-probe', 'graph-probe', 'addon.node',
           'cargo.jsonl', 'source-inventory.json'}


def digest(path):
    data = Path(path).read_bytes()
    return {'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}


def record(directory, arm, revision, tree, source_inventory_sha256, compiler, features, profile):
    directory = Path(directory)
    assert arm in ('baseline', 'candidate')
    entries = list(directory.iterdir())
    assert {p.name for p in entries} == ALLOWED, 'missing or extra private transfer entry'
    assert all(p.is_file() and not p.is_symlink() for p in entries), 'private transfer entry is not a regular file'
    files = {p.name: digest(p) for p in entries}
    manifest = dict(arm=arm, revision=revision, tree=tree,
                source_inventory_sha256=source_inventory_sha256,
                compiler=compiler, features=features, profile=profile, files=files)
    (directory / 'transfer-manifest.json').write_text(json.dumps(manifest, sort_keys=True) + '\n')
    return manifest


def verify(directory, manifest, expected):
    directory = Path(directory)
    assert set(manifest) == {'arm', 'revision', 'tree', 'source_inventory_sha256',
                             'compiler', 'features', 'profile', 'files'}
    for key, value in expected.items():
        assert manifest[key] == value, f'{key} differs from frozen pair'
    assert set(manifest['files']) == ALLOWED
    assert {p.name for p in directory.iterdir()} == ALLOWED | {'transfer-manifest.json'}
    assert all(p.is_file() and not p.is_symlink() for p in directory.iterdir())
    assert json.loads((directory / 'transfer-manifest.json').read_text()) == manifest
    for name in ALLOWED:
        assert digest(directory / name) == manifest['files'][name], f'{name} changed in transfer'
    return True
