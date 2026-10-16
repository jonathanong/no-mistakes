"""Whole-job clock anchored to GitHub's actual started_at, including checkout."""
import datetime
import json
import os
from pathlib import Path
import subprocess
import signal
import time
from common import require, write


def start(root, job_name):
    path = Path(root) / 'public/job-clock.json'
    require(not path.exists(), 'job_clock_cannot_restart')
    run = os.environ['GITHUB_RUN_ID']
    attempt = os.environ['GITHUB_RUN_ATTEMPT']
    argv=['gh','api',f"repos/{os.environ['GITHUB_REPOSITORY']}/actions/runs/{run}/attempts/{attempt}/jobs?per_page=100"]
    process=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    stdout=b'';stderr=b'';error=None
    try:
        stdout,stderr=process.communicate(timeout=30)
    except BaseException as exc:
        error=repr(exc)
        try:os.killpg(process.pid,signal.SIGTERM)
        except ProcessLookupError:pass
        try:stdout,stderr=process.communicate(timeout=2)
        except subprocess.TimeoutExpired:
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            stdout,stderr=process.communicate(timeout=2)
    finally:
        try:os.killpg(process.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        process.wait(timeout=2)
        path.parent.mkdir(parents=True,exist_ok=True)
        (path.parent/'job-bootstrap.stdout').write_bytes(stdout)
        (path.parent/'job-bootstrap.stderr').write_bytes(stderr)
        write(path.parent/'job-bootstrap.json',{'argv':argv,'exit':process.returncode,'error':error,'timeout_seconds':30})
    require(error is None and process.returncode==0,'actual_job_bootstrap_failed')
    data=json.loads(stdout)
    rows = [row for row in data['jobs'] if row['name'] == job_name and row['status'] == 'in_progress']
    require(len(rows) == 1, 'actual_job_identity_ambiguous')
    started = datetime.datetime.fromisoformat(rows[0]['started_at'].replace('Z', '+00:00')).timestamp()
    elapsed = time.time() - started
    require(0 <= elapsed < 1620, 'job_setup_used_upload_reserve')
    # A small clock/network guard reduces, never extends, the engine's 1800s cap.
    deadline = time.monotonic() + 1800 - elapsed - 2
    record = {'run_id': run, 'attempt': attempt, 'job_id': rows[0]['id'],
        'job_name': job_name, 'started_at': rows[0]['started_at'], 'started_unix': started,
        'deadline_monotonic': deadline, 'job_timeout_seconds': 1800}
    write(path, record)
    return record


def clock(root):
    row = json.loads((Path(root) / 'public/job-clock.json').read_text())
    require(row['run_id'] == os.environ['GITHUB_RUN_ID']
            and row['attempt'] == os.environ['GITHUB_RUN_ATTEMPT'], 'foreign_job_clock')
    require(row['job_name']==os.environ['DIAG_JOB_NAME'],'foreign_job_name_clock')
    require(row['job_timeout_seconds'] == 1800, 'job_timeout_changed')
    return row


def remaining(root, reserve=180):
    require(reserve >= 180, 'upload_reserve_cannot_shrink')
    seconds = clock(root)['deadline_monotonic'] - time.monotonic() - reserve
    require(seconds > 0, 'whole_job_deadline_or_upload_reserve')
    return seconds


def sampling_deadline(root):
    require(remaining(root) >= 1500, 'insufficient_complete_sampling_window')
    return time.monotonic() + 1500
