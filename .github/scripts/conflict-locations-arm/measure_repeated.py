"""Three same-VM main A/A and candidate BCCB blocks with complete raw evidence."""
import json
import os
import shutil
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
config_dir = Path(os.environ.get('RUNNER_TEMP', '/tmp')) / f'ordering-policy-{os.getpid()}'
config_dir.mkdir(mode=0o700)
config = config_dir / 'ordering-only.yml'
# Config-parent gitignore augmentation must see the original consumer rules.
shutil.copy2(root / '.gitignore', config_dir / '.gitignore')
assert common.digest(config_dir / '.gitignore') == '116a90411abf2bb9f2efef56dd64f7ba8a02ab13cc4caa047c5bfcba55e68e57'
config_proof = derive_config(root, config, saved_config)
config_proof.update(gitignoreSha256=common.digest(config_dir / '.gitignore'),
                    gitignoreCopiedAlongsideConfig=True)
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
    'jobs': jobs, 'blocks': 3, 'sameVmAA': True,
    'diagnostics': diagnostics, 'ledger': ledger, 'order': 'BCCB',
    'collectionCapSecondsPerArm': CAP_SECONDS,
    'stepBudgetMinutes': 245,
    'jobBudgetMinutes': JOB_MINUTES})
rows, status = [], 'incomplete'
try:
    for stage in ('aa', 'bccb'):
        for block in range(3):
            folder = output / stage / str(block)
            folder.mkdir(parents=True)
            current = []

            def sample_arm(role, use_ledger, arm, ordinal):
                names = set(subprocess.check_output(['ps', '-e', '-o', 'comm='], text=True).split())
                assert not names.intersection({'cargo', 'rustc', 'rustdoc', 'cc1', 'ld.lld'})
                row = worker.sample(binaries[role], root, config, jobs, diagnostics,
                                    use_ledger, folder, arm, ordinal)
                row.update(stage=stage, block=block, sourceRole=role,
                           ledgerRequested=use_ledger)
                rows.append(row)
                common.save(output / 'samples.json', rows)
                print(f"[sample] stage={stage} block={block} arm={arm} role={role} "
                      f"wall={row['wallSeconds']} user={row['userSeconds']} "
                      f"system={row['systemSeconds']} rssKiB={row['peakRssKiB']}", flush=True)
                return row

            if stage == 'aa':
                for ordinal, arm in enumerate(('b', 'c', 'c', 'b')):
                    row = sample_arm('base', False, arm, ordinal)
                    current.append(row)
                    common.save(folder / 'samples.json', current)
                    assert row['eligible']
                    assert row['stdoutSha256'] == current[0]['stdoutSha256']
                    assert row['semanticStderr'] == current[0]['semanticStderr']
                    assert row['exit'] == current[0]['exit']
                parity = {'identicalMainBinary': True, 'rawRepeatHashesMatch': True,
                          'semanticStderrMatch': True, 'exitStatusMatch': True}
            else:
                run_bccb(plan, sample_arm, folder, ledger == 'on', current,
                         lambda saved: common.save(folder / 'samples.json', saved))
                parity = validate_repeats(current, folder, ledger == 'on')
            common.save(folder / 'block.json', {
                'stage': stage, 'block': block, 'parity': parity,
                'completedArms': len(current), 'order': 'BCCB'})
    assert common.workload(root) == physical
    status = 'passed'
finally:
    common.save(output / 'status.json', {
        'status': status, 'eligibleForGate': False, 'rows': len(rows),
        'completedArms': sum(row.get('eligible', False) for row in rows),
        'deadlineCensoredArms': sum(row.get('deadlineHit', False) for row in rows),
        'invalidArms': sum(not row.get('eligible', False) and
                           not row.get('deadlineHit', False) for row in rows),
        'blocksPerStage': 3, 'stages': ['aa', 'bccb'], 'order': 'BCCB'})
    shutil.rmtree(config_dir)
