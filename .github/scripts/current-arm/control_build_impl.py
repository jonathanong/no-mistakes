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

from common import SOURCES, compile_cpu_probe, digest, install_cancel_handler, machine, run, save
from frozen import PINS, copy_frozen
from build_size import target_size

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
    if role in PINS:
        proof = copy_frozen(role, proof, output)
    else:
        with (output / 'build.log').open('wb') as log:
            process = subprocess.Popen(command, cwd=root, env=env, stdout=log,
                                       stderr=subprocess.STDOUT, start_new_session=True)
            started = time.monotonic()
            while process.poll() is None:
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    size = target_size(target)
                    elapsed = time.monotonic() - started
                    if int(elapsed // 60) > int((elapsed - 5) // 60):
                        print(f'BUILD role={role} elapsedSeconds={elapsed:.1f} targetBytes={size}', flush=True)
                    if size >= 8_000_000_000 or time.monotonic() - started >= 24 * 60:
                        raise RuntimeError('owned build exceeded time or target-size safety bound')
            assert process.returncode == 0, process.returncode
        assert not run('git', 'status', '--porcelain', cwd=root)
        binary = output / 'no-mistakes'
        shutil.copy2(target / 'release/no-mistakes', binary)
    binary = output / 'no-mistakes'
    probe = compile_cpu_probe('harness/fixtures/performance/matched-arm/cpu-probe-lto',
                              root / 'Cargo.lock', output / 'cpu-probe')
    proof.update(binarySha256=digest(binary), cpuProbeSha256=digest(output / 'cpu-probe'),
                 cpuProbeBuild=probe)
    proof['artifactHarnessCommit'] = os.environ['GITHUB_SHA']
    save(output / 'provenance.json', proof)
finally:
    if process is not None and process.poll() is None:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
    if target.exists():
        shutil.rmtree(target)
