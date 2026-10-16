"""Retain partial and complete evidence without upload-invalid inner names."""
import os
from pathlib import Path
import sys
from cohort_transport import pack
from retain import compress

cohort = sys.argv[1]
assert cohort in ['A', 'B']
root = Path(os.environ['RUNNER_TEMP']) / ('mutable-root-B26-' + cohort)
if not root.exists():
    import json
    root.mkdir()
    (root / 'closure.json').write_text(json.dumps({'complete': False, 'cohort': cohort, 'failure': 'Dependency/setup phase ended before native entrypoint'}))
assert root.is_dir()
dependency = Path(os.environ['RUNNER_TEMP']) / ('B26-dependencies-' + cohort)
if dependency.exists():
    import shutil
    shutil.copytree(dependency, root / 'JS-dependency-setup')
# Native assets remain staged separately and are already immutable API objects;
# this carrier retains all raw execution reports, not redundant executable copies.
pair = root / 'pair'
if pair.exists():
    import shutil
    shutil.rmtree(pair)
compress(root)
pack(root, Path(os.environ['RUNNER_TEMP']) / ('B26-transport-' + cohort), cohort)
