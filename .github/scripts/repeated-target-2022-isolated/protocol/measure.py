"""Bounded native paired samples; file-backed stdout avoids wait4 pipe deadlocks."""
import argparse
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

from pair import HERE, SPEC
from common import digest, resources, require, write, failure_record
from paired_summary import assess, describe_pilot, stable, validate
from semantics import inspect_sample

CURRENT = {'workload': None, 'arm': None, 'phase': None}
LOCAL_DEADLINE = None


def sample(root, workload, arm, count, affinity):
    require(LOCAL_DEADLINE is not None and time.monotonic() < LOCAL_DEADLINE - 65, 'local_child_reserve_exhausted')
    directory = root / 'private' / f"{workload['name']}-{arm}-{time.time_ns()}"
    directory.mkdir(parents=True)
    source = root / 'inputs' / SPEC['inputs'][workload['input']]['path']
    if workload['kind'] == 'probe':
        args = [str(root / arm / 'transfer/timing-probe'), workload['mode'], str(source), str(count)]
    else:
        args = [sys.executable, str(HERE / 'cli-worker.py'), str(root / arm / 'transfer/cli'),
                str(source), str(count), 'on' if workload['diagnostics'] else 'off', str(directory / 'requests'), json.dumps(workload['command'])]
    with (directory / 'stdout').open('wb') as stdout, (directory / 'stderr').open('wb') as stderr:
        started = time.perf_counter()
        process = subprocess.Popen(args, stdout=stdout, stderr=stderr, start_new_session=True,
            env=dict(os.environ, RAYON_NUM_THREADS='4'), preexec_fn=lambda: os.sched_setaffinity(0, affinity))
        timed_out = False
        def expired(_signal, _frame):
            nonlocal timed_out
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
        previous = signal.signal(signal.SIGALRM, expired)
        signal.alarm(60)
        try:
            _, status, usage = os.wait4(process.pid, 0)
            elapsed = time.perf_counter() - started
            process.returncode = os.waitstatus_to_exitcode(status)
        finally:
            signal.alarm(0)
            signal.signal(signal.SIGALRM, previous)
    return complete_sample_row(root,directory,workload,arm,count,process.returncode,
                               timed_out,usage,elapsed,affinity)


def complete_sample_row(root,directory,workload,arm,count,returncode,timed_out,usage,elapsed,affinity):
    # Preserve wait4 resources even if output inspection cannot read a file.
    try:
        verified = inspect_sample(root, directory, workload, arm, count)
        semantic_valid = verified.pop('valid')
    except Exception:
        verified = {'error_code':'sample_output_inspection_failed'}
        semantic_valid = False
    return dict(arm=arm, count=count, exit=returncode, timed_out=timed_out,
        valid=returncode == 0 and not timed_out and semantic_valid,
        **verified, user_s=usage.ru_utime, system_s=usage.ru_stime,
        cpu_s=usage.ru_utime + usage.ru_stime, wall_s=elapsed, rss_kib=usage.ru_maxrss,
        affinity=affinity, rayon_workers=4)


def collect(root, workload, phase, count, blocks, affinity, deadline):
    CURRENT['phase'] = phase
    output = root / f"public/{workload['name']}-{phase}-rows.json"
    require(not output.exists(), 'sample_rows_already_exist')
    rows = []
    for order in ('ABBA', 'BAAB'):
        for block in range(blocks):
            for position, letter in enumerate(order):
                require(time.monotonic() < deadline - 60, 'global_sampling_deadline_near')
                arm = 'baseline' if letter == 'A' else 'candidate'
                CURRENT['arm'] = arm
                row = sample(root, workload, arm, count, affinity)
                row.update(order=order, block=block, position=position)
                rows.append(row)
                write(output, rows)
                require(row['valid'], 'sample_exit_output_or_semantics_invalid')
    validate(rows, blocks)
    return rows


def capture_two_arms(root, workload, phase, count, affinity):
    output=root/f"public/{workload['name']}-{phase}.json"
    require(not output.exists(),'sample_rows_already_exist')
    rows=[]
    for arm in ('baseline','candidate'):
        CURRENT.update(phase=phase,arm=arm)
        row=sample(root,workload,arm,count,affinity)
        rows.append(row)
        write(output,rows)
        require(row['valid'],phase+'_semantics_invalid')
    return rows


