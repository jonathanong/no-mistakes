#!/usr/bin/env python3
"""Build one source-pinned, default-feature CLI for a matched hosted pair."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

from tool_build import pinned_environment


def digest(path):
    with open(path, 'rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def compile_cpu_probe(source, dependency, dependency_dir, output):
    subprocess.run(['rustc', str(source), '--edition=2021', '-Clto=fat', '-L',
                    'dependency='+str(dependency_dir), '--extern',
                    'num_cpus='+str(dependency), '-o', str(output)], check=True, env=pinned_environment())


def build():
    output = Path(os.environ['BUILD_OUTPUT']).resolve()
    output.mkdir(parents=True, exist_ok=True)
    target = Path.cwd() / 'target'
    require(not target.exists(), 'Build target must be isolated and fresh')
    expected = os.environ['SOURCE_SHA']
    actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    require(actual == expected and len(expected) == 40, 'Checkout does not match the requested source SHA')
    command = ['cargo', 'build', '--release', '--locked', '-p', 'no-mistakes', '--bin', 'no-mistakes']
    environment = pinned_environment()
    environment.update(RUSTUP_TOOLCHAIN='1.99.1', CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0',
                       CARGO_PROFILE_RELEASE_LTO='fat', CARGO_PROFILE_RELEASE_CODEGEN_UNITS='1',
                       CARGO_PROFILE_RELEASE_DEBUG='0', RUSTC_WRAPPER='', RUSTC_WORKSPACE_WRAPPER='')
    rustc = subprocess.check_output(['rustc', '-Vv'], env=environment, text=True)
    require(rustc.startswith('rustc 1.99.1 '), 'Expected Rust 1.99.1')
    provenance = dict(source=actual, tree=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD^{tree}'], text=True).strip(), command=command,
        rustc=rustc, rustup_toolchain='1.99.1', features='default',
        lto='fat', codegen_units=1, build_jobs=2, rustflags='', debug=0, cargo_lock_sha256=digest('Cargo.lock'),
        image_os=os.environ.get('ImageOS'), image_version=os.environ.get('ImageVersion'))
    started = time.monotonic()
    peak = 0
    child = None
    try:
        with (output / 'build.log').open('wb') as log:
            child = subprocess.Popen(command, env=environment, stdout=log, stderr=subprocess.STDOUT,
                                     start_new_session=True)
            while child.poll() is None:
                size = sum(p.stat().st_size for p in target.rglob('*') if p.is_file())
                peak = max(peak, size)
                if size >= 10_000_000_000 or time.monotonic() - started > 1_620:
                    os.killpg(child.pid, signal.SIGKILL)
                    raise RuntimeError('Isolated target size or build deadline exceeded')
                time.sleep(5)
            require(child.wait() == 0, 'Source build failed; inspect build.log')
        probe = Path(__file__).resolve().parents[2] / 'fixtures/performance/branch-state-gate/cpu-probe/main.rs'
        dependency = next((target / 'release/deps').glob('libnum_cpus-*.rlib'))
        compile_cpu_probe(probe, dependency, target / 'release/deps', output / 'cpu-probe')
        binary = output / 'no-mistakes'
        shutil.copy2(target / 'release/no-mistakes', binary)
        provenance.update(binary_sha256=digest(binary), elapsed_seconds=time.monotonic()-started,
                          target_peak_bytes=peak)
    finally:
        if child and child.poll() is None:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
        (output / 'provenance.json').write_text(json.dumps(provenance, indent=2)+'\n')
        shutil.rmtree(target, ignore_errors=True)


if __name__ == '__main__':
    build()
