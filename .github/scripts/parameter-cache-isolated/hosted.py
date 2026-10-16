"""One hosted invocation; downloads reviewed assets, verifies, controls, measures."""
import datetime
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import time

from process_owner import OwnedProcess

D = pathlib.Path(__file__).resolve().parent
O = pathlib.Path(os.environ['RUNNER_TEMP']) / 'parameter-cache-isolated'
O.mkdir(exist_ok=False)
deadline = time.monotonic() + 1260
ledger = []


def sha(path):
    return hashlib.file_digest(pathlib.Path(path).open('rb'), 'sha256').hexdigest()


def run(argv, label, seconds=120, env=None, cwd=None):
    process = owner = None
    record = {'argv': argv, 'label': label, 'exit': None}
    try:
        with (O / (label + '.stdout')).open('wb') as out, (O / (label + '.stderr')).open('wb') as err:
            process = subprocess.Popen(argv, stdout=out, stderr=err, env=env, cwd=cwd, start_new_session=True)
            owner = OwnedProcess(process)
            while process.poll() is None:
                assert time.monotonic() < min(deadline, started + seconds)
                time.sleep(.05)
            record['exit'] = process.wait()
            assert process.returncode == 0
            assert time.monotonic() < min(deadline, started + seconds)
    finally:
        if owner:
            owner.finish()
            assert not owner.members()
        ledger.append(record)
        (O / 'ledger.json').write_text(json.dumps(ledger, indent=2))


def command(argv, label, seconds=120, env=None, cwd=None):
    global started
    started = time.monotonic()
    run(argv, label, seconds, env, cwd)


spec = json.loads((D / 'transport-REVIEWED.json').read_text())
assert spec['status'] == 'REVIEWED_ISOLATED_HOSTED'
assert spec['repository'] == os.environ['GITHUB_REPOSITORY'] == 'jonathanong/no-mistakes'
assert platform.machine() == 'x86_64' and {0, 1, 2, 3} <= os.sched_getaffinity(0)
assert subprocess.check_output(['node', '--version'], text=True).strip() == spec['node_version']
assert not pathlib.Path(spec['source_root']).exists()
assert subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd='candidate-source', text=True).strip() == spec['candidate_tree']
shutil.copytree('candidate-source', spec['source_root'], ignore=shutil.ignore_patterns('.git'), symlinks=True)
assert len(spec['files']) == 97 and len(spec['assets']) == 4
assert spec['assets'] == [r for r in spec['files'] if 'asset_id' in r]
assert len({r['path'] for r in spec['files']}) == 97
for row in spec['files']:
    target = pathlib.Path(row['path'])
    assert str(target).startswith('/tmp/') and '..' not in target.parts
    target.parent.mkdir(parents=True, exist_ok=True)
    if 'payload' in row:
        source = D / 'payload' / row['payload']
        assert source.parent == D / 'payload'
        shutil.copyfile(source, target)
    else:
        assert type(row['asset_id']) is int and row['asset_id'] > 0
        command(['gh', 'api', '--header', 'Accept: application/octet-stream',
                 f"repos/{spec['repository']}/releases/assets/{row['asset_id']}"],
                'download-' + row['name'], 190)
        shutil.copyfile(O / ('download-' + row['name'] + '.stdout'), target)
        target.chmod(0o755)
    assert target.stat().st_size == row['bytes'] and sha(target) == row['sha256']
freeze = json.loads((D / 'payload/original-freeze.json').read_text())
assert freeze['files'] == [{k: r[k] for k in ['path', 'bytes', 'sha256']} for r in spec['files']]
F = pathlib.Path(spec['frozen_plan']).parent
assert sha(spec['frozen_plan']) == freeze['plan_sha256']
for p, h in json.loads(pathlib.Path(spec['frozen_plan']).read_text())['physical_inputs'].items():
    assert sha(p) == h
host = {'platform': platform.platform(), 'affinity': sorted(os.sched_getaffinity(0)),
        'cpuinfo': pathlib.Path('/proc/cpuinfo').read_text(), 'meminfo': pathlib.Path('/proc/meminfo').read_text(),
        'image': {k: os.environ.get(k) for k in ['ImageOS', 'ImageVersion', 'RUNNER_ARCH', 'GITHUB_RUN_ID', 'GITHUB_SHA']},
        'producer_labels_unchanged': True, 'no_local_quiet_lease': True}
(O / 'host-provenance.json').write_text(json.dumps(host, indent=2))
eligibility_index = 0

