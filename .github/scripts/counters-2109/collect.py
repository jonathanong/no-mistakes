"""Fresh-process CLI preflight or one complete balanced block; DRAFT fails closed."""
import time
ENTRY_WALL=time.time()
import argparse,hashlib,json,math,os,resource,sys
from pathlib import Path
from common import package_guard,run,save,sha,git,machine,validate_producer
from contracts import CASES,ORDERS,parsed,parity,seal,fit
from transport import verify
import subreaper
P=Path(__file__).resolve().parent

def fixture(root):
    expected=json.loads((P/'consumer-canonical.json').read_text());actual=[]
    raw=git(root,'ls-files','--stage','-z')
    entries=[]
    for value in raw.split(b'\0'):
        if not value:continue
        meta,name=value.split(b'\t');mode,blob,stage=meta.decode().split();assert stage=='0'
        entries.append((os.fsdecode(name),mode,blob))
    assert len(entries)==21088 and sum(m=='120000'for _,m,_ in entries)==34
    for name,mode,blob in sorted(entries):
        assert '..'not in Path(name).parts and not Path(name).is_absolute()
        path=root/name
        data=os.fsencode(os.readlink(path))if mode=='120000'else path.read_bytes()
        if mode!='120000':assert path.is_file()and not path.is_symlink()and bool(path.stat().st_mode&0o111)==(mode=='100755')
        actual.append({'path':name,'mode':mode,'blob':blob,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    assert actual==expected,'tracked tree/mode/blob/content drift; index timestamps ignored'
    assert git(root,'rev-parse','HEAD^{tree}').decode().strip()=='d811b8d2299b36192996a2944c1ea13c1c9bf9ad'
    assert not git(root,'status','--porcelain','--untracked-files=all')
    return sha(P/'consumer-canonical.json')

def invoke(binary,root,out,case,arm,cap,deadline):
    runtime=out/'runtime';runtime.mkdir(mode=0o700,parents=True)
    config=root/'.no-mistakes.yml'if case.startswith('consumer')else P/'configs/no-annotation.yml'
    argv=[str(binary),'--timeout','180','--jobs','4','check','--root',str(root),'--config',str(config),'--format','json']
    if case.endswith('-on'):argv+=['--verbose-timings','--progress','--progress-interval','1000']
    env={k:v for k,v in os.environ.items()if not k.startswith(('NO_MISTAKES','RAYON_','CARGO_','RUST','GITHUB_','GH_'))and k not in['NODE_OPTIONS','LD_PRELOAD','LD_LIBRARY_PATH']}
    env.update(RAYON_NUM_THREADS='4',XDG_RUNTIME_DIR=str(runtime),TMPDIR=str(runtime))
    rss=out/'child-rss.txt'
    wrapped=['/usr/bin/time','--quiet','--format=%M','--output='+str(rss),'--',*argv]
    row=run(wrapped,out/'raw',min(cap,deadline-time.monotonic()),env)
    raw=rss.read_bytes();assert raw.endswith(b'\n')and raw[:-1].isdigit()and int(raw)>0
    row.update(arm=arm,case=case,peak_rss_kib=int(raw),wrapper_wait4_rss_diagnostic_only=row['rss_kib'],cpu_scope='GNUtime-wrapper-plus-child',wall_scope='GNUtime-wrapper-plus-child;25ms observer precision',cli_argv=argv)
    row['parsed']=parsed((out/'raw.stdout').read_text(),(out/'raw.stderr').read_text(),case)
    return row

def current_artifact(root):
    verify(root)
    receipt=json.loads((root/'transport-terminal.json').read_text())
    assert receipt['stage_outcome']=='success'
    assert receipt['run_id']==os.environ['GITHUB_RUN_ID']and receipt['attempt']==os.environ['GITHUB_RUN_ATTEMPT']
    source=json.loads((root/'job-metadata.json').read_text())
    assert source['GITHUB_SHA']==source['GITHUB_WORKFLOW_SHA']==os.environ['APPROVED_SOURCE_SHA']

def dependency_pair(artifacts,producers):
    projections=[];files=[]
    for arm in ['baseline','candidate']:
        directory=artifacts/arm/'native'
        cargo=json.loads((directory/'cargo-artifacts.json').read_text())
        root_package=producers[arm]['cargo_selected']['package_id']
        selected=[r for r in cargo if r.get('reason')=='compiler-artifact'and r['package_id']!=root_package]
        assert selected and all(r['fresh']is False for r in selected)
        projections.append(sorted(json.dumps({k:r[k]for k in ['package_id','target','features','profile']},sort_keys=True)for r in selected))
        deps=json.loads((directory/'completed-dependencies.json').read_text())
        files.append({r['path']:(r['bytes'],r['sha256'])for r in deps if 'no_mistakes'not in Path(r['path']).name and 'no-mistakes'not in Path(r['path']).name})
    assert projections[0]==projections[1],'third-party source/features/profile mismatch'
    assert files[0]and files[0]==files[1],'third-party object/dependency bytes differ'
    return {'matched_compiler_artifacts':len(projections[0]),'matched_dependency_files':len(files[0])}

def main():
    a=argparse.ArgumentParser();a.add_argument('phase',choices=['preflight','block']);a.add_argument('--artifacts',type=Path,required=True);a.add_argument('--root',type=Path,required=True);a.add_argument('--output',type=Path,required=True);a.add_argument('--preflight',type=Path);a.add_argument('--case',choices=CASES);a.add_argument('--block',type=int,choices=range(4));args=a.parse_args()
    spec=json.loads((P/'protocol.json').read_text());assert spec['status']=='REVIEWED_2109_MATCHED_DIAGNOSTIC','DRAFT cannot execute'
    package_guard(P,os.environ);assert os.environ['PROTOCOL_SHA256']==sha(P/'protocol.json')
    assert not args.output.exists();args.output.mkdir(parents=True);out=args.output
    m=machine();assert sorted(os.sched_getaffinity(0))==[0,1,2,3]
    seconds=1800
    job=float(os.environ['JOB_STARTED_UNIX']);assert job<=ENTRY_WALL<=time.time()
    deadline=time.monotonic()+min(job+seconds-180,ENTRY_WALL+seconds-240)-time.time()
    assert deadline>time.monotonic();before=fixture(args.root);assert before==spec['consumer_canonical_sha256'];save(out/'machine.json',m)
    producers={};binaries={};rows=[];failure=None
    subreaper.enable()
    for arm in ['baseline','candidate']:
        current_artifact(args.artifacts/arm);d=args.artifacts/arm/'native';v=json.loads((d/'producer.json').read_text());elf=(d/'cli').read_bytes()
        terminal=json.loads((d/'terminal.json').read_text());assert terminal['status']=='PASS'and terminal['source']==spec[arm]
        validate_producer(v,arm,spec,m,sha(d/'cli'),len(elf),int.from_bytes(elf[18:20],'little'))
        assert v['tree']==spec['source_trees'][arm]and elf[:7]==b'\x7fELF\x02\x01\x01'
        assert v['cargo_selected']['features']==['default','mermaid-validation']and v['profile']==spec['profile']
        assert json.loads((d/'cleanup.json').read_text())=={'target_absent':True,'source_exact':True}
        producers[arm]=v;binaries[arm]=d/'cli';os.chmod(d/'cli',0o755)
    assert producers['baseline']['compiler']==producers['candidate']['compiler']
    assert producers['baseline']['cargo_selected']['manifest_path']==producers['candidate']['cargo_selected']['manifest_path'],'matching compiler source prefix required'
    assert producers['baseline']['cargo_lock_sha256']==producers['candidate']['cargo_lock_sha256']
    assert producers['baseline']['cli_sha256']!=producers['candidate']['cli_sha256']
    save(out/'dependency-pair.json',dependency_pair(args.artifacts,producers))
    versions={}
    for arm,binary in binaries.items():
        v=run([str(binary),'--version'],out/('version-'+arm),min(20,deadline-time.monotonic()));assert v['exit']==0
        versions[arm]=(out/('version-'+arm+'.stdout')).read_text()
    assert versions['baseline']==versions['candidate'];save(out/'runtime-versions.json',versions)
    tool=run(['/usr/bin/time','--version'],out/'gnu-time-version',min(20,deadline-time.monotonic()));assert tool['exit']==0
    save(out/'gnu-time-identity.json',{'sha256':sha('/usr/bin/time'),'version':(out/'gnu-time-version.stdout').read_text()})
    pair=hashlib.sha256(json.dumps(producers,sort_keys=True).encode()).hexdigest();machine_sha=hashlib.sha256(json.dumps(m,sort_keys=True).encode()).hexdigest()
    resource.setrlimit(resource.RLIMIT_AS,(9000000000,9000000000));resource.setrlimit(resource.RLIMIT_FSIZE,(33554432,33554432))
    try:
        if args.phase=='preflight':
            assert 8*150+180<=deadline-time.monotonic()
            for case in CASES:
                for arm in ['baseline','candidate']:
                    row=invoke(binaries[arm],args.root,out/(case+'-'+arm),case,arm,150,deadline);rows.append(row);save(out/'partial.json',rows);parity(rows)
            save(out/'duration-seal.json',seal(rows))
        else:
            assert args.case is not None and args.block is not None and args.preflight
            current_artifact(args.preflight.parent);pre=json.loads((args.preflight/'terminal.json').read_text());assert pre['status']=='PASS'and pre['phase']=='preflight'and pre['pair_sha256']==pair
            cap=json.loads((args.preflight/'duration-seal.json').read_text())['child_seconds'];fit(cap,deadline-time.monotonic())
            reference=json.loads((args.preflight/'rows.json').read_text())
            for arm in ['baseline','candidate']:
                warm=invoke(binaries[arm],args.root,out/('warm-'+arm),args.case,arm,cap,deadline);parity(reference+[warm])
            fit(cap,deadline-time.monotonic(),True)
            order=ORDERS[args.block]
            for position,label in enumerate(order):
                assert (4-position)*cap+180<=deadline-time.monotonic()
                arm='baseline'if label=='A'else'candidate';row=invoke(binaries[arm],args.root,out/str(position),args.case,arm,cap,deadline)
                row.update(block=args.block,position=position,order=order,machine_sha256=machine_sha,pair_sha256=pair,fixture_sha256=before,protocol_sha256=sha(P/'protocol.json'))
                rows.append(row);save(out/'partial.json',rows);parity(reference+rows)
        save(out/'rows.json',rows)
    except BaseException as error:failure=repr(error);raise
    finally:
        try:
            assert fixture(args.root)==before
            assert all(sha(binaries[a])==producers[a]['cli_sha256']for a in binaries)
            assert not subreaper.children();package_guard(P,os.environ)
        except BaseException as error:failure='POSTCHECK '+repr(error);raise
        finally:save(out/'terminal.json',{'status':'FAILED'if failure else'PASS','phase':args.phase,'rows':len(rows),'error':failure,'pair_sha256':pair,'scientific':'NOT AUTOMATIC ACCEPTANCE','postchecks_pass':failure is None})
if __name__=='__main__':main()
