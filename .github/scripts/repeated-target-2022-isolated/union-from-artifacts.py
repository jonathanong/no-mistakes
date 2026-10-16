"""Run locally after root independently downloads/digest-verifies five artifacts."""
import argparse
import json
from pathlib import Path
from retention import verify
from union import verify as verify_union

D = Path(__file__).resolve().parent
p = argparse.ArgumentParser()
p.add_argument('--artifact', type=Path, action='append', required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
assert len(a.artifact) == 5 and not a.output.exists()
roots = {}
for artifact in a.artifact:
    ends = list(artifact.rglob('cohort-END.json'))
    indices = list(artifact.rglob('retained-evidence-index.json'))
    closures = list(artifact.rglob('closure.json'))
    assert len(ends) == len(indices) == len(closures) == 1
    root = ends[0].parent.parent
    end = json.loads(ends[0].read_text())
    closure = json.loads(closures[0].read_text())
    assert closure['source_pins_after_exact'] and closure['within_job_budget']
    assert closure['cohort'] == end['cohort'] and end['cohort'] not in roots
    verify(root, json.loads(indices[0].read_text())['files'])
    roots[end['cohort']] = root
assert sorted(roots) == list(range(5))
plan = json.loads((D / 'plan.json').read_text())
assert plan['status'] == 'REVIEWED_ISOLATED_FIVE_COHORTS'
a.output.write_text(json.dumps(verify_union([roots[i] for i in range(5)], plan), indent=2))
