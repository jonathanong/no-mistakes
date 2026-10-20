#!/usr/bin/env python3
"""One small fresh interpreter owns one native wait4 sample, then exits."""
import json
import os
from pathlib import Path
import resource
import runpy
import sys


def launcher():
    # Current resident size is separate from high-water history across exec.
    fields = {}
    status = Path('/proc/self/status')
    if status.exists():
        for line in status.read_text().splitlines():
            key, _, value = line.partition(':')
            if key in ('VmRSS', 'VmHWM'):
                fields[key + 'KiB'] = int(value.split()[0])
    else:
        import subprocess
        fields['VmRSSKiB'] = int(subprocess.check_output(
            ['ps', '-o', 'rss=', '-p', str(os.getpid())], text=True).strip())
    return {'pid': os.getpid(), 'freshInterpreter': True,
            'ru_maxrss_raw': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
            'ru_maxrss_units': 'bytes' if sys.platform == 'darwin' else 'KiB', **fields}


def main():
    payload = json.load(sys.stdin)
    shared = runpy.run_path(str(Path(__file__).with_name('mimalloc-measure.py')))
    payload['metadata']['sampler_launcher'] = launcher()
    shared['measure_child'](**payload)


if __name__ == '__main__':
    main()
