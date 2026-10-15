"""Verify the public package without rebuilding or sampling."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
mapping = json.loads((root / 'public-copy-mapping.json').read_text())
for row in mapping['files']:
    data = (root / row['path']).read_bytes()
    assert len(data) == row['public_bytes'], row['path']
    assert hashlib.sha256(data).hexdigest() == row['public_sha256'], row['path']
    if row['byte_exact']:
        assert row['public_sha256'] == row['original_sha256'], row['path']
counts = {
    'empty-import-guard/pilot-rows.json': 64,
    'empty-import-guard/long-rows.json': 256,
    'empty-import-guard/clean-repeat-rows.json': 256,
    'inline-scope/fixed-rows.json': 256,
    'final-three-arm/A_B-pilot-new-rows.json': 112,
    'final-three-arm/A_B-long-controls-rows.json': 256,
    'final-three-arm/A_B-long-hot-facts-rows.json': 160,
    'final-three-arm/A_B-long-graph-effects-rows.json': 288,
    'final-three-arm/B_C-long-controls-rows.json': 256,
}
for name, expected in counts.items():
    rows = json.loads((root / name).read_text())
    assert len(rows) == expected, name
    assert all(row['exit'] == 0 for row in rows), name
    assert all(abs(row['cpu_s'] - row['user_s'] - row['system_s']) < 1e-9 for row in rows), name
print(f'Verified {len(mapping["files"])} public copies and {sum(counts.values())} retained raw rows.')
