"""Pure saved-protocol controls; no native execution or invented timing claims."""
import copy
import hashlib
import json
import unittest
from pathlib import Path
from schedule import cohorts, expected_keys, verify_rows, ORDERS

D = Path(__file__).resolve().parent


class ScheduleTests(unittest.TestCase):
    def test_complete_partition(self):
        plan, workloads = cohorts()
        self.assertEqual(sum(c['rows'] for c in plan.values()), 704)
        self.assertEqual(set(plan['A']['workloads']) & set(plan['B']['workloads']), set())
        self.assertEqual(len(workloads), 22)

    def rows(self, cohort):
        plan, workloads = cohorts()
        lookup = {w['id']: w for w in workloads}
        signatures = json.loads((D / 'timing-signatures.json').read_text())
        manifests = json.loads((D / 'producer-manifests.json').read_text())
        calibration = json.loads((D / 'calibration-batches.json').read_text())
        rows = []
        for block, workload, position, arm in expected_keys(cohort):
            source = 'A28' if arm == 'baseline' else 'B28'
            signature = signatures[workload][source]
            rows.append(dict(pair='A28_B28',phase='long',group='all',block=block,workload=workload,
                position=position,arm=arm,source_arm=source,order=ORDERS[block],
                revision=manifests[source]['revision'],binary_sha256=manifests[source]['binary_sha256'],
                signature=signature,exit=0,affinity=[0,1,2,3],rayon_workers=4,
                collections=lookup[workload].get('fixed_batch',calibration.get(workload)),
                cpu_s=1.,user_s=.75,system_s=.25,wall_s=2.,rss_kib=100,
                stdout_sha256=hashlib.sha256((str(signature)+'\n').encode()).hexdigest(),
                stderr_sha256=hashlib.sha256(b'').hexdigest()))
        return rows, manifests, signatures

    def test_full_and_partial_schedules(self):
        for cohort in ['A','B']:
            rows, manifests, signatures = self.rows(cohort)
            verify_rows(rows,cohort,manifests,signatures)
            for mutate in [lambda r:r.pop(),lambda r:r.reverse(),lambda r:r.append(r[0]),
                           lambda r:r[0].update(workload='other-cohort'),lambda r:r[0].update(signature='wrong'),
                           lambda r:r[0].update(exit=1),lambda r:r[0].update(cpu_s=float('nan')),
                           lambda r:r[0].update(collections=0),lambda r:r[0].update(binary_sha256='other')]:
                altered=copy.deepcopy(rows);mutate(altered)
                with self.assertRaises(AssertionError):verify_rows(altered,cohort,manifests,signatures)
