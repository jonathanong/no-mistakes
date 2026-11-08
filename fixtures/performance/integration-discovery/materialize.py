"""Expand saved source into one immutable performance corpus, outside test code."""
import pathlib
import shutil
import subprocess
import sys

source = pathlib.Path(__file__).resolve().parent
root = pathlib.Path(sys.argv[1]).resolve()
root.mkdir()
(root / 'selected').mkdir()
(root / 'source').mkdir()
shutil.copy(source / 'helper.ts', root / 'selected/helper.ts')
shutil.copy(source / 'suite.test.ts', root / 'selected/suite.test.ts')
shutil.copytree(source / 'configs', root / 'configs')
seed = (source / 'seed.tsx').read_text()
for index in range(256):
    body = ["import { readBoundary } from '../selected/helper';\n"]
    for repeat in range(64):
        body.append(seed.split('\n', 1)[1].replace('Panel', f'Panel{repeat}').replace('renderCard', f'renderCard{repeat}'))
    (root / f'source/panel-{index:03}.tsx').write_text(''.join(body))
subprocess.run(['git', 'init', '-q', str(root)], check=True)
