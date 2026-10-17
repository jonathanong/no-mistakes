"""Fixed whole-workload cohorts; no estimator or acceptance changes."""
import hashlib
import math
import json
from pathlib import Path

D = Path(__file__).resolve().parent
ORDERS = ['ABBA', 'BAAB'] * 4


def cohorts():
    plan = json.loads((D / 'cohorts-DRAFT.json').read_text())['cohorts']
    workloads = json.loads((D / 'workloads.json').read_text())
    names = [w['id'] for w in workloads]
    assert len(names) == len(set(names)) == 22
    flat = [w for c in plan.values() for w in c['workloads']]
    assert len(flat) == len(set(flat)) == 22 and set(flat) == set(names)
    assert set(plan) == {'A', 'B'}
    for c in plan.values():
        assert len(c['workloads']) == 11 and c['rows'] == 352
    return plan, workloads


def expected_keys(cohort):
    plan, _ = cohorts()
    return [(b, w, p, 'baseline' if arm == 'A' else 'candidate')
            for b, order in enumerate(ORDERS)
            for w in plan[cohort]['workloads'] for p, arm in enumerate(order)]


def verify_rows(rows, cohort, manifests, signatures):
    keys = [(r['block'], r['workload'], r['position'], r['arm']) for r in rows]
    assert keys == expected_keys(cohort), 'Incomplete, duplicate, reordered, or split workload schedule'
    _, workloads = cohorts()
    by_name = {w['id']: w for w in workloads}
    calibration = json.loads((D / 'calibration-batches.json').read_text())
    for r in rows:
        assert (r['pair'], r['phase'], r['group']) == ('A28_B28', 'long', 'all')
        w = by_name[r['workload']]
        assert r['collections'] == w.get('fixed_batch', calibration.get(w['id']))
        assert r['stderr_sha256'] == hashlib.sha256(b'').hexdigest()
        assert r['stdout_sha256'] == hashlib.sha256((str(r['signature']) + '\n').encode()).hexdigest()
        source = 'A28' if r['arm'] == 'baseline' else 'B28'
        assert r['source_arm'] == source
        assert r['order'] == ORDERS[r['block']]
        assert r['revision'] == manifests[source]['revision']
        assert r['binary_sha256'] == manifests[source]['binary_sha256']
        assert r['signature'] == signatures[r['workload']][source]
        assert r['exit'] == 0 and r['affinity'] == [0, 1, 2, 3] and r['rayon_workers'] == 4
        for metric in ['cpu_s', 'user_s', 'system_s', 'wall_s', 'rss_kib']:
            assert type(r[metric]) in (int, float) and math.isfinite(r[metric]) and r[metric] >= 0
        assert r['cpu_s'] > 0 and r['wall_s'] > 0 and r['rss_kib'] > 0
        assert abs(r['cpu_s'] - r['user_s'] - r['system_s']) < 1e-12
    return rows
