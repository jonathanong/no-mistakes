"""Measure frozen release binaries in balanced fresh-process blocks (Linux)."""
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time

corpus = pathlib.Path(sys.argv[1]).resolve()
binaries = {'A': sys.argv[2], 'B': sys.argv[3]}
output = pathlib.Path(sys.argv[4]).resolve()
rows = []
for mode in ['off', 'on']:
    for scope in ['consuming', 'broad', 'disabled']:
        for jobs in ['1', '4', 'default']:
            orders = ['ABBA' if block % 2 == 0 else 'BAAB' for block in range(12)]
            orders += ['AAAA', 'BBBB']
            for block, order in enumerate(orders):
                for position, arm in enumerate(order):
                    command = [binaries[arm]]
                    if jobs != 'default':
                        command += ['--jobs', jobs]
                    command += ['check', '--root', str(corpus), '--config',
                                str(corpus / 'configs' / (scope + '.yml')), '--format', 'json']
                    if mode == 'on':
                        command += ['--timings', '--verbose-timings']
                    stdout_path = output.with_suffix('.stdout')
                    stderr_path = output.with_suffix('.stderr')
                    with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
                        started = time.perf_counter()
                        process = subprocess.Popen(command, stdout=stdout, stderr=stderr)
                        _, status, usage = os.wait4(process.pid, 0)
                        process.returncode = os.waitstatus_to_exitcode(status)
                        wall = time.perf_counter() - started
                    report = stdout_path.read_bytes()
                    json.loads(report)
                    stderr = stderr_path.read_bytes()
                    row = dict(mode=mode, scope=scope, jobs=jobs, block=block,
                               position=position, arm=arm, calibration=block >= 12,
                               wall=wall, user=usage.ru_utime, system=usage.ru_stime,
                               rss=usage.ru_maxrss, exit=process.returncode,
                               sha256=hashlib.sha256(report).hexdigest(),
                               stderr_sha256=hashlib.sha256(stderr).hexdigest())
                    rows.append(row)
                    output.write_text(json.dumps(rows, indent=2) + '\n')
            cell = [r for r in rows if (r['mode'], r['scope'], r['jobs']) == (mode, scope, jobs)]
            assert len({(r['exit'], r['sha256']) for r in cell}) == 1, (mode, scope, jobs)
            print(json.dumps(dict(mode=mode, scope=scope, jobs=jobs, samples=len(cell),
                                  identical_stdout_and_status=True)), flush=True)
