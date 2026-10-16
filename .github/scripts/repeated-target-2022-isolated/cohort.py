"""One complete-workload cohort on one isolated VM; no cross-VM statistics."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

D = Path(__file__).resolve().parent


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def execute(plan, root, cohort, job_deadline):
    from contracts import approved
    approved(plan, os.environ)
    assert 0 <= cohort < 5 and root.is_dir() and not root.is_symlink()
    selected = plan['rounds'][cohort]
    assert selected['id'] == cohort
    sys.path.insert(0, str(D / 'protocol'))
    import measure
    from common import identity
    original = measure.SPEC
    workloads = [original['workloads'][i] for i in selected['indices']]
    assert [w['name'] for w in workloads] == selected['workloads']
    proof = json.loads((root / 'public/hosted-preflight.json').read_text())
    assert proof['status'] == 'PASS actual fresh VM controls'
    assert proof['semantic_raw_files'] == 182 and proof['semantic_groups_per_arm'] == 27
    assert proof['sql_conditional_passes_per_arm'] == 1 and proof['sql_source_passes_per_arm'] == 34
    assert proof['skipped'] == 0 and proof['layout_records'] == 24
    assert proof['run_id'] == os.environ['GITHUB_RUN_ID']
    assert proof['run_attempt'] == os.environ['GITHUB_RUN_ATTEMPT']
    assert proof['workflow_sha'] == os.environ['GITHUB_SHA']
    for row in proof['pins']:
        assert digest(row['path']) == row['sha256']
    # The original measure function checks CPU quota, memory, affinity, Rayon,
    # binary manifests, inputs, and per-sample full output signatures itself.
    before = identity(root)
    assert job_deadline - time.monotonic() >= 600 + 120
    started = time.monotonic()
    start = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
             'run_id': os.environ['GITHUB_RUN_ID'], 'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
             'workflow_sha': os.environ['GITHUB_SHA'], 'cohort': cohort,
             'indices': selected['indices'], 'preflight_sha256': digest(root / 'public/hosted-preflight.json')}
    (root / 'public/cohort-START.json').write_text(json.dumps(start, indent=2))
    measure.SPEC = {**original, 'workloads': workloads}
    measure.measure(root, min(started + 600, job_deadline - 120), cohort)
    elapsed = time.monotonic() - started
    after = identity(root)
    assert elapsed <= 600
    for key in ['allowed_cpus', 'selected_cpus', 'cpu_quota', 'architecture', 'kernel', 'cpu_model', 'image_os', 'image_version', 'rayon_workers']:
        assert before[key] == after[key]
    limits = lambda snapshot: [{k: r[k] for k in ['version', 'ancestor_distance', 'limit_bytes']} for r in snapshot['visible_cgroup_memory_limits']]
    assert limits(before) == limits(after)
    for row in proof['pins']:
        assert digest(row['path']) == row['sha256']
    # Completeness is checked independently by union.py; failure and adverse
    # assessment files remain present and are never converted into PASS.
    (root / 'public/cohort-END.json').write_text(json.dumps({**start,
        'status': 'COMPLETE invocation; numerical and row eligibility separate',
        'elapsed_s': elapsed, 'resources_before': before, 'resources_after': after,
        'end_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--cohort', type=int, required=True)
    parser.add_argument('--job-deadline', type=float, required=True)
    args = parser.parse_args()
    execute(json.loads((D / 'plan.json').read_text()), args.root, args.cohort, args.job_deadline)
