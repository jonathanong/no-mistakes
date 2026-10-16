"""Predeclared fresh-process samples; run only in a coordinated idle window."""
import hashlib,json,os
from pathlib import Path
import subprocess,sys,time
from datetime import datetime, timezone
launched_at=datetime.now(timezone.utc)

out=Path(__file__).resolve().parent
from protocol_guard import require_reviewed_pair
reviewed_plan=require_reviewed_pair(out, live=True)
assert len(sys.argv)==1
phase,pair,group='long','ARM_BASE_ARM_CAND','all'
source=Path('/tmp/nm2019-current-main-pair-evidence')
original=Path('/tmp/nm2019-three-arm-evidence')
inputs=original/'inputs'
rows_path=out/'ARM_BASE_ARM_CAND-long-all-rows.json'
assert not rows_path.exists(), 'Never overwrite prior observations'
frozen=json.loads((out/'frozen-run-manifest.json').read_text())
assert frozen['status']=='FROZEN before first row'
# A completed gate cannot reconstruct a missing pre-start record after sampling.
start=json.loads((out/'sampling-start-record.json').read_text())
assert start['status']=='READY before sampling'
assert start['frozen_manifest_sha256']==hashlib.sha256((out/'frozen-run-manifest.json').read_bytes()).hexdigest()
assert start['candidate_revision']==reviewed_plan['candidate_revision']
assert start['baseline_revision']==reviewed_plan['baseline_revision']
recorded_at=datetime.fromisoformat(start['recorded_at_utc'])
assert recorded_at.tzinfo is not None and recorded_at<=launched_at
assert not start.get('recorded_after_run',False)
for path,digest in frozen['files'].items():
 assert Path(path).is_absolute()
 assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
workloads=json.loads((out/'workloads.json').read_text())
selected=workloads
expected=json.loads((out/'timing-signatures.json').read_text())
assert (out/'semantic-manifest.json').exists()
assert len(selected)==22
cpus=sorted(os.sched_getaffinity(0))[:4]
assert cpus==[0,1,2,3]
arm_map={'baseline':'ARM_BASE','candidate':'ARM_CAND'}
manifests={arm:json.loads((source/arm/'manifest.json').read_text()) for arm in arm_map.values()}
for arm,manifest in manifests.items():
 assert hashlib.sha256((source/arm/'probe').read_bytes()).hexdigest()==manifest['binary_sha256']
for item in json.loads((original/'input-manifest.json').read_text())['files']:
 assert hashlib.sha256((inputs/item['path']).read_bytes()).hexdigest()==item['sha256']
calibration=json.loads((out/'calibration-batches.json').read_text())
rows=[]
orders=['ABBA','BAAB']*4
def affinity():os.sched_setaffinity(0,cpus)
for block,order in enumerate(orders):
 for workload in selected:
  count=workload.get('fixed_batch',calibration.get(workload['id']))
  assert type(count) is int and count>0
  arguments=[arg.replace('@INPUT@',str(inputs)).replace('@COUNT@',str(count)) for arg in workload['args']]
  for position,letter in enumerate(order):
   arm='baseline' if letter=='A' else 'candidate'
   source_arm=arm_map[arm];manifest=manifests[source_arm]
   start=time.perf_counter()
   child=subprocess.Popen([str(source/source_arm/'probe'),*arguments],stdout=subprocess.PIPE,stderr=subprocess.PIPE,
                          env=dict(os.environ,RAYON_NUM_THREADS='4'),preexec_fn=affinity)
   _,status,usage=os.wait4(child.pid,0)
   child.returncode=os.waitstatus_to_exitcode(status)
   stdout=child.stdout.read();stderr=child.stderr.read()
   signature=stdout.decode(errors='replace').strip()
   if not signature.isdigit():signature=None
   if child.returncode!=0 or stderr or signature!=expected[workload['id']][source_arm]:
    private=out/'measurement-failures-private';private.mkdir(exist_ok=True)
    label=f'{pair}-{phase}-{group}-{block}-{workload["id"]}-{position}-{source_arm}'
    (private/(label+'.stdout')).write_bytes(stdout);(private/(label+'.stderr')).write_bytes(stderr)
   row=dict(pair=pair,phase=phase,group=group,block=block,order=order,position=position,
            workload=workload['id'],arm=arm,source_arm=source_arm,revision=manifest['revision'],
            binary_sha256=manifest['binary_sha256'],collections=count,cpu_s=usage.ru_utime+usage.ru_stime,
            user_s=usage.ru_utime,system_s=usage.ru_stime,wall_s=time.perf_counter()-start,
            rss_kib=usage.ru_maxrss,affinity=cpus,rayon_workers=4,exit=child.returncode,
            signature=signature,stdout_sha256=hashlib.sha256(stdout).hexdigest(),
            stderr_sha256=hashlib.sha256(stderr).hexdigest())
   rows.append(row)
   temporary=rows_path.with_suffix('.tmp')
   temporary.write_text(json.dumps(rows,indent=2)+'\n');temporary.replace(rows_path)
   assert child.returncode==0 and not stderr and row['signature']==expected[workload['id']][source_arm],row
 print(pair,phase,group,'block',block+1,'/',len(orders),'complete',flush=True)
