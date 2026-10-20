"""Saved tiny native-child control; its own current image should stay small."""
import json
import os
from pathlib import Path
import resource

status = Path('/proc/self/status')
current = {}
if status.exists():
    for line in status.read_text().splitlines():
        key, _, value = line.partition(':')
        if key in ('VmRSS', 'VmHWM'):
            current[key] = int(value.split()[0])
print(json.dumps({'pid': os.getpid(), 'usagePeak': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
                  'procKiB': current}))
