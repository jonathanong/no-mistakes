#!/usr/bin/env python3
"""Summarize complete paired evidence; never turn a timeout or mismatched report into a win."""
from collections import defaultdict
import json
from pathlib import Path
import statistics
import sys

from measure import ORACLE


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_sample(sample, key, pair):
    require(sample['complete'] and sample['parity'] and sample['report_sha256'] == ORACLE,
            'Incomplete sample or full-JSON oracle mismatch')
    require((sample['jobs'], sample['diagnostics'], sample['pair']) == (*key, pair),
            'Sample metadata differs from its matched pair')
    require(sample['resources']['exit'] == sample['wrapper_exit'] == 1,
            'Sample did not complete with the expected consumer finding status')


def nearest_rank(values, percentile):
    ordered = sorted(values)
    rank = (len(ordered) * percentile + 99) // 100
    return ordered[rank - 1]


def validate_cells(groups):
    required = {(jobs, diagnostics) for jobs in ('1', '4', 'default') for diagnostics in ('off', 'on')}
    require(set(groups) == required, 'Missing a jobs/diagnostics measurement cell')


def validate_pairs(blocks):
    require(sorted(block['pair'] for block in blocks) == [1, 2, 3, 4, 5, 6], 'Missing or duplicate pair')


def summarize(root):
    groups = defaultdict(list)
    baseline_sources, candidate_sources = set(), set()
    for metadata_file in sorted(root.glob('*/environment.json')):
        metadata = json.loads(metadata_file.read_text())
        key = (metadata['jobs'], metadata['diagnostics'])
        pair = metadata['pair']
        require(key[0] in ('1', '4', 'default') and key[1] in ('off', 'on'), 'Invalid matrix cell')
        variants = metadata['binaries']
        baseline_sources.add(variants['baseline']['source'])
        candidate_sources.add(variants['candidate']['source'])
        require(variants['baseline']['binary_sha256'] != variants['candidate']['binary_sha256'],
                'Baseline and candidate are the same binary')
        for field in ('rustc', 'features', 'lto', 'codegen_units', 'build_jobs', 'cargo_lock_sha256'):
            require(variants['baseline'][field] == variants['candidate'][field],
                    f'Baseline and candidate differ in {field}')
        samples = []
        for path in metadata_file.parent.glob('[0-3]-*.json'):
            sample = json.loads(path.read_text())
            validate_sample(sample, key, pair)
            samples.append(sample)
        expected = ('baseline', 'candidate', 'candidate', 'baseline') if pair % 2 else (
            'candidate', 'baseline', 'baseline', 'candidate')
        require(sorted((s['position'], s['variant']) for s in samples) == list(enumerate(expected)),
                'Missing or incorrectly ordered source sample')
        metrics = {}
        for metric in ('elapsed', 'cpu', 'rss_kib'):
            values = {}
            for variant in ('baseline', 'candidate'):
                values[variant] = [s['resources']['user']+s['resources']['system'] if metric == 'cpu'
                                   else s['resources'][metric] for s in samples if s['variant'] == variant]
                require(all(value > 0 for value in values[variant]), f'Invalid {metric} sample')
            metrics[metric] = dict(baseline=values['baseline'], candidate=values['candidate'],
                                   ratio=statistics.mean(values['candidate'])/statistics.mean(values['baseline']))
        groups[key].append(dict(pair=pair, order='ABBA' if pair % 2 else 'BAAB', metrics=metrics))
    require(len(baseline_sources) == len(candidate_sources) == 1, 'Source revisions vary across samples')
    require(baseline_sources != candidate_sources, 'Baseline and candidate source revisions match')
    validate_cells(groups)
    summary = dict(
        baseline=next(iter(baseline_sources)), candidate=next(iter(candidate_sources)),
        percentile_method='nearest rank; p95 is the maximum with 12 variant samples and 6 paired ratios',
        groups=[])
    for (jobs, diagnostics), blocks in sorted(groups.items()):
        validate_pairs(blocks)
        metrics = {}
        for metric in ('elapsed', 'cpu', 'rss_kib'):
            ratios = [b['metrics'][metric]['ratio'] for b in blocks]
            values = {v: [x for b in blocks for x in b['metrics'][metric][v]]
                      for v in ('baseline', 'candidate')}
            metrics[metric] = dict(paired_ratios=ratios, median_ratio=statistics.median(ratios),
                                   improved_pairs=sum(r < 1 for r in ratios),
                                   by_order={o: [b['metrics'][metric]['ratio'] for b in blocks if b['order'] == o]
                                             for o in ('ABBA', 'BAAB')},
                                   maximum={v: max(xs) for v, xs in values.items()},
                                   median={v: statistics.median(xs) for v, xs in values.items()},
                                   p95={v: nearest_rank(xs, 95) for v, xs in values.items()},
                                   paired_ratio_p95=nearest_rank(ratios, 95))
        summary['groups'].append(dict(jobs=jobs, diagnostics=diagnostics, pairs=6,
                                     runs_per_variant=12, metrics=metrics))
    return summary


if __name__ == '__main__':
    directory = Path(sys.argv[1])
    result = summarize(directory)
    (directory / 'summary.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
