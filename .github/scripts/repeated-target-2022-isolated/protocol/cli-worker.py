"""Timed CLI request batch: file-backed output, no report parsing in the batch."""
import os
import json
from pathlib import Path
import subprocess
import sys

binary, source, count, diagnostics, output, command_json = sys.argv[1:]
directory = Path(output)
directory.mkdir()
for index in range(int(count)):
    flags = ['--timings', '--verbose-timings'] if diagnostics == 'on' else []
    with (directory / f'{index}.stdout').open('wb') as stdout, (directory / f'{index}.stderr').open('wb') as stderr:
        result = subprocess.run([binary, *flags, *json.loads(command_json), '--root', source, '--format', 'json'],
                                stdout=stdout, stderr=stderr, env=os.environ)
    (directory / f'{index}.exit').write_text(str(result.returncode))
    if result.returncode not in (0, 1):
        sys.exit(2)
print(count)
