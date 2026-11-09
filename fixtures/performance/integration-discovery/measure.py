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
policies = source_provenance.get('policies', ['consuming', 'broad', 'disabled'])


def executable_digest(path):
    with pathlib.Path(path).open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


for output_path in [output, output.with_suffix('.jsonl'), output.with_suffix('.warmups.jsonl'),
                    output.with_suffix('.stdout'), output.with_suffix('.stderr'),
                    output.with_suffix('.provenance.json')]:
    if output_path.exists():
        raise FileExistsError(f'refusing to overwrite prior cohort: {output_path}')

binary_hashes = {arm: executable_digest(path) for arm, path in binaries.items()}
assert len(set(binary_hashes.values())) == 2, 'baseline and candidate must be distinct'
driver_sha256 = executable_digest(__file__)
source_provenance['measurementDriverSha256'] = driver_sha256


def corpus_digest():
    digest = hashlib.sha256()
    for path in sorted(p for p in corpus.rglob('*') if p.is_file() and '.git' not in p.parts):
        digest.update(str(path.relative_to(corpus)).encode() + b'\0' + path.read_bytes())
    return digest.hexdigest()


input_digest = corpus_digest()
assert input_digest == source_provenance['corpusSha256']
metadata = dict(binaryHashes=binary_hashes, binaries=binaries, corpusSha256=input_digest,
                sourceProvenance=source_provenance, platform=platform.platform(),
                affinity=sorted(os.sched_getaffinity(0)),
                measurementDriverSha256=driver_sha256,
                warmupProtocol='four untimed fresh-process runs in ABBA arm order before every mode/scope/jobs cell',
                warmupJsonl=str(output.with_suffix('.warmups.jsonl')))
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
with output.with_suffix('.jsonl').open('w') as trace, output.with_suffix('.warmups.jsonl').open('w') as warm_trace:
    for mode in ['off', 'on']:
        for scope in policies:
            for jobs in ['1', '4', 'default']:
                warmup_rows = []
                for warm_position, arm in enumerate('ABBA'):
                    command = ['taskset', '--cpu-list', ','.join(map(str, affinities[jobs])), binaries[arm]]
                    if jobs != 'default':
                        command += ['--jobs', jobs]
                    command += ['check', '--root', str(corpus), '--config',
                                str(corpus / 'configs' / (scope + '.yml')), '--format', 'json']
                    if mode == 'on':
                        command += ['--timings', '--verbose-timings']
                    warm = subprocess.run(command, capture_output=True, env=child_environment)
                    warm_stderr = warm.stderr
                    warm_authoritative_stderr = b'\n'.join(
                        line for line in warm_stderr.splitlines()
                        if not line.startswith((b'[timing]', b'[cpu]', b'[memory]', b'[hot]', b'[work]')))
                    expected = source_provenance['expectedFindings'][scope]
                    warm_row = dict(mode=mode, scope=scope, jobs=jobs, warmup_position=warm_position,
                                    arm=arm, command=command, exit=warm.returncode,
                                    sha256=hashlib.sha256(warm.stdout).hexdigest(),
                                    stderr_sha256=hashlib.sha256(warm_stderr).hexdigest(),
                                    authoritative_stderr_sha256=hashlib.sha256(warm_authoritative_stderr).hexdigest())
                    warmup_rows.append(warm_row)
                    warm_trace.write(json.dumps(warm_row) + '\n')
                    warm_trace.flush()
                    warm_report = json.loads(warm.stdout)
                    assert warm.returncode == int(sum(expected.values()) > 0), warm_row
                    assert all(len(warm_report[key]) == expected.get(key, 0) for key in
                               ['integration', 'advisories', 'codebase', 'queues', 'react', 'rules', 'warnings']), warm_row
                    assert not warm_report.get('incomplete'), warm_row
                    if scope.startswith('sql-'):
                        assert warm_report['rules'][0]['rule'] == 'postgres-no-offset', warm_row
                        assert warm_report['rules'][0]['target'] == 'offset', warm_row
                assert len({(row['exit'], row['sha256']) for row in warmup_rows}) == 1, (mode, scope, jobs, warmup_rows)
                assert len({row['authoritative_stderr_sha256'] for row in warmup_rows}) == 1, (mode, scope, jobs)
                if mode == 'off':
                    assert len({row['stderr_sha256'] for row in warmup_rows}) == 1, (scope, jobs)

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
                        expected = source_provenance['expectedFindings'][scope]
                        assert process.returncode == int(sum(expected.values()) > 0), row
                        assert all(len(value[key]) == expected.get(key, 0) for key in
                                   ['integration', 'advisories', 'codebase', 'queues', 'react', 'rules', 'warnings']), row
                        assert not value.get('incomplete'), row
                        if scope.startswith('sql-'):
                            assert value['rules'][0]['rule'] == 'postgres-no-offset', row
                            assert value['rules'][0]['target'] == 'offset', row
                cell = [r for r in rows if (r['mode'], r['scope'], r['jobs']) == (mode, scope, jobs)]
                assert len({(r['exit'], r['sha256']) for r in cell}) == 1, (mode, scope, jobs)
                assert len({r['authoritative_stderr_sha256'] for r in cell}) == 1, (mode, scope, jobs)
                if mode == 'off':
                    assert len({r['stderr_sha256'] for r in cell}) == 1, (scope, jobs)
                output.write_text(json.dumps(rows, indent=2) + '\n')
                print(json.dumps(dict(mode=mode, scope=scope, jobs=jobs, samples=len(cell),
                                      identical_stdout_and_status=True)), flush=True)
for scope in policies:
    for jobs in ['1', '4', 'default']:
        cell = [r for r in rows if (r['scope'], r['jobs']) == (scope, jobs)]
        assert len({(r['exit'], r['sha256']) for r in cell}) == 1, (scope, jobs, 'observer parity')
assert corpus_digest() == input_digest, 'input corpus changed while sampling'
final_binary_hashes = {arm: executable_digest(path) for arm, path in binaries.items()}
assert final_binary_hashes == binary_hashes, 'a frozen executable changed while sampling'
metadata['binaryHashesAfter'] = final_binary_hashes
output.with_suffix('.provenance.json').write_text(json.dumps(metadata, indent=2) + '\n')
