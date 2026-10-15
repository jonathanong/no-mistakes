"""Summarize every preserved paired observation without filtering or pooling series."""
import hashlib
import json
import math
from pathlib import Path
import random
import statistics
import sys

out=Path(sys.argv[1])
results=[]
for series in ['fixed']:
    rows=json.loads((out/f'{series}-rows.json').read_text())
    expected=64 if series=='pilot' else 256
    assert len(rows)==expected
    assert all(r['exit']==0 and math.isclose(r['cpu_s'],r['user_s']+r['system_s']) for r in rows)
    for case in sorted({r['case'] for r in rows}):
        for demand in ['values','imports']:
            selected=[r for r in rows if r['case']==case and r['demand']==demand]
            assert len({r['signature'] for r in selected})==1
            blocks=[]
            for block in sorted({r['block'] for r in selected}):
                group=[r for r in selected if r['block']==block]
                assert len(group)==4
                means={arm:statistics.mean(r['cpu_s'] for r in group if r['arm']==arm) for arm in ['baseline','candidate']}
                blocks.append({'block':block,'order':group[0]['order'],'log_ratio':math.log(means['candidate']/means['baseline'])})
            point=math.expm1(statistics.mean(b['log_ratio'] for b in blocks))*100
            rng=random.Random(1972)
            draws=sorted(math.expm1(statistics.mean(rng.choice(blocks)['log_ratio'] for _ in blocks))*100 for _ in range(10000))
            orders={order:math.expm1(statistics.mean(b['log_ratio'] for b in blocks if b['order']==order))*100 for order in ['ABBA','BAAB']}
            arms={}
            for arm in ['baseline','candidate']:
                group=[r for r in selected if r['arm']==arm]
                cpu=[r['cpu_s'] for r in group]
                arms[arm]={'samples':len(group),'collections_per_sample':group[0]['collections'],'mean_cpu_s':statistics.mean(cpu),'mean_wall_s':statistics.mean(r['wall_s'] for r in group),'peak_rss_kib':max(r['rss_kib'] for r in group),'cpu_cv':statistics.stdev(cpu)/statistics.mean(cpu)}
            results.append({'series':series,'case':case,'demand':demand,'paired_cpu_percent':point,'descriptive_bootstrap_95_percent':[draws[249],draws[9749]],'orders_percent':orders,'arms':arms,'eligible_idle_window':series=='fixed'})
(out/'summary.json').write_text(json.dumps(results,indent=2)+'\n')
for r in results:
    print(r['series'],r['case'],r['demand'],f"{r['paired_cpu_percent']:+.3f}%",[round(x,3) for x in r['descriptive_bootstrap_95_percent']],r['orders_percent'])
