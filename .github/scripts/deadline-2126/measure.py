"""One same-VM diagnostics/jobs case; preflight precedes every timed row."""
import time
ENTRY_WALL=time.time()
import json,os,re,sys,time
from pathlib import Path
from common import package_guard,command_deadline,command,git,inventory,machine,parity,run,save,schedule,sha,validate_producer
import subreaper
P=Path(__file__).resolve().parent

def capacity(durations,remaining):
    assert len(durations)==4 and all(0<x<120 for x in durations)
    # Fixed child cap and complete schedule; no retuning after a fast pilot.
    needed=16*125+200
    assert needed<=remaining,'whole fixed schedule cannot fit; STOP before timing'
    return needed

def main():
    artifacts=Path(sys.argv[1]).resolve();fixture=Path(sys.argv[2]).resolve();out=Path(sys.argv[3]).resolve();diag,jobs=sys.argv[4:6];out.mkdir(parents=True,exist_ok=True)
    package_guard(P,os.environ);deadline=command_deadline(ENTRY_WALL,os.environ)
    spec=json.loads((P/'protocol.json').read_text());assert spec['status']=='REVIEWED_ACTIVE_DEADLINE_DIAGNOSTIC'
    assert os.environ.get('PROTOCOL_SHA256')==sha(P/'protocol.json'),'exact protocol confirmation required'
    assert diag in ['off','on'] and jobs in ['1','4','default'];m=machine();before=inventory(fixture)
    expected=json.loads((P/'fixture-inventory.json').read_text());assert before==expected['files']
    assert git(fixture,'rev-parse','HEAD').decode().strip()==spec['baseline']
    assert not git(fixture,'status','--porcelain','--untracked-files=all')
    for entry in before:
        path=fixture/entry['path']
        if entry['mode'] not in ['120000','160000']:os.chmod(path,0o555 if entry['mode']=='100755'else 0o444)
    save(out/'machine.json',m);save(out/'fixture-before.json',before)
    producers={};binaries={}
    for arm in ['baseline','candidate']:
        d=artifacts/arm;producer=json.loads((d/'producer.json').read_text())
        elf=(d/'cli').read_bytes();assert elf[:4]==b'\x7fELF'
        validate_producer(producer,arm,spec,m,sha(d/'cli'),(d/'cli').stat().st_size,int.from_bytes(elf[18:20],'little'))
        assert json.loads((d/'cleanup.json').read_text())=={'target_absent':True,'source_exact':True}
        producers[arm]=producer;binaries[arm]=d/'cli';os.chmod(d/'cli',0o755)
    assert producers['baseline']['cli_sha256']!=producers['candidate']['cli_sha256']
    assert producers['baseline']['compiler']==producers['candidate']['compiler']
    save(out/'producers.json',producers)
    rows=[];subreaper.enable()
    def invoke(arm,timeout,label):
        runtime=out/(label+'-runtime');runtime.mkdir(mode=0o700)
        env={k:v for k,v in os.environ.items()if k not in ['RAYON_NUM_THREADS','RUST_TEST_THREADS']};env['XDG_RUNTIME_DIR']=str(runtime)
        argv=command(binaries[arm],fixture,P/'empty-rules.json',timeout,diag,jobs)
        row=run(argv,out/label,min(125,deadline-time.monotonic()),env)
        row.update(arm=arm,timeout=timeout,diagnostics=diag,jobs=jobs,label=label)
        json.loads((out/(label+'.stdout')).read_bytes())
        stderr=(out/(label+'.stderr')).read_text();row['work']={k:int(v)for k,v in re.findall(r'^\[work\] ([\w.]+): (\d+)$',stderr,re.M)}
        row['timings_ms']={k:float(v)for k,v in re.findall(r'^\[timing\] (.+?): ([\d.]+)ms',stderr,re.M)}
        if diag=='on':
            assert row['work'].get('discovery.candidates',0)>20000,'representative discovery scope missing'
            assert 'discovery.classify' in row['timings_ms'],'classify timing missing'
        return row
    try:
        preflight=[invoke(arm,timeout,f'preflight-{timeout}-{arm}')for timeout in [120,0]for arm in ['baseline','candidate']]
        parity(preflight)
        if diag=='on':assert len({r['work']['discovery.candidates']for r in preflight})==1,'discovery scope differs'
        save(out/'preflight.json',preflight)
        capacity([r['wall']for r in preflight],deadline-time.monotonic())
        save(out/'duration-seal.json',{'child_cap_seconds':125,'remaining_seconds':deadline-time.monotonic(),'fixed_rows':16,'schedule':['ABBA','BAAB'],'resampling':False})
        for timeout in [120,0]:
            for block,order,index,arm in schedule():
                r=invoke('baseline'if arm=='A'else'candidate',timeout,f'row-{timeout}-{block}-{index}-{arm}')
                r.update(block=block,order=order,index=index);rows.append(r);save(out/'rows.json',rows);parity(preflight+rows)
        assert len(rows)==16
    except BaseException as error:
        save(out/'terminal.json',{'technical':'FAILED','error':repr(error),'rows':len(rows)})
        raise
    finally:
        try:
            post={'fixture_exact':inventory(fixture)==before,'binary_exact':all(sha(binaries[a])==producers[a]['cli_sha256']for a in binaries),'owned_children':subreaper.children()}
            save(out/'postcheck.json',post)
            assert post['fixture_exact'] and post['binary_exact'] and not post['owned_children'],'postchecks failed'
            package_guard(P,os.environ)
        except BaseException as error:
            save(out/'terminal.json',{'technical':'FAILED','postcheck_error':repr(error),'rows':len(rows)})
            raise
    save(out/'terminal.json',{'technical':'PASS','rows':16,'scientific':'SEE INDEPENDENT DESCRIPTIVE ANALYSIS, NOT CI SUCCESS'})
if __name__=='__main__':
    try:main()
    except BaseException as error:
        failure=Path(sys.argv[3]);failure.mkdir(parents=True,exist_ok=True)
        prior=json.loads((failure/'terminal.json').read_text())if(failure/'terminal.json').exists()else{}
        save(failure/'terminal.json',dict(prior,technical='FAILED',outer_error=repr(error)))
        raise
