"""Measure frozen release binaries in balanced fresh-process blocks (Linux)."""
import hashlib
import json
import os
import pathlib
import platform
import subprocess
import sys
import time

corpus = pathlib.Path(sys.argv[1]).resolve()
binaries = {'A': sys.argv[2], 'B': sys.argv[3]}
output = pathlib.Path(sys.argv[4]).resolve()
source_provenance = json.loads(pathlib.Path(sys.argv[5]).read_text())
def executable_digest(path):
    with pathlib.Path(path).open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


binary_hashes = {arm: executable_digest(path) for arm, path in binaries.items()}
assert len(set(binary_hashes.values())) == 2, 'baseline and candidate must be distinct'


def corpus_digest():
    digest = hashlib.sha256()
    for path in sorted(p for p in corpus.rglob('*') if p.is_file() and '.git' not in p.parts):
        digest.update(str(path.relative_to(corpus)).encode() + b'\0' + path.read_bytes())
    return digest.hexdigest()


input_digest = corpus_digest()
assert input_digest == source_provenance['corpusSha256']
metadata = dict(binaryHashes=binary_hashes, binaries=binaries, corpusSha256=input_digest,
                sourceProvenance=source_provenance, platform=platform.platform(),
                affinity=sorted(os.sched_getaffinity(0)))
output.with_suffix('.provenance.json').write_text(json.dumps(metadata, indent=2) + '\n')
allowed_cpus = sorted(os.sched_getaffinity(0))
affinities = json.loads(os.environ.get('NM_BENCH_AFFINITY_JSON', '{}'))
for jobs in ['1', '4', 'default']:
    affinities.setdefault(jobs, allowed_cpus)
    assert affinities[jobs] and set(affinities[jobs]) <= set(allowed_cpus)
metadata['affinityByJobs'] = affinities
output.with_suffix('.provenance.json').write_text(json.dumps(metadata, indent=2) + '\n')
child_environment = os.environ.copy()
child_environment.pop('RAYON_NUM_THREADS', None)
rows = []
with output.with_suffix('.jsonl').open('w') as trace:
    for mode in ['off', 'on']:
        for scope in ['consuming', 'broad', 'disabled']:
            for jobs in ['1', '4', 'default']:
                orders = ['ABBA' if block % 2 == 0 else 'BAAB' for block in range(12)]
                orders += ['AAAA', 'BBBB']
                for block, order in enumerate(orders):
                    for position, arm in enumerate(order):
                        command = ['taskset', '--cpu-list', ','.join(map(str, affinities[jobs])), binaries[arm]]
                        if jobs != 'default':
                            command += ['--jobs', jobs]
                        command += ['check', '--root', str(corpus), '--config',
                                    str(corpus / 'configs' / (scope + '.yml')), '--format', 'json']
                        if mode == 'on':
                            command += ['--timings', '--verbose-timings']
                        stdout_path = output.with_suffix('.stdout')
                        stderr_path = output.with_suffix('.stderr')
                        with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
                            started = time.perf_counter()
                            process = subprocess.Popen(command, stdout=stdout, stderr=stderr, env=child_environment)
                            _, status, usage = os.wait4(process.pid, 0)
                            process.returncode = os.waitstatus_to_exitcode(status)
                            wall = time.perf_counter() - started
                        report = stdout_path.read_bytes()
                        stderr = stderr_path.read_bytes()
                        authoritative_stderr = b'\n'.join(
                            line for line in stderr.splitlines()
                            if not line.startswith((b'[timing]', b'[cpu]', b'[memory]', b'[hot]', b'[work]')))
                        row = dict(mode=mode, scope=scope, jobs=jobs, block=block,
                                   position=position, arm=arm, calibration=block >= 12,
                                   wall=wall, user=usage.ru_utime, system=usage.ru_stime,
                                   rss=usage.ru_maxrss, exit=process.returncode,
                                   sha256=hashlib.sha256(report).hexdigest(),
                                   stderr_sha256=hashlib.sha256(stderr).hexdigest(),
                                   authoritative_stderr_sha256=hashlib.sha256(authoritative_stderr).hexdigest(),
                                   load1=os.getloadavg()[0])
                        rows.append(row)
                        trace.write(json.dumps(row) + '\n')
                        trace.flush()
                        value = json.loads(report)
                        assert process.returncode == int(scope != 'disabled'), row
                        assert len(value['integration']) == int(scope != 'disabled'), row
                        assert all(value[key] == [] for key in
                                   ['advisories', 'codebase', 'queues', 'react', 'rules', 'warnings']), row
                        assert not value.get('incomplete'), row
                cell = [r for r in rows if (r['mode'], r['scope'], r['jobs']) == (mode, scope, jobs)]
                assert len({(r['exit'], r['sha256']) for r in cell}) == 1, (mode, scope, jobs)
                assert len({r['authoritative_stderr_sha256'] for r in cell}) == 1, (mode, scope, jobs)
                if mode == 'off':
                    assert len({r['stderr_sha256'] for r in cell}) == 1, (scope, jobs)
                output.write_text(json.dumps(rows, indent=2) + '\n')
                print(json.dumps(dict(mode=mode, scope=scope, jobs=jobs, samples=len(cell),
                                      identical_stdout_and_status=True)), flush=True)
for scope in ['consuming', 'broad', 'disabled']:
    for jobs in ['1', '4', 'default']:
        cell = [r for r in rows if (r['scope'], r['jobs']) == (scope, jobs)]
        assert len({(r['exit'], r['sha256']) for r in cell}) == 1, (scope, jobs, 'observer parity')
assert corpus_digest() == input_digest, 'input corpus changed while sampling'
