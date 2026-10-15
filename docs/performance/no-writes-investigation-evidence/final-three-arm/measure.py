"""Predeclared fresh-process samples; run only in a coordinated idle window."""
import hashlib,json,math,os
from pathlib import Path
import statistics,subprocess,sys,time

out=Path(__file__).resolve().parent
phase,pair,group=sys.argv[1:]
assert phase in ('pilot','long')
assert pair in ('A_B','B_C')
assert (phase,pair,group)==('pilot','A_B','new') or (phase=='long' and
       ((pair=='A_B' and group in ('controls','hot-facts','graph-effects')) or
        (pair=='B_C' and group=='controls')))
rows_path=out/f'{pair}-{phase}-{group}-rows.json'
assert not rows_path.exists(), 'Never overwrite prior observations'
workloads=json.loads((out/'workloads.json').read_text())
selected=[row for row in workloads if ('fixed_batch' not in row if group=='new' else row['group']==group)]
expected=json.loads((out/'timing-signatures.json').read_text())
assert (out/'semantic-manifest.json').exists() and (out/'semantic-diff-A-B.json').exists()
assert len(selected)==({'new':14,'controls':8,'hot-facts':5,'graph-effects':9}[group])
cpus=sorted(os.sched_getaffinity(0))[:4]
assert cpus==[0,1,2,3]
arm_map={'baseline':pair[0],'candidate':pair[2]}
manifests={arm:json.loads((out/arm/'manifest.json').read_text()) for arm in arm_map.values()}
for arm,manifest in manifests.items():
 assert hashlib.sha256((out/arm/'probe').read_bytes()).hexdigest()==manifest['binary_sha256']
for item in json.loads((out/'input-manifest.json').read_text())['files']:
 assert hashlib.sha256((out/'inputs'/item['path']).read_bytes()).hexdigest()==item['sha256']
calibration=json.loads((out/'calibration-batches.json').read_text()) if phase=='long' and group!='controls' else {}
rows=[]
orders=['ABBA','BAAB']*(1 if phase=='pilot' else 4)
def affinity():os.sched_setaffinity(0,cpus)
for block,order in enumerate(orders):
 for workload in selected:
  count=1 if phase=='pilot' else workload.get('fixed_batch',calibration.get(workload['id']))
  assert type(count) is int and count>0
  arguments=[arg.replace('@INPUT@',str(out/'inputs')).replace('@COUNT@',str(count)) for arg in workload['args']]
  for position,letter in enumerate(order):
   arm='baseline' if letter=='A' else 'candidate'
   source_arm=arm_map[arm];manifest=manifests[source_arm]
   start=time.perf_counter()
   child=subprocess.Popen([str(out/source_arm/'probe'),*arguments],stdout=subprocess.PIPE,stderr=subprocess.PIPE,
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
if phase=='pilot':
 path=out/'calibration-batches.json';assert not path.exists()
 batches={}
 for workload in selected:
  samples=[row['cpu_s'] for row in rows if row['workload']==workload['id']]
  assert len(samples)==8 and all(cpu>0 for cpu in samples)
  batches[workload['id']]=max(1,min(100000,math.floor(1.2/statistics.median(samples))))
 path.write_text(json.dumps(batches,indent=2)+'\n')
 print('All112pilotrows retained; new-workload batches frozen.',flush=True)
