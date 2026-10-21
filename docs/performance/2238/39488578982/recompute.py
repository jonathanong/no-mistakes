"""Verify preserved file hashes and recompute all raw resources without harness imports."""
from pathlib import Path
import hashlib
import json
import math
import statistics
import sys

root = Path(__file__).resolve().parent
for name, digest in json.loads((root/'checksums.json').read_text()).items():
    assert hashlib.sha256((root/name).read_bytes()).hexdigest() == digest, name
summary = json.loads((root/'summary.json').read_text())
results=[]; count=0
for cell in summary['cells']:
    raw=root/cell['artifact']/'raw'; rows=json.loads((raw/'samples.json').read_text()); stages={}
    assert len(rows)==24
    for stage in ('aa','bccb'):
        values=[]; pairs=[]
        for block in range(3):
            current=[r for r in rows if r['stage']==stage and r['block']==block]
            assert len(current)==4
            assert [r['arm'] for r in current]==['b','c','c','b']
            for ordinal,row in enumerate(current):
                assert row==rows[(0 if stage=='aa' else 12)+block*4+ordinal]
                assert row['eligible'] and not row['deadlineHit'] and row['exit']==1
                values.append({'arm':row['arm'],'wallSeconds':row['wallSeconds'],
                    'cpuSeconds':row['userSeconds']+row['systemSeconds'],'peakRssKiB':row['peakRssKiB']}); count+=1
            for b,c in [(0,1),(3,2)]:
                a,z=values[-4+b],values[-4+c]
                pairs.append({k:100*(z[k]/a[k]-1) for k in ('wallSeconds','cpuSeconds','peakRssKiB')})
        means={k:statistics.mean(p[k] for p in pairs) for k in pairs[0]}
        spread={k:max(abs(p[k]) for p in pairs) for k in pairs[0]}; declared=cell['stages'][stage]
        for key in means:
            assert math.isclose(means[key],declared['meanPairPercentDelta'][key],abs_tol=1e-9)
            assert math.isclose(spread[key],declared['maxAbsolutePairPercentDelta'][key],abs_tol=1e-9)
            for role,arm in [('baseline','b'),('candidate','c')]:
                actual=statistics.mean(v[key] for v in values if v['arm']==arm)
                assert math.isclose(actual,declared['roleStats'][role][key]['mean'],abs_tol=1e-9)
        stages[stage]={'meanPairPercentDelta':means,'maxAbsolutePairPercentDelta':spread}
    decision=abs(stages['bccb']['meanPairPercentDelta']['wallSeconds'])<=stages['aa']['maxAbsolutePairPercentDelta']['wallSeconds']
    assert decision==cell['withinObservedAAWallSpread']
    results.append({'jobs':cell['jobs'],'diagnostics':cell['diagnostics'],'ledger':cell['ledger'],'withinSpread':decision,'stages':stages})
assert count==288 and len(results)==12
assert {'count':count,'cells':results} == json.loads((root/'root-independent-resource-recompute.json').read_text())
print(json.dumps({'arms':count,'offCells':[{'jobs':c['jobs'],'diagnostics':c['diagnostics'],'withinSpread':c['withinSpread'],
    'wallPct':c['stages']['bccb']['meanPairPercentDelta']['wallSeconds'],
    'aaSpread':c['stages']['aa']['maxAbsolutePairPercentDelta']['wallSeconds']}
    for c in results if c['ledger']=='off']}))
