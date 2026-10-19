#!/usr/bin/env python3
from pathlib import Path
import subprocess
import sys

# A successful leader exit must not hide an unobserved descendant's resources.
child = subprocess.Popen([sys.executable, str(Path(__file__).with_name('sleeper.py'))])
print(f'fixture-child:{child.pid}', file=sys.stderr)
print('{"advisories":[],"codebase":[],"integration":[],"queues":[],"react":[],"rules":[],"warnings":[]}')
