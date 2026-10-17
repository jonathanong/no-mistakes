#!/usr/bin/env python3
"""Offline only. Never accepts incomplete rows or transforms diagnostic outputs."""
import argparse
import hashlib
import json
import random
import statistics
from pathlib import Path


def load(path):
    return json.loads(Path(path).read_text())


def summary(blocks):
    ratios = [statistics.mean(b) / statistics.mean(a) - 1 for _, a, b in blocks]
    generator = random.Random(1971)
    samples = sorted(statistics.mean(generator.choices(ratios, k=len(ratios))) for _ in range(10000))
    lo, hi = samples[249], samples[9749]
    orders = {order: statistics.mean(r for r, (o, _, _) in zip(ratios, blocks) if o == order)
              for order in ('ABBA', 'BAAB')}
    return {'mean_block_relative_change': statistics.mean(ratios), 'ci95': [lo, hi],
            'order_relative_changes': orders, 'block_relative_changes': ratios,
            'status': 'FLAT' if all(x == 0 for x in ratios) else 'REGRESSION' if lo > 0 and min(orders.values()) > 0 else
                      'IMPROVEMENT' if hi < 0 and max(orders.values()) < 0 else 'INCONCLUSIVE'}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--request', required=True)
    parser.add_argument('--round', action='append', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    request = load(args.request)
    assert request['status'] == 'REVIEWED consumer protocol; untimed preflight first'
    output = Path(args.output)
    assert not output.exists()
    rows, pins = [], []
    for name in args.round:
        receipt = load(name)
        assert receipt['status'] == 'PASS' and receipt['phase'] == 'round'
        assert receipt['request_sha256'] == hashlib.sha256(Path(args.request).read_bytes()).hexdigest()
        assert receipt['pins_after_exact'] and receipt['elapsed_seconds'] <= 600
        rows += receipt['rows']
        pins.append({'path': name, 'sha256': hashlib.sha256(Path(name).read_bytes()).hexdigest()})
    assert len(rows) == request['rows_total'] == 64
    identities = [(x['case'], x['block'], x['position']) for x in rows]
    assert len(set(identities)) == len(identities)
    metrics = {}
    for case in request['cases']:
        selected = [x for x in rows if x['case'] == case['id']]
        assert len(selected) == 16
        fields = ['wall_seconds', 'cpu_seconds', 'user_seconds', 'sys_seconds', 'peak_rss_kib']
        if case['diagnostics'] and case['annotation']:
            fields += ['projection_wall_ms']
        for field in fields:
            blocks = []
            for block in range(4):
                values = sorted((x for x in selected if x['block'] == block), key=lambda x: x['position'])
                assert len(values) == 4 and all(x['status'] == 'COMPLETE' for x in values)
                assert [x['position'] for x in values] == list(range(4))
                assert all(x['order'] == values[0]['order'] for x in values)
                assert ''.join('A' if x['arm'] == 'baseline' else 'B' for x in values) == values[0]['order']
                def metric(x):
                    return x['parsed'][field] if field == 'projection_wall_ms' else x[field]
                a = [metric(x) for x in values if x['arm'] == 'baseline']
                b = [metric(x) for x in values if x['arm'] == 'candidate']
                assert min(a) > 0 and min(b) > 0
                blocks.append((values[0]['order'], a, b))
            metrics[case['id'] + '/' + field] = summary(blocks)
    # Descriptive source proposal; final acceptance remains an independent root review.
    gate_metrics = {k: v for k, v in metrics.items() if not k.endswith(('/user_seconds', '/sys_seconds'))}
    key = 'consumer-on/projection_wall_ms'
    consuming = [x for x in rows if x['case'] == 'consumer-on']
    scope_equal_positive = all(
        len({x['parsed']['work'][counter] for x in consuming}) == 1 and
        min(x['parsed']['work'][counter] for x in consuming) > 0
        for counter in ('postgres.annotation.modules', 'postgres.annotation.target_resolutions'))
    counts_decline = all(
        max(x['parsed']['work'].get(counter, 0) for x in consuming if x['arm'] == 'candidate') <
        min(x['parsed']['work'][counter] for x in consuming if x['arm'] == 'baseline')
        for counter in ('postgres.annotation.reachability_walks', 'postgres.annotation.captured_reader_rebuilds'))
    verdict = ('FAIL' if not scope_equal_positive or not counts_decline or any(x['status'] == 'REGRESSION' for x in gate_metrics.values()) else
               'PASS_PENDING_INDEPENDENT_REVIEW' if metrics[key]['status'] == 'IMPROVEMENT' else 'INCONCLUSIVE')
    uncertainty = [k for k, v in gate_metrics.items() if v['status'] == 'INCONCLUSIVE']
    output.write_text(json.dumps({'status': verdict, 'rows': len(rows), 'metrics': metrics,
                                 'same_positive_module_target_demand': scope_equal_positive,
                                 'both_actual_counts_decline': counts_decline, 'uncertain_metrics': uncertainty,
                                 'round_pins': pins, 'policy': 'No favorable pooling, omitted rows, or changed thresholds'}, indent=2) + '\n')


if __name__ == '__main__':
    main()
