# cspell:ignore baab maxrss
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import statistics
import sys
import tempfile
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'postgres-bounded-3114-exact'))
from measurement_support import validate_sample

parser = argparse.ArgumentParser()
parser.add_argument('--before', required=True, type=Path)
parser.add_argument('--after', required=True, type=Path)
parser.add_argument('--before-revision')
parser.add_argument('--after-revision')
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--order', choices=['abba', 'baab'], default='abba')
options = parser.parse_args()
root = Path(__file__).resolve().parents[3]
base = root / 'test-cases/rules/postgres-bounded-statements/materialized-bodies'
binaries = {'before': str(options.before.resolve()), 'after': str(options.after.resolve())}
environment = dict(os.environ, XDG_RUNTIME_DIR='/tmp/nm-3136-perf-runtime')


def command(directory, config):
    return ['check', '--root', str(directory), '--config', str(directory / config),
            '--format', 'json', '--jobs', '1']


def run(side, arguments, expected):
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        start = time.perf_counter()
        pid = os.posix_spawn(binaries[side], [binaries[side], *arguments], environment,
                            file_actions=[(os.POSIX_SPAWN_DUP2, stdout.fileno(), 1),
                                          (os.POSIX_SPAWN_DUP2, stderr.fileno(), 2)])
        _, status, usage = os.wait4(pid, 0)
        wall = time.perf_counter() - start
        stdout.seek(0)
        stderr.seek(0)
        data, errors = stdout.read(), stderr.read()
    report = json.loads(data)
    exit_code = os.waitstatus_to_exitcode(status)
    if len(report['advisories']) != expected[1]:
        raise ValueError('unexpected advisory count')
    checked = dict(report, advisories=[])
    try:
        validate_sample(checked, exit_code, errors, expected[0])
    except ValueError as error:
        raise ValueError(f"{side}: expected rules/advisories {expected}, got "
                         f"{len(report['rules'])}/{len(report['advisories'])}; "
                         f"status {exit_code}, warnings {len(report['warnings'])}") from error
    return dict(userSeconds=usage.ru_utime, systemSeconds=usage.ru_stime,
                wallSeconds=wall, peakRssKiB=usage.ru_maxrss, exitCode=exit_code,
                stdoutSha256=hashlib.sha256(data).hexdigest(),
                stderrSha256=hashlib.sha256(errors).hexdigest(),
                findings=len(report['rules']), advisories=len(report['advisories']),
                warnings=len(report['warnings']))


def digest(binary):
    with open(binary, 'rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


result = dict(
    baseline={'revision': options.before_revision, 'verification': 'unverified caller label'},
    candidate={'revision': options.after_revision, 'verification': 'unverified caller label'},
    profile='caller-provided executables; CLI jobs1', counters='not measured by this harness',
    binaryHashes={side: digest(binary) for side, binary in binaries.items()},
    order=options.order, controls={}, workloads={})
assert len(set(result['binaryHashes'].values())) == 2
for label, directory, config, expected in [
    ('original-off', base / 'fixture', 'off.yml', (2, 3)),
    ('original-ignore', base / 'fixture', 'ignore.yml', (2, 3)),
    ('probe-off', base / 'probes', 'off.yml', (0, 0)),
    ('probe-ignore', base / 'probes', 'ignore.yml', (0, 0)),
    ('deep', base / 'deep', '.no-mistakes.yml', (20, 0)),
]:
    samples = {side: run(side, command(directory, config), expected) for side in binaries}
    for key in ('stdoutSha256', 'stderrSha256', 'exitCode'):
        assert samples['before'][key] == samples['after'][key], (label, key)
    result['controls'][label] = samples

for label, arguments, repeats, expected in [
    ('repository', ['check', '--root', str(root), '--format', 'json', '--jobs', '1'], 2,
     {'before': (0, 0), 'after': (0, 0)}),
    ('policy-off', command(base / 'fixture', 'off.yml'), 5,
     {'before': (2, 3), 'after': (2, 3)}),
    ('probe-off', command(base / 'probes', 'off.yml'), 5,
     {'before': (0, 0), 'after': (0, 0)}),
    ('policy-advise', command(base / 'probes', 'advise.yml'), 5,
     {'before': (0, 29), 'after': (0, 17)}),
    ('deep-report', command(base / 'deep', '.no-mistakes.yml'), 5,
     {'before': (20, 0), 'after': (20, 0)}),
]:
    for side in binaries:
        run(side, arguments, expected[side])
    samples = {'before': [], 'after': []}
    for _ in range(repeats):
        for side in (('before', 'after', 'after', 'before') if options.order == 'abba'
                     else ('after', 'before', 'before', 'after')):
            samples[side].append(run(side, arguments, expected[side]))
    if label != 'policy-advise':
        for key in ('stdoutSha256', 'stderrSha256', 'exitCode'):
            assert {sample[key] for sample in samples['before']} == {
                sample[key] for sample in samples['after']}, (label, key)
    summary = {side: {key: statistics.mean(sample[key] for sample in rows)
                     for key in ('userSeconds', 'wallSeconds', 'peakRssKiB')}
               for side, rows in samples.items()}
    result['workloads'][label] = dict(samples=samples, summary=summary)
    print(label, json.dumps(summary), flush=True)

options.output.mkdir(parents=True, exist_ok=True)
(options.output / 'measurements.json').write_text(json.dumps(result, indent=2) + '\n')
with open(options.output / 'samples.csv', 'w', newline='') as output:
    fields = ['workload', 'side', 'sample', *result['workloads']['repository']['samples']['before'][0]]
    writer = csv.DictWriter(output, fieldnames=fields, lineterminator='\n')
    writer.writeheader()
    for label, workload in result['workloads'].items():
        for side, rows in workload['samples'].items():
            for index, row in enumerate(rows):
                writer.writerow(dict(workload=label, side=side, sample=index + 1, **row))
