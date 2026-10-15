"""Native child-only samples; no builds or analysis run concurrently with children."""
import argparse
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

from common import FIXTURE, FIXTURE_SHA, digest, resources, write, require, failure_record
from summary import assess, stable, validate
from outputs import curate

CURRENT_SAMPLE = {'batch': None, 'arm': None, 'demand': None}


def sample(root, profile, affinity, demand, arm, count):
    binary = root / f'{profile}-{arm}-facts'
    started = time.perf_counter()
    process = subprocess.Popen([str(binary), str(root / 'baseline' / FIXTURE), demand, str(count)],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True,
        env=dict(os.environ, RAYON_NUM_THREADS='4'), preexec_fn=lambda: os.sched_setaffinity(0, affinity))
    def expired(_signal, _frame):
        os.killpg(process.pid, signal.SIGKILL)
    previous = signal.signal(signal.SIGALRM, expired)
    signal.alarm(60)
    try:
        _, status, usage = os.wait4(process.pid, 0)
        elapsed = time.perf_counter() - started
        signal.alarm(0)
        process.returncode = os.waitstatus_to_exitcode(status)
        stdout, stderr = process.communicate()
    finally:
        signal.alarm(0)
        signal.signal(signal.SIGALRM, previous)
    output, private = curate(stdout, stderr, demand)
    for kind, data in private.items():
        (root / 'private' / f'{profile}-{demand}-{arm}-{time.time_ns()}.{kind}').write_bytes(data)
    return dict(arm=arm, demand=demand, collections=count, exit=process.returncode, **output,
                user_s=usage.ru_utime, system_s=usage.ru_stime,
                cpu_s=usage.ru_utime + usage.ru_stime, wall_s=elapsed,
                rss_kib=usage.ru_maxrss, child_affinity=affinity, rayon_workers=4)


def collect(root, profile, tag, count, blocks, affinity):
    target = root / f'public/{profile}-{tag}-rows.json'
    require(not target.exists(), 'sample_output_already_exists')
    CURRENT_SAMPLE['batch'] = tag
    rows = []
    # No summary/bootstrap/hash workload until the whole batch finishes.
    for order in ('ABBA', 'BAAB'):
        for demand in ('values', 'imports'):
            for block in range(blocks):
                for position, letter in enumerate(order):
                    arm = 'baseline' if letter == 'A' else 'candidate'
                    CURRENT_SAMPLE.update(arm=arm, demand=demand)
                    row = sample(root, profile, affinity, demand, arm, count)
                    rows.append(dict(row, order=order, block=block, position=position))
                    write(target, rows)  # Retain partial failures, never drop rows.
                    require(row['exit'] == 0, 'sample_child_nonzero_exit', exit_code=row['exit'])
                    require(not row['stderr'], 'sample_child_stderr_present')
                    require(row['signature'] == ('128' if demand == 'values' else '0'),
                            'sample_fact_signature_mismatch', expected='128' if demand == 'values' else '0')
    try:
        validate(rows, blocks)
    except ValueError as error:
        require(False, 'sample_integrity_invalid', invariant=str(error))
    return rows


def measure(root, profile):
    snapshot = resources(root)
    affinity = snapshot['selected_cpus']
    preflight = json.loads((root / 'public/preflight.json').read_text())
    require(affinity == preflight['selected_cpus'], 'sampling_affinity_changed_since_preflight')
    require(snapshot['cpu_quota'] == preflight['cpu_quota'], 'sampling_cpu_quota_changed_since_preflight')
    for arm in ('baseline', 'candidate'):
        manifest = json.loads((root / f'public/{profile}-{arm}-manifest.json').read_text())
        CURRENT_SAMPLE['arm'] = arm
        require(digest(root / f'{profile}-{arm}-facts') == manifest['binary_sha256'], 'frozen_binary_hash_mismatch')
        require(digest(root / arm / FIXTURE) == FIXTURE_SHA, 'physical_fixture_hash_mismatch')
    write(root / f'public/{profile}-sample-resources.json', snapshot)
    collect(root, profile, 'original100', 100, 4, affinity)
    pilot_count = 1000 if profile == 'opt3' else 100
    pilot = collect(root, profile, 'pilot', pilot_count, 1, affinity)
    if not stable(pilot):
        write(root / f'public/{profile}-assessment.json', {'status': 'inconclusive',
              'reason': 'pilot CPU duration, arm spread or order difference outside preset bounds; no larger rerun'})
        return
    minimum = min(r['cpu_s'] for r in pilot)
    cap = 10000 if profile == 'opt3' else 1000
    count = min(cap, math.ceil(5 / minimum) * pilot_count)
    rows = collect(root, profile, 'long', count, 4, affinity)
    write(root / f'public/{profile}-assessment.json', assess(rows))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('profile', choices=['opt3', 'opt0'])
    parser.add_argument('--root', required=True, type=Path)
    args = parser.parse_args()
    try:
        measure(args.root.resolve(), args.profile)
    except Exception as error:
        write(args.root / f'public/{args.profile}-assessment.json', failure_record(
            error, 'measure', profile=args.profile, **CURRENT_SAMPLE))
        sys.exit(1)
