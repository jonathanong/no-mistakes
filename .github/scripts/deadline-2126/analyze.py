"""Descriptive within-cell paired block ratios; two blocks cannot establish tails."""
import json,math,sys
from pathlib import Path

def summarize(rows):
    assert len(rows)==16
    result={}
    for timeout in [120,0]:
        for metric in ['wall','cpu','rss_kib']:
            differences=[]
            for block,order in enumerate(['ABBA','BAAB']):
                r=[x for x in rows if x['timeout']==timeout and x['block']==block]
                assert ''.join('A'if x['arm']=='baseline'else'B'for x in r)==order
                assert all(x['exit']==0 and x[metric]>0 for x in r)
                means={a:sum(math.log(x[metric])for x in r if x['arm']==a)/2 for a in ['baseline','candidate']}
                differences.append(means['candidate']-means['baseline'])
            result[f'{timeout}/{metric}']={'geomean_percent':100*math.expm1(sum(differences)/2),'order_percent':[100*math.expm1(x)for x in differences],'blocks':2,'inference':'DESCRIPTIVE_ONLY; no reliable CI/tail or speedup gate from two blocks'}
    return result
if __name__=='__main__':
    rows=json.loads(Path(sys.argv[1]).read_text());print(json.dumps(summarize(rows),indent=2))
