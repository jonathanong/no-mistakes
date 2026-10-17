"""Command-boundary ownership only; no per-sample observer overhead."""
import json
import os
import subprocess
import time
from process_owner import OwnedProcess
import subreaper
from deadline import remaining


def command(argv, label, output, deadline, seconds, env=None):
    subreaper.require_empty()
    row = {'argv': argv, 'exit': None, 'complete': False}
    process = owner = None
    started = time.monotonic()
    try:
        with (output / (label + '.stdout')).open('wb') as out, (output / (label + '.stderr')).open('wb') as err:
            process = subprocess.Popen(argv, stdout=out, stderr=err, env=env,
                                       start_new_session=True)
            owner = OwnedProcess(process)
            process.wait(timeout=remaining(deadline, seconds))
            row['exit'] = process.returncode
            assert process.returncode == 0, (label, process.returncode)
            assert time.monotonic() < deadline
            row['complete'] = True
    finally:
        try:
            if owner is not None:
                row['cleanup'] = subreaper.finish(process, owner)
        finally:
            row['exit'] = process.returncode if process else None
            row['elapsed_seconds'] = time.monotonic() - started
            (output / (label + '.command.json')).write_text(json.dumps(row, indent=2))
    return row
