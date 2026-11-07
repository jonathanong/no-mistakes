"""Verify native wait4 includes resources of a completed, waited descendant."""
import os
import time

child = os.fork()
if child == 0:
    data = bytearray(64 * 1024 * 1024)
    for offset in range(0, len(data), 4096):
        data[offset] = 1
    until = time.process_time() + .15
    while time.process_time() < until:
        pass
    os._exit(0)
_, status = os.waitpid(child, 0)
raise SystemExit(os.waitstatus_to_exitcode(status))
