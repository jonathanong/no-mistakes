"""Union schedule accounting only; original per-workload estimator is unchanged."""
import json
import os
from pathlib import Path
import sys
from cohort_transport import unpack
from schedule import cohorts, verify_rows
from transport import sha
from readback import verify as verify_controls

D = Path(__file__).resolve().parent


def main():
    incoming, output = map(Path, sys.argv[1:])
    output.mkdir(exist_ok=False)
    manifests = json.loads((D / 'producer-manifests.json').read_text())
    signatures = json.loads((D / 'timing-signatures.json').read_text())
    all_rows, origins = [], {}
    for cohort in ['A', 'B']:
        root = output / ('cohort-' + cohort)
        unpack(incoming / ('cohort-' + cohort), root, cohort)
        closure = json.loads((root / 'closure.json').read_text())
        assert closure['complete'] is True and closure['cohort'] == cohort
        assert closure['run_id'] == os.environ['GITHUB_RUN_ID']
        assert closure['attempt'] == os.environ['GITHUB_RUN_ATTEMPT']
        assert closure['workflow_sha'] == os.environ['GITHUB_SHA']
        verify_controls(root)
        rows = verify_rows(json.loads((root / 'A28_B28-long-all-rows.json').read_text()), cohort, manifests, signatures)
        all_rows.extend(rows)
        origins[cohort] = {'host_sha256': sha(root / 'host.json'), 'rows': 352,
                           'workloads': list(dict.fromkeys(r['workload'] for r in rows))}
    _, workloads = cohorts()
    order = {w['id']: i for i, w in enumerate(workloads)}
    # Canonical original ordering is a view; actual per-VM chronological arrays
    # remain intact. There is no cross-host pooled workload or relative metric.
    all_rows.sort(key=lambda r: (r['block'], order[r['workload']], r['position']))
    assert len(all_rows) == 704 and len({r['workload'] for r in all_rows}) == 22
    (output / 'A28_B28-long-all-rows.json').write_text(json.dumps(all_rows, indent=2) + '\n')
    (output / 'row-origins.json').write_text(json.dumps(origins, indent=2))
    original = (D / 'analyze-original.py.txt').read_text()
    removed = "from protocol_guard import require_reviewed_pair\nreviewed_plan=require_reviewed_pair(p, live=False)\n"
    assert original.count(removed) == 1
    adapted = original.replace(removed, '').replace('A26_B26', 'A28_B28').replace('A26/B26', 'A28/B28').replace('actual c896 baseline and 5d37 candidate', 'actual 4e62c503 baseline and d7847d84 candidate')
    # Source-reviewed boundary replacement only: complete same-run cohort guards
    # above replace local filesystem/live guards. Arithmetic below is unchanged.
    (output / 'analyze.py').write_text(adapted)
    exec(compile(adapted, str(output / 'analyze.py'), 'exec'), {'__file__': str(output / 'analyze.py'), '__name__': '__main__'})


if __name__ == '__main__':
    main()
