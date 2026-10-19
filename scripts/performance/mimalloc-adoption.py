#!/usr/bin/env python3
"""Require complete replicated evidence before recommending maintainer review."""
import argparse
from collections import defaultdict
import json
import math
from pathlib import Path
import runpy

HERE = Path(__file__).parent
RULE = runpy.run_path(str(HERE / 'mimalloc-adoption-rule.py'))
MATRIX = runpy.run_path(str(HERE / 'mimalloc-report-matrix.py'))
SIZE_TARGETS = {'aarch64-apple-darwin', 'aarch64-unknown-linux-gnu',
                'x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc'}
WORKLOAD_PINS = {'consumer': 'f38a048f5eeff0438fc59a15e9eb742def575a36',
                 'hosted-acceptance': '22edf4bfe0a782ac0d030c916dedfd6f8acb499c',
                 'hosted-passing-failing': '92d0a2061bde8d260aa0b86a2d71110ece1a43ca'}
SCOPES = ('hosted', 'napi', 'many-core-linux', 'many-core-macos')


def validate_summary(groups, scope):
    if not groups:
        raise ValueError('empty supplied summary')
    kind = 'many-core' if scope.startswith('many-core') else scope
    platforms = {g['platform'] for g in groups}
    platform = None
    if kind == 'many-core':
        allowed = {'linux-arm64', 'linux-x64'} if scope.endswith('linux') else {'macos-arm64'}
        if len(platforms) != 1 or not platforms <= allowed:
            raise ValueError('wrong many-core platform scope')
        platform = next(iter(platforms))
    if kind == 'many-core':
        for group in groups:
            for pair in group['pairs']:
                p = pair['provenance']
                if (pair.get('hostFacts', {}).get('available_cpus', 0) < 16
                        or p.get('num_cpus_default', 0) < 16 or p.get('automatic_rayon_jobs', 0) < 16
                        or p.get('rayon_environment')):
                    raise ValueError('many-core summary lacks physical capacity evidence')
    expected, _ = MATRIX['matrix_cells'](kind, platform)
    keys = [tuple(g[k] for k in MATRIX['CELL_FIELDS']) for g in groups]
    if len(keys) != len(set(keys)) or set(keys) != expected:
        raise ValueError('incomplete or duplicated supplied matrix')


def validate_sizes(sizes):
    if sizes is None:
        return False
    if len(sizes) != 4 or {s['target'] for s in sizes} != SIZE_TARGETS:
        raise ValueError('incomplete four-target size evidence')
    for row in sizes:
        if row.get('valid') is not True or not row.get('source'):
            raise ValueError('invalid release size evidence')
        for label in ('cli', 'addon'):
            values = row[label]
            b, c = values['baselineBytes'], values['candidateBytes']
            if type(b) is not int or type(c) is not int or min(b, c) <= 0:
                raise ValueError('invalid artifact bytes')
            if values['deltaBytes'] != c - b or not math.isclose(values['percentDelta'], 100 * (c - b) / b):
                raise ValueError('inconsistent size delta')
    return True


