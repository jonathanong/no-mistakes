"""Linux process-tree RSS sampler; source inputs are copied saved fixtures."""
import json
from pathlib import Path
import resource
import subprocess
import sys
import time

root, label, *command = sys.argv[1:]
root = Path(root)
started = time.perf_counter()
peak = 0
with (root / f'{label}.stdout').open('wb') as output, (root / f'{label}.stderr').open('wb') as errors:
    process = subprocess.Popen(command, stdout=output, stderr=errors)
    while process.poll() is None:
        pending = [process.pid]
        seen = set()
        rss = 0
        while pending:
            pid = pending.pop()
            if pid in seen:
                continue
            seen.add(pid)
            try:
                for line in Path(f'/proc/{pid}/status').read_text().splitlines():
                    if line.startswith('VmRSS:'):
                        rss += int(line.split()[1])
                for task in Path(f'/proc/{pid}/task').iterdir():
                    try:
                        pending.extend(int(child) for child in (task / 'children').read_text().split())
                    except (FileNotFoundError, ProcessLookupError):
                        pass
            except (FileNotFoundError, ProcessLookupError):
                pass
        peak = max(peak, rss)
        time.sleep(0.025)
    status = process.wait()
usage = resource.getrusage(resource.RUSAGE_CHILDREN)
result = dict(wallSeconds=time.perf_counter()-started, userSeconds=usage.ru_utime,
              systemSeconds=usage.ru_stime, peakTreeRssKiB=peak,
              maxProcessRssKiB=usage.ru_maxrss, exitCode=status)
(root / f'{label}.metrics.json').write_text(json.dumps(result))
print(json.dumps(result))
