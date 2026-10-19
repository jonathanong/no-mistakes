"""One BCCB ordering-only observation; fail closed and retain every raw arm."""
import json
import os
from pathlib import Path
import subprocess
import sys

from inputs import (BUILD_FIELDS, CAP_SECONDS, CONSUMER, JOB_MINUTES, SOURCES)
from ordering import (derive_config, run_bccb, sample_plan,
                      validate_fixture_contract, validate_repeats)
import common
import long_frozen
import worker
from image_contract import image_provenance

common.install_cancel_handler()
common.SOURCES = SOURCES
common.CAP_SECONDS = CAP_SECONDS
worker.CAP_SECONDS = CAP_SECONDS
assert os.uname().machine == 'aarch64'
assert os.environ['ImageOS'] == 'ubuntu24-arm64'
jobs, diagnostics, ledger = sys.argv[1:]
plan = sample_plan(jobs, diagnostics, ledger)
output = Path('results')
output.mkdir(exist_ok=True)
common.save(output / 'status.json', {
    'status': 'incomplete', 'eligibleForGate': False, 'rows': 0,
    'completedArms': 0, 'deadlineCensoredArms': 0, 'invalidArms': 0,
    'order': 'BCCB', 'jobs': jobs,
    'diagnostics': diagnostics, 'ledger': ledger})
root = Path('fixture').resolve()
physical = common.workload(root)
assert physical['commit'] == CONSUMER
fixture_proof = json.loads(Path('prepared/fixture-provenance.json').read_text())
assert physical == fixture_proof
archive = Path('prepared/fixture.tar.gz')
saved_config = Path('harness/fixtures/performance/current-verdict-arm/ordering-only.yml')
config = Path(os.environ.get('RUNNER_TEMP', '/tmp')) / f'ordering-only-{os.getpid()}.yml'
config_proof = derive_config(root, config, saved_config)
validate_fixture_contract(
    'harness/fixtures/performance/current-verdict-arm/ordering-only-provenance.json',
    common.digest(archive))
origin = long_frozen.artifact_metadata('matched-consumer')
assert origin['id'] == 12158768554
assert origin['digest'] == 'sha256:080223598499563b3f21bdd29e450536db72183c5ef65e3d3f221976273c6911'
proofs, binaries = {}, {}
for role in SOURCES:
    folder = Path('cli') / role
    proof = json.loads((folder / 'provenance.json').read_text())
    assert proof['source'] == SOURCES[role]
    assert proof['artifactHarnessCommit'] == os.environ['GITHUB_SHA']
    assert not proof.get('frozenCliBuild')
    assert proof['machine']['imageOS'] == os.environ['ImageOS']
    assert proof['machine']['uname'][-1] == os.uname().machine
    for filename, field in [('no-mistakes', 'binarySha256'), ('cpu-probe', 'cpuProbeSha256')]:
        path = folder / filename
        assert common.digest(path) == proof[field]
        path.chmod(0o755)
    proof['effectiveDefaultJobs'] = int(subprocess.check_output(
        [str((folder / 'cpu-probe').resolve())]))
    proofs[role] = proof
    binaries[role] = (folder / 'no-mistakes').resolve()
common.assert_same_machine_identity(proofs['base']['machine'],
                                    proofs['candidate']['machine'])
assert proofs['base']['cpuProbeSha256'] == proofs['candidate']['cpuProbeSha256']
assert proofs['base']['cpuProbeBuild'] == proofs['candidate']['cpuProbeBuild']
assert proofs['base']['binarySha256'] != proofs['candidate']['binarySha256']
assert all(proofs['base'][key] == proofs['candidate'][key] for key in BUILD_FIELDS)
assert proofs['base']['effectiveDefaultJobs'] == proofs['candidate']['effectiveDefaultJobs']
measurement_machine = common.machine()
images = image_provenance(proofs['base']['machine'], measurement_machine)
common.save(output / 'provenance.json', {
    'purpose': 'ordering-only measurement; not a full-config or full-consumer gate',
    'eligibleForGate': False, 'sourcePins': SOURCES, 'consumer': physical,
    'consumerArtifact': origin, 'consumerArtifactZipSha256': origin['digest'].removeprefix('sha256:'),
    'fixtureTarSha256': common.digest(archive), 'imageVersions': images,
    'orderingConfig': config_proof, 'builds': proofs, 'machine': measurement_machine,
    'harnessCommit': os.environ['GITHUB_SHA'],
    'cliBuilderHarnessCommit': os.environ['GITHUB_SHA'],
    'jobs': jobs,
    'diagnostics': diagnostics, 'ledger': ledger, 'order': 'BCCB',
    'collectionCapSecondsPerArm': CAP_SECONDS,
    'stepBudgetMinutes': 245,
    'jobBudgetMinutes': JOB_MINUTES})
rows, status = [], 'incomplete'
try:
    def sample_arm(role, use_ledger, arm, ordinal):
        names = set(subprocess.check_output(['ps', '-e', '-o', 'comm='], text=True).split())
        assert not names.intersection({'cargo', 'rustc', 'rustdoc', 'cc1', 'ld.lld'})
        row = worker.sample(binaries[role], root, config, jobs, diagnostics,
                            use_ledger, output, arm, ordinal)
        return row

    run_bccb(plan, sample_arm, output, ledger == 'on', rows,
             lambda saved: common.save(output / 'samples.json', saved))
    assert common.workload(root) == physical
    parity = validate_repeats(rows, output, ledger == 'on')
    status = 'passed'
    common.save(output / 'block.json', {
        'eligible': True, 'eligibleForGate': False, 'observationsOnly': True,
        'purpose': 'ordering-only configuration; not full-config or full-consumer evidence',
        'ratios': [
            {'wallSeconds': rows[1]['wallSeconds'] / rows[0]['wallSeconds'],
             'cpuSeconds': ((rows[1]['userSeconds'] + rows[1]['systemSeconds']) /
                            (rows[0]['userSeconds'] + rows[0]['systemSeconds'])),
             'peakRssKiB': rows[1]['peakRssKiB'] / rows[0]['peakRssKiB']},
            {'wallSeconds': rows[2]['wallSeconds'] / rows[3]['wallSeconds'],
             'cpuSeconds': ((rows[2]['userSeconds'] + rows[2]['systemSeconds']) /
                            (rows[3]['userSeconds'] + rows[3]['systemSeconds'])),
             'peakRssKiB': rows[2]['peakRssKiB'] / rows[3]['peakRssKiB']}],
        'parity': parity, 'completedArms': len(rows), 'order': 'BCCB'})
except BaseException as error:
    status = 'failed'
    common.save(output / 'block.json', {
        'eligible': False, 'eligibleForGate': False, 'status': status,
        'reason': f'{type(error).__name__}: {error}',
        'completedArms': sum(row.get('eligible', False) for row in rows),
        'deadlineCensoredArms': sum(row.get('deadlineHit', False) for row in rows),
        'invalidArms': sum(not row.get('eligible', False) and
                           not row.get('deadlineHit', False) for row in rows),
        'order': 'BCCB', 'retainedRows': len(rows)})
    raise
finally:
    common.save(output / 'status.json', {
        'status': status, 'eligibleForGate': False, 'rows': len(rows),
        'completedArms': sum(row.get('eligible', False) for row in rows),
        'deadlineCensoredArms': sum(row.get('deadlineHit', False) for row in rows),
        'invalidArms': sum(not row.get('eligible', False) and
                           not row.get('deadlineHit', False) for row in rows),
        'order': 'BCCB'})
    try:
        config.unlink()
    except FileNotFoundError:
        pass
