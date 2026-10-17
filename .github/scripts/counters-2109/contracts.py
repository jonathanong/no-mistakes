"""Pure admission rules, including exact counter presence; no native calls."""
import json,math
from pathlib import Path
CASES=('consumer-off','consumer-on','control-off','control-on')
ORDERS=('ABBA','BAAB','BAAB','ABBA')

def parsed(stdout,stderr,case):
    report=json.loads(stdout)
    assert set(report)=={'advisories','codebase','integration','queues','react','rules','warnings'}
    work={};snapshots=[]
    for line in stderr.splitlines():
        if line.startswith('[work] '):
            key,value=line[7:].rsplit(': ',1)
            assert key not in work and value.isdecimal()
            work[key]=int(value)
        elif line.startswith('[progress] '):snapshots.append(json.loads(line[11:]))
    projection=None
    if case.endswith('-on'):
        assert snapshots and not snapshots[-1]['active']
        phases=[p for p in snapshots[-1]['completed']if p['label']=='postgres.annotation.project']
        if case.startswith('consumer'):
            assert len(phases)==1 and not phases[0]['parallel'] and phases[0]['durationMs']>0
            projection=phases[0]['durationMs']
            for name in ['modules','target_resolutions','reachability_walks']:
                assert work.get('postgres.annotation.'+name,0)>0
        else:
            assert not phases and all(v==0 for k,v in work.items()if k.startswith('postgres.annotation.'))
    else:assert not work and not snapshots
    return {'report':report,'work':work,'projection_wall_ms':projection}

def parity(rows):
    for scope in ['consumer','control']:
        selected=[r for r in rows if r['case'].startswith(scope)]
        if selected:
            assert len({(r['exit'],r['stdout_sha256'],r['stdout_bytes'])for r in selected})==1,'whole output/exit mismatch'
            assert all(r['exit']in[0,1]for r in selected),'invalid analysis exit'
    for case in CASES:
        selected=[r for r in rows if r['case']==case]
        if selected:assert all(r['parsed']['work']==selected[0]['parsed']['work']for r in selected),'full counter ledger or explicit presence changed'

def seal(rows):
    assert len(rows)==8 and {(r['case'],r['arm'])for r in rows}=={(c,a)for c in CASES for a in ['baseline','candidate']}
    parity(rows)
    for arm in ['baseline','candidate']:
        for mode in ['off','on']:
            cross_scope(next(r for r in rows if(r['arm'],r['case'])==(arm,'consumer-'+mode)),next(r for r in rows if(r['arm'],r['case'])==(arm,'control-'+mode)))
    cap=math.ceil(1.25*max(r['wall']for r in rows))+5
    assert 0<cap<=180,'STOP before timing: preflight capacity'
    return {'child_seconds':cap,'blocks_per_case':4,'orders':list(ORDERS),'rows_total':64,'rule':'duration only; frozen before first target sample'}

def fit(cap,remaining,after_warmup=False):
    assert 0<cap<=180
    assert (4 if after_warmup else 6)*cap+(180 if after_warmup else 300)<=remaining,'STOP before timing: whole block cannot fit'

def rows_complete(rows):
    assert len(rows)==64
    assert len({(r['case'],r['block'],r['position'])for r in rows})==64
    parity(rows)
    for case in CASES:
        for block,order in enumerate(ORDERS):
            group=sorted([r for r in rows if(r['case'],r['block'])==(case,block)],key=lambda r:r['position'])
            assert ''.join('A'if r['arm']=='baseline'else'B'for r in group)==order
            assert len({r['machine_sha256']for r in group})==1
    for key in ['pair_sha256','fixture_sha256','protocol_sha256']:
        assert len({r[key]for r in rows})==1

def cross_scope(consumer,control):
    """Only the intentionally removed annotation rule findings may disappear."""
    left=json.loads(json.dumps(consumer['parsed']['report']))
    removed=[r for r in left['rules']if r.get('rule')=='postgres-require-query-annotation']
    left['rules']=[r for r in left['rules']if r.get('rule')!='postgres-require-query-annotation']
    assert left==control['parsed']['report'],'non-annotation report fields/findings changed across control'
    if not removed:assert consumer['exit']==control['exit'],'exit changed without removed findings'
    else:assert consumer['exit']==1 and control['exit']in[0,1]
    return len(removed)
