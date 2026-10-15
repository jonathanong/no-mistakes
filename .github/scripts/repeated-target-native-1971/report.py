"""Curated completion ledger. Incomplete/noisy outcomes are retained explicitly."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

def completion_pair(checkout):
    from pair import current_pair
    from common import ExperimentFailure
    try:
        _, main = current_pair(checkout)
        return {'eligible': True, 'status': 'source pair still current', 'actual_main_ref': main}
    except Exception as error:
        # Preserve every sample and layout even when the source pair has drifted
        # or the final remote check is unavailable. Never publish raw errors.
        return {'eligible': False, 'status': 'historical or unverified; ineligible for merge gate',
                'error_code': error.code if isinstance(error, ExperimentFailure)
                else 'final_pair_check_failed'}


def summary(spec, assessments, counts, currentness=None):
    outcomes = {row.get('workload'): row for row in assessments}
    lines = ['# Frozen pair: actual retained native results', '',
        'A: `' + spec['baseline']['revision'] + '` (baseline frozen at preflight).',
        'B: `' + spec['candidate']['revision'] + '`; PR head `' + spec['diagnostic_parent'] + '`.', '',
        'Default-feature release opt3/full-LTO/codegen1. CodSpeed instrumented observations are separate.',
        'Direct prepared lookup timing is unmeasured. Actual incoming/counter proof comes from exact CI internal assertions.', '',
        '| Workload | Outcome | CPU | Wall | Peak RSS | Evidence phase / rows | Retained long rows |', '|---|---|---|---|---|---|---|']
    incomplete = currentness is not None and not currentness['eligible']
    if currentness is not None:
        lines[5:5] = ['Final source-pair check: ' + currentness['status'] + '.', '']
    for workload in spec['workloads']:
        row = outcomes.get(workload['name'], {'status':'incomplete'})
        count = counts.get(workload['name'], 0)
        status = row.get('status', 'incomplete')
        incomplete |= status == 'incomplete' or count != 32
        metrics = row.get('metrics', {})
        columns = [metrics.get(key, {}).get('status', 'unmeasured') for key in ('cpu_s', 'wall_s', 'rss_kib')]
        phase = row.get('phase', 'unmeasured')
        samples = row.get('samples', count)
        lines.append(f"| {workload['name']} | {status} | {' | '.join(columns)} | {phase} / {samples} | {count} |")
    lines += ['', 'All pilot/startup/partial/adverse numeric rows are retained. No outlier removal, clock subtraction, or favorable retry.',
        'CPU, wall and peak RSS have separate paired intervals and order diagnostics in assessments.json.',
        'Higher-cost observations require adjacent reversed confirmation before attribution. No automatic READY decision.',
        'Missing/noisy work remains incomplete/inconclusive. Flat CPU with higher RSS requires a design decision.']
    return '\n'.join(lines) + '\n', incomplete


def report(root):
    from pair import SPEC, HERE
    from common import write, require, digest
    public = root / 'public'
    public.mkdir(parents=True, exist_ok=True)
    assessments = json.loads((public / 'assessments.json').read_text()) if (public / 'assessments.json').exists() else []
    counts = {}
    for workload in SPEC['workloads']:
        rows = public / (workload['name'] + '-long-rows.json')
        counts[workload['name']] = len(json.loads(rows.read_text())) if rows.exists() else 0
    currentness = completion_pair(HERE.parents[2])
    write(public / 'final-pair.json', currentness)
    markdown, incomplete = summary(SPEC, assessments, counts, currentness)
    (public / 'summary.md').write_text(markdown)
    inventory = []
    for path in sorted(public.iterdir()):
        require(path.is_file() and not path.is_symlink() and path.suffix in ('.json', '.md'), 'unexpected_public_payload_type')
        require(path.stat().st_size <= 2 * 1024**2, 'public_payload_too_large')
        data = path.read_text()
        require(str(root) not in data and '/home/runner/' not in data, 'private_absolute_path_in_public_payload')
        inventory.append({'file':path.name, 'bytes':path.stat().st_size, 'sha256':digest(path)})
    write(public / 'artifact-manifest.json', {'status':'incomplete' if incomplete else 'retained results require review',
        'final_source_pair': currentness,
        'payload':inventory, 'excluded':'No executables/libraries/raw cargo/LCOV/source dumps/private request reports/profiles'})
    return int(incomplete)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    sys.exit(report(parser.parse_args().root.resolve()))
