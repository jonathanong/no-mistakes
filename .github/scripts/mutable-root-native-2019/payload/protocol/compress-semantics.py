"""Lossless storage only, after full raw semantic comparisons have passed."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil

out = Path(__file__).resolve().parent
from protocol_guard import require_reviewed_pair
reviewed_plan=require_reviewed_pair(out, live=False)
assert (out / 'semantic-manifest.json').exists()
manifest = out / 'semantic-compression.json'
assert not manifest.exists(), 'Preserve earlier compression evidence'
rows = []

def identity(path, opener=open):
    result = hashlib.sha256()
    size = 0
    with opener(path, 'rb') as source:
        while chunk := source.read(1024 * 1024):
            result.update(chunk)
            size += len(chunk)
    return result.hexdigest(), size

for raw in sorted((out / 'semantic-private').glob('*.stdout')):
    if raw.stat().st_size < 1024 * 1024:
        continue
    packed = raw.with_suffix(raw.suffix + '.gz')
    assert not packed.exists()
    raw_hash, raw_bytes = identity(raw)
    with raw.open('rb') as source, packed.open('xb') as destination:
        with gzip.GzipFile(filename='', mode='wb', fileobj=destination, mtime=0) as compressed:
            shutil.copyfileobj(source, compressed, 1024 * 1024)
    assert identity(packed, gzip.open) == (raw_hash, raw_bytes)
    packed_hash, packed_bytes = identity(packed)
    rows.append(dict(path=str(raw.relative_to(out)), raw_sha256=raw_hash,
                     raw_bytes=raw_bytes, gzip_path=str(packed.relative_to(out)),
                     gzip_sha256=packed_hash, gzip_bytes=packed_bytes,
                     lossless_roundtrip_verified=True))
    manifest.write_text(json.dumps(rows, indent=2) + '\n')
    raw.unlink()
if not manifest.exists():
    manifest.write_text('[]\n')
print(json.dumps({'compressed_files': len(rows), 'raw_bytes': sum(row['raw_bytes'] for row in rows),
                  'gzip_bytes': sum(row['gzip_bytes'] for row in rows)}))
