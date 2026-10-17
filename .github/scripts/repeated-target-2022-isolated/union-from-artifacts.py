"""Run locally after root independently downloads/digest-verifies eight artifacts."""
import argparse
import os
import json
from pathlib import Path
from cohort_transport import unpack
from retention import verify
from union import verify as verify_union

D = Path(__file__).resolve().parent
p = argparse.ArgumentParser()
p.add_argument('--artifact', type=Path, action='append', required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--unpacked-output', type=Path, required=True)
p.add_argument('--run-id', required=True)
p.add_argument('--attempt', required=True)
p.add_argument('--workflow-sha', required=True)
a = p.parse_args()
assert len(a.artifact) == 8 and not a.output.exists() and not a.unpacked_output.exists()
os.environ.update(GITHUB_REPOSITORY='jonathanong/no-mistakes', GITHUB_RUN_ID=a.run_id, GITHUB_RUN_ATTEMPT=a.attempt, GITHUB_SHA=a.workflow_sha, APPROVED_SHA=a.workflow_sha)
a.unpacked_output.mkdir()
roots = {}
for cohort, carrier in enumerate(a.artifact):
    artifact = a.unpacked_output / str(cohort)
    unpack(carrier, artifact, str(cohort))
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
assert sorted(roots) == list(range(8))
plan = json.loads((D / 'plan.json').read_text())
assert plan['status'] == 'REVIEWED_ISOLATED_EIGHT_COHORTS'
a.output.write_text(json.dumps(verify_union([roots[i] for i in range(8)], plan), indent=2))
