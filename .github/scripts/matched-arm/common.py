"""Immutable inputs for the temporary #2215/#2118 ARM evaluation."""
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess

SOURCES = {
    'base': '8de12b05c425a45bbab806b140721e80db7e1bbb',
    'verdicts': 'd8af2ab71154d5bf74c2ff43ef715358cbb668c7',
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
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


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
