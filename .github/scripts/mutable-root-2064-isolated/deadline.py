"""Charge setup and controls to the actual job clock; reserve five minutes."""
import datetime
import json
import os
from pathlib import Path
import subprocess
import time
from process_owner import OwnedProcess
import subreaper


def install(output, expected_name):
    before = time.monotonic()
    argv = ['gh', 'api', '--paginate', '--slurp',
            f'repos/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{os.environ["GITHUB_RUN_ID"]}/attempts/{os.environ["GITHUB_RUN_ATTEMPT"]}/jobs?per_page=100']
    with (output / 'job-clock.stdout').open('wb') as out, (output / 'job-clock.stderr').open('wb') as err:
        process = owner = None
        cleanup = []
        receipt = {'argv': argv, 'exit': None, 'owned_identity': None,
                   'owned_cleanup': [], 'cleanup_complete': False}
        try:
            subreaper.require_empty()
            process = subprocess.Popen(argv, stdout=out, stderr=err, start_new_session=True)
            owner = OwnedProcess(process)
            receipt['owned_identity'] = owner.initial
            (output / 'job-clock-command.json').write_text(json.dumps(receipt, indent=2))
            process.wait(timeout=30)
        finally:
            try:
                if owner is not None:
                    cleanup = subreaper.finish(process, owner)
                    receipt['cleanup_complete'] = True
            finally:
                receipt.update(exit=process.returncode if process else None,
                               lookup_seconds=time.monotonic() - before,
                               owned_cleanup=cleanup)
                (output / 'job-clock-command.json').write_text(json.dumps(receipt, indent=2))
    (output / 'job-clock-command.json').write_text(json.dumps(receipt, indent=2))
    assert process.returncode == 0
    pages = json.loads((output / 'job-clock.stdout').read_text())
    jobs = [job for page in pages for job in page['jobs'] if job['name'] == expected_name]
    assert len(jobs) == 1, 'Actual job clock must identify exactly one matrix job'
    job = jobs[0]
    assert job['status'] == 'in_progress'
    start = datetime.datetime.fromisoformat(job['started_at'].replace('Z', '+00:00')).timestamp()
    elapsed = time.time() - start
    assert 0 <= elapsed < 1500, 'Setup exhausted the whole operation budget'
    record = {'job_id': job['id'], 'started_at': job['started_at'], 'observed_elapsed': elapsed,
              'operation_budget_seconds': 1500, 'job_budget_seconds': 1800, 'transfer_reserve_seconds': 300}
    (output / 'actual-job-clock.json').write_text(json.dumps(record, indent=2))
    return time.monotonic() + 1500 - elapsed


def remaining(deadline, requested):
    value = min(requested, deadline - time.monotonic())
    assert value > 0, 'Actual job operation deadline exhausted; preserve partial outputs'
    return value
