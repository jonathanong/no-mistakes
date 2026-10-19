"""Bounded feasibility pairs or balanced same-VM B-C-C-B measurements."""
import json
import os
from pathlib import Path
import subprocess
import sys

from common import SOURCES, digest, install_cancel_handler, machine, save, workload
from report import compare_reports
from worker import sample

install_cancel_handler()
case, jobs, diagnostics, order = sys.argv[1:]
assert case in {'verdicts-default', 'verdicts-ledger', 'phases-full', 'phases-control'}
assert jobs in {'1', '4', 'default'} and diagnostics in {'on', 'off'}
assert order in {'bc', 'bccb'}
output = Path('results')
output.mkdir()
root = Path('fixture').resolve()
physical = workload(root)
config = root / '.no-mistakes.yml'
if case == 'phases-control':
    import yaml
    original = yaml.safe_load(config.read_text())
    assert len(original.pop('rules')) == 133
    config = root / '.matched-empty.yml'
    assert not config.exists()
    with (root / '.git/info/exclude').open('a') as excluded:
        excluded.write('\n/.matched-empty.yml\n')
    config.write_text(yaml.safe_dump(original, sort_keys=False))
    assert yaml.safe_load(config.read_text()) == original
roles = {'b': 'base', 'c': 'phases'}
if case == 'verdicts-default':
    roles['c'] = 'verdicts'
elif case == 'verdicts-ledger':
    roles = {'b': 'verdicts', 'c': 'verdicts'}
proofs, binaries = {}, {}
for arm, role in roles.items():
    folder = Path('cli') / ('matched-cli-' + role)
    proof = json.loads((folder / 'provenance.json').read_text())
    binary = (folder / 'no-mistakes').resolve()
    binary.chmod(0o755)
    probe = folder / 'cpu-probe'
    probe.chmod(0o755)
    assert proof['source'] == SOURCES[role]
    assert digest(binary) == proof['binarySha256']
    assert digest(probe) == proof['cpuProbeSha256']
    proof['effectiveDefaultJobs'] = int(subprocess.check_output([str(probe)]))
    proofs[arm], binaries[arm] = proof, binary
for field in ('lockSha256', 'rustc', 'cargo', 'features', 'profile', 'command', 'buildJobs', 'incremental'):
    assert proofs['b'][field] == proofs['c'][field], field
if case != 'verdicts-ledger':
    assert proofs['b']['binarySha256'] != proofs['c']['binarySha256']
assert proofs['b']['effectiveDefaultJobs'] == proofs['c']['effectiveDefaultJobs']
provenance = {'case': case, 'jobs': jobs, 'diagnostics': diagnostics, 'order': order,
              'workload': physical, 'configSha256': digest(config), 'builds': proofs,
              'machine': machine(), 'harnessCommit': os.environ['GITHUB_SHA'],
              'baseAncestry': 'Phases descends from401; 401-to-8de changes only tests, fixtures, docs and ESLint. Native source, manifests and lock are identical.',
              'pairing': 'All arms on one VM; bc is feasibility only, bccb is balanced.'}
save(output / 'provenance.json', provenance)
rows = []
for ordinal, arm in enumerate(order):
    # Installation/build processes must have finished before either clock starts.
    names = set(subprocess.check_output(['ps', '-e', '-o', 'comm='], text=True).split())
    assert not names.intersection({'cargo', 'rustc', 'rustdoc', 'cc1', 'ld.lld'})
    row = sample(binaries[arm], root, config, jobs, diagnostics,
                 case == 'verdicts-ledger' and arm == 'c', output, arm, ordinal)
    rows.append(row)
    save(output / 'samples.json', rows)
    if not row['eligible']:
        save(output / 'block.json', {'eligible': False, 'reason': 'censored or invalid arm'})
        raise SystemExit(1)
assert workload(root) == physical
if not all(row['eligible'] for row in rows):
    save(output / 'pair.json', {'eligible': False, 'reason': 'censored or invalid arm; no performance delta'})
    raise SystemExit(1)
assert all(row['exit'] == rows[0]['exit'] for row in rows)
assert all(row['semanticStderr'] == rows[0]['semanticStderr'] for row in rows)
parity = compare_reports(output / rows[0]['stdout'], output / rows[1]['stdout'],
                         with_ledger=case == 'verdicts-ledger',
                    feature_delta=case == 'verdicts-default')
if order == 'bccb':
    compare_reports(output / rows[3]['stdout'], output / rows[2]['stdout'],
                    with_ledger=case == 'verdicts-ledger',
                    feature_delta=case == 'verdicts-default')
    assert digest(output / rows[0]['stdout']) == digest(output / rows[3]['stdout'])
    assert digest(output / rows[1]['stdout']) == digest(output / rows[2]['stdout'])
if 'ledger' in parity:
    parity['ledger'] = {key: value for key, value in parity['ledger'].items()
                        if key != 'by_rule_application_profile_file_span_subcheck'}
if diagnostics == 'on' and case == 'phases-full':
    assert all(row['workCounts'].get('check.fact_collections') == 1
               for row in rows if row['arm'] == 'c')
ratios = []
for baseline, candidate in [(rows[0], rows[1])] + ([(rows[3], rows[2])] if order == 'bccb' else []):
    ratio = {field: candidate[field] / baseline[field] for field in ('wallSeconds', 'peakRssKiB')}
    ratio['cpuSeconds'] = ((candidate['userSeconds'] + candidate['systemSeconds']) /
                           (baseline['userSeconds'] + baseline['systemSeconds']))
    ratios.append(ratio)
save(output / 'block.json', {'eligible': True, 'ratios': ratios, 'parity': parity,
                           'balanced': order == 'bccb', 'observationsOnly': True,
                           'needsReverseRecheckOnSurprisingDelta': True})
