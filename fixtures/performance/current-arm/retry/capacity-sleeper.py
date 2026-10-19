#!/usr/bin/env python3
import os
import time
from pathlib import Path
Path(os.environ['CAPACITY_CHILD_PID']).write_text(str(os.getpid()))
time.sleep(30)
