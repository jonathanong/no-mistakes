"""Create a separate immutable service corpus; never alter existing corpora."""
import pathlib
import shutil
import subprocess
import sys

source = pathlib.Path(__file__).resolve().parent
root = pathlib.Path(sys.argv[1]).resolve()
root.mkdir()
(root / 'source').mkdir()
(root / 'shared').mkdir()
(root / 'configs').mkdir()
for name in ['default.yml', 'import-only.yml']:
    shutil.copy(source / 'configs' / name, root / 'configs' / name)
shutil.copy(source / 'helper.mts', root / 'shared/helper.mts')
seed = (source / 'seed.mts').read_text()
for index in range(256):
    body = ["import { helper } from '../shared/helper.mts';\n"]
    body.extend(seed.replace('BLOCK', str(block)) for block in range(64))
    (root / f'source/service-{index:03}.mts').write_text(''.join(body))
subprocess.run(['git', 'init', '-q', str(root)], check=True)
