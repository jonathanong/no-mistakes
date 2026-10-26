#!/usr/bin/env python3
"""Run fresh CLI processes in balanced ABBA/BAAB order and retain raw evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import statistics
import time

parser = argparse.ArgumentParser()
parser.add_argument('--baseline', required=True)
parser.add_argument('--candidate', required=True)
parser.add_argument('--fixture', required=True)
parser.add_argument('--out', required=True)
args = parser.parse_args()
root = Path(args.fixture).resolve()
out = Path(args.out).resolve()
out.mkdir(parents=True, exist_ok=True)
binaries = {name: Path(getattr(args, name)).resolve() for name in ('baseline', 'candidate')}
hashes = {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in binaries.items()}
assert hashes['baseline'] != hashes['candidate'], 'comparison needs distinct binaries'
# Identical write sizes avoid different file-cache residency from artifact copies.
# Verify every copy; this changes neither executable bytes nor analyzer settings.
(out / 'binaries').mkdir(exist_ok=True)
for name, original in binaries.items():
    frozen = out / 'binaries' / name
    with original.open('rb') as source, frozen.open('wb') as destination:
        while block := source.read(4096):
            destination.write(block)
    frozen.chmod(0o755)
    assert hashlib.sha256(frozen.read_bytes()).hexdigest() == hashes[name]
    binaries[name] = frozen
env = os.environ.copy()
env['XDG_RUNTIME_DIR'] = str(out / 'runtime')
Path(env['XDG_RUNTIME_DIR']).mkdir(mode=0o700, exist_ok=True)
env.pop('RAYON_NUM_THREADS', None)
order = ['baseline', 'candidate', 'candidate', 'baseline', 'candidate', 'baseline', 'baseline', 'candidate'] * 2
records = []


def sample(variant, cohort, jobs, observer, sequence, warmup=False):
    stem = f'{cohort}-j{jobs}-{observer}-{sequence}-{variant}'
    stdout_path, stderr_path = out / f'{stem}.stdout', out / f'{stem}.stderr'
    command = [str(binaries[variant]), '--timeout', '0', '--lock-timeout', '0', 'check', '--root', str(root), '--config', str(root / f'{cohort}.yml'), '--format', 'json']
    if cohort == 'startup':
        command = [str(binaries[variant]), '--version']
    if jobs not in ('auto', 'none'):
        command.extend(['--jobs', jobs])
    if observer == 'on':
        command.append('--verbose-timings')
    with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
        started = time.perf_counter()
        pid = os.fork()
        if pid == 0:
            os.setpgrp()
            os.dup2(stdout.fileno(), 1)
            os.dup2(stderr.fileno(), 2)
            os.execve(command[0], command, env)
        def deadline(_sig, _frame):
            os.killpg(pid, signal.SIGKILL)
            raise TimeoutError(stem)
        previous = signal.signal(signal.SIGALRM, deadline)
        signal.alarm(60)
        try:
            _, status, usage = os.wait4(pid, 0)
        finally:
            signal.alarm(0)
            signal.signal(signal.SIGALRM, previous)
        elapsed = time.perf_counter() - started
    raw = stdout_path.read_bytes()
    report = {} if cohort == 'startup' else json.loads(raw)
    code = os.waitstatus_to_exitcode(status)
    expected = 1 if cohort == 'broad' else 0
    assert code == expected, (stem, code, stderr_path.read_text()[:1500], report)
    result = dict(variant=variant, cohort=cohort, jobs=jobs, observer=observer,
                  sequence=sequence, warmup=warmup, wall_s=elapsed,
                  user_s=usage.ru_utime, system_s=usage.ru_stime,
                  cpu_s=usage.ru_utime + usage.ru_stime, rss_kib=usage.ru_maxrss,
                  exit=code, report_sha256=hashlib.sha256(raw).hexdigest(),
                  findings=len(report.get('rules', [])), warnings=report.get('warnings', []))
    with (out / 'samples.jsonl').open('a') as stream:
        stream.write(json.dumps(result) + '\n')
    return result


for variant in binaries:
    sample(variant, 'startup', 'none', 'off', 'warmup', True)
for sequence, variant in enumerate(order):
    records.append(sample(variant, 'startup', 'none', 'off', sequence))

for cohort in ('consuming', 'control', 'broad'):
    for jobs in ('1', '4', 'auto'):
        for observer in ('off', 'on'):
            baseline = sample('baseline', cohort, jobs, observer, 'warmup', True)
            candidate = sample('candidate', cohort, jobs, observer, 'warmup', True)
            assert baseline['report_sha256'] == candidate['report_sha256'], (cohort, jobs, observer, 'report parity')
            expected_hash = baseline['report_sha256']
            for sequence, variant in enumerate(order):
                row = sample(variant, cohort, jobs, observer, sequence)
                assert row['report_sha256'] == expected_hash, (cohort, jobs, observer, sequence, 'nondeterministic report')
                records.append(row)
            print(json.dumps(dict(cohort=cohort, jobs=jobs, observer=observer, samples=16)), flush=True)

summary = []
startup = {}
for variant in binaries:
    chosen = [r for r in records if r['cohort'] == 'startup' and r['variant'] == variant]
    startup[variant] = {m: statistics.median([r[m] for r in chosen]) for m in ('wall_s', 'cpu_s', 'rss_kib')}
(out / 'startup.json').write_text(json.dumps(startup, indent=2) + '\n')
for cohort in ('consuming', 'control', 'broad'):
    for jobs in ('1', '4', 'auto'):
        for observer in ('off', 'on'):
            row = dict(cohort=cohort, jobs=jobs, observer=observer)
            for variant in binaries:
                selected = [r for r in records if (r['cohort'], r['jobs'], r['observer'], r['variant']) == (cohort, jobs, observer, variant)]
                row[variant] = dict(n=len(selected), exit=selected[0]['exit'], findings=selected[0]['findings'], report_sha256=selected[0]['report_sha256'])
                for metric in ('wall_s', 'cpu_s', 'rss_kib'):
                    values = [r[metric] for r in selected]
                    row[variant][metric] = dict(mean=statistics.mean(values), median=statistics.median(values), minimum=min(values), maximum=max(values))
            summary.append(row)
(out / 'summary.json').write_text(json.dumps(dict(binaries={k: str(v) for k,v in binaries.items()}, hashes=hashes, order=order, summaries=summary), indent=2) + '\n')