def eligibility():
    global eligibility_index
    eligibility_index += 1
    suffix = str(eligibility_index)
    command(['gh', 'api', 'repos/jonathanong/no-mistakes/git/ref/heads/main'], 'live-main-' + suffix)
    command(['gh', 'api', 'repos/jonathanong/no-mistakes/pulls/2086'], 'live-pr-' + suffix)
    main = json.loads((O / ('live-main-' + suffix + '.stdout')).read_text())
    pr = json.loads((O / ('live-pr-' + suffix + '.stdout')).read_text())
    command(['gh', 'api', 'repos/jonathanong/no-mistakes/actions/runs/38941195465'], 'live-ci-' + suffix)
    ci = json.loads((O / ('live-ci-' + suffix + '.stdout')).read_text())
    assert ci['head_sha'] == spec['candidate'] and ci['status'] == 'completed' and ci['conclusion'] == 'success'
    command(['gh', 'api', f"repos/jonathanong/no-mistakes/commits/{spec['candidate']}/check-runs?per_page=100"], 'live-checks-' + suffix)
    checks = json.loads((O / ('live-checks-' + suffix + '.stdout')).read_text())
    assert checks['total_count'] <= 100
    for name in ['codecov/patch/rust', 'codecov/project/rust', 'codecov/patch/javascript', 'codecov/project/javascript']:
        matches = [c for c in checks['check_runs'] if c['name'] == name]
        assert len(matches) == 1 and matches[0]['status'] == 'completed' and matches[0]['conclusion'] == 'success'
    assert main['object']['sha'] == spec['main']
    assert pr['head']['sha'] == spec['candidate'] and pr['draft'] is True and pr['state'] == 'open'



eligibility()
command(['ldd', '--version'], 'glibc')
for arm in ['baseline', 'candidate']:
    artifact = pathlib.Path(json.loads(pathlib.Path(spec['frozen_plan']).read_text())['artifacts'][arm])
    command(['ldd', str(artifact / 'cli')], arm + '-cli-dependencies')
    command(['ldd', str(artifact / 'addon.node')], arm + '-addon-dependencies')
controls = O / 'controls'
controls.mkdir()
for name in ['functional-once.py', 'node_test_counts.py', 'public-reports.cjs', 'process_owner.py']:
    shutil.copyfile(D / name, controls / name)
shutil.copyfile(spec['frozen_plan'], controls / 'performance-source-plan.json')
clean = os.environ.copy()
clean.pop('GH_TOKEN', None)
command(['python3', str(controls / 'functional-once.py')], 'fresh-host-functional-controls', 610, clean)
for arm in ['baseline', 'candidate']:
    assert (controls / 'functional' / (arm + '-prepared-result.json')).read_bytes() == (F / 'functional' / (arm + '-prepared-result.json')).read_bytes()
for workload in json.loads(pathlib.Path(spec['frozen_plan']).read_text())['workloads']:
    for arm in ['baseline', 'candidate']:
        name = arm + '-' + workload['id'] + '.stdout'
        assert (controls / 'functional' / name).read_bytes() == (pathlib.Path(json.loads(pathlib.Path(spec['frozen_plan']).read_text())['functional_golden_paths'][workload['id']]) / name).read_bytes()



eligibility()
M = O / 'round'
M.mkdir()
for name in ['measure-once.py', 'analyze-once.py', 'process_owner.py']:
    shutil.copyfile(D / name, M / name)
shutil.copyfile(spec['frozen_plan'], M / 'performance-source-plan.json')
shutil.copyfile(D / 'payload/original-freeze.json', M / 'measurement-freeze.json')
(M / 'functional').mkdir()
shutil.copyfile(controls / 'functional/actual-functional-result.json', M / 'functional/actual-functional-result.json')
assert deadline - time.monotonic() >= 600 + 120, 'Insufficient outer budget for complete sampling and final assessment'
start = {'status': 'ACTUAL START on isolated hosted VM after current pair and host controls',
         'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'plan_sha256': freeze['plan_sha256'],
         'actual_main': spec['main'], 'actual_head': spec['candidate'], 'actual_draft': True,
         'full_current_head_ci_success': True, 'host_provenance_sha256': sha(O / 'host-provenance.json'),
         'adapter_sha256': sha(D / 'measure-once.py'), 'isolated_vm': True}
(M / 'measurement-START.json').write_text(json.dumps(start, indent=2))
command(['python3', str(M / 'measure-once.py')], 'fixed-256-rows', 600, clean)
eligibility()
command(['python3', str(M / 'analyze-once.py')], 'unchanged-assessor', 120, clean)
for row in spec['files']:
    assert sha(row['path']) == row['sha256']
assert time.monotonic() < deadline
(O / 'closure.json').write_text(json.dumps({'status': 'COMPLETE; inspect unchanged numerical gates', 'rows': 256,
    'transport_sha256': sha(D / 'transport-REVIEWED.json'), 'sampler_sha256': sha(M / 'measure-once.py'),
    'assessor_sha256': sha(M / 'analyze-once.py'), 'no_historical_rows': True}, indent=2))
