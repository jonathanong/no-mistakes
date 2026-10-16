from pathlib import Path
import subprocess
import sys
import time

here = Path(__file__).resolve().parent
child = subprocess.Popen([sys.executable, str(here / 'orphan_child.py'), sys.argv[1]], start_new_session=True)
deadline = time.monotonic() + 3
while not Path(sys.argv[1]).exists():
    assert time.monotonic() < deadline
    time.sleep(.01)
if sys.argv[2] == 'timeout':
    time.sleep(30)
# Deliberately do not wait: the host subreaper must adopt this separate SID.
