"""Explicit upload scope: evidence only, never Git stores or executable transfers."""
import hashlib
import json
from pathlib import Path


def inventory(root):
    rows = []
    for name in ['public', 'private', 'semantic-manifest.json', 'reviewed-expected-parity.json']:
        item = root / name
        paths = sorted(item.rglob('*')) if item.is_dir() else [item]
        for path in paths:
            assert not path.is_symlink()
            if path.is_file():
                with path.open('rb') as stream:
                    digest = hashlib.file_digest(stream, 'sha256').hexdigest()
                rows.append({'path': str(path.relative_to(root)), 'bytes': path.stat().st_size, 'sha256': digest})
    return rows


def verify(root, rows):
    assert len({r['path'] for r in rows}) == len(rows)
    for row in rows:
        path = root / row['path']
        assert not Path(row['path']).is_absolute() and '..' not in Path(row['path']).parts
        assert Path(row['path']).parts[0] in ['public', 'private', 'semantic-manifest.json', 'reviewed-expected-parity.json']
        assert path.is_file() and not path.is_symlink() and path.stat().st_size == row['bytes']
        with path.open('rb') as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest() == row['sha256']
