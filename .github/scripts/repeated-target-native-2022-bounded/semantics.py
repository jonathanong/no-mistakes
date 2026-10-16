"""Untimed full-report preflight and per-sample signature verification."""
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import io
import tarfile

from common import digest, identity, require, write
from pair import HERE, ROOT, SPEC, reviewed_stage
from transfer import verify


def sha(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def normalized(data, source):
    return json.loads(data.decode().replace(str(source),'<fixture>'))


def canonical_graph(data):
    graph=data['semantic']
    require(set(graph)=={'sites','nodes','edges','importer_view'},'structured_graph_schema_changed')
    for site in graph['sites']:
        site['known'].sort(key=lambda item:json.dumps(item,sort_keys=True))
    for key in ('sites','nodes','edges','importer_view'):
        graph[key].sort(key=lambda item:json.dumps(item,sort_keys=True))
    return graph


def receiver_order_controls(graph):
    """Retain the prior once-call positive and conservative repeated-call controls."""
    sites = graph['sites']
    for file, caller, callee, known, unknown in (
        ('repeated-binding-acquisitions', 'runBefore/takeBefore', 'fn', ['beforeLoop.get'], False),
        ('repeated-binding-acquisitions', 'oneShot/takeDeferred', 'fn', ['deferredCopy.get'], False),
        ('deferred-double-call', 'calledTwice/takeTwice', 'fn', [], True),
        ('deferred-call-proof-controls', None, 'emptyDeferred', [], True),
        ('deferred-call-proof-controls', None, 'conditionalDeferred', [], True),
        ('deferred-call-proof-controls', None, 'escapedDeferred', [], True),
    ):
        rows = [site for site in sites if site['file'] == 'src/' + file + '.mts'
                and site['caller'] == caller and site['callee'] == callee]
        require(len(rows) == 1, 'receiver_order_control_site_changed', file=file, callee=callee)
        require(sorted(target['name'] for target in rows[0]['known']) == known
                and rows[0]['unknown'] is unknown,
                'receiver_order_control_target_changed', file=file, callee=callee)


def verify_transfer(root, arm):
    folder=root/arm/'transfer'
    manifest=json.loads((folder/'transfer-manifest.json').read_text())
    expected={'arm':arm,'revision':SPEC[arm]['revision'],'tree':SPEC[arm]['tree'],
              'source_inventory_sha256':SPEC['source_git_inventory_sha256'][arm],
              'features':SPEC['features'],'profile':SPEC['release_profile'],
              'graph_probe_cfg':'alternatives_api' if arm=='candidate' else None}
    require(verify(folder,manifest,expected),'transfer_changed',arm=arm)
    require(manifest['compiler'].startswith('rustc 1.99.1 (58cf8f9c3 ') and 'host: '+SPEC['target'] in manifest['compiler'],'compiler_changed',arm=arm)
    inventory=json.loads((folder/'source-inventory.json').read_text())
    require(inventory['revision']==SPEC[arm]['revision'] and inventory['tree']==SPEC[arm]['tree']
            and inventory['sha256']==SPEC['source_git_inventory_sha256'][arm], 'source_inventory_transfer_changed',arm=arm)
    revision=SPEC[arm]['revision']
    present=subprocess.run(['git','cat-file','-e',revision+'^{commit}'],cwd=ROOT,capture_output=True)
    if present.returncode:
        fetched=subprocess.run(['git','fetch','--no-tags','--depth=5','origin',revision],cwd=ROOT,capture_output=True)
        require(fetched.returncode==0,'source_revision_fetch_failed',arm=arm)
    # One immutable archive proves every selected Git blob; no per-file Git process loop.
    archive=subprocess.check_output(['git','archive','--format=tar',revision],cwd=ROOT)
    actual={}
    with tarfile.open(fileobj=io.BytesIO(archive),mode='r:') as source:
        for member in source:
            path=member.name
            selected=path in ('Cargo.toml','Cargo.lock','crates/no-mistakes/Cargo.toml') or path.startswith('crates/no-mistakes/') and path.endswith('.rs')
            if selected:
                require(member.isfile(),'selected_source_not_regular',path=path)
                raw=source.extractfile(member).read()
                actual[path]={'path':path,'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
    require([actual[key] for key in sorted(actual)]==inventory['rows'],'source_inventory_git_blob_changed',arm=arm)
    require(sha(inventory['rows'])==inventory['sha256'],'source_inventory_digest_changed',arm=arm)
    artifacts=[]
    for line in (folder/'cargo.jsonl').read_bytes().splitlines():
        try:record=json.loads(line)
        except ValueError:continue
        if record.get('reason')=='compiler-artifact' and record['target']['name'] in ('no_mistakes','no-mistakes'):
            artifacts.append(record)
    for kind in ('rlib','bin','cdylib'):
        name='no-mistakes' if kind=='bin' else 'no_mistakes'
        rows=[record for record in artifacts if record['target']['name']==name and kind in record['target']['kind']]
        require(len(rows)==1 and rows[0]['fresh'] is False
                and sorted(rows[0]['features'])==sorted(SPEC['features']),
                'cargo_owned_artifact_provenance_changed',arm=arm,kind=kind)
    for binary in ('cli','timing-probe','graph-probe','addon.node'):
        result=subprocess.run(['readelf','-h',str(folder/binary)],capture_output=True,text=True)
        require(result.returncode==0 and 'AArch64' in result.stdout,'not_aarch64_binary',arm=arm,binary=binary)
    return manifest


def copy_inputs(root):
    target=root/'inputs'
    require(not target.exists(),'input_snapshot_already_exists')
    target.mkdir()
    all_rows={row['path']:row for item in SPEC['inputs'].values() for row in item['files']}
    for path,row in sorted(all_rows.items()):
        raw=subprocess.check_output(['git','show',SPEC['candidate']['revision']+':'+path],cwd=ROOT)
        require(hashlib.sha256(raw).hexdigest()==row['sha256'] and len(raw)==row['bytes'],'input_blob_changed',path=path)
        destination=target/path
        destination.parent.mkdir(parents=True,exist_ok=True)
        destination.write_bytes(raw)
    # Exact v4 overlay applies to copied input only, identically for both arms.
    for key,name in [('configured-consumer','consumer-overlay.yml'),('default-cold','plain-cold-overlay.yml')]:
        overlay=target/SPEC['inputs'][key]['path']/'.no-mistakes.yml'
        shutil.copyfile(HERE/name,overlay)
    return target


def cli_once(root, workload, arm, source, private):
    directory=private/(workload['name']+'-'+arm)
    args=[sys.executable,str(HERE/'cli-worker.py'),str(root/arm/'transfer/cli'),
          str(source),'1','on' if workload['diagnostics'] else 'off',str(directory),json.dumps(workload['command'])]
    result=subprocess.run(args,capture_output=True,timeout=120)
    require(result.returncode==0 and not result.stderr,'cli_worker_preflight_failed',workload=workload['name'],arm=arm)
    code=int((directory/'0.exit').read_text())
    require(code in (0,1),'unexpected_cli_exit',workload=workload['name'],arm=arm)
    stderr=(directory/'0.stderr').read_bytes()
    require(bool(stderr)==workload['diagnostics'],'diagnostics_stderr_changed',workload=workload['name'],arm=arm)
    report=normalized((directory/'0.stdout').read_bytes(),source)
    if workload['command'][0]=='check':
        require(set(report)=={'react','queues','rules','integration','codebase','warnings','advisories'},'check_schema_changed')
        require(not report['warnings'],'check_warnings',workload=workload['name'],arm=arm)
    else:
        require(set(report)=={'kind','entry','callSites','byCategory'} and code==0,'effects_report_changed',workload=workload['name'],arm=arm)
    return report,code


def preflight(root):
    reviewed_stage('semantic-preflight')
    root=Path(root)
    root.mkdir(parents=True,exist_ok=True)
    manifests={arm:verify_transfer(root,arm) for arm in ('baseline','candidate')}
    for binary in ('cli','timing-probe','graph-probe','addon.node'):
        require(manifests['baseline']['files'][binary]['sha256']!=manifests['candidate']['files'][binary]['sha256'],'same_pair_binary',binary=binary)
    copy_inputs(root)
    machine=identity(root)
    require(machine['architecture']=='aarch64','measurement_runner_not_arm')
    private=root/'private/semantics';private.mkdir(parents=True,exist_ok=True)
    expected=SPEC['semantic_policy']['report_hashes']
    require(set(expected)=={'graph','cli','probe','napi'},'semantic_hash_schema_incomplete')
    graph_inputs=SPEC['semantic_policy']['untimed_graph_inputs']
    require(set(expected['graph'])==set(graph_inputs),'graph_hash_input_coverage_incomplete')
    for key in SPEC['semantic_policy']['unchanged_graph_controls']:
        require(expected['graph'][key]['baseline']==expected['graph'][key]['candidate'],
                'cold_or_inherited_graph_control_changed',input=key)
    graph_records={}
    for key in graph_inputs:
        input_key=key.split(':',1)[0]
        source=root/'inputs'/SPEC['inputs'][input_key]['path']
        mode='no-calls' if key.endswith(':no-calls') else 'calls'
        graph_records[key]={}
        for arm in ('baseline','candidate'):
            proc=subprocess.run([str(root/arm/'transfer/graph-probe'),str(source),'1','off',mode],capture_output=True,timeout=120)
            (private/f'{key}-{arm}.graph.stdout').write_bytes(proc.stdout)
            (private/f'{key}-{arm}.graph.stderr').write_bytes(proc.stderr)
            require(proc.returncode==0 and not proc.stderr,'graph_preflight_failed',input=key,arm=arm)
            graph=canonical_graph(normalized(proc.stdout,source))
            if key == 'receiver-order:calls' and arm == 'candidate':
                receiver_order_controls(graph)
            if mode=='no-calls':
                require(not graph['sites'] and graph['nodes'] and graph['edges'],
                        'no_calls_graph_empty_or_has_call_sites',input=key,arm=arm)
            write(root/'private/reports/graph'/f'{key}-{arm}.json',graph)
            h=sha(graph)
            require(h==expected['graph'][key][arm],'full_graph_hash_changed',input=key,arm=arm,actual=h)
            graph_records[key][arm]={'full_sha256':h,'sites':len(graph['sites']),'nodes':len(graph['nodes']),'edges':len(graph['edges'])}
    signatures={}
    require(set(expected['cli'])=={w['name'] for w in SPEC['workloads'] if w['kind']=='cli'},'cli_hash_coverage_incomplete')
    for key in SPEC['semantic_policy']['unchanged_cli_controls']:
        require(expected['cli'][key]['baseline']==expected['cli'][key]['candidate'],
                'inherited_cli_control_changed',workload=key)
    require(set(expected['probe'])=={w['name'] for w in SPEC['workloads'] if w['kind']=='probe'},'probe_signature_coverage_incomplete')
    for work in SPEC['workloads']:
        key=work['name'];source=root/'inputs'/SPEC['inputs'][work['input']]['path'];signatures[key]={}
        for arm in ('baseline','candidate'):
            if work['kind']=='cli':
                report,code=cli_once(root,work,arm,source,private)
                write(root/'private/reports/cli'/f'{key}-{arm}.json',report)
                h=sha(report)
                require({'sha256':h,'exit':code}==expected['cli'][key][arm],'full_cli_report_changed',workload=key,arm=arm,actual=h)
                signatures[key][arm]={'report_sha256':h,'exit':code}
            else:
                proc=subprocess.run([str(root/arm/'transfer/timing-probe'),work['mode'],str(source),'1'],capture_output=True,timeout=120)
                (private/f'{key}-{arm}.probe.stdout').write_bytes(proc.stdout)
                (private/f'{key}-{arm}.probe.stderr').write_bytes(proc.stderr)
                require(proc.returncode==0 and not proc.stderr and proc.stdout.strip().isdigit(),'probe_preflight_failed',workload=key,arm=arm)
                value=proc.stdout.decode().strip()
                require(value==expected['probe'][key][arm],'probe_signature_changed',workload=key,arm=arm,actual=value)
                signatures[key][arm]=value
    write(root/'public/preflight.json',dict(machine,pair_revision={arm:SPEC[arm]['revision'] for arm in ('baseline','candidate')},
          artifact_hashes={arm:manifests[arm]['files'] for arm in manifests},status='untimed full semantic preflight passed'))
    write(root/'public/semantic-preflight.json',{'signatures':signatures,'graphs':graph_records,
          'consumer_overlay_sha256':digest(root/'inputs'/SPEC['inputs']['configured-consumer']['path'] /'.no-mistakes.yml'),
          'plain_cold_overlay_sha256':digest(root/'inputs'/SPEC['inputs']['default-cold']['path'] /'.no-mistakes.yml')})
    return signatures


def inspect_sample(root,directory,workload,arm,count):
    result={'valid':False,'stdout_sha256':digest(directory/'stdout'),'stdout_bytes':(directory/'stdout').stat().st_size,
            'stderr_sha256':digest(directory/'stderr'),'stderr_bytes':(directory/'stderr').stat().st_size}
    try:
        require(not result['stderr_bytes'],'worker_stderr_present')
        expected=json.loads((root/'public/semantic-preflight.json').read_text())['signatures'][workload['name']][arm]
        if workload['kind']=='probe':
            require((directory/'stdout').read_text().strip()==expected,'probe_signature_changed')
        else:
            require((directory/'stdout').read_text().strip()==str(count),'cli_batch_incomplete')
            requests=directory/'requests';source=root/'inputs'/SPEC['inputs'][workload['input']]['path']
            require(len(list(requests.glob('*.exit')))==count,'request_count_changed')
            for index in range(count):
                report=normalized((requests/f'{index}.stdout').read_bytes(),source)
                require(sha(report)==expected['report_sha256'] and int((requests/f'{index}.exit').read_text())==expected['exit'],'cli_report_or_exit_changed')
                require(bool((requests/f'{index}.stderr').stat().st_size)==workload['diagnostics'],'cli_diagnostics_changed')
        result['valid']=True
    except Exception:
        result['error_code']='sample_output_or_semantics_invalid'
    return result
