"""Read-only hosted controller; privileged job contains no repository execution."""
import atexit
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

from process_owner import OwnedProcess
import subreaper

D = Path(__file__).resolve().parent
O = Path(os.environ['RUNNER_TEMP']) / 'repeated-target-2022-isolated'
O.mkdir(exist_ok=False)

def retain_partial():
    from retention import inventory
    root = Path('/tmp/nm2022-main281-current-pair-functional/current-pair-semantics')
    (O / 'retained-evidence-index.json').write_text(json.dumps({'scope': 'Only public/private/raw semantic evidence; partial on failure', 'files': inventory(root)}, indent=2))

atexit.register(retain_partial)
subreaper.enable()
def source_pins():
    for row in json.loads((D / 'source-manifest.json').read_text())['files']:
        path = D / row['path']
        assert path.is_file() and not path.is_symlink() and path.stat().st_size == row['bytes']
        with path.open('rb') as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest() == row['sha256']

source_pins()
plan = json.loads((D / 'plan.json').read_text())
from contracts import approved, transfer
approved(plan, os.environ)
cohort = int(os.environ['COHORT'])
assert 0 <= cohort < 8
ledger = []
deadline = time.monotonic() + 1080  # Replaced with stricter actual job-start bound below.


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def command(argv, label, seconds=60, env=None, cwd=None):
    subreaper.require_empty()
    process = owner = None
    row = {'argv': argv, 'label': label, 'exit': None}
    started = time.monotonic()
    try:
        with (O / (label + '.stdout')).open('wb') as out, (O / (label + '.stderr')).open('wb') as err:
            process = subprocess.Popen(argv, stdout=out, stderr=err, env=env, cwd=cwd, start_new_session=True)
            owner = OwnedProcess(process)
            while process.poll() is None:
                assert time.monotonic() < min(deadline, started + seconds)
                time.sleep(.05)
            row['exit'] = process.wait()
            assert row['exit'] == 0 and time.monotonic() < min(deadline, started + seconds)
    finally:
        try:
            if owner:
                row['cleanup'] = subreaper.finish(process, owner)
        finally:
            row['exit'] = process.returncode if process else None
            row['elapsed_s'] = time.monotonic() - started
            ledger.append(row)
            (O / 'ledger.json').write_text(json.dumps(ledger, indent=2))


def api(endpoint, label):
    command(['gh', 'api', endpoint], label)
    return json.loads((O / (label + '.stdout')).read_text())


jobs = api(f"repos/jonathanong/no-mistakes/actions/runs/{os.environ['GITHUB_RUN_ID']}/attempts/{os.environ['GITHUB_RUN_ATTEMPT']}/jobs?per_page=100", 'actual-jobs')
assert jobs['total_count'] <= 100
matches = [j for j in jobs['jobs'] if j['name'] == f'Complete workload cohort {cohort}']
assert len(matches) == 1
job = matches[0]
started_at = datetime.datetime.fromisoformat(job['started_at'].replace('Z', '+00:00'))
remaining = 1800 - (datetime.datetime.now(datetime.timezone.utc) - started_at).total_seconds() - 420
assert remaining > 720
deadline = min(deadline, time.monotonic() + remaining)
(O / 'actual-job-budget.json').write_text(json.dumps({'job': job, 'remaining_s': remaining, 'retention_reserve_s': 360, 'cleanup_reserve_s': 60}, indent=2))


def eligibility(suffix):
    main = api('repos/jonathanong/no-mistakes/git/ref/heads/main', 'main-' + suffix)
    pr = api('repos/jonathanong/no-mistakes/pulls/2022', 'pr-' + suffix)
    ci = api('repos/jonathanong/no-mistakes/actions/runs/' + str(plan['candidate_successful_ci_run']), 'ci-' + suffix)
    assert main['object']['sha'] == plan['baseline_equivalent_main']
    assert pr['head']['sha'] == plan['candidate_current_source'] and pr['draft'] and pr['state'] == 'open'
    assert ci['head_sha'] == plan['candidate_current_source'] and ci['status'] == 'completed' and ci['conclusion'] == 'success'
    checks = api(f"repos/jonathanong/no-mistakes/commits/{plan['candidate_current_source']}/check-runs?per_page=100", 'checks-' + suffix)
    assert checks['total_count'] <= 100
    for name in ['codecov/patch/rust', 'codecov/project/rust', 'codecov/patch/javascript', 'codecov/project/javascript']:
        rows = [r for r in checks['check_runs'] if r['name'] == name]
        assert rows
        latest = max(rows, key=lambda r: r['id'])
        assert latest['status'] == 'completed' and latest['conclusion'] == 'success'


