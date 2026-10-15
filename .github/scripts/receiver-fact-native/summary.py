"""Integrity and uncertainty checks, independent of measurement execution."""
import math
import random
import statistics


def validate(rows, blocks):
    expected = 2 * 2 * blocks * 4
    if len(rows) != expected:
        raise ValueError('incomplete balanced sample set')
    counts = set()
    for demand in ('values', 'imports'):
        for order in ('ABBA', 'BAAB'):
            for block in range(blocks):
                group = [r for r in rows if (r['demand'], r['order'], r['block']) == (demand, order, block)]
                if [r['position'] for r in group] != list(range(4)):
                    raise ValueError('invalid sample positions')
                if [r['arm'] for r in group] != ['baseline' if c == 'A' else 'candidate' for c in order]:
                    raise ValueError('missing, duplicated or reordered balanced block')
                for row in group:
                    if row['exit'] != 0 or row['stderr'] or row['signature'] != ('128' if demand == 'values' else '0'):
                        raise ValueError('failed child or fact signature mismatch')
                    if any(not math.isfinite(row[k]) or row[k] <= 0 for k in ('cpu_s', 'wall_s', 'rss_kib')):
                        raise ValueError('invalid resources')
                    if abs(row['cpu_s'] - row['user_s'] - row['system_s']) > 1e-8:
                        raise ValueError('CPU accounting mismatch')
                    if row['rayon_workers'] != 4 or len(row['child_affinity']) != 4:
                        raise ValueError('unmatched worker or affinity control')
                    counts.add(row['collections'])
    if len({tuple(r['child_affinity']) for r in rows}) != 1:
        raise ValueError('affinity differs between samples')
    if len(counts) != 1 or next(iter(counts)) <= 0:
        raise ValueError('unmatched batch sizes')


def stable(rows, minimum_cpu=1, demands=('values', 'imports')):
    if min(r['cpu_s'] for r in rows) < minimum_cpu:
        return False
    for demand in demands:
        changes = []
        for arm in ('baseline', 'candidate'):
            values = [r['cpu_s'] for r in rows if r['demand'] == demand and r['arm'] == arm]
            if max(values) / min(values) > 1.10:
                return False
        for order in ('ABBA', 'BAAB'):
            group = [r for r in rows if r['demand'] == demand and r['order'] == order]
            a = statistics.mean(r['cpu_s'] for r in group if r['arm'] == 'baseline')
            b = statistics.mean(r['cpu_s'] for r in group if r['arm'] == 'candidate')
            changes.append(100 * (b / a - 1))
        if abs(changes[0] - changes[1]) > 5:
            return False
    return True


def assess(rows, blocks=4):
    try:
        validate(rows, blocks)
    except (ValueError, KeyError, TypeError) as error:
        return {'status': 'incomplete', 'reason': str(error)}
    result = {'status': 'inconclusive', 'demands': {}}
    for demand in ('values', 'imports'):
        group = [r for r in rows if r['demand'] == demand]
        paired = []
        orders = {}
        for order in ('ABBA', 'BAAB'):
            selected = [r for r in group if r['order'] == order]
            means = {arm: statistics.mean(r['cpu_s'] for r in selected if r['arm'] == arm)
                     for arm in ('baseline', 'candidate')}
            orders[order] = 100 * (means['candidate'] / means['baseline'] - 1)
            for block in range(blocks):
                selected_block = [r for r in selected if r['block'] == block]
                means = {arm: statistics.mean(r['cpu_s'] for r in selected_block if r['arm'] == arm)
                         for arm in ('baseline', 'candidate')}
                paired.append(math.log(means['candidate'] / means['baseline']))
        rng = random.Random(2012)
        boot = sorted(100 * math.expm1(statistics.mean(rng.choices(paired, k=len(paired)))) for _ in range(10000))
        ci = [boot[249], boot[9749]]
        cvs = {arm: statistics.stdev([r['cpu_s'] for r in group if r['arm'] == arm]) /
               statistics.mean(r['cpu_s'] for r in group if r['arm'] == arm) for arm in ('baseline', 'candidate')}
        consistent = stable(group, demands=(demand,)) and max(cvs.values()) <= .03
        status = 'inconclusive'
        if consistent and min(orders.values()) > 0 and ci[0] > 0:
            status = 'material regression observed' if ci[0] >= 3 else 'positive cost observed'
        elif consistent and max(orders.values()) < 0 and ci[1] < 0:
            status = 'lower cost observed'
        result['demands'][demand] = {'status': status, 'orders_percent': orders,
            'paired_cpu_percent': 100 * math.expm1(statistics.mean(paired)),
            'descriptive_bootstrap_95_percent': ci, 'arm_cpu_cv': cvs,
            'cpu_range_s': [min(r['cpu_s'] for r in group), max(r['cpu_s'] for r in group)],
            'mean_cpu_ms_per_collection': {arm: statistics.mean(r['cpu_s'] * 1000 / r['collections']
                for r in group if r['arm'] == arm) for arm in ('baseline', 'candidate')}}
    statuses = [d['status'] for d in result['demands'].values()]
    for status in ('material regression observed', 'positive cost observed', 'lower cost observed'):
        if status in statuses:
            result['status'] = status
            break
    return result
