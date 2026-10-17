"""Copy reviewed static inputs and closed provenance; native bytes use asset IDs."""
from pathlib import Path
import hashlib
import json

D = Path(__file__).resolve().parent
C = Path('/tmp/nm2022-main281-current-pair-functional/consumed-source-inputs.json')
closure = json.loads(C.read_text())
rows = closure['files']
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
paths = {row['path'] for row in rows}
for row in rows:
    p = Path(row['path'])
    assert p.stat().st_size == row['bytes'] and sha(p) == row['sha256']
paths.add(str(C))
A = Path('/tmp/nm2022-final-cost-native-source-draft/candidate')
assert json.loads((A / 'cleanup.json').read_text()) == {'target_absent': True, 'source_after_exact': True}
assert json.loads((A / 'closed-artifacts.json').read_text())['head'] == 'e193aeca201a346115ebe0ee552d6cb1ec995447'
paths.update(str(A / name) for name in ['closed-artifacts.json', 'cleanup.json'])
gate = json.loads((D / 'plan.json').read_text())['layout_gate']
assert gate['status'] == 'PASS exact current compiler layouts'
for key in ['proof', 'root_review']:
    pin = gate[key]
    assert sha(pin['path']) == pin['sha256']
    paths.add(pin['path'])
assert not any('/current-pair-semantics/' in name or '/node_modules/' in name or '/functional-deps/' in name or '/cli-completed-dependencies/' in name for name in paths)
# New producer receipts supplement, never replace, the frozen source/expectations.
paths.update(str(Path('/tmp/nm2022-final-cost-native-source-draft') / name) for name in ['actual-owner-terminal.json', 'request.json', 'source-inventory.json', 'candidate/closed-cli-stage.json'])
paths.update(str(Path('/tmp/nm2022-final-cost-helper-links') / name) for name in ['receipt.json', 'actual-owner-terminal.json'])
paths.update(str(Path('/tmp/nm2022-final-cost-functional') / name) for name in ['actual-functional-result.json', 'functional-api/PASS.json'])
paths.add('/tmp/nm2022-4a-completed-ci/exact-head-ci-receipt.json')
for key in ['proof', 'root_review']:
    pin = gate['source_carry'][key]
    assert sha(pin['path']) == pin['sha256']
    paths.add(pin['path'])
(D / 'payload').mkdir(exist_ok=True)
out = []
for index, name in enumerate(sorted(paths)):
    p = Path(name)
    assert p.is_file() and not p.is_symlink() and str(p).startswith('/tmp/')
    assert '.git' not in p.parts
    data = ((D / 'current-pair-collector.py').read_bytes() if name == '/tmp/nm2022-main281-current-pair-functional/collect-current-pair-semantics.py' else p.read_bytes())
    assert len(data) < 10_000_000 and not data.startswith(b'\x7fELF')
    payload = f'{index:04}.data'
    (D / 'payload' / payload).write_bytes(data)
    out.append({'path': name, 'payload': payload, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
used = {row['payload'] for row in out}
for path in (D / 'payload').iterdir():
    assert path.is_file() and not path.is_symlink()
    if path.name not in used:
        path.unlink()
(D / 'payload-index.json').write_text(json.dumps(out, indent=2) + '\n')
print(json.dumps({'files': len(out), 'native_bytes_in_source_payload': False}))
