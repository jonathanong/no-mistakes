"""Build/freeze one reviewed source arm; never runs measurements."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

source, destination = map(lambda p: Path(p).resolve(), sys.argv[1:3])
revision = sys.argv[3]
harness = Path(__file__).with_name('harness')
target = source / 'target'
assert str(source).startswith('/tmp/'), 'Dedicated temporary worktree required'
assert not target.exists() and not target.is_symlink(), 'Require absent dedicated target'
assert not destination.exists(), 'Never overwrite frozen evidence'

def command(args, **kwargs):
    return subprocess.check_output(args, cwd=source, text=True, **kwargs).strip()

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

assert command(['git', 'rev-parse', '--show-toplevel']) == str(source)
assert command(['git', 'rev-parse', 'HEAD']) == revision
assert not command(['git', 'status', '--porcelain'])
profile = tomllib.loads((source/'Cargo.toml').read_text())['profile']['release']
assert profile['lto'] is True and profile['codegen-units'] == 1
assert profile.get('opt-level', 3) == 3
build_env = {key: value for key, value in os.environ.items()
             if not key.startswith('CARGO_PROFILE_') and key not in
             ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER')}
build_env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
                 RUSTUP_TOOLCHAIN='stable', RUSTFLAGS='', CARGO_ENCODED_RUSTFLAGS='',
                 RUSTC_WRAPPER='', RUSTC_WORKSPACE_WRAPPER='')
rustc = command(['rustc', '+stable', '-Vv'], env=build_env)
assert rustc.startswith('rustc 1.99.0 (b940084d7 ')
destination.mkdir(parents=True)
artifact_log = destination/'cargo-artifacts.jsonl'
try:
    with artifact_log.open('w') as stdout, (destination/'build.stderr').open('w') as stderr:
        subprocess.run(['cargo', '+stable', 'build', '--locked', '-p', 'no-mistakes', '--lib',
                        '--release', '--message-format=json'], cwd=source, env=build_env,
                       stdout=stdout, stderr=stderr, check=True)
    artifacts = [json.loads(line) for line in artifact_log.read_text().splitlines()]
    libraries = [row for row in artifacts if row.get('reason') == 'compiler-artifact'
                 and row.get('target', {}).get('name') == 'no_mistakes'
                 and row['target']['kind'] == ['rlib']]
    assert len(libraries) == 1
    artifact = libraries[0]
    assert Path(artifact['target']['src_path']).resolve() == source/'crates/no-mistakes/src/lib.rs'
    assert artifact['features'] == ['default', 'mermaid-validation']
    assert not artifact['fresh'] and artifact['profile']['opt_level'] == '3'
    assert artifact['profile']['debuginfo'] == 0 and not artifact['profile']['test']
    library, = [Path(path) for path in artifact['filenames'] if path.endswith('.rlib')]
    shutil.copy2(library, destination/'lib.rlib')
    link = ['rustc', '+stable', '--edition=2021', '--crate-name', 'nm_shared_analysis_probe',
            '-C', 'opt-level=3', '-C', 'lto=true', '-C', 'codegen-units=1', '-C', 'debuginfo=0',
            '-L', 'dependency='+str(target/'release/deps'),
            '--extern', 'no_mistakes='+str(destination/'lib.rlib'),
            str(harness/'main.rs'), '-o', str(destination/'probe')]
    (destination/'link-command.json').write_text(json.dumps(link, indent=2)+'\n')
    with (destination/'link.stdout').open('w') as stdout, (destination/'link.stderr').open('w') as stderr:
        subprocess.run(link, cwd=source, env=build_env, stdout=stdout, stderr=stderr, check=True)
    files = command(['git', 'ls-files']).splitlines()
    selected = [path for path in files if path in ('Cargo.toml', 'Cargo.lock', 'crates/no-mistakes/Cargo.toml')
                or path.startswith('crates/no-mistakes/') and path.endswith('.rs')]
    sources = [{'path': path, 'sha256': digest(source/path)} for path in sorted(selected)]
    (destination/'source-manifest.json').write_text(json.dumps(sources, indent=2)+'\n')
    harness_sources = [{'path': path.name, 'sha256': digest(path)} for path in sorted(harness.glob('*.rs'))]
    assert command(['git', 'rev-parse', 'HEAD']) == revision
    assert not command(['git', 'status', '--porcelain'])
    manifest = dict(revision=revision, tree=command(['git', 'rev-parse', 'HEAD^{tree}']), rustc=rustc,
                    lock_sha256=digest(source/'Cargo.lock'), features=artifact['features'],
                    release_profile=dict(opt_level=3,lto=True,codegen_units=1,debug=0,build_jobs=2,incremental=0),
                    source_manifest_sha256=digest(destination/'source-manifest.json'),
                    harness=harness_sources, library_sha256=digest(destination/'lib.rlib'),
                    binary_sha256=digest(destination/'probe'), compiler_artifact=artifact)
    (destination/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({'frozen':str(destination),'revision':revision,'binary_sha256':manifest['binary_sha256']}),flush=True)
finally:
    assert not target.is_symlink()
    if target.exists():
        shutil.rmtree(target)
