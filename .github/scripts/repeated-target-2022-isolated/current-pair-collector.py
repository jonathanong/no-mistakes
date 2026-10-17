"""Untimed historical637 release versus actual prospective2087-integrated release candidate functional reports."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

CLOSURE_PATH=Path('/tmp/nm2022-main5de-integration/dev-semantic-source-closure.json')
assert hashlib.sha256(CLOSURE_PATH.read_bytes()).hexdigest()=='9341cbcf7d7940035a4bec7d7bc69fe6bcbb0efff435e3504f69c13bb8d946f9'
CLOSURE=json.loads(CLOSURE_PATH.read_text())
def verify_source_closure():
    assert hashlib.sha256(CLOSURE_PATH.read_bytes()).hexdigest()=='9341cbcf7d7940035a4bec7d7bc69fe6bcbb0efff435e3504f69c13bb8d946f9'
    for row in CLOSURE['files']:
        path=Path(row['path'])
        assert path.stat().st_size==row['bytes'] and hashlib.sha256(path.read_bytes()).hexdigest()==row['sha256'],str(path)
verify_source_closure()

PROPOSAL = Path('/tmp/nm2022-next-arm-proposal-v4')
sys.path.insert(0, str(PROPOSAL))
import semantics  # noqa: E402
import napi_parity  # noqa: E402
from pair import SPEC  # noqa: E402

GIT = Path('/tmp/no-mistakes-2022-assignment-target-opt')
semantics.ROOT = GIT
napi_parity.ROOT = GIT
ROOT = Path('/tmp/nm2022-main281-current-pair-functional/current-pair-semantics')
ROOT.mkdir(parents=True, exist_ok=False)
RUNTIME = ROOT / 'runtime'
RUNTIME.mkdir(mode=0o700)
os.environ['XDG_RUNTIME_DIR'] = str(RUNTIME)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


manifest = {'status':'incomplete','purpose':'UNTIMED actual main281-equivalent baseline099+sealed lifetime overlay versus fresh e193 candidate (public4a test-only carry); properLTO helpers. No performance claim.','pair':{'baseline':{'actual_producer':'099cfed26a23050aee6e9e8427af4de9b7eab4ef','overlay':'68c36c8537b61d863ab9242c65bcb3c4c5af20eceb649b8e55a4b158d3f4bcd4','equivalent_main':'281673146341e6915c051aedfde6fd05ae6dd055'},'candidate':{'actual_producer':'e193aeca201a346115ebe0ee552d6cb1ec995447','public_source':'4a322424894b3d5f965d2e04fad808baa13da392'}},'report_hashes':{k:{} for k in ['graph','cli','probe','napi']},'raw_files':{},'adapter_lock_notice_counts':{},'xdg_runtime_dir':str(RUNTIME),'failures':[],'external_source_closure_sha256':hashlib.sha256(CLOSURE_PATH.read_bytes()).hexdigest()}
write(ROOT / 'semantic-manifest.json', manifest)
private = ROOT / 'private'
try:
    build = Path('/tmp/nm2022-final-cost-native-source-draft/candidate')
    assert json.loads((build/'cleanup.json').read_text()) == {'target_absent': True, 'source_after_exact': True}
    baseline = Path('/tmp/nm2076-cold-lifetime-release/candidate')
    baseline_helpers = Path('/tmp/nm-main281-shared-baseline')
    actual_helpers = json.loads((baseline_helpers/'actual.json').read_text())
    for arm in ('baseline', 'candidate'):
        artifact_dir = baseline if arm == 'baseline' else build
        provenance = json.loads((artifact_dir/'closed-artifacts.json').read_text())
        assert provenance['head'] == ('099cfed26a23050aee6e9e8427af4de9b7eab4ef' if arm == 'baseline' else 'e193aeca201a346115ebe0ee552d6cb1ec995447')
        transfer = ROOT/arm/'transfer'
        transfer.mkdir(parents=True)
        for name in ('cli', 'addon.node', 'graph-probe', 'timing-probe'):
            key = 'addon' if name == 'addon.node' else name
            if arm == 'baseline' and name.endswith('probe'):
                artifact = baseline_helpers/(name+'-2022')
                records = [r for r in actual_helpers['ledger'] if r['label'] == name+'-2022']
                assert len(records) == 1 and records[0]['exit'] == 0
                assert sha(artifact) == records[0]['sha256'] and artifact.stat().st_size == records[0]['bytes']
                assert 'lto=true' in records[0]['argv'] and 'codegen-units=1' in records[0]['argv']
                manifest.setdefault('baseline_helper_actual_receipt', {})['sha256'] = sha(baseline_helpers/'actual.json')
            elif arm == 'candidate' and name.endswith('probe'):
                helper_dir = Path('/tmp/nm2022-final-cost-helper-links')
                helper_receipt = json.loads((helper_dir/'receipt.json').read_text())
                assert helper_receipt['status'].startswith('PASS proper-LTO helpers only') and helper_receipt['head'] == 'e193aeca201a346115ebe0ee552d6cb1ec995447'
                records = [r for r in helper_receipt['commands'] if r['label'] == name]
                assert len(records) == 1 and records[0]['exit'] == 0
                artifact = helper_dir/name
                assert sha(artifact) == records[0]['artifact']['sha256'] and artifact.stat().st_size == records[0]['artifact']['bytes']
                assert 'lto=true' in records[0]['argv'] and 'codegen-units=1' in records[0]['argv']
            else:
                artifact = artifact_dir/name
                assert sha(artifact) == provenance['artifacts'][key]['sha256']
            (transfer/name).symlink_to(artifact)
    original_input_revision = SPEC['candidate']['revision']
    try:
        # Lookup the exact frozen bytes at the actual public candidate, not an
        # unreachable historical producer object. Binary labels remain explicit.
        SPEC['candidate']['revision'] = '4a322424894b3d5f965d2e04fad808baa13da392'
        semantics.copy_inputs(ROOT)
    finally:
        SPEC['candidate']['revision'] = original_input_revision
    inputs = ROOT / 'inputs'
    for key in SPEC['semantic_policy']['untimed_graph_inputs']:
        input_key = key.split(':', 1)[0]
        source = inputs / SPEC['inputs'][input_key]['path']
        mode = 'no-calls' if key.endswith(':no-calls') else 'calls'
        manifest['report_hashes']['graph'][key] = {}
        for arm in ('baseline', 'candidate'):
            command = [str(ROOT / arm / 'transfer/graph-probe'), str(source), '1', 'off', mode]
            completed = subprocess.run(command, capture_output=True, timeout=120)
            raw = private / 'graphs' / f'{key}-{arm}'
            raw.parent.mkdir(parents=True, exist_ok=True)
            (raw.with_suffix('.stdout')).write_bytes(completed.stdout)
            (raw.with_suffix('.stderr')).write_bytes(completed.stderr)
            assert completed.returncode == 0 and not completed.stderr, (key, arm, completed.returncode)
            graph = semantics.canonical_graph(semantics.normalized(completed.stdout, source))
            if key == 'receiver-order:calls' and arm == 'candidate':
                semantics.receiver_order_controls(graph)
            report = private / 'reports' / 'graph' / f'{key}-{arm}.json'
            write(report, graph)
            manifest['report_hashes']['graph'][key][arm] = semantics.sha(graph)
            manifest['raw_files'][str(report.relative_to(ROOT))] = sha(report)
        write(ROOT / 'semantic-manifest.json', manifest)
    (private / 'cli-raw').mkdir(parents=True, exist_ok=False)
    for workload in SPEC['workloads']:
        key = workload['name']
        source = inputs / SPEC['inputs'][workload['input']]['path']
        category = 'cli' if workload['kind'] == 'cli' else 'probe'
        manifest['report_hashes'][category][key] = {}
        for arm in ('baseline', 'candidate'):
            if category == 'cli':
                report, exit_code = semantics.cli_once(ROOT, workload, arm, source, private / 'cli-raw')
                path = private / 'reports' / 'cli' / f'{key}-{arm}.json'
                write(path, report)
                manifest['report_hashes']['cli'][key][arm] = {'sha256': semantics.sha(report), 'exit': exit_code}
                manifest['raw_files'][str(path.relative_to(ROOT))] = sha(path)
            else:
                command = [str(ROOT / arm / 'transfer/timing-probe'), workload['mode'], str(source), '1']
                completed = subprocess.run(command, capture_output=True, timeout=120)
                raw = private / 'probes' / f'{key}-{arm}'
                raw.parent.mkdir(parents=True, exist_ok=True)
                (raw.with_suffix('.stdout')).write_bytes(completed.stdout)
                (raw.with_suffix('.stderr')).write_bytes(completed.stderr)
                assert completed.returncode == 0 and not completed.stderr and completed.stdout.strip().isdigit(), (key, arm)
                manifest['report_hashes']['probe'][key][arm] = completed.stdout.decode().strip()
        write(ROOT / 'semantic-manifest.json', manifest)
    original_revisions = {arm: SPEC[arm]['revision'] for arm in ('baseline', 'candidate')}
    try:
        SPEC['baseline']['revision'] = '281673146341e6915c051aedfde6fd05ae6dd055'
        SPEC['candidate']['revision'] = '4a322424894b3d5f965d2e04fad808baa13da392'
        napi_parity.facade_identity()
    finally:
        for arm, revision in original_revisions.items(): SPEC[arm]['revision'] = revision
    for case in SPEC['semantic_policy']['napi_cases']:
        prefix = case['prefix']
        fixture = inputs / SPEC['inputs'][case['input']]['path']
        reports = [{'type': 'effects', 'id': f'{prefix}-{i}', 'kind': case['kind'], 'entry': entry}
                   for i, entry in enumerate(case['entries'])]
        reports.append({'type': 'check', 'id': f'{prefix}-check'})
        manifest['report_hashes']['napi'][prefix] = {}
        manifest['adapter_lock_notice_counts'][prefix] = {}
        for arm in ('baseline', 'candidate'):
            addon = ROOT / arm / 'transfer/addon.node'
            env = dict(os.environ, NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(addon))
            command = ['node', str(PROPOSAL / 'napi-parity.mjs'), str(GIT / 'packages/no-mistakes'),
                       str(fixture), json.dumps(reports)]
            completed = subprocess.run(command, env=env, capture_output=True, timeout=120)
            raw = private / 'napi' / f'{prefix}-{arm}'
            raw.parent.mkdir(parents=True, exist_ok=True)
            (raw.with_suffix('.stdout')).write_bytes(completed.stdout)
            (raw.with_suffix('.stderr')).write_bytes(completed.stderr)
            assert completed.returncode == 0, (prefix, arm, completed.returncode)
            manifest['adapter_lock_notice_counts'][prefix][arm] = napi_parity.adapter_lock_notice_count(completed.stderr)
            report = napi_parity.normalized(completed.stdout, fixture)
            assert len(report['reports']) == len(reports)
            for i, entry in enumerate(case['entries']):
                row = report['reports'][i]
                assert row['id'] == f'{prefix}-{i}'
                cli = subprocess.run([str(ROOT / arm / 'transfer/cli'), 'effects', case['kind'],
                                      '--entry', entry, '--root', str(fixture), '--format', 'json'],
                                     capture_output=True, timeout=60)
                (private / 'napi' / f'{prefix}-{arm}-{i}.cli.stdout').write_bytes(cli.stdout)
                (private / 'napi' / f'{prefix}-{arm}-{i}.cli.stderr').write_bytes(cli.stderr)
                assert cli.returncode == 0 and not cli.stderr, (prefix, arm, entry)
                assert row['result'] == napi_parity.normalized(cli.stdout, fixture)
            check = subprocess.run([str(ROOT / arm / 'transfer/cli'), 'check', '--root', str(fixture),
                                    '--format', 'json'], capture_output=True, timeout=60)
            (private / 'napi' / f'{prefix}-{arm}-check.cli.stdout').write_bytes(check.stdout)
            (private / 'napi' / f'{prefix}-{arm}-check.cli.stderr').write_bytes(check.stderr)
            assert check.returncode in (0, 1) and not check.stderr
            assert report['reports'][-1]['id'] == f'{prefix}-check'
            assert report['reports'][-1]['result'] == napi_parity.normalized(check.stdout, fixture)
            assert not report['reports'][-1]['result']['warnings']
            path = private / 'reports' / 'napi' / f'{prefix}-{arm}.json'
            write(path, report)
            manifest['report_hashes']['napi'][prefix][arm] = napi_parity.sha(report)
            manifest['raw_files'][str(path.relative_to(ROOT))] = sha(path)
        write(ROOT / 'semantic-manifest.json', manifest)
    manifest['status'] = 'complete'
except Exception as error:
    manifest['failures'].append({'type': type(error).__name__, 'message': str(error)})
    raise
finally:
    if private.exists():
        manifest['raw_files'] = {str(path.relative_to(ROOT)): sha(path)
                                 for path in sorted(private.rglob('*')) if path.is_file()}
    write(ROOT / 'semantic-manifest.json', manifest)
print('complete', len(manifest['report_hashes']['graph']), len(manifest['report_hashes']['cli']),
      len(manifest['report_hashes']['probe']), len(manifest['report_hashes']['napi']), flush=True)

expected_path = Path('/tmp/nm2022-stage2-native-semantics/collection-v2/semantic-manifest.json')
expected = json.loads(expected_path.read_text())
assert manifest['report_hashes'] == expected['report_hashes'], 'full normalized per-arm semantic groups changed'
write(ROOT / 'reviewed-expected-parity.json', {'status':'PASS all27 normalized groups per arm exactly match reviewed historical expectations', 'actual_new_collection':str(ROOT/'semantic-manifest.json'),'reviewed_expected_manifest':str(expected_path),'reviewed_expected_manifest_sha256':sha(expected_path),'pair':manifest['pair'],'scope':'Complete fresh functional reports; no timing claim; authentic baseline overlay preserved.'})

verify_source_closure()
