"""Saved lookup surrogate: successful JSON plus a separate-SID descendant."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

here = Path(__file__).resolve().parent
pidfile = Path(sys.argv[1])
subprocess.Popen([sys.executable, str(here / 'orphan_child.py'), str(pidfile)], start_new_session=True)
deadline = time.monotonic() + 3
while not pidfile.exists():
    assert time.monotonic() < deadline
    time.sleep(.01)
print((here / 'actual-A-job-clock.json').read_text(), end='')
