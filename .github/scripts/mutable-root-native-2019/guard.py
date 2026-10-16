"""One source snapshot per stage; no remote waiting or generated required checks."""
from common import *
import re

def reviewed():
    require(SPEC['status']=='REVIEWED AND FROZEN','DRAFT: no execution')
    approved=os.environ.get('APPROVED_DIAGNOSTIC_COMMIT','')
    require(re.fullmatch('[0-9a-f]{40}',approved),'missing approved diagnostic SHA')
    require(git('rev-parse','HEAD')==approved,'diagnostic source changed')
    require(git('merge-base',SPEC['diagnostic_parent'],'HEAD')==SPEC['diagnostic_parent'],'wrong diagnostic parent')
    require(not git('status','--porcelain'),'dirty diagnostic source')
    changed=git('diff','--name-only',SPEC['diagnostic_parent'],'HEAD').splitlines()
    require(changed and all(p==SPEC['workflow']or p.startswith(SPEC['directory']+'/')for p in changed),
            'diagnostic modifies production source')
    manifest=json.loads((HERE/'source-manifest.json').read_text())
    actual={str(p.relative_to(ROOT))for p in HERE.rglob('*')if p.is_file()and
            p.name!='source-manifest.json'and '__pycache__'not in p.parts}
    actual.add(SPEC['workflow'])
    require(actual==set(manifest['files']),'source manifest incomplete or extra files')
    for name,digest in manifest['files'].items():require(sha(ROOT/name)==digest,'source bytes changed: '+name)
    snap=api('repos/jonathanong/no-mistakes/git/ref/heads/main')
    pr=api('repos/jonathanong/no-mistakes/pulls/2054')
    require(snap['object']['sha']==SPEC['baseline']['revision'],'main changed')
    require(pr['head']['sha']==SPEC['candidate']['revision']and pr['draft']is True,'head/draft changed')
    require(pr['base']['ref']=='main','base changed')
    ci=api('repos/jonathanong/no-mistakes/actions/runs/'+str(SPEC['ci']['run']))
    require(ci['head_sha']==SPEC['candidate']['revision']and ci['conclusion']=='success','CI not eligible')
    for name,identifier in SPEC['ci']['codecov'].items():
        row=api('repos/jonathanong/no-mistakes/check-runs/'+str(identifier))
        require(row['name']==name and row['head_sha']==SPEC['candidate']['revision']and
                row['status']=='completed'and row['conclusion']=='success','Codecov changed')
    return {'main':snap['object']['sha'],'head':pr['head']['sha'],'draft':True,'ci':ci['id']}

def coverage():
    job=api('repos/jonathanong/no-mistakes/actions/jobs/'+str(SPEC['ci']['rust_job']))
    require(job['head_sha']==SPEC['candidate']['revision']and job['conclusion']=='success','Rust job mismatch')
    raw=run(['gh','api','repos/jonathanong/no-mistakes/actions/jobs/'+str(job['id'])+'/logs',
             '--allow-escape-sequences'],timeout=120)
    require(__import__('hashlib').sha256(raw).hexdigest()==SPEC['ci']['raw_log_sha256'],'raw CI log changed')
    text=raw.decode();values=re.findall(r'\bTOTAL\s+\d+\s+\d+\s+([\d.]+)%\s+\d+\s+\d+\s+([\d.]+)%\s+\d+\s+\d+\s+([\d.]+)%',text)
    require(len(values)==1 and all(float(x)>=98 for x in values[0]),'global coverage below98')
    for kind in ['LINES','FUNCTIONS','REGIONS']:
        require(set(re.findall('RUST_COVERAGE_FAIL_UNDER_'+kind+r':\s*(\d+)',text))=={'98'},'floor changed')
        require('--fail-under-'+kind.lower()in text,'enforcement missing')
    (WORK/'public').mkdir(parents=True,exist_ok=True);(WORK/'public/ci-raw.log').write_bytes(raw)
    write(WORK/'public/ci-proof.json',{'head':SPEC['candidate']['revision'],'job':job,'coverage':values[0]})
