"""Workload-local balanced integrity and CPU/wall/RSS uncertainty summaries."""
import math
import random
import statistics


def validate(rows, blocks):
    assert len(rows) == 2 * blocks * 4
    assert len({row['count'] for row in rows}) == 1
    assert len({tuple(row['affinity']) for row in rows}) == 1
    assert len(rows[0]['affinity']) == 4
    for order in ('ABBA', 'BAAB'):
        for block in range(blocks):
            group = [r for r in rows if (r['order'], r['block']) == (order, block)]
            assert [r['position'] for r in group] == list(range(4))
            assert [r['arm'] for r in group] == ['baseline' if c == 'A' else 'candidate' for c in order]
    for row in rows:
        assert row['valid'] and row['count'] > 0 and row['rayon_workers'] == 4
        assert abs(row['cpu_s'] - row['user_s'] - row['system_s']) < 1e-8
        assert all(math.isfinite(row[key]) and row[key] > 0 for key in ('cpu_s', 'wall_s', 'rss_kib'))


def metric_stable(rows, metric, minimum_cpu=1):
    if metric == 'cpu_s' and min(row['cpu_s'] for row in rows) < minimum_cpu:
        return False
    changes = []
    for arm in ('baseline', 'candidate'):
        values = [r[metric] for r in rows if r['arm'] == arm]
        if max(values) / min(values) > 1.10:
            return False
    for order in ('ABBA', 'BAAB'):
        group = [r for r in rows if r['order'] == order]
        a, b = [statistics.mean(r[metric] for r in group if r['arm'] == arm) for arm in ('baseline', 'candidate')]
        changes.append(100 * (b / a - 1))
    return abs(changes[0] - changes[1]) <= 5


def stable(rows, minimum_cpu=1):
    return metric_stable(rows, 'cpu_s', minimum_cpu)


def describe_pilot(rows):
    validate(rows, 1)
    metrics = {}
    for metric in ('cpu_s', 'wall_s', 'rss_kib'):
        means = {arm: statistics.mean(r[metric] for r in rows if r['arm'] == arm) for arm in ('baseline', 'candidate')}
        percent = 100 * (means['candidate'] / means['baseline'] - 1)
        direction = 'higher' if percent > 0 else 'lower' if percent < 0 else 'flat'
        metrics[metric] = {'status': f'pilot {direction} ({percent:+.2f}%); long withheld',
            'descriptive_mean_percent':percent,
            'range':{arm:[min(r[metric] for r in rows if r['arm'] == arm), max(r[metric] for r in rows if r['arm'] == arm)] for arm in ('baseline', 'candidate')}}
    return {'phase':'pilot', 'samples':len(rows), 'metrics':metrics,
            'inference':'Descriptive8rowpilot only; no long-run confirmation or bootstrap claim'}


def assess(rows, blocks=4):
    validate(rows, blocks)
    result = {'status': 'inconclusive', 'metrics': {}}
    for metric in ('cpu_s', 'wall_s', 'rss_kib'):
        paired, orders = [], {}
        for order in ('ABBA', 'BAAB'):
            group = [r for r in rows if r['order'] == order]
            means = {arm: statistics.mean(r[metric] for r in group if r['arm'] == arm) for arm in ('baseline', 'candidate')}
            orders[order] = 100 * (means['candidate'] / means['baseline'] - 1)
            for block in range(blocks):
                group = [r for r in rows if (r['order'], r['block']) == (order, block)]
                a, b = [statistics.mean(r[metric] for r in group if r['arm'] == arm) for arm in ('baseline', 'candidate')]
                paired.append(math.log(b / a))
        rng = random.Random(1971)
        bootstrap = sorted(100 * math.expm1(statistics.mean(rng.choices(paired, k=len(paired)))) for _ in range(10000))
        cv = {arm: statistics.stdev([r[metric] for r in rows if r['arm'] == arm]) /
              statistics.mean(r[metric] for r in rows if r['arm'] == arm) for arm in ('baseline', 'candidate')}
        interval = [bootstrap[249], bootstrap[9749]]
        status = 'inconclusive'
        if metric_stable(rows, metric, minimum_cpu=5) and max(cv.values()) <= .03:
            if min(orders.values()) > 0 and interval[0] > 0:
                status = 'higher cost observed; adjacent reverse confirmation required'
            elif max(orders.values()) < 0 and interval[1] < 0:
                status = 'lower cost observed'
        result['metrics'][metric] = {'status': status, 'paired_percent': 100 * math.expm1(statistics.mean(paired)),
            'orders_percent': orders, 'descriptive_bootstrap_95_percent': interval, 'arm_cv': cv,
            'median_per_request_ms' if metric != 'rss_kib' else 'median_peak_rss_kib': {
                arm: statistics.median(r[metric] * 1000 / r['count'] if metric != 'rss_kib' else r[metric]
                    for r in rows if r['arm'] == arm) for arm in ('baseline', 'candidate')},
            'range': {arm: [min(r[metric] for r in rows if r['arm'] == arm), max(r[metric] for r in rows if r['arm'] == arm)] for arm in ('baseline', 'candidate')}}
    statuses = [metric['status'] for metric in result['metrics'].values()]
    if any(status.startswith('higher cost observed') for status in statuses):
        result['status'] = 'higher cost observed; adjacent reverse confirmation required'
    elif any(status == 'lower cost observed' for status in statuses):
        result['status'] = 'lower cost observed; other metrics require independent review'
    return result
