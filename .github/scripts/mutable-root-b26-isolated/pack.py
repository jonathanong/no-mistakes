"""Retain partial and complete evidence without upload-invalid inner names."""
import os
from pathlib import Path
import sys
from cohort_transport import pack
from retain import compress

cohort = sys.argv[1]
assert cohort in ['A', 'B']
root = Path(os.environ['RUNNER_TEMP']) / ('mutable-root-B26-' + cohort)
assert root.is_dir()
# Native assets remain staged separately and are already immutable API objects;
# this carrier retains all raw execution reports, not redundant executable copies.
pair = root / 'pair'
if pair.exists():
    import shutil
    shutil.rmtree(pair)
compress(root)
pack(root, Path(os.environ['RUNNER_TEMP']) / ('B26-transport-' + cohort), cohort)
