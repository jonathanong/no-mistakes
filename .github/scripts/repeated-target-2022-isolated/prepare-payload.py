"""Copy only reviewed source and metadata; native files use fixed release IDs."""
from pathlib import Path
import hashlib
import json
import shutil

D = Path(__file__).resolve().parent
C = Path('/tmp/nm2022-main5de-integration/dev-semantic-source-closure.json')
rows = json.loads(C.read_text())['files']
paths = {row['path'] for row in rows}
paths.add(str(C))
paths.add('/tmp/nm2022-2084-release/collect-current-pair-semantics.py')
for arm, artifact in [('baseline', '/tmp/nm-shared-baseline-2087-c896-final-artifacts'), ('candidate', '/tmp/nm2022-2084-release/candidate')]:
    paths.add(artifact + '/closed-artifacts.json')
    paths.add('/tmp/nm2022-matched-LTO-probes/' + arm + '/PASS.json')
for name in ['actual-complete24-layouts.json']:
    paths.add('/tmp/nm2022-metadata-layout-source-prototype/' + name)
paths.add('/tmp/nm2022-root-actual-layout-readback.json')
# Small preflight compatibility metadata only; inputs and artifacts are produced
# from the fresh collector on the VM, never substituted with old raw results.
layout = Path('/tmp/nm2022-local-five-round-source-draft/prepared-layout')
for p in layout.rglob('*.json'):
    if '/transfer/' in str(p) or p.name in ['preflight.json', 'semantic-preflight.json']:
        paths.add(str(p))
(D / 'payload').mkdir(exist_ok=True)
out = []
for index, name in enumerate(sorted(paths)):
    p = Path(name)
    assert p.is_file() and not p.is_symlink() and str(p).startswith('/tmp/')
    assert '.git' not in p.parts
    data = p.read_bytes()
    assert len(data) < 10_000_000
    payload = f'{index:04}.data'
    (D / 'payload' / payload).write_bytes(data)
    out.append({'path': name, 'payload': payload, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
(D / 'payload-index.json').write_text(json.dumps(out, indent=2) + '\n')
