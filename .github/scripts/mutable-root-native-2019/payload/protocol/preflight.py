"""Untimed full-output verification; records evidence, never starts timing."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

results = Path(__file__).resolve().parent
from protocol_guard import require_reviewed_pair
reviewed_plan=require_reviewed_pair(results, live=True)
out = Path('/tmp/nm2019-three-arm-evidence')
arms_root = Path('/tmp/nm2019-current-main-pair-evidence')
root = out/'inputs'
plan = json.loads((out/'workload-plan-draft.json').read_text())
manifests = {arm:json.loads((arms_root/arm/'manifest.json').read_text()) for arm in ('ARM_BASE', 'ARM_CAND')}
assert manifests['ARM_BASE']['revision']=='e5ae00bd6353a28045a5344f4accac411a2ccd74'
assert manifests['ARM_CAND']['revision']==json.loads((results/'measurement-plan.json').read_text())['candidate_revision']
for arm,manifest in manifests.items():
    for filename,key in (('probe','binary_sha256'),('lib.rlib','library_sha256'),
                         ('source-manifest.json','source_manifest_sha256')):
        assert hashlib.sha256((arms_root/arm/filename).read_bytes()).hexdigest() == manifest[key], (arm,filename)
    profile = manifest['compiler_artifact']['profile']
    assert profile['opt_level'] == '3' and profile['debuginfo'] == 0 and not profile['test']
    assert profile['debug_assertions'] is False and profile['overflow_checks'] is False
    current_harness = [{'path':path.name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
                       for path in sorted((out/'harness').glob('*.rs'))]
    assert manifest['harness'] == current_harness, (arm,'harness changed')
for key in ('rustc','features','release_profile','harness'):
    assert len({json.dumps(manifest[key],sort_keys=True) for manifest in manifests.values()}) == 1, key
assert len({manifest['binary_sha256'] for manifest in manifests.values()}) == 2
# Only own package release metadata may differ; no third-party normalization.
import tomllib
lockdir = results/'lockfiles'
lockdir.mkdir(exist_ok=False)
parsed, lock_records = {}, {}
for arm, manifest in manifests.items():
    raw = subprocess.check_output(['git', 'show', f"{manifest['revision']}:Cargo.lock"], cwd=os.environ['GITHUB_WORKSPACE'])
    assert hashlib.sha256(raw).hexdigest() == manifest['lock_sha256']
    (lockdir/f'{arm}.Cargo.lock').write_bytes(raw)
    data = tomllib.loads(raw.decode())
    own = [package for package in data['package'] if package['name'] == 'no-mistakes']
    assert len(own) == 1
    own_version = own[0]['version']
    assert own_version == {'ARM_BASE': reviewed_plan['baseline_version'], 'ARM_CAND': reviewed_plan['candidate_version']}[arm]
    own[0]['version'] = '<own-package-version>'
    parsed[arm] = data
    lock_records[arm] = dict(raw_sha256=manifest['lock_sha256'], own_version=own_version)
assert lock_records['ARM_BASE']['raw_sha256'] == lock_records['ARM_CAND']['raw_sha256'], 'Fresh pair raw lock mismatch'
assert parsed['ARM_BASE'] == parsed['ARM_CAND'], 'third-party dependencies changed'
(results/'lock-parity.json').write_text(json.dumps(dict(status='PASS: exact approved raw locks, explicit actual own versions and complete third-party identity', arms=lock_records, third_party_exact=True), indent=2)+'\n')
for row in json.loads((out/'input-manifest.json').read_text())['files']:
    assert hashlib.sha256((root/row['path']).read_bytes()).hexdigest() == row['sha256']
private = results/'semantic-private'
assert not private.exists(), 'Preserve earlier semantic evidence'
private.mkdir()
controls = {
 'no_writes':'test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts',
 'writes':'test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts',
 'shared_histories':'test-cases/codebase-analysis/import-facts/fixture/many-late-defaults.mts',
 'jsx':'test-cases/ast-snippets/selectors/fixture/app-text-targets.tsx',
}
facts = [(case,demand,path) for case,path in controls.items() for demand in ('values','imports')]
facts += [(Path(name).stem,'values',plan['extraction_hot_directory']+'/'+name)
          for name in plan['extraction_hot_files']]
records = {}
prior = {'ARM_BASE': 'A22', 'ARM_CAND': 'B23'}
prior_dirs = {arm:Path('/tmp/nm2019-arm2019-expected-prior') for arm in prior}
previous = {arm:json.loads((directory/'semantic-manifest.json').read_text()) for arm,directory in prior_dirs.items()}
def old_graph(arm, mode):
    raw = (prior_dirs[arm]/f'semantic-private/graph-{mode}-{prior[arm]}.stdout').read_bytes()
    data = json.loads(raw.replace(str(root).encode(), b'<inputs>'))
    return data['semantic']

def run(arm,label,args):
    result = subprocess.run([str(arms_root/arm/'probe'),*args],capture_output=True,
                            env=dict(os.environ,RAYON_NUM_THREADS='4'),check=False)
    (private/f'{label}-{arm}.stdout').write_bytes(result.stdout)
    (private/f'{label}-{arm}.stderr').write_bytes(result.stderr)
    assert result.returncode == 0 and not result.stderr, (arm,label,result.returncode)
    data = result.stdout.replace(str(root).encode(), b'<inputs>')
    records.setdefault(label,{})[arm] = {'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
    return data

for case,demand,path in facts:
    outputs = {arm:run(arm,f'facts-{case}-{demand}',[str(root/path),demand,'1','dump']) for arm in ('ARM_BASE', 'ARM_CAND')}
    assert all(records[f'facts-{case}-{demand}'][arm] == previous[arm]['outputs'][f'facts-{case}-{demand}'][prior[arm]] for arm in prior), (case,demand,'full prior-arm facts changed')
    print('fact parity immediateB23 ARM_BASE/ARM_BASE and B23/ARM_CAND:',case,demand,flush=True)
graph_root = root/plan['graph']['root']
graphs = {}
for mode in ('off','work'):
    graphs[mode] = {arm:json.loads(run(arm,f'graph-{mode}',[str(graph_root),'1',mode,'calls'])) for arm in ('ARM_BASE', 'ARM_CAND')}
    assert all(graphs[mode][arm]['semantic'] == old_graph(arm, mode) for arm in prior), 'full prior-arm graph changed'
for arm in ('ARM_BASE', 'ARM_CAND'):
    assert graphs['off'][arm]['semantic'] == graphs['work'][arm]['semantic'], 'observer changed graph'
identity = lambda graph:[(site['file'],site['offset'],site['caller_id'],site['invocation'],site['callee'])
                        for site in graph['semantic']['sites']]
assert identity(graphs['off']['ARM_BASE']) == identity(graphs['off']['ARM_CAND']), 'ARM_BASE/ARM_CAND changed physical call identity'
no_call_graphs = {}
for arm in ('ARM_BASE', 'ARM_CAND'):
    no_calls=json.loads(run(arm,'graph-no-calls',[str(graph_root),'1','off','no-calls']))
    no_call_graphs[arm] = no_calls['semantic']
    assert not no_calls['semantic']['sites']
    assert no_calls['semantic']['nodes'] and no_calls['semantic']['edges']
assert all(no_call_graphs[arm] == old_graph(arm, 'no-calls') for arm in prior), 'prior-arm non-consuming graph changed'
for index,entry in enumerate(plan['effects']['entries']):
    outputs={arm:run(arm,f'effects-{index}', ['verify',str(graph_root),plan['effects']['kind'],entry,'1']) for arm in ('ARM_BASE', 'ARM_CAND')}
    assert all(records[f'effects-{index}'][arm] == previous[arm]['outputs'][f'effects-{index}'][prior[arm]] for arm in prior), 'full prior-arm effects changed'
    print('effects parity immediateB23 ARM_BASE/ARM_BASE and B23/ARM_CAND:',entry,flush=True)
(results/'semantic-manifest.json').write_text(json.dumps({'status':'Full outputs retained for independent review; immediateB23 ARM_BASE/ARM_BASE and B23/ARM_CAND parity must be independently reviewed before timing',
 'source_revisions':{arm:manifest['revision'] for arm,manifest in manifests.items()},'outputs':records, 'prior_arm_carry':prior, 'prior_manifests':{arm:{'path':str(prior_dirs[arm]/'semantic-manifest.json'),'sha256':hashlib.sha256((prior_dirs[arm]/'semantic-manifest.json').read_bytes()).hexdigest()} for arm in prior}, 'intended_delta_review_reference':str(out/'semantic-diff-A-B.json'),
 'assertions':['all13 full fact snapshots per prior arm exact','full graph immediateB23 ARM_BASE/ARM_BASE and B23/ARM_CAND exact','graph observer/off semantic exact per arm',
               'ARM_BASE/ARM_CAND physical call identity exact','non-consuming graph has nodes/edges and no calls',
               'all3 full effects reports immediateB23 ARM_BASE/ARM_BASE and B23/ARM_CAND exact'],
 'effects_observer_limit':'verify emits observer-off full reports; observer collection timed separately; count alone is not full semantic proof'},indent=2)+'\n')
assert len(records) == 19 and sum(len(arms) for arms in records.values()) == 38
print('Untimed full ARM_BASE/ARM_CAND outputs and independent prior-arm carry complete; root semantic review required before timing.',flush=True)
