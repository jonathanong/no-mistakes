"""Instrumented guest instruction work; these are not native CPU measurements."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time

out = Path(__file__).resolve().parent
profiles = out/'profiles'/'original-unoptimized'
profiles.mkdir()
tool_root = Path('<PROFILER_ROOT>')
valgrind = tool_root/'usr/bin/valgrind'
fixture = Path('<WORKSPACE>/no-mistakes-1972/test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts')
assert not (profiles/'work-raw.json').exists(), 'Do not overwrite profiles'
def affinity():
    os.sched_setaffinity(0, {0})
rows = []
for comparison, prefix in [('historical_original_unoptimized','')]:
    for demand in ['values','imports']:
        for index,label in enumerate(['baseline','candidate','candidate','baseline']):
            name = f'{prefix}{demand}-{index}-{label}-collect10'
            binary = Path('<WORKSPACE>')/f'receiver-{label}-perf'
            profile = profiles/f'{name}.callgrind'
            stdout = profiles/f'{name}.stdout'
            stderr = profiles/f'{name}.stderr'
            assert not any(p.exists() for p in [profile,stdout,stderr])
            command = [str(valgrind),'--tool=callgrind',
                f'--extra-debuginfo-path={tool_root}<SYSTEM>/lib/debug',
                '--collect-jumps=yes','--dump-instr=yes',f'--callgrind-out-file={profile}',
                str(binary),str(fixture),demand,'10']
            start = time.perf_counter()
            with stdout.open('wb') as output,stderr.open('wb') as errors:
                process = subprocess.Popen(command,stdout=output,stderr=errors,
                    preexec_fn=affinity,env=dict(os.environ,RAYON_NUM_THREADS='1',
                        VALGRIND_LIB=str(tool_root/'usr/libexec/valgrind'),DEBUGINFOD_URLS=''))
                _,status,usage = os.wait4(process.pid,0)
                process.returncode = os.waitstatus_to_exitcode(status)
            assert process.returncode == 0, stderr.read_text()
            content = profile.read_text()
            match = re.search(r'^summary:\s+(\d+)\s*$',content,re.M)
            assert match, profile
            row = dict(comparison=comparison,demand=demand,binary=label,order='ABBA',position=index,
                collections=10,rayon_workers=1,child_cpu_affinity=[0],
                instruction_metric='Callgrind guest Ir on protected original OPT0 artifacts, whole process including startup; not native retired instructions',
                guest_instructions=int(match.group(1)),profile=profile.name,
                profile_sha256=hashlib.sha256(profile.read_bytes()).hexdigest(),
                binary_sha256=hashlib.file_digest(binary.open('rb'),'sha256').hexdigest(),
                instrumented_wall_s=time.perf_counter()-start,
                instrumented_cpu_s=usage.ru_utime+usage.ru_stime,instrumented_rss_kib=usage.ru_maxrss,
                exit=process.returncode,report=stdout.read_text().strip(),command=command)
            assert row['report'] == ('128' if demand=='values' else '0'),row
            rows.append(row)
            (profiles/'work-raw.json').write_text(json.dumps(rows,indent=2)+'\n')
            print(json.dumps({key:row[key] for key in ['comparison','demand','binary','position','guest_instructions','instrumented_wall_s']}),flush=True)
