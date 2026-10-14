"""Balanced fresh-process fact controls; run only in a coordinated idle window."""
import hashlib
import argparse
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

out = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--comparison', choices=['historical', 'current'], default='historical')
parser.add_argument('--scale', type=int, default=1, help='Separately labeled multiplier for every batch size')
parser.add_argument('--tag', default='', help='Distinct filename tag for an additional preserved run')
parser.add_argument('--case', choices=['no_writes','writes','shared_histories','jsx'])
parser.add_argument('--iterations', type=int, help='Override one selected case; requires --case and --tag')
parser.add_argument('--threads', type=int, default=4, choices=[1,4])
parser.add_argument('--cpu', type=int, help='Set child-only affinity to one allowed CPU')
args = parser.parse_args()
assert args.scale >= 1
assert all(c.isalnum() or c in '-_' for c in args.tag)
parent_affinity = sorted(os.sched_getaffinity(0))
if args.threads != 4 or args.cpu is not None:
    assert args.tag, 'Separately label altered worker/affinity diagnostics'
if args.cpu is not None:
    assert args.cpu in parent_affinity
def set_child_affinity():
    os.sched_setaffinity(0, {args.cpu})
root = Path('<WORKSPACE>/no-mistakes-1972')
cases = {
    'no_writes': ('test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts', 100),
    'writes': ('test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts', 100),
    'shared_histories': ('test-cases/codebase-analysis/import-facts/fixture/many-late-defaults.mts', 1),
    'jsx': ('test-cases/ast-snippets/selectors/fixture/app-text-targets.tsx', 100),
}
cases = {name:(path, count*args.scale) for name,(path,count) in cases.items()}
if args.case:
    assert args.tag, 'Label focused diagnostics with --tag'
    cases = {args.case:cases[args.case]}
if args.iterations is not None:
    assert args.case and args.tag and args.iterations > 0 and args.scale == 1
    cases = {args.case:(cases[args.case][0],args.iterations)}
def digest(path):
    return hashlib.file_digest(path.open('rb'), 'sha256').hexdigest()
labels = ['baseline', 'candidate']
prefix = '' if args.comparison == 'historical' else 'current-'
result_prefix = prefix + (f'scale{args.scale}-' if args.scale != 1 else '') + (args.tag+'-' if args.tag else '')
assert not (out/f'{result_prefix}optimized-raw.json').exists(), 'Preserve prior raw samples; choose a new --tag'
assert not (out/f'{result_prefix}optimized-summary.json').exists()
manifests = {label: json.loads((out/f'{prefix}{label}-manifest.json').read_text()) for label in labels}
assert manifests['baseline']['lock_sha256'] == manifests['candidate']['lock_sha256']
for key in ['rustc', 'release_profile']:
    assert manifests['baseline'][key] == manifests['candidate'][key]
assert manifests['baseline']['artifact']['features'] == manifests['candidate']['artifact']['features']
assert len({digest(out/f'{prefix}{label}-facts') for label in labels}) == 2
rows = []
for order in ['ABBA', 'BAAB']:
    for case, (relative, count) in cases.items():
        for demand in ['values', 'imports']:
            for round_number in range(4):
                for name in order:
                    label = labels[name == 'B']
                    start = time.perf_counter()
                    process = subprocess.Popen([str(out/f'{prefix}{label}-facts'), str(root/relative), demand, str(count)],
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                        env=dict(os.environ, RAYON_NUM_THREADS=str(args.threads)),
                        preexec_fn=set_child_affinity if args.cpu is not None else None)
                    _, status, usage = os.wait4(process.pid, 0)
                    process.returncode = os.waitstatus_to_exitcode(status)
                    row = dict(order=order, case=case, demand=demand, binary=label, round=round_number, iterations=count,
                        rayon_workers=args.threads,child_cpu_affinity=[args.cpu] if args.cpu is not None else parent_affinity,
                        wall_s=time.perf_counter()-start, cpu_s=usage.ru_utime+usage.ru_stime,
                        rss_kib=usage.ru_maxrss, report=process.stdout.read().decode().strip(),
                        stderr=process.stderr.read().decode(), exit=process.returncode)
                    assert row['exit'] == 0 and not row['stderr'], row
                    rows.append(row)
                    (out/f'{result_prefix}optimized-raw.json').write_text(json.dumps(rows, indent=2)+'\n')
results = []
for order in ['ABBA', 'BAAB']:
    for case, (relative, count) in cases.items():
        source = root/relative
        for demand in ['values', 'imports']:
            selected = [r for r in rows if (r['order'],r['case'],r['demand']) == (order,case,demand)]
            assert len({r['report'] for r in selected}) == 1
            result = dict(order=order,case=case,demand=demand,iterations=count,samples_per_binary=8,
                          fixture=dict(path=relative,sha256=digest(source),bytes=source.stat().st_size,lines=len(source.read_text().splitlines())))
            for label in labels:
                group = [r for r in selected if r['binary'] == label]
                result[label] = {k:statistics.median(r[k] for r in group) for k in ['wall_s','cpu_s','rss_kib']}
            results.append(result)
summary = dict(scope='Historical optimized 57c927df to 8d81aa1; not current-main measurements' if not prefix else 'Merged #1961 integration optimized b97b2cb to 917c9ea1; no later follow-up implementations',
               manifests=manifests,rayon_workers=args.threads,parent_cpu_affinity=parent_affinity,
               child_cpu_affinity=[args.cpu] if args.cpu is not None else parent_affinity,
               batch_multiplier=args.scale,run_tag=args.tag,
               binary_hashes={label:digest(out/f'{prefix}{label}-facts') for label in labels},cases=results)
(out/f'{result_prefix}optimized-summary.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(results, indent=2))
