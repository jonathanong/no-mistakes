"""One owned sampling subprocess; the parent enforces the actual-job deadline."""
import json
import os
from pathlib import Path
import sys
from cohort_sample import sample

D = Path(__file__).resolve().parent
output, source, cohort, end = sys.argv[1:]
output, source = Path(output), Path(source)
manifests = json.loads((D / 'producer-manifests.json').read_text())
signatures = json.loads((D / 'timing-signatures.json').read_text())
sample(output, source, Path('/tmp/nm2019-three-arm-evidence/inputs'), cohort,
       manifests, signatures, float(end))
