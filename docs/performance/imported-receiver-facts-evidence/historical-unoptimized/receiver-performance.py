import hashlib
import json
import os
import re
import statistics
import subprocess
import time
import tomllib
from pathlib import Path

root = Path('<WORKSPACE>/no-mistakes-receiver')
out = Path('<WORKSPACE>')
labels = ['baseline', 'candidate']
fact_bins = {label: out / f'receiver-{label}-perf' for label in labels}
graph_bins = {label: out / f'receiver-{label}-graph-perf' for label in labels}
fact_cases = {
    'writes': ('test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts', 100),
    'no_writes': ('test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts', 100),
    'shared_histories': ('test-cases/codebase-analysis/import-facts/fixture/many-late-defaults.mts', 1),
    'jsx_control': ('test-cases/ast-snippets/selectors/fixture/app-text-targets.tsx', 100),
}
graph_cases = {
    'origin_small': 'test-cases/codebase-analysis/value-flow-origin-small/fixture',
    'origin_large': 'test-cases/codebase-analysis/value-flow-origin-large/fixture',
    'lazy_imported': 'test-cases/codebase-analysis/value-flow-imported-receiver-cache/fixture',
}
env = dict(os.environ, RAYON_NUM_THREADS='4')
rows = []

def digest(file):
    result = hashlib.sha256()
    with file.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1048576), b''):
            result.update(chunk)
    return result.hexdigest()

def file_info(file):
    source = file.read_text()
    return dict(bytes=file.stat().st_size, lines=len(source.splitlines()),
                jsx_lines=sum(bool(re.search(r'<[A-Za-z]', line)) for line in source.splitlines()),
                sha256=digest(file))

def lock_provenance(revision):
    lock = tomllib.loads(subprocess.check_output(['git', 'show', f'{revision}:Cargo.lock'], cwd=root, text=True))
    version = None
    for package in lock['package']:
        if package['name'] == 'no-mistakes':
            version = package['version']
            package['version'] = '<workspace-version>'
    assert version is not None
    return version, hashlib.sha256(json.dumps(lock, sort_keys=True).encode()).hexdigest()

def sample(kind, case, demand, label, binary, args, round_number):
    start = time.perf_counter()
    process = subprocess.Popen([str(binary), *map(str, args)], stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, env=env)
    _, status, usage = os.wait4(process.pid, 0)
    process.returncode = os.waitstatus_to_exitcode(status)
    row = dict(kind=kind, case=case, demand=demand, binary=label, round=round_number,
               wall_s=time.perf_counter()-start, cpu_s=usage.ru_utime+usage.ru_stime,
               rss_kib=usage.ru_maxrss, report=process.stdout.read().decode().strip(),
               exit=process.returncode, stderr=process.stderr.read().decode())
    assert row['exit'] == 0 and not row['stderr'], row
    rows.append(row)
    (out / 'receiver-performance-raw.json').write_text(json.dumps(rows, indent=2))

assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=root, text=True).strip(), 'Commit the measured source first'
for bins in [fact_bins, graph_bins]:
    assert len({digest(binary) for binary in bins.values()}) == 2
baseline_artifacts = json.loads((out / 'receiver-baseline-artifacts.json').read_text())
candidate_artifacts = json.loads((out / 'receiver-candidate-artifacts.json').read_text())
assert baseline_artifacts['features'] == candidate_artifacts['features']
assert baseline_artifacts['profile'] == candidate_artifacts['profile']
assert baseline_artifacts['lock_sha256'] == candidate_artifacts['lock_sha256']
for artifact in [baseline_artifacts, candidate_artifacts]:
    artifact['crate_version'], artifact['normalized_lock_sha256'] = lock_provenance(artifact['revision'])
assert baseline_artifacts['normalized_lock_sha256'] == candidate_artifacts['normalized_lock_sha256']
assert baseline_artifacts['harness_hashes'] == candidate_artifacts['harness_hashes']

for case, (path, iterations) in fact_cases.items():
    for demand in ['values', 'imports']:
        for round_number in range(4):
            for label in ['baseline', 'candidate', 'candidate', 'baseline']:
                sample('facts', case, demand, label, fact_bins[label],
                       [root / path, demand, iterations], round_number)
for case, path in graph_cases.items():
    for round_number in range(4):
        for label in ['baseline', 'candidate', 'candidate', 'baseline']:
            sample('graph', case, None, label, graph_bins[label], [root / path, 1], round_number)

summary = dict(baseline_revision=baseline_artifacts['revision'],
               candidate_revision=candidate_artifacts['revision'],
               checkout_revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
               artifact_provenance={'baseline': baseline_artifacts, 'candidate': candidate_artifacts},
               samples_per_binary_per_case=8, rayon_threads=4, cases=[],
               binary_hashes={kind: {label: digest(binary)
                                     for label, binary in bins.items()}
                              for kind, bins in [('facts', fact_bins), ('graph', graph_bins)]})
for kind, case, demand in dict.fromkeys((row['kind'], row['case'], row['demand']) for row in rows):
    selected = [row for row in rows if (row['kind'], row['case'], row['demand']) == (kind, case, demand)]
    result = dict(kind=kind, case=case, demand=demand,
                  iterations_per_sample=fact_cases[case][1] if kind == 'facts' else 1)
    if kind == 'facts':
        result['source'] = file_info(root / fact_cases[case][0])
    else:
        folder = root / graph_cases[case]
        files = subprocess.check_output(['git', 'ls-files', '-z', str(folder.relative_to(root) / '*.mts')], cwd=root).decode().split('\0')
        result['sources'] = {str((root / file).relative_to(folder)): file_info(root / file)
                             for file in files if file}
    for label in labels:
        group = [row for row in selected if row['binary'] == label]
        assert len(group) == 8 and len({row['report'] for row in group}) == 1
        result[label] = {metric: statistics.median(row[metric] for row in group)
                         for metric in ['wall_s', 'cpu_s', 'rss_kib']}
        result[label]['report'] = group[0]['report']
    if case == 'lazy_imported':
        baseline = result['baseline']['report'].split(':')
        candidate = result['candidate']['report'].split(':')
        assert baseline[0] == candidate[0] and int(baseline[1]) == 0 and int(candidate[1]) == 128, result
    else:
        assert result['baseline']['report'] == result['candidate']['report'], result
    summary['cases'].append(result)
(out / 'receiver-performance-summary.json').write_text(json.dumps(summary, indent=2))
print(json.dumps(summary, indent=2))