def measure(root, deadline, round_id):
    global LOCAL_DEADLINE
    LOCAL_DEADLINE = deadline
    snapshot = resources(root)
    before = json.loads((root / 'public/preflight.json').read_text())
    require(snapshot['selected_cpus'] == before['selected_cpus'] and snapshot['cpu_quota'] == before['cpu_quota'], 'sampling_resources_changed')
    for arm in ('baseline', 'candidate'):
        manifest = json.loads((root / arm / 'transfer/transfer-manifest.json').read_text())
        require(digest(root / arm / 'transfer/timing-probe') == manifest['files']['timing-probe']['sha256'], 'timing_binary_changed')
        require(digest(root / arm / 'transfer/cli') == manifest['files']['cli']['sha256'], 'cli_binary_changed')
    for input_key, item in SPEC['inputs'].items():
        for row in item['files']:
            # The configured-consumer overlay is separately hashed in semantic preflight.
            if input_key == 'configured-consumer' and row['path'].endswith('.no-mistakes.yml'):
                continue
            require(digest(root / 'inputs' / row['path']) == row['sha256'], 'common_input_changed')
    semantics = json.loads((root / 'public/semantic-preflight.json').read_text())
    for input_key, hash_key in [('configured-consumer', 'consumer_overlay_sha256'), ('default-cold', 'plain_cold_overlay_sha256')]:
        overlay = root / 'inputs' / SPEC['inputs'][input_key]['path'] / '.no-mistakes.yml'
        require(digest(overlay) == semantics[hash_key], 'effective_config_overlay_changed')
    write(root / f'public/sample-resources-round-{round_id}.json', snapshot)
    outcomes = []
    for workload in SPEC['workloads']:
        CURRENT.update(workload=workload['name'], arm=None, phase='startup')
        try:
            require(time.monotonic() < deadline - 65, 'sampling_deadline_reserve_for_partial_rows')
            # One per arm, retained independently of pilot/long distributions.
            startup = capture_two_arms(root,workload,'startup',1,snapshot['selected_cpus'])
            # A single process startup can dominate tiny fact requests. Use a
            # separately retained common calibration batch to choose the pilot,
            # without subtracting startup or altering any reported resource row.
            calibration_count = 128 if workload['kind'] == 'probe' else 16
            calibration = capture_two_arms(root,workload,'calibration',calibration_count,snapshot['selected_cpus'])
            minimum = min(r['cpu_s'] for r in calibration)
            count = min(SPEC['iterations_cap'], max(1, math.ceil(2 * calibration_count / max(minimum, 1e-6))))
            pilot = collect(root, workload, 'pilot', count, 1, snapshot['selected_cpus'], deadline)
            if not stable(pilot):
                outcomes.append(dict(workload=workload['name'], count=count, status='inconclusive; long withheld',
                    reason='pilot duration/spread/order exceeded preset bounds; no favorable retry', **describe_pilot(pilot)))
            else:
                count = min(SPEC['iterations_cap'], max(count, math.ceil(5 / min(r['cpu_s'] for r in pilot) * count)))
                rows = collect(root, workload, 'long', count, 4, snapshot['selected_cpus'], deadline)
                outcomes.append(dict(workload=workload['name'], count=count, phase='long', samples=len(rows), **assess(rows)))
        except Exception as error:
            outcomes.append(failure_record(error, 'sample', **CURRENT))
            write(root / f'public/assessments-round-{round_id}.json', outcomes)
            if time.monotonic() >= deadline - 60:
                break
        write(root / f'public/assessments-round-{round_id}.json', outcomes)
    visited = {row.get('workload') for row in outcomes}
    outcomes.extend({'workload': w['name'], 'status': 'incomplete', 'reason': 'sampling deadline; not executed'} for w in SPEC['workloads'] if w['name'] not in visited)
    write(root / f'public/assessments-round-{round_id}.json', outcomes)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    try:
        raise RuntimeError('use reviewed run-round.py with an actual bounded START')
    except Exception as error:
        write(args.root / 'public/measure-failure.json', failure_record(error, 'measure', **CURRENT))
        sys.exit(1)
