"""Recompute every workload locally; never pool measurements across VMs."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

D = Path(__file__).resolve().parent
sys.path.insert(0, str(D / 'protocol'))
from paired_summary import assess, stable, validate


def verify(roots, plan):
    assert len(roots) == 5
    results, pins, indices = [], {}, []
    context = None
    count = 0
    for cohort, root in zip(plan['rounds'], roots):
        end = json.loads((root / 'public/cohort-END.json').read_text())
        assert end['cohort'] == cohort['id'] and end['indices'] == cohort['indices']
        assert end['status'] == 'COMPLETE invocation; numerical and row eligibility separate'
        assert 0 < end['elapsed_s'] <= 600
        actual = [end[k] for k in ['run_id', 'run_attempt', 'workflow_sha']]
        if context is None:
            context = actual
        assert actual == context
        indices.extend(cohort['indices'])
        manifest = json.loads((root / 'semantic-manifest.json').read_text())
        assert manifest['status'] == 'complete' and not manifest['failures'] and len(manifest['raw_files']) == 182
        for rel, expected in manifest['raw_files'].items():
            assert hashlib.sha256((root / rel).read_bytes()).hexdigest() == expected
        recorded = json.loads((root / f'public/assessments-round-{cohort["id"]}.json').read_text())
        assert [r['workload'] for r in recorded] == cohort['workloads']
        for name in cohort['workloads']:
            phases = {}
            for phase, expected in [('startup', 2), ('calibration', 2), ('pilot', 8), ('long', 32)]:
                suffix = '-rows' if phase in ['pilot', 'long'] else ''
                path = root / f'public/{name}-{phase}{suffix}.json'
                raw = path.read_bytes()
                rows = json.loads(raw)
                assert len(rows) == expected and all(r['valid'] for r in rows)
                assert all(r['affinity'] == end['resources_before']['selected_cpus'] and r['rayon_workers'] == 4 for r in rows)
                if phase in ['pilot', 'long']:
                    validate(rows, 1 if phase == 'pilot' else 4)
                phases[phase] = rows
                pins[f'{cohort["id"]}/{path.name}'] = hashlib.sha256(raw).hexdigest()
                count += expected
            assert stable(phases['pilot'])
            derived = assess(phases['long'])
            original = next(r for r in recorded if r['workload'] == name)
            assert derived['metrics'] == original['metrics'] and derived['status'] == original['status']
            results.append({'workload': name, 'cohort': cohort['id'], **derived})
    assert sorted(indices) == list(range(16)) and len(indices) == 16
    assert count == 704 and len(results) == 16 and sum(len(r['metrics']) for r in results) == 48
    return {'integrity': 'PASS', 'rows': count, 'workloads': results, 'raw_pins': pins,
            'cross_vm_pooling': False, 'numerical_verdict': 'UNCHANGED workload-local outcomes; root review required'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--cohort', action='append', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    plan = json.loads((D / 'plan.json').read_text())
    assert plan['status'] == 'REVIEWED_ISOLATED_FIVE_COHORTS'
    args.output.write_text(json.dumps(verify(args.cohort, plan), indent=2))
