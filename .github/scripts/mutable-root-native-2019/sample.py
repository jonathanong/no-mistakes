"""One isolated VM, one retained attempt, explicit 1500s hosted budget; no local hold."""
from common import *
from guard import reviewed
from package import verify
from machine import snapshot
from datetime import datetime,timezone
import shutil

def sample():
    eligibility=reviewed();machine=snapshot();out=WORK/'evidence'
    require((out/'semantic-closure.json').is_file(),'semantic closure missing')
    closure=json.loads((out/'semantic-closure.json').read_text())
    require(closure['run_id']==os.environ['GITHUB_RUN_ID']and closure['source']==git('rev-parse','HEAD'),'semantic run/source mismatch')
    for name,digest in closure['proofs'].items():require(sha(out/name)==digest,'semantic receipt changed')
    # Fresh measurement VM reconstructs fixed absolute paths from immutable payload.
    original=Path('/tmp/nm2019-three-arm-evidence');prior=Path('/tmp/nm2019-arm2019-expected-prior')
    for source,dest in [(HERE/'payload/original',original),(HERE/'payload/expected-prior',prior)]:
        require(not dest.exists(),'input root occupied');shutil.copytree(source,dest)
    shutil.rmtree(original/'harness');materialize_harness(original/'harness')
    shutil.copy2(out/'workloads.json',original/'workloads.json')
    for arm,label in ARM.items():
        verify(arm);src=WORK/'closed'/arm;dest=ARTIFACTS/label;dest.mkdir(parents=True,exist_ok=False)
        for name in ['probe','lib.rlib']:shutil.copy2(src/'binaries'/name,dest/name)
        for name in ['manifest.json','source-manifest.json']:shutil.copy2(src/name,dest/name)
    files={str(p):sha(p)for prefix in [out,original,prior,HERE]for p in sorted(prefix.rglob('*'))if p.is_file()and '__pycache__'not in p.parts}
    files[str(ROOT/SPEC['workflow'])]=sha(ROOT/SPEC['workflow'])
    for arm,label in ARM.items():
        for p in (ARTIFACTS/label).iterdir():files[str(p)]=sha(p)
    write(out/'frozen-run-manifest.json',{'status':'FROZEN before first row','files':files,'run_id':os.environ['GITHUB_RUN_ID']})
    rows=out/'ARM_BASE_ARM_CAND-long-all-rows.json';require(not rows.exists(),'prior rows exist')
    start=time.time();deadline=start+SPEC['measurement_budget_seconds']
    write(out/'sampling-start-record.json',{'status':'READY before sampling','recorded_at_utc':datetime.fromtimestamp(start,timezone.utc).isoformat(),
          'baseline_revision':SPEC['baseline']['revision'],'candidate_revision':SPEC['candidate']['revision'],
          'frozen_manifest_sha256':sha(out/'frozen-run-manifest.json'),'budget_seconds':1500,
          'scope':'isolated hosted ARM VM; no local coordinated hold','machine':machine,'eligibility':eligibility})
    env=dict(os.environ,ARM2019_SOURCE=str(HERE));error=None;complete=False
    try:
        run(['python3',str(out/'measure.py')],env=env,log=out/'actual-sampler',timeout=1490)
        data=json.loads(rows.read_text());workloads=json.loads((out/'workloads.json').read_text())
        expected=[(block,order,w['id'],position,'baseline'if letter=='A'else'candidate')
                  for block,order in enumerate(['ABBA','BAAB']*4)for w in workloads for position,letter in enumerate(order)]
        actual=[(r['block'],r['order'],r['workload'],r['position'],r['arm'])for r in data]
        require(actual==expected and len(data)==704,'incomplete or reordered rows')
        require(all(r['exit']==0 and r['signature']==json.loads((out/'timing-signatures.json').read_text())[r['workload']][r['source_arm']] and r['stderr_sha256']==hashlib.sha256(b'').hexdigest()for r in data),'failed observations')
        require(time.time()<=deadline,'late completion')
        for path,digest in files.items():require(sha(path)==digest,'frozen input/source/artifact changed')
        reviewed();require(time.time()<=deadline,'late final eligibility readback');complete=True
    except BaseException as exc:error=repr(exc);raise
    finally:
        write(out/'sampling-end.json',{'complete':complete,'error':error,'ended_unix':time.time(),'deadline_unix':deadline,
              'raw_rows_sha256':sha(rows)if rows.exists()else None,'run_id':os.environ['GITHUB_RUN_ID']})
    run(['python3',str(out/'analyze.py')],env=env,log=out/'actual-analysis',timeout=120)
