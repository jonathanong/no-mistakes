"""Lossless raw carriers; retain original hashes and verify every gzip roundtrip."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil
from transport import sha


def compress(root):
    rows = []
    for path in sorted(Path(root).rglob('*.stdout')):
        if path.stat().st_size < 1000000:
            continue
        raw_sha, size = sha(path), path.stat().st_size
        zipped = path.with_suffix(path.suffix + '.gz')
        assert not zipped.exists()
        with path.open('rb') as source, gzip.open(zipped, 'wb', compresslevel=1) as target:
            shutil.copyfileobj(source, target)
        with gzip.open(zipped, 'rb') as restored:
            digest = hashlib.file_digest(restored, 'sha256').hexdigest()
        assert digest == raw_sha
        rows.append({'raw_path': str(path.relative_to(root)), 'raw_sha256': raw_sha, 'raw_bytes': size,
                     'gzip_path': str(zipped.relative_to(root)), 'gzip_sha256': sha(zipped), 'gzip_bytes': zipped.stat().st_size})
        path.unlink()
    (Path(root) / 'lossless-raw-carriers.json').write_text(json.dumps(rows, indent=2))
