"""Untimed full-output verification; records evidence, never starts timing."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

out = Path(__file__).resolve().parent
root = out/'inputs'
plan = json.loads((out/'workload-plan-draft.json').read_text())
manifests = {arm:json.loads((out/arm/'manifest.json').read_text()) for arm in 'ABC'}
for arm,manifest in manifests.items():
    for filename,key in (('probe','binary_sha256'),('lib.rlib','library_sha256'),
                         ('source-manifest.json','source_manifest_sha256')):
        assert hashlib.sha256((out/arm/filename).read_bytes()).hexdigest() == manifest[key], (arm,filename)
    profile = manifest['compiler_artifact']['profile']
    assert profile['opt_level'] == '3' and profile['debuginfo'] == 0 and not profile['test']
    assert profile['debug_assertions'] is False and profile['overflow_checks'] is False
    current_harness = [{'path':path.name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
                       for path in sorted((out/'harness').glob('*.rs'))]
    assert manifest['harness'] == current_harness, (arm,'harness changed')
for key in ('rustc','lock_sha256','features','release_profile','harness'):
    assert len({json.dumps(manifest[key],sort_keys=True) for manifest in manifests.values()}) == 1, key
assert len({manifest['binary_sha256'] for manifest in manifests.values()}) == 3
for row in json.loads((out/'input-manifest.json').read_text())['files']:
    assert hashlib.sha256((root/row['path']).read_bytes()).hexdigest() == row['sha256']
private = out/'semantic-private'
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

def run(arm,label,args):
    result = subprocess.run([str(out/arm/'probe'),*args],capture_output=True,
                            env=dict(os.environ,RAYON_NUM_THREADS='4'),check=False)
    (private/f'{label}-{arm}.stdout').write_bytes(result.stdout)
    (private/f'{label}-{arm}.stderr').write_bytes(result.stderr)
    assert result.returncode == 0 and not result.stderr, (arm,label,result.returncode)
    data = result.stdout.replace(str(root).encode(), b'<inputs>')
    records.setdefault(label,{})[arm] = {'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
    return data

for case,demand,path in facts:
    outputs = {arm:run(arm,f'facts-{case}-{demand}',[str(root/path),demand,'1','dump']) for arm in 'ABC'}
    assert outputs['B'] == outputs['C'], (case,demand,'optimization changed facts')
    print('fact parity B/C:',case,demand,flush=True)
graph_root = root/plan['graph']['root']
graphs = {}
for mode in ('off','work'):
    graphs[mode] = {arm:json.loads(run(arm,f'graph-{mode}',[str(graph_root),'1',mode,'calls'])) for arm in 'ABC'}
    assert graphs[mode]['B']['semantic'] == graphs[mode]['C']['semantic'], 'optimization changed graph'
for arm in 'ABC':
    assert graphs['off'][arm]['semantic'] == graphs['work'][arm]['semantic'], 'observer changed graph'
identity = lambda graph:[(site['file'],site['offset'],site['caller_id'],site['invocation'],site['callee'])
                        for site in graph['semantic']['sites']]
assert identity(graphs['off']['A']) == identity(graphs['off']['B']), '2019 changed physical call identity'
no_call_graphs = {}
for arm in 'ABC':
    no_calls=json.loads(run(arm,'graph-no-calls',[str(graph_root),'1','off','no-calls']))
    no_call_graphs[arm] = no_calls['semantic']
    assert not no_calls['semantic']['sites']
    assert no_calls['semantic']['nodes'] and no_calls['semantic']['edges']
assert no_call_graphs['A'] == no_call_graphs['B'] == no_call_graphs['C'], 'non-consuming graph changed'
for index,entry in enumerate(plan['effects']['entries']):
    outputs={arm:run(arm,f'effects-{index}', ['verify',str(graph_root),plan['effects']['kind'],entry,'1']) for arm in 'ABC'}
    assert outputs['B'] == outputs['C'], 'optimization changed consuming report'
    print('effects parity B/C:',entry,flush=True)
(out/'semantic-manifest.json').write_text(json.dumps({'status':'Full outputs retained for independent review; A/B approved deltas must be reviewed before timing',
 'source_revisions':{arm:manifest['revision'] for arm,manifest in manifests.items()},'outputs':records,
 'assertions':['all13 fact snapshots B/C exact','full graph B/C exact','graph observer/off semantic exact per arm',
               'A/B physical call identity exact','non-consuming graph has nodes/edges and no calls',
               'all3 full effects reports B/C exact'],
 'effects_observer_limit':'verify emits observer-off full reports; observer collection timed separately; count alone is not full semantic proof'},indent=2)+'\n')
print('Untimed preflight complete; independent A/B semantic review required before timing.',flush=True)
