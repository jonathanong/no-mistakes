"""Retain one installed physical consumer tree for all matched sample pairs."""
from pathlib import Path
import subprocess

from common import save, workload

root = Path('fixture').resolve()
save('fixture-provenance.json', workload(root))
subprocess.run(['tar', '-czf', 'fixture.tar.gz', 'fixture'], check=True)
