"""Unchanged original sampling kernel, fixed whole-workload cohort boundary."""
import hashlib,json,os
from pathlib import Path
import subprocess,sys,time
from schedule import cohorts,verify_rows

def sample(out, source, inputs, cohort, manifests, expected, deadline):
 assert cohort in ('A','B')
 plan,workloads=cohorts()
 selected=[w for w in workloads if w['id'] in plan[cohort]['workloads']]
 phase,pair,group='long','A28_B28','all'
 rows_path=out/'A28_B28-long-all-rows.json'
 assert not rows_path.exists(), 'Never overwrite prior observations'
 cpus=sorted(os.sched_getaffinity(0))[:4]
 assert cpus==[0,1,2,3]
 arm_map={'baseline':'A28','candidate':'B28'}
 calibration=json.loads((Path(__file__).parent/'calibration-batches.json').read_text())
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
 assert time.monotonic() < deadline, 'Cohort post-verification exceeded operation deadline'
 verify_rows(rows,cohort,manifests,expected)
 return rows
