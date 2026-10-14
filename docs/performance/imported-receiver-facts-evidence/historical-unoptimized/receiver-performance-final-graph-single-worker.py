import json, os, statistics, subprocess, time
from pathlib import Path
root=Path('<WORKSPACE>/no-mistakes-receiver')
cases=[
 ('origin_small','graph',['test-cases/codebase-analysis/value-flow-origin-small/fixture','10']),
 ('origin_large','graph',['test-cases/codebase-analysis/value-flow-origin-large/fixture','10']),
 ('lazy_imported','graph',['test-cases/codebase-analysis/value-flow-imported-receiver-cache/fixture','10']),
]
rows=[]
for case,kind,args in cases:
 for round_number in range(4):
  for label in ['candidate','baseline','baseline','candidate']:
   binary=Path('<WORKSPACE>')/f'receiver-{label}-{"graph-perf" if kind=="graph" else "perf"}'
   start=time.perf_counter()
   process=subprocess.Popen([str(binary),str(root/args[0]),*args[1:]],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=dict(os.environ,RAYON_NUM_THREADS='1'))
   _,status,usage=os.wait4(process.pid,0)
   row=dict(case=case,kind=kind,binary=label,round=round_number,iterations=int(args[-1]),wall_s=time.perf_counter()-start,cpu_s=usage.ru_utime+usage.ru_stime,rss_kib=usage.ru_maxrss,report=process.stdout.read().decode().strip(),exit=os.waitstatus_to_exitcode(status),stderr=process.stderr.read().decode())
   process.returncode=row['exit']
   assert row['exit']==0 and not row['stderr'],row
   rows.append(row)
   Path('<WORKSPACE>/receiver-performance-final-graph-single-worker-raw.json').write_text(json.dumps(rows,indent=2))
summary=[]
for case,kind,args in cases:
 item=dict(case=case,kind=kind,iterations=int(args[-1]),samples_per_binary=8)
 for label in ['baseline','candidate']:
  group=[row for row in rows if row['case']==case and row['binary']==label]
  assert len({row['report'] for row in group})==1
  item[label]={metric:statistics.median(row[metric] for row in group) for metric in ['wall_s','cpu_s','rss_kib']}
  item[label]['report']=group[0]['report']
 if case!='lazy_imported': assert item['baseline']['report']==item['candidate']['report']
 else: assert int(item['baseline']['report'].split(':')[1])==0 and int(item['candidate']['report'].split(':')[1])==128
 summary.append(item)
Path('<WORKSPACE>/receiver-performance-final-graph-single-worker-summary.json').write_text(json.dumps(summary,indent=2))
print(json.dumps(summary,indent=2))
