import time
ENTRY_WALL=time.time()
import json,os,sys
from pathlib import Path
from common import package_guard,command_deadline,inventory,git,machine,run,save,sha
from requests import make_request,nonvacuous
import subreaper
P=Path(__file__).resolve().parent

def validate(p,arm,pair,spec,m,d):
    assert p['arm']==arm and p['pair']==pair and p['source']==spec['pairs'][pair][arm]
    assert p['tree']==spec['pairs'][pair]['trees'][arm]
    assert json.loads((d/'source-inventory.json').read_text())==json.loads((d/'source-after.json').read_text())
    assert sha(d/'source-after.json')==p['source_after_sha256']
    assert p['machine']['run']==m['run'] and p['machine']['attempt']==m['attempt']
    assert sha(d/'addon.node')==p['addon_sha256'] and (d/'addon.node').stat().st_size==p['bytes']
    assert p['cargo_selected']['fresh'] is False and p['cargo_selected']['target']['crate_types']==['cdylib']
    assert p['cargo_selected']['features']==['default','mermaid-validation']
    assert json.loads((d/'cleanup.json').read_text())=={'target_absent':True,'source_exact':True}
    assert sha(d/'source-inventory.json')==p['source_inventory_sha256']
    b=(d/'addon.node').read_bytes();assert b[:4]==b'\x7fELF' and int.from_bytes(b[18:20],'little')==183

def main():
    artifacts,root,out=map(lambda x:Path(x).resolve(),sys.argv[1:4]);pair,case,diag,jobs=sys.argv[4:8]
    out.mkdir(parents=True,exist_ok=True);rows=[];before=None;producers={}
    try:
        package_guard(P,os.environ);deadline=command_deadline(ENTRY_WALL,os.environ);spec=json.loads((P/'protocol.json').read_text())
        assert spec['status']=='REVIEWED_ASYNC_API_DIAGNOSTIC' and sha(P/'protocol.json')==os.environ['PROTOCOL_SHA256']
        assert diag in ['off','on'] and jobs in ['1','4','default'];m=machine();save(out/'machine.json',m)
        assert git(root,'rev-parse','HEAD').decode().strip()==spec['pairs'][pair]['baseline']
        before=inventory(root);frozen=json.loads((P/f'fixture-{pair}.json').read_text());assert frozen['source']==spec['pairs'][pair]['baseline'] and before==frozen['files'] and git(root,'rev-parse','HEAD^{tree}').decode().strip()==frozen['tree'],'frozen fixture inventory mismatch';save(out/'fixture-before.json',before);binaries={}
        for arm in ['baseline','candidate']:
            d=artifacts/(pair+'-'+arm);p=json.loads((d/'producer.json').read_text());validate(p,arm,pair,spec,m,d);producers[arm]=p;binaries[arm]=d/'addon.node';os.chmod(binaries[arm],0o755)
        assert producers['baseline']['compiler']==producers['candidate']['compiler']
        assert producers['baseline']['addon_sha256']!=producers['candidate']['addon_sha256'],'distinct pair binaries required'
        save(out/'producers.json',producers);subreaper.enable()
        request=out/'request.json';save(request,make_request(pair,case,root))
        def invoke(arm,mode,label):
            d=out/label;d.mkdir();facade=root/'packages/no-mistakes'/('index.mjs'if mode=='facade-esm'else'index.js')
            c={'status':'REVIEWED_WORKER_INVOCATION','addon':str(binaries[arm]),'addon_sha256':producers[arm]['addon_sha256'],'facade':str(facade),'facade_sha256':sha(facade),'request':str(request),'request_sha256':sha(request),'mode':mode,'jobs':jobs,'diagnostics':diag,'output':str(d/'report.json'),'receipt':str(d/'worker.json'),'snapshots':str(d/'snapshots.json'),'observer_error':str(d/'observer-error.txt')};save(d/'input.json',c)
            env={k:v for k,v in os.environ.items()if k not in ['RAYON_NUM_THREADS','RUST_TEST_THREADS']};(d/'runtime').mkdir(mode=0o700);env['XDG_RUNTIME_DIR']=str(d/'runtime')
            r=run(['node',str(P/'worker.cjs'),str(d/'input.json')],d/'process',min(180,deadline-time.monotonic()),env);assert r['exit']==0
            worker=json.loads((d/'worker.json').read_text());assert worker['status']=='COMPLETE';report=json.loads((d/'report.json').read_text());nonvacuous(pair,case,report)
            r.update(arm=arm,mode=mode,label=label,worker=worker,report_sha256=sha(d/'report.json'),report_bytes=(d/'report.json').stat().st_size);return r,report
        pre=[];reports={}
        for arm in ['baseline','candidate']:
            for mode in ['raw-native','facade-cjs','facade-esm']:
                r,report=invoke(arm,mode,'pre-'+arm+'-'+mode);pre.append(r);reports[(arm,mode)]=report
        assert all(x==reports[('baseline','raw-native')] for x in reports.values()),'whole report parity'
        assert len({(r['report_sha256'],r['report_bytes'])for r in pre if r['mode']=='raw-native'})==1,'native literal bytes differ'
        save(out/'preflight.json',pre)
        assert deadline-time.monotonic()>=8*180+200,'fixed8rows cannotfit; no retuning'
        for block,order in enumerate(['ABBA','BAAB']):
            for position,letter in enumerate(order):
                arm='baseline'if letter=='A'else'candidate';r,report=invoke(arm,'facade-cjs',f'row-{block}-{position}-{letter}');assert report==reports[('baseline','raw-native')]
                r.update(block=block,order=order,position=position,pair=pair,case=case,diagnostics=diag,jobs=jobs);rows.append(r);save(out/'rows.json',rows)
        assert len(rows)==8
    finally:
        checks={'fixture_exact':before is not None and inventory(root)==before,'binary_exact':all(sha(artifacts/(pair+'-'+a)/'addon.node')==p['addon_sha256']for a,p in producers.items()),'children':subreaper.children()};save(out/'postcheck.json',checks)
        package_guard(P,os.environ);assert checks['fixture_exact'] and checks['binary_exact'] and not checks['children']
    save(out/'terminal.json',{'technical':'PASS','rows':8,'pair':pair,'case':case,'diagnostics':diag,'jobs':jobs,'run':os.environ['GITHUB_RUN_ID'],'attempt':os.environ['GITHUB_RUN_ATTEMPT'],'source_sha':os.environ['GITHUB_SHA'],'interpretation':'Descriptive only; no automatic performance PASS'})
if __name__=='__main__':
    try:main()
    except BaseException as e:
        d=Path(sys.argv[3]);d.mkdir(parents=True,exist_ok=True);save(d/'terminal.json',{'technical':'FAILED','error':repr(e)});raise
