"""Matched fresh-process fact collection; run only in a coordinated idle window."""
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time

root, out = map(Path, sys.argv[1:3])
mode = sys.argv[3]
assert mode == 'fixed'
raw_path = out / f'{mode}-rows.json'
assert not raw_path.exists(), 'Preserve existing observations'
cpus = sorted(os.sched_getaffinity(0))[:4]
assert len(cpus) == 4
cases = {
    'no_writes': ('test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts', 1000),
    'writes': ('test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts', 20),
    'shared_histories': ('test-cases/codebase-analysis/import-facts/fixture/many-late-defaults.mts', 1),
    'jsx': ('test-cases/ast-snippets/selectors/fixture/app-text-targets.tsx', 100),
}
manifests = {arm: json.loads((out/f'{arm}-manifest.json').read_text()) for arm in ['baseline','candidate']}
assert manifests['baseline']['lock_sha256'] == manifests['candidate']['lock_sha256']
for key in ['rustc','release_profile','features','harness_sha256']:
    assert manifests['baseline'][key] == manifests['candidate'][key]
assert manifests['baseline']['binary_sha256'] != manifests['candidate']['binary_sha256']

def affinity():
    os.sched_setaffinity(0, cpus)

preset = json.loads((out/'batches.json').read_text())
batches = {(case,demand):preset[case+'|'+demand] for case in cases for demand in ['values','imports']}
rows=[]
orders=['ABBA','BAAB']*4
for block, order in enumerate(orders):
    for case,(relative,_) in cases.items():
        for demand in ['values','imports']:
            for position, letter in enumerate(order):
                arm = 'baseline' if letter=='A' else 'candidate'
                start=time.perf_counter()
                child=subprocess.Popen([str(out/f'{arm}-facts'),str(root/relative),demand,str(batches[(case,demand)])],
                    stdout=subprocess.PIPE,stderr=subprocess.PIPE,
                    env=dict(os.environ,RAYON_NUM_THREADS='4'),preexec_fn=affinity)
                _,status,usage=os.wait4(child.pid,0)
                child.returncode=os.waitstatus_to_exitcode(status)
                stdout=child.stdout.read();stderr=child.stderr.read()
                row=dict(block=block,order=order,position=position,case=case,demand=demand,arm=arm,
                    collections=batches[(case,demand)],cpu_s=usage.ru_utime+usage.ru_stime,user_s=usage.ru_utime,
                    system_s=usage.ru_stime,wall_s=time.perf_counter()-start,rss_kib=usage.ru_maxrss,
                    affinity=cpus,rayon_workers=4,exit=child.returncode,signature=stdout.decode().strip(),
                    stdout_sha256=hashlib.sha256(stdout).hexdigest(),stderr_sha256=hashlib.sha256(stderr).hexdigest())
                rows.append(row)
                raw_path.write_text(json.dumps(rows,indent=2)+'\n')
                assert child.returncode==0 and not stderr,row
                assert len({r['signature'] for r in rows if r['case']==case and r['demand']==demand})==1,row
    print(f'{mode} block {block+1}/{len(orders)} complete',flush=True)
