import json,os,subprocess,statistics,time
from pathlib import Path
w=Path('<WORKSPACE>');fixture=w/'no-mistakes-receiver/test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts';rows=[]
for demand in ['values','imports']:
 for round in range(4):
  for label in ['candidate','baseline','baseline','candidate']:
   start=time.perf_counter();p=subprocess.Popen([str(w/f'receiver-{label}-perf'),str(fixture),demand,'100'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=dict(os.environ,RAYON_NUM_THREADS='4'))
   _,status,u=os.wait4(p.pid,0);p.returncode=os.waitstatus_to_exitcode(status)
   row=dict(demand=demand,binary=label,round=round,cpu_s=u.ru_utime+u.ru_stime,wall_s=time.perf_counter()-start,rss_kib=u.ru_maxrss,report=p.stdout.read().decode().strip(),stderr=p.stderr.read().decode(),exit=p.returncode)
   assert not row['exit'] and not row['stderr'],row
   rows.append(row);(w/'receiver-no-writes-recheck-raw.json').write_text(json.dumps(rows,indent=2))
summary=[]
for demand in ['values','imports']:
 item=dict(demand=demand,iterations=100,samples_per_binary=8,order='BAAB')
 for label in ['baseline','candidate']:
  g=[r for r in rows if r['demand']==demand and r['binary']==label];assert len({r['report'] for r in g})==1
  item[label]={k:statistics.median(r[k] for r in g) for k in ['cpu_s','wall_s','rss_kib']};item[label]['report']=g[0]['report']
 assert item['baseline']['report']==item['candidate']['report'];summary.append(item)
(w/'receiver-no-writes-recheck-summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary,indent=2))