def evaluate(scopes, sizes=None):
    missing, cells, sources, locks, trees = [], [], set(), set(), set()
    resource_builds, workloads = defaultdict(set), defaultdict(set)
    contracts = defaultdict(set)
    for scope in SCOPES:
        supplied = scopes.get(scope, [])
        if not supplied:
            missing.append(scope)
            continue
        grouped = defaultdict(list)
        for summary in supplied:
            validate_summary(summary, scope)
            for group in summary:
                key = tuple(group[k] for k in MATRIX['CELL_FIELDS'])
                grouped[key].append(group)
                for pair in group['pairs']:
                    p = pair['provenance']
                    workload = 'consumer' if group['workload'] == 'consumer-napi' else group['workload']
                    if p.get('workload_commit') != WORKLOAD_PINS[workload] or not p.get('workload_tree'):
                        raise ValueError('wrong or missing pinned workload identity')
                    workloads[workload].add((p['workload_commit'], p['workload_tree']))
                    sources.add(p['source'])
                    trees.add(p['tree'])
                    locks.add(p['lock']['sha256'])
                    build = {k: p[k] for k in ('rustc', 'cargo', 'lto', 'codegenUnits', 'build_flags')}
                    resource_builds[p['target']].add(json.dumps(build, sort_keys=True))
                    contract = {k: p[k] for k in ('tree', 'rustc', 'cargo', 'target', 'lto', 'codegenUnits', 'build_flags')}
                    contract['artifacts'] = group['artifact_sha256']
                    if scope == 'napi':
                        contract['node'] = pair['node']
                    contracts[(group['platform'], scope == 'napi')].add(json.dumps(contract, sort_keys=True))
        for key, groups in sorted(grouped.items()):
            cell = dict(zip(MATRIX['CELL_FIELDS'], key))
            cell.update(RULE['evaluate_cell'](groups, scope.startswith('many-core')))
            cells.append({'scope': scope, **cell})
    if any(len(pins) > 1 for pins in workloads.values()):
        raise ValueError('workload identity changed across evidence')
    size_complete = validate_sizes(sizes)
    if not size_complete:
        missing.append('release-sizes')
    else:
        for row in sizes:
            sources.add(row['source'])
            trees.add(row['tree'])
            locks.add(row['lockSha256'])
            profile = row['profile']
            build = {'rustc': row['rustc'], 'cargo': row['cargo'],
                     'lto': profile['CARGO_PROFILE_RELEASE_LTO'],
                     'codegenUnits': int(profile['CARGO_PROFILE_RELEASE_CODEGEN_UNITS']),
                     'build_flags': {k: profile[k] for k in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS',
                                     'RUSTC_WRAPPER', 'CARGO_PROFILE_RELEASE_DEBUG',
                                     'CARGO_INCREMENTAL', 'NO_MISTAKES_BUILD_NAPI')}}
            if resource_builds[row['target']] and resource_builds[row['target']] != {json.dumps(build, sort_keys=True)}:
                raise ValueError('resource and size evidence use different compiler or flags')
    if (len(sources) > 1 or len(locks) > 1 or len(trees) > 1 or any(len(values) > 1 for values in contracts.values())):
        raise ValueError('source, lock, compiler, flags, or artifact drift across evidence')
    complete = not missing
    eligible = complete and bool(cells) and all(c['eligible'] for c in cells)
    failures = any(c['observedThresholdFailures'] for c in cells)
    status = ('eligible-for-maintainer-review' if eligible else 'observed-threshold-failure'
              if failures else 'insufficient-evidence')
    return {'status': status, 'adopt': False, 'eligibleForMaintainerReview': eligible,
            'maintainerReviewRequired': True, 'completeRequiredScopes': complete,
            'missingScopes': missing, 'rule': RULE['ADOPTION_RULE'], 'cells': cells,
            'releaseSizes': sizes,
            'limits': ['Four-pair dispatches are exploratory; repeat independent collections to reach 12 distinct pairs per cell.',
                       'The binomial median bound assumes independent paired observations from a stable sampling process.',
                       'All resource ratios compare variants within each physical pair. Size deltas need maintainer review.']}


def main():
    parser = argparse.ArgumentParser()
    for scope in SCOPES:
        parser.add_argument('--' + scope, action='append', default=[])
    parser.add_argument('--sizes')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    scopes = {s: [json.loads(Path(p).read_text()) for p in getattr(args, s.replace('-', '_'))] for s in SCOPES}
    sizes = json.loads(Path(args.sizes).read_text()) if args.sizes else None
    result = evaluate(scopes, sizes)
    Path(args.output).write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: result[k] for k in ('status', 'adopt', 'missingScopes', 'eligibleForMaintainerReview')}))


if __name__ == '__main__':
    main()
