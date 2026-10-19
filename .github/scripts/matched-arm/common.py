"""Immutable inputs for the temporary #2215/#2118 ARM evaluation."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import tomllib

SOURCES = {
    'base': '8de12b05c425a45bbab806b140721e80db7e1bbb',
    'verdicts': 'ef45fdbffb7050b55677ca4d2b5b3b5e1e2b73ab',
    'phases': '956f213d635eb793fe61ac0d32822be08b37f205',
}
CONSUMER = 'f38a048f5eeff0438fc59a15e9eb742def575a36'
CONFIG_HASH = 'c9fe0bae7cb65b06cbe89b66773bac19b3176a8601f4860e857d42dce169897c'
LOCK_HASH = '000a40975d95b994e55d7c53ca43912d74dcd90f859b7010674f0661ebaf29e5'
CAP_SECONDS = 600


def install_cancel_handler():
    def cancelled(_signal, _frame):
        raise InterruptedError('evaluation cancelled; owned subprocess cleanup follows')
    signal.signal(signal.SIGTERM, cancelled)


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def run(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def compile_cpu_probe(fixture, production_lock, output):
    # Link the release-LTO dependency with Cargo; its rlib may contain only LLVM bitcode.
    fixture = Path(fixture)
    def versions(lock):
        packages = tomllib.loads(Path(lock).read_text())['package']
        return {p['name']: {key: p[key] for key in ('version', 'source', 'checksum')}
                for p in packages if p['name'] in {'num_cpus', 'libc', 'hermit-abi'}}
    assert versions(fixture / 'Cargo.lock') == versions(production_lock)
    profile = tomllib.loads((fixture / 'Cargo.toml').read_text())['profile']['release']
    assert profile['lto'] is True and profile['codegen-units'] == 1
    with tempfile.TemporaryDirectory(prefix='matched-arm-cpu-probe-',
                                     dir=os.environ.get('RUNNER_TEMP')) as directory:
        target = Path(directory) / 'target'
        env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0',
                   CARGO_BUILD_JOBS='2')
        result = subprocess.run(['cargo', 'build', '--release', '--locked',
                                 '--manifest-path', str(fixture / 'Cargo.toml')],
                                env=env, capture_output=True, timeout=120)
        Path(str(output) + '.build.log').write_bytes(result.stdout + result.stderr)
        result.check_returncode()
        shutil.copy2(target / 'release/matched-arm-cpu-probe-lto', output)
    assert int(run(str(output))) > 0
    return {'fixtureLockSha256': digest(fixture / 'Cargo.lock'),
            'fixtureManifestSha256': digest(fixture / 'Cargo.toml'),
            'dependencyVersions': {name: row['version'] for name, row in versions(production_lock).items()},
            'dependencyPackages': versions(production_lock)}


def machine():
    return {'uname': list(os.uname()), 'affinity': sorted(os.sched_getaffinity(0)),
            'imageOS': os.environ.get('ImageOS'),
            'imageVersion': os.environ.get('ImageVersion'),
            'cpuinfo': Path('/proc/cpuinfo').read_text(),
            'meminfo': Path('/proc/meminfo').read_text()}


def workload(root):
    assert run('git', 'rev-parse', 'HEAD', cwd=root) == CONSUMER
    assert not run('git', 'status', '--porcelain', '--untracked-files=all', cwd=root)
    assert digest(root / '.no-mistakes.yml') == CONFIG_HASH
    assert digest(root / 'pnpm-lock.yaml') == LOCK_HASH
    files = subprocess.check_output(['git', 'ls-files', '-z'], cwd=root).split(b'\0')
    sources = [root / os.fsdecode(p) for p in files if p and
               Path(os.fsdecode(p)).suffix in {'.ts', '.tsx', '.mts', '.cts', '.js', '.jsx', '.mjs', '.cjs'}]
    return {'commit': CONSUMER, 'tree': run('git', 'rev-parse', 'HEAD^{tree}', cwd=root),
            'configSha256': CONFIG_HASH, 'lockSha256': LOCK_HASH,
            'sourceFiles': len(sources), 'sourceBytes': sum(p.stat().st_size for p in sources),
            'sourceLines': sum(p.read_bytes().count(b'\n') for p in sources)}
