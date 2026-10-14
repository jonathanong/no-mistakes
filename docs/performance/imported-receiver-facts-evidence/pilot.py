"""Raw batch-size pilot; run only after all builds/CLI workloads are idle."""
import json
import os
from pathlib import Path
import subprocess
import time

out = Path(__file__).resolve().parent
assert not (out/'pilot-raw.json').exists(), 'Do not overwrite the pilot evidence'
root = Path('<WORKSPACE>/no-mistakes-1972')
fixtures = {
    'no_writes': root/'test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts',
    'shared_histories': root/'test-cases/codebase-analysis/import-facts/fixture/many-late-defaults.mts',
}
rows = []
for prefix in ['', 'current-']:
    for case, fixture in fixtures.items():
        for count in ([1,100,1000] if case == 'no_writes' else [1,10]):
            for demand in ['values', 'imports']:
                for label in ['baseline','candidate','candidate','baseline']:
                    start = time.perf_counter()
                    process = subprocess.Popen([str(out/f'{prefix}{label}-facts'),str(fixture),demand,str(count)],
                        stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=dict(os.environ,RAYON_NUM_THREADS='4'))
                    _,status,usage = os.wait4(process.pid,0)
                    process.returncode = os.waitstatus_to_exitcode(status)
                    row = dict(comparison='historical' if not prefix else 'merged_1961_integration',case=case,
                        iterations=count,demand=demand,binary=label,wall_s=time.perf_counter()-start,
                        cpu_s=usage.ru_utime+usage.ru_stime,rss_kib=usage.ru_maxrss,
                        report=process.stdout.read().decode().strip(),stderr=process.stderr.read().decode(),exit=process.returncode)
                    assert row['exit'] == 0 and not row['stderr'], row
                    rows.append(row)
                    (out/'pilot-raw.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2))
