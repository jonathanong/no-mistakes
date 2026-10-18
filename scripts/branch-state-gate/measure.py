#!/usr/bin/env python3
"""Measure exact consumer JSON parity and raw resources in balanced source-build pairs."""
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

CONSUMER = 'f38a048f5eeff0438fc59a15e9eb742def575a36'
CONFIG = 'c9fe0bae7cb65b06cbe89b66773bac19b3176a8601f4860e857d42dce169897c'
LOCK = '000a40975d95b994e55d7c53ca43912d74dcd90f859b7010674f0661ebaf29e5'
ORACLE = '74a840c1cf712381f89787a052dca418ff4ed22c6eade305fa91e14df33a65b5'


def digest(path):
    with open(path, 'rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def identity(fixture):
    def git(*args):
        return subprocess.check_output(['git', '-C', str(fixture), *args], text=True).strip()
    require(git('rev-parse', 'HEAD') == CONSUMER, 'Consumer commit differs from the pinned revision')
    require(digest(fixture / '.no-mistakes.yml') == CONFIG, 'Consumer config differs from the pinned file')
    require(digest(fixture / 'pnpm-lock.yaml') == LOCK, 'Consumer lockfile differs from the pinned file')
    require(git('status', '--porcelain', '--untracked-files=normal') == '', 'Consumer checkout is not clean')
    return dict(commit=CONSUMER, tree=git('rev-parse', 'HEAD^{tree}'), config_sha256=CONFIG,
                lock_sha256=LOCK)


def measure():
    fixture = Path(os.environ['CONSUMER_ROOT']).resolve()
    artifacts = Path(os.environ['ARTIFACT_ROOT']).resolve()
    output = Path(os.environ['MEASUREMENT_ROOT']).resolve()
    output.mkdir(parents=True, exist_ok=True)
    jobs, diagnostics = os.environ['CHECK_JOBS'], os.environ['CHECK_DIAGNOSTICS']
    sample = int(os.environ['PAIR_NUMBER'])
    require(jobs in ('1', '4', 'default') and diagnostics in ('off', 'on'), 'Invalid measurement cell')
    require(not any(os.environ.get(k) for k in ('RAYON_NUM_THREADS', 'NO_MISTAKES_DIAGNOSTICS',
                                                 'NO_MISTAKES_TIMINGS')),
            'Unexpected Rayon or diagnostics environment override')
    before = identity(fixture)
    metadata = dict(consumer=before, jobs=jobs, diagnostics=diagnostics, pair=sample,
                    collection_cap_seconds=300, platform=platform.platform(), affinity=sorted(os.sched_getaffinity(0)),
                    image_os=os.environ.get('ImageOS'), image_version=os.environ.get('ImageVersion'),
                    cpu=Path('/proc/cpuinfo').read_text(), binaries={})
    for variant in ('baseline', 'candidate'):
        directory = artifacts / variant
        provenance = json.loads((directory / 'provenance.json').read_text())
        binary = directory / 'no-mistakes'
        binary.chmod(0o755)
        require(digest(binary) == provenance['binary_sha256'], f'{variant} binary hash differs from provenance')
        probe = directory / 'cpu-probe'
        probe.chmod(0o755)
        provenance['num_cpus'] = int(subprocess.check_output([str(probe)], text=True))
        metadata['binaries'][variant] = provenance
    require(metadata['binaries']['baseline']['source'] != metadata['binaries']['candidate']['source'],
            'Baseline and candidate use the same source commit')
    require(metadata['binaries']['baseline']['binary_sha256'] != metadata['binaries']['candidate']['binary_sha256'],
            'Baseline and candidate are the same binary')
    (output / 'environment.json').write_text(json.dumps(metadata, indent=2)+'\n')
    order = ('baseline', 'candidate', 'candidate', 'baseline') if sample % 2 else (
        'candidate', 'baseline', 'baseline', 'candidate')
    failed = False
    for position, variant in enumerate(order):
        prefix = output / f'{position}-{variant}'
        args = [str(artifacts / variant / 'no-mistakes'), 'check', '--root', str(fixture),
                '--config', str(fixture / '.no-mistakes.yml'), '--profile', 'ci', '--format', 'json']
        if jobs != 'default':
            args.extend(('--jobs', jobs))
        if diagnostics == 'on':
            args.extend(('--timings', '--verbose-timings'))
        time_format = '{"elapsed":%e,"user":%U,"system":%S,"rss_kib":%M,"exit":%x}'
        command = ['/usr/bin/time', '-q', '-o', str(prefix)+'.time', '-f', time_format,
                   'timeout', '--signal=TERM', '--kill-after=5s', '300s', *args]
        started = time.monotonic()
        with open(str(prefix)+'.stdout', 'wb') as stdout, open(str(prefix)+'.stderr', 'wb') as stderr:
            child = subprocess.run(command, cwd=fixture, stdout=stdout, stderr=stderr, check=False)
        try:
            resources = json.loads(Path(str(prefix)+'.time').read_text())
        except (ValueError, OSError):
            resources = None
        report_hash = digest(str(prefix)+'.stdout')
        eligible = child.returncode == 1 and resources is not None and resources['exit'] == 1
        parity = report_hash == ORACLE
        record = dict(variant=variant, position=position, jobs=jobs, diagnostics=diagnostics,
                      pair=sample, command=args, wrapper=command[:8], wrapper_exit=child.returncode,
                      resources=resources, complete=eligible, parity=parity, report_sha256=report_hash,
                      observed_seconds=time.monotonic()-started, metadata='environment.json')
        Path(str(prefix)+'.json').write_text(json.dumps(record, indent=2)+'\n')
        failed |= not eligible or not parity
    require(identity(fixture) == before, 'Consumer source/config/lock changed during measurement')
    if failed:
        raise SystemExit('Incomplete sample or full-JSON oracle mismatch; retain raw evidence')


if __name__ == '__main__':
    measure()