eligibility('before')
assets = json.loads((D / 'assets.json').read_text())
stage = Path(os.environ['RUNNER_TEMP']) / 'native-stage'
proof = transfer(stage, assets, os.environ)
(O / 'verified-same-run-transfer.json').write_text(json.dumps(proof, indent=2))
G = Path('/tmp/no-mistakes-2022-region-production')
assert not G.exists()
# Historical source-closure snapshot: restore authenticated frozen bytes only.
# The separate CURRENT_GIT checkout below owns all live Git/facade queries.
CURRENT_GIT = Path('/tmp/no-mistakes-2022-assignment-target-opt')
assert not CURRENT_GIT.exists()
command(['git', 'clone', '--shared', '--no-checkout', str(Path('candidate-source').resolve()), str(CURRENT_GIT)], 'current-pair-local-git-clone')
command(['git', 'checkout', '--detach', plan['candidate_actual_compiler']], 'current-pair-producer-checkout', cwd=CURRENT_GIT)
for row in json.loads((D / 'payload-index.json').read_text()):
    path = Path(row['path'])
    assert str(path).startswith('/tmp/') and '..' not in path.parts and '.git' not in path.parts
    assert not any(parent.is_symlink() for parent in [path, *path.parents])
    payload = D / 'payload' / row['payload']
    assert payload.parent == D / 'payload' and sha(payload) == row['sha256'] and payload.stat().st_size == row['bytes']
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        assert sha(path) == row['sha256']
    else:
        shutil.copyfile(payload, path)
for row in assets:
    from contracts import asset_role
    arm, name = asset_role(row)
    if arm == 'candidate':
        destination = Path('/tmp/nm2022-combined-release-source-draft/candidate') / name
    elif name in ['cli', 'addon.node']:
        destination = Path('/tmp/nm2076-cold-lifetime-release/candidate') / name
    else:
        destination = Path('/tmp/nm-main281-shared-baseline') / (name + '-2022')
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(stage / row['name'], destination)
    destination.chmod(0o755)
clean = {k: v for k, v in os.environ.items() if k not in ['GH_TOKEN', 'GITHUB_TOKEN', 'ACTIONS_RUNTIME_TOKEN', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN', 'ACTIONS_ID_TOKEN_REQUEST_URL']}
assert subprocess.check_output(['node', '--version'], text=True).strip() == 'v26.11.1'
command(['python3', '-m', 'unittest', 'discover', '-s', str(D), '-p', 'test_*.py'], 'source-boundaries', 60, clean)
command(['python3', str(D / 'preflight.py')], 'fresh-182-plus-sql', 610, clean)
eligibility('before-sampling')
root = Path('/tmp/nm2022-main281-current-pair-functional/current-pair-semantics')
assert deadline - time.monotonic() >= 720
command(['python3', str(D / 'cohort.py'), '--root', str(root), '--cohort', str(cohort), '--job-deadline', str(deadline)], 'fixed-complete-cohort', 605, clean)
eligibility('after')
for row in json.loads((D / 'payload-index.json').read_text()):
    assert sha(row['path']) == row['sha256']
for row in assets:
    assert sha(stage / row['name']) == row['sha256']
source_pins()
# The workflow uploads this directory and only functional/sample outputs, never
# the local Git store, credentials, environment dumps or executable files.
(O / 'closure.json').write_text(json.dumps({'status': 'COMPLETE invocation; union and numerical verdict separate', 'cohort': cohort, 'source_pins_after_exact': True, 'within_job_budget': time.monotonic() < deadline}, indent=2))
