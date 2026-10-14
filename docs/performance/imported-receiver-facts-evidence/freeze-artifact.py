import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

label, checkout = sys.argv[1:]
out = Path('<WORKSPACE>/performance-1972-evidence')
target = Path('<WORKSPACE>/performance-1972-target')
records = [json.loads(line) for line in Path(f'<WORKSPACE>/performance-1972-{label}-build.jsonl').read_text().splitlines()]
artifacts = [r for r in records if r.get('reason') == 'compiler-artifact' and r['target']['name'] == 'no_mistakes' and any(p.endswith('.rlib') for p in r['filenames'])]
assert len(artifacts) == 1, artifacts
artifact = artifacts[0]
assert not artifact['fresh'], 'A fresh cached artifact is insufficient source provenance'
assert artifact['profile']['opt_level'] == '3'
assert Path(artifact['target']['src_path']).resolve() == Path(checkout).resolve()/'crates/no-mistakes/src<SYSTEM_LIB>.rs'
compiler = subprocess.check_output(['rustc', '-Vv'], cwd=checkout, text=True)
cache = json.loads((target/'.rustc_info.json').read_text())
assert any(item.get('stdout') == compiler for item in cache['outputs'].values()), 'Compiler must match Cargo recorded compiler'
source = next(Path(p) for p in artifact['filenames'] if p.endswith('.rlib'))
frozen = out / f'lib{label}.rlib'
assert not frozen.exists(), 'Do not overwrite a frozen artifact'
assert not (out/f'{label}-manifest.json').exists(), 'Do not overwrite artifact provenance'
shutil.copy2(source, frozen)
def git(*args):
    return subprocess.check_output(['git', '-C', checkout, *args], text=True).strip()
assert not git('status', '--porcelain')
manifest = dict(revision=git('rev-parse', 'HEAD'), tree=git('rev-parse', 'HEAD^{tree}'),
                analyzer_tree=git('rev-parse', 'HEAD:crates/no-mistakes/src'),
                lock_sha256=hashlib.sha256((Path(checkout)/'Cargo.lock').read_bytes()).hexdigest(),
                rustc=compiler,
                artifact=artifact, frozen=str(frozen),
                sha256=hashlib.sha256(frozen.read_bytes()).hexdigest(),
                release_profile={'lto': True, 'codegen_units': 1, 'opt_level': 3},
                build_command='cargo build --release --locked -p no-mistakes --lib --message-format=json')
(out/f'{label}-manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
print(json.dumps({k:manifest[k] for k in ['revision','tree','sha256']}))
