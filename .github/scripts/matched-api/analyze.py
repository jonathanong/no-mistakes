"""Two within-cell matched block ratios; no cross-pair pooling or invented CI."""
import json,math,sys
from pathlib import Path

def summary(rows):
    assert len(rows)==8
    result={}
    for field in ['wall_seconds','cpu_seconds','user_seconds','sys_seconds','peak_rss_kib']:
        change=[]
        for block,order in enumerate(['ABBA','BAAB']):
            group=[r for r in rows if r['block']==block];assert ''.join('A'if r['arm']=='baseline'else'B'for r in group)==order
            assert [r['position']for r in group]==list(range(4));assert all(r['exit']==0 and r['worker'][field]>=0 for r in group)
            if any(r['worker'][field]==0 for r in group):change.append(None);continue
            a=sum(math.log(r['worker'][field])for r in group if r['arm']=='baseline')/2;b=sum(math.log(r['worker'][field])for r in group if r['arm']=='candidate')/2;change.append(100*math.expm1(b-a))
        result[field]={'ABBA_percent':change[0],'BAAB_percent':change[1],'inference':'DESCRIPTIVE_ONLY_TWO_BLOCKS; no universalwin/regressionclassification'}
    return result
if __name__=='__main__':print(json.dumps(summary(json.loads(Path(sys.argv[1]).read_text())),indent=2))
