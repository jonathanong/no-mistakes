"""All retained long rows; paired four-process block is resampling unit."""
import json,math,random,statistics,hashlib
from pathlib import Path
p=Path(__file__).resolve().parent
metrics=['cpu_s','user_s','system_s','wall_s','rss_kib']
files=['A_B-long-controls-rows.json','A_B-long-hot-facts-rows.json','A_B-long-graph-effects-rows.json','B_C-long-controls-rows.json']
rng=random.Random(1972)
# Same 10,000 bootstrap block-index draws for every metric/workload.
draws=[[rng.randrange(8) for _ in range(8)] for _ in range(10000)]
def delta(logs):return 100*math.expm1(statistics.mean(logs))
def quantile(xs,q):
 xs=sorted(xs);v=(len(xs)-1)*q;i=int(v);return xs[i]+(xs[min(i+1,len(xs)-1)]-xs[i])*(v-i)
result={'method':{'point':'geometric mean of eight paired-block candidate/baseline ratios; each block averages its two processes per arm','interval':'95% percentile bootstrap of eight complete paired four-process blocks, 10000 draws, seed1972; descriptive small-sample intervals, not causal proof','order_split':'geometric mean block ratios within four ABBA or four BAAB blocks','raw_arm_means':'arithmetic mean across16 processes/arm; CPU is user+system, wall includes startup, RSS is process high-water KiB','no_exclusions':True,'historical1972':'Current A/B correctness costs and B/C optimization do not establish resolution of the original historical regression'},'files':{},'workloads':[]}
for name in files:
 raw=(p/name).read_bytes();rows=json.loads(raw);result['files'][name]={'rows':len(rows),'sha256':hashlib.sha256(raw).hexdigest()}
 for w in dict.fromkeys(r['workload'] for r in rows):
  selected=[r for r in rows if r['workload']==w];assert len(selected)==32
  entry={'pair':selected[0]['pair'],'group':selected[0]['group'],'workload':w,'collections':selected[0]['collections'],'metrics':{}}
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
pilot=p/'A_B-pilot-new-rows.json';assert len(json.loads(pilot.read_text()))==112
assert sum(x['rows'] for x in result['files'].values())==960
(p/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
lines=['All112pilot+960long rows retained. Units and bootstrap method: analysis.json.','pair workload CPU% [95%] ABBA% BAAB% wall% RSS%']
for x in result['workloads']:
 c=x['metrics']['cpu_s'];lines.append(f"{x['pair']} {x['workload']} {c['paired_delta_pct']:+.3f} [{c['ci95_pct'][0]:+.3f},{c['ci95_pct'][1]:+.3f}] {c['ABBA_pct']:+.3f} {c['BAAB_pct']:+.3f} {x['metrics']['wall_s']['paired_delta_pct']:+.3f} {x['metrics']['rss_kib']['paired_delta_pct']:+.3f}")
(p/'analysis-summary.txt').write_text('\n'.join(lines)+'\n');print('\n'.join(lines))
