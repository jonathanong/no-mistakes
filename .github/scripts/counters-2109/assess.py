"""Offline complete-row review, using the unchanged earlier block estimator."""
import argparse,json,hashlib
from pathlib import Path
from contracts import CASES,ORDERS,rows_complete,parsed
from kernel_assess import summary
from transport import verify

def main():
    p=argparse.ArgumentParser();p.add_argument('--block',type=Path,action='append',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    assert len(a.block)==16 and len(set(a.block))==16 and not a.output.exists()
    rows=[]
    for path in a.block:
        verify(path.parent)
        terminal=json.loads((path/'terminal.json').read_text())
        assert terminal['status']=='PASS'and terminal['phase']=='block'and terminal['postchecks_pass']and terminal['rows']==4
        selected=json.loads((path/'rows.json').read_text())
        for row in selected:
            raw=path/str(row['position']);receipt=json.loads((raw/'raw.json').read_text())
            stdout=(raw/'raw.stdout').read_bytes();stderr=(raw/'raw.stderr').read_text()
            assert hashlib.sha256(stdout).hexdigest()==row['stdout_sha256']and len(stdout)==row['stdout_bytes']
            assert parsed(stdout,stderr,row['case'])==row['parsed']
            for key in ['exit','cpu','user','sys','wall']:assert row[key]==receipt[key]
            assert row['peak_rss_kib']==int((raw/'child-rss.txt').read_text())
        rows+=selected
    rows_complete(rows);metrics={}
    for case in CASES:
        fields=['wall','cpu','user','sys','peak_rss_kib']+(['projection_wall_ms']if case=='consumer-on'else[])
        for field in fields:
            blocks=[]
            for block,order in enumerate(ORDERS):
                selected=[r for r in rows if(r['case'],r['block'])==(case,block)]
                def value(r):return r['parsed'][field]if field=='projection_wall_ms'else r[field]
                left=[value(r)for r in selected if r['arm']=='baseline'];right=[value(r)for r in selected if r['arm']=='candidate']
                assert min(left)>0 and min(right)>0
                blocks.append((order,left,right))
            metrics[case+'/'+field]=summary(blocks)
    gate_metrics={k:v for k,v in metrics.items()if not k.endswith(('/user','/sys'))}
    result={'status':'COMPLETE_EVIDENCE_REQUIRES_INDEPENDENT_CORE_REVIEW','rows':64,'metrics':metrics,'full_counter_and_report_parity':True,'reproduced_regression_flags':[k for k,v in gate_metrics.items()if v['status']=='REGRESSION'],'uncertain_metrics':[k for k,v in gate_metrics.items()if v['status']=='INCONCLUSIVE'],'no_automatic_readiness':True,'interpretation':'Original4block estimator; smallsampleuncertainty retained. No2-10% target, gates changed, rawcrossVMpool, selectivecases or resampling.'}
    a.output.write_text(json.dumps(result,indent=2)+'\n')
if __name__=='__main__':main()
