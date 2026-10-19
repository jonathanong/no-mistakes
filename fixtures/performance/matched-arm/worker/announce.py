#!/usr/bin/env python3
import os
import sys
import time

print(f'fixture-leader:{os.getpid()}', file=sys.stderr, flush=True)
time.sleep(120)
