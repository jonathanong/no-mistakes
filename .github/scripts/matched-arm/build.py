"""Build a pinned production CLI, retain provenance, remove only its own target."""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
import tomllib

from common import SOURCES, digest, install_cancel_handler, machine, run, save

install_cancel_handler()
role = sys.argv[1]
assert role in SOURCES
root = Path('source').resolve()
output = Path('out') / role
output.mkdir(parents=True)
target = Path(os.environ['RUNNER_TEMP']) / ('matched-arm-target-' + role)
assert not target.exists()
assert run('git', 'rev-parse', 'HEAD', cwd=root) == SOURCES[role]
assert not run('git', 'status', '--porcelain', cwd=root)
profile = tomllib.loads((root / 'Cargo.toml').read_text())['profile']['release']
assert profile['lto'] is True and profile['codegen-units'] == 1
env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0',
           CARGO_BUILD_JOBS='2')
for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER', 'NO_MISTAKES_BUILD_NAPI'):
    assert not env.get(key), key
assert not any(k.startswith('CARGO_PROFILE_RELEASE_') for k in env)
command = ['cargo', 'build', '--release', '--locked', '-p', 'no-mistakes', '--bin', 'no-mistakes']
proof = {'source': SOURCES[role], 'tree': run('git', 'rev-parse', 'HEAD^{tree}', cwd=root),
         'lockSha256': digest(root / 'Cargo.lock'), 'rustc': run('rustc', '-Vv'),
         'cargo': run('cargo', '-V'), 'features': 'default', 'profile': profile,
         'command': command, 'buildJobs': 2, 'incremental': False, 'machine': machine()}
save(output / 'source.json', proof)
process = None
try:
    with (output / 'build.log').open('wb') as log:
        process = subprocess.Popen(command, cwd=root, env=env, stdout=log,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        started = time.monotonic()
        while process.poll() is None:
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                size = int(run('du', '-sb', str(target)).split()[0])
                if size >= 8_000_000_000 or time.monotonic() - started >= 24 * 60:
                    raise RuntimeError('owned build exceeded time or target-size safety bound')
        assert process.returncode == 0, process.returncode
    assert not run('git', 'status', '--porcelain', cwd=root)
    binary = output / 'no-mistakes'
    shutil.copy2(target / 'release/no-mistakes', binary)
    libraries = list((target / 'release/deps').glob('libnum_cpus-*.rlib'))
    assert len(libraries) == 1
    subprocess.run(['rustc', 'harness/.github/scripts/matched-arm/cpu-probe.rs',
                    '--edition=2021', '-L', 'dependency=' + str(target / 'release/deps'),
                    '--extern', 'num_cpus=' + str(libraries[0]), '-o', str(output / 'cpu-probe')], check=True)
    proof.update(binarySha256=digest(binary), cpuProbeSha256=digest(output / 'cpu-probe'))
    save(output / 'provenance.json', proof)
finally:
    if process is not None and process.poll() is None:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
    if target.exists():
        shutil.rmtree(target)
