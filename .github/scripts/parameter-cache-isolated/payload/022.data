"""All retained long rows; paired four-process block is resampling unit."""
import json,math,random,statistics,hashlib
from pathlib import Path
p=Path(__file__).resolve().parent
plan=json.loads((p/'performance-source-plan.json').read_text());assert plan['status']=='REVIEWED'
assert json.loads((p/'measurement/END.json').read_text())['complete'] is True
assert not (p/'analysis.json').exists()
metrics=['cpu_s','wall_s','rss_kib']
files=['measurement/rows.json']
rng=random.Random(1972)
# Same 10,000 bootstrap block-index draws for every metric/workload.
draws=[[rng.randrange(8) for _ in range(8)] for _ in range(10000)]
def delta(logs):return 100*math.expm1(statistics.mean(logs))
def quantile(xs,q):
 xs=sorted(xs);v=(len(xs)-1)*q;i=int(v);return xs[i]+(xs[min(i+1,len(xs)-1)]-xs[i])*(v-i)
result={'method':{'point':'geometric mean of eight paired-block candidate/baseline ratios; each block averages its two processes per arm','interval':'95% percentile bootstrap of eight complete paired four-process blocks, 10000 draws, seed1972; descriptive small-sample intervals, not causal proof','order_split':'geometric mean block ratios within four ABBA or four BAAB blocks','raw_arm_means':'arithmetic mean across16 processes/arm; CPU is user+system, wall includes startup, RSS is process high-water KiB','no_exclusions':True,'scope':'Current 2076 source-equivalent release artifacts only; no historical pooling or causal allocation claims'},'files':{},'workloads':[]}
for name in files:
 raw=(p/name).read_bytes();rows=json.loads(raw)
 expected=[(b,w['id'],i,'baseline' if letter=='A' else 'candidate') for b,order in enumerate(plan['orders']) for w in plan['workloads'] for i,letter in enumerate(order)]
 assert [(r['block'],r['workload'],r['position'],r['arm']) for r in rows]==expected
 result['files'][name]={'rows':len(rows),'sha256':hashlib.sha256(raw).hexdigest()}
 for w in dict.fromkeys(r['workload'] for r in rows):
  selected=[r for r in rows if r['workload']==w];assert len(selected)==32
  entry={'pair':'current-main/candidate','group':'diagnostics-on' if w.endswith('-timings') else 'diagnostics-off','workload':w,'metrics':{}}
  assert all(r['exit']==0 for r in selected)
  for m in metrics:
   blocks=[]
   for b in range(8):
    a=[r[m] for r in selected if r['block']==b and r['arm']=='baseline'];c=[r[m] for r in selected if r['block']==b and r['arm']=='candidate'];assert len(a)==len(c)==2
    assert statistics.mean(a)>0 and statistics.mean(c)>0
    blocks.append(math.log(statistics.mean(c)/statistics.mean(a)))
   bs=[delta([blocks[i] for i in draw]) for draw in draws]
   entry['metrics'][m]={'baseline_mean':statistics.mean(r[m] for r in selected if r['arm']=='baseline'),'candidate_mean':statistics.mean(r[m] for r in selected if r['arm']=='candidate'),'paired_delta_pct':delta(blocks),'ci95_pct':[quantile(bs,.025),quantile(bs,.975)],'ABBA_pct':delta(blocks[::2]),'BAAB_pct':delta(blocks[1::2]),'block_delta_pct':[100*math.expm1(x) for x in blocks],'position_means':{str(i):statistics.mean(r[m] for r in selected if r['position']==i) for i in range(4)}}
  result['workloads'].append(entry)
assert sum(x['rows'] for x in result['files'].values())==256
result['gate']={'adverse':[{'workload':x['workload'],'metric':m} for x in result['workloads'] for m,v in x['metrics'].items() if v['ci95_pct'][0]>0], 'scale_cpu_benefit':all(next(x for x in result['workloads'] if x['workload']==w)['metrics']['cpu_s']['ci95_pct'][1]<0 for w in ['scale','scale-timings'])}
result['gate']['pass']=not result['gate']['adverse'] and result['gate']['scale_cpu_benefit']
(p/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
lines=['All256 current paired rows retained; no pilot or calibration. Units and bootstrap method: analysis.json.','pair workload CPU% [95%] ABBA% BAAB% wall% RSS%']
for x in result['workloads']:
 c=x['metrics']['cpu_s'];lines.append(f"{x['pair']} {x['workload']} {c['paired_delta_pct']:+.3f} [{c['ci95_pct'][0]:+.3f},{c['ci95_pct'][1]:+.3f}] {c['ABBA_pct']:+.3f} {c['BAAB_pct']:+.3f} {x['metrics']['wall_s']['paired_delta_pct']:+.3f} {x['metrics']['rss_kib']['paired_delta_pct']:+.3f}")
(p/'analysis-summary.txt').write_text('\n'.join(lines)+'\n');print('\n'.join(lines))
