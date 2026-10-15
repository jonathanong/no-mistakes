"""Synthetic accounting integrity only; never launches a compiler or workload."""
import copy
import unittest

from paired_summary import assess, describe_pilot, stable, validate
from report import summary


def rows(blocks=4):
    result = []
    for order in ('ABBA', 'BAAB'):
        for block in range(blocks):
            for position, letter in enumerate(order):
                result.append(dict(order=order, block=block, position=position,
                    arm='baseline' if letter == 'A' else 'candidate', count=100,
                    valid=True, rayon_workers=4, affinity=[0, 1, 2, 3],
                    user_s=4, system_s=1, cpu_s=5, wall_s=6, rss_kib=2048))
    return result


class BalancedIntegrity(unittest.TestCase):
    def test_one_workload_has_32_rows_and_flat_data_is_inconclusive(self):
        data = rows()
        self.assertEqual(len(data), 32)
        validate(data, 4)
        report = assess(data)
        self.assertTrue(all(m['status'] == 'inconclusive' for m in report['metrics'].values()))

    def test_partial_duplicate_reordered_and_unmatched_batches_are_rejected(self):
        original = rows()
        for mutate in [lambda r: r.pop(), lambda r: r.append(dict(r[-1])),
                       lambda r: r[0].update(position=1), lambda r: r[0].update(count=101),
                       lambda r: r[0].update(affinity=[1, 2, 3, 4]),
                       lambda r: r[0].update(cpu_s=0), lambda r: r[0].update(valid=False),
                       lambda r: r[0].update(cpu_s=6), lambda r: r[0].update(rayon_workers=8)]:
            data = copy.deepcopy(original)
            mutate(data)
            with self.assertRaises(AssertionError):
                validate(data, 4)

    def test_short_or_order_biased_pilots_are_inconclusive(self):
        data = rows(1)
        data[0].update(cpu_s=.5, user_s=.4, system_s=.1)
        self.assertFalse(stable(data))
        data = rows(1)
        for row in data:
            if row['arm'] == 'candidate' and row['order'] == 'ABBA':
                row.update(cpu_s=6, user_s=5)
        self.assertFalse(stable(data))

    def test_higher_rss_remains_visible_when_cpu_is_flat(self):
        data = rows()
        for row in data:
            if row['arm'] == 'candidate':
                row['rss_kib'] = 3072
        report = assess(data)
        self.assertEqual(report['metrics']['cpu_s']['status'], 'inconclusive')
        self.assertIn('higher cost observed', report['metrics']['rss_kib']['status'])
        self.assertEqual(report['metrics']['rss_kib']['paired_percent'], 50)

    def test_rendered_report_exposes_higher_rss_despite_noisy_cpu(self):
        data = rows()
        for row in data:
            row['cpu_s'] = 4 if row['position'] % 2 == 0 else 6
            row['user_s'] = row['cpu_s'] - row['system_s']
            if row['arm'] == 'candidate':
                row['rss_kib'] = 3072
        assessment = dict(workload='cold-control', **assess(data))
        self.assertEqual(assessment['metrics']['cpu_s']['status'], 'inconclusive')
        self.assertIn('higher cost observed', assessment['metrics']['rss_kib']['status'])
        self.assertIn('higher cost observed', assessment['status'])
        spec = {'baseline': {'revision':'A'}, 'candidate': {'revision':'B'},
                'diagnostic_parent':'HEAD', 'workloads':[{'name':'cold-control'}]}
        markdown, incomplete = summary(spec, [assessment], {'cold-control':32})
        self.assertFalse(incomplete)
        self.assertIn('| Peak RSS |', markdown)
        line = next(line for line in markdown.splitlines() if line.startswith('| cold-control |'))
        self.assertIn('| inconclusive |', line)
        self.assertGreaterEqual(line.count('higher cost observed'), 2)

    def test_unstable_pilot_retains_descriptive_rss_direction_without_long_claim(self):
        data = rows(1)
        for row in data:
            row['cpu_s'] = 4 if row['position'] % 2 == 0 else 6
            row['user_s'] = row['cpu_s'] - row['system_s']
            if row['arm'] == 'candidate':
                row['rss_kib'] = 3072
        self.assertFalse(stable(data))
        assessment = dict(workload='cold-control', status='inconclusive; long withheld', **describe_pilot(data))
        spec = {'baseline':{'revision':'A'}, 'candidate':{'revision':'B'},
                'diagnostic_parent':'HEAD', 'workloads':[{'name':'cold-control'}]}
        markdown, incomplete = summary(spec, [assessment], {})
        self.assertTrue(incomplete)
        line = next(line for line in markdown.splitlines() if line.startswith('| cold-control |'))
        self.assertIn('pilot higher (+50.00%); long withheld', line)
        self.assertIn('| pilot / 8 | 0 |', line)
        self.assertNotIn('unmeasured', line)


if __name__ == '__main__':
    unittest.main()
