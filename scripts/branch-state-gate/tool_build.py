#!/usr/bin/env python3
"""Build and freeze the pinned Selene tool natively on the hosted ARM runner."""
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import time

VERSION = '0.29.0'
MAX_TARGET = 10_000_000_000
DEADLINE = 900


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def target_size(target):
    total = 0
    for path in target.rglob('*'):
        try:
            if path.is_file():
                total += path.stat().st_size
        except FileNotFoundError:
            pass
    return total


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def pinned_environment():
    environment = os.environ.copy()
    environment['RUSTUP_TOOLCHAIN'] = '1.99.1'
    for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS'):
        environment.pop(key, None)
    return environment


def build():
    source = Path.cwd().resolve()
    output = Path(os.environ['TOOL_OUTPUT']).resolve()
    target = source / 'target'
    require(not target.exists(), 'Build requires a fresh workspace-root target')
    require(not output.is_relative_to(target), 'Frozen tool output must be outside target')
    output.mkdir(parents=True, exist_ok=True)
    require(not any(output.iterdir()), 'TOOL_OUTPUT must be fresh and empty')
    require(platform.system() == 'Linux' and platform.machine().lower() in ('aarch64', 'arm64'),
            'Selene must be built natively on Linux ARM')
    expected = os.environ['SOURCE_SHA']
    actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=source, text=True).strip()
    require(actual == expected and len(actual) == 40, 'Source checkout differs from pinned candidate')
    require(not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=source, text=True),
            'Source checkout must be clean')
    environment = pinned_environment()
    rustc = subprocess.check_output(['rustc', '-Vv'], cwd=source, env=environment, text=True)
    require(rustc.startswith('rustc 1.99.1 '), 'Expected Rust 1.99.1')
    install_root = target / 'selene-install'
    command = ['cargo', 'install', 'selene', '--version', VERSION, '--locked', '--jobs', '2', '--root', str(install_root)]
    environment.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
                       RUSTC_WRAPPER='', RUSTC_WORKSPACE_WRAPPER='')
    lock_hash = digest(source / 'Cargo.lock')
    provenance = dict(source=actual, tree=tree, command=command, rustc=rustc, tool='selene',
                      tool_version=VERSION, features='default', default_features=True,
                      target_triple=subprocess.check_output(['rustc', '-vV'], cwd=source, env=environment, text=True).split('host: ')[1].splitlines()[0],
                      workspace_cargo_lock_sha256=lock_hash, build_jobs=2, rustflags='', rustup_toolchain='1.99.1',
                      install_root='target/selene-install', image_os=os.environ.get('ImageOS'),
                      image_version=os.environ.get('ImageVersion'), complete=False)
    started, peak, child = time.monotonic(), 0, None
    try:
        with (output / 'build.log').open('wb') as log:
            child = subprocess.Popen(command, cwd=source, env=environment, stdout=log, stderr=subprocess.STDOUT,
                                     start_new_session=True)
            while child.poll() is None:
                size = target_size(target)
                peak = max(peak, size)
                require(size < MAX_TARGET, 'Isolated tool target reached 10GB')
                require(time.monotonic() - started < DEADLINE, 'Selene build exceeded its 15-minute cap')
                time.sleep(3)
            require(child.wait() == 0, 'Selene build failed; inspect build.log')
        peak = max(peak, target_size(target))
        require(peak < MAX_TARGET, 'Completed target reached 10GB')
        require(time.monotonic() - started < DEADLINE, 'Selene build exceeded its 15-minute cap')
        binary = install_root / 'bin/selene'
        version = subprocess.check_output([str(binary), '--version'], text=True).strip()
        require(version.split() == ['selene', VERSION], f'Unexpected Selene version: {version}')
        frozen = output / 'selene'
        shutil.copy2(binary, frozen)
        require(digest(frozen) == digest(binary), 'Frozen Selene binary hash mismatch')
        require(digest(source / 'Cargo.lock') == lock_hash, 'Cargo.lock changed during build')
        require(not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=source, text=True),
                'Source checkout changed during build')
        provenance.update(binary_sha256=digest(frozen), version_output=version, elapsed_seconds=time.monotonic()-started,
                          target_peak_bytes=peak, complete=True)
    finally:
        if child and child.poll() is None:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            child.wait()
        provenance.update(elapsed_seconds=time.monotonic()-started, target_peak_bytes=peak)
        try:
            (output / 'provenance.json').write_text(json.dumps(provenance, indent=2)+'\n')
        finally:
            if target.exists():
                shutil.rmtree(target)


if __name__ == '__main__':
    build()
