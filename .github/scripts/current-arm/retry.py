"""Verify pinned ARM artifacts, run one capacity arm, then finalize its retry."""
import json
import os
from pathlib import Path
import subprocess
import sys
import urllib.request

from inputs import CAP_SECONDS, SOURCES, outcome, validate_parity
import common
import worker

RUN, ARTIFACT, HARNESS = 39282454396, 12204356184, '4bee78e73b19f571f766011c34b43395936d54f9'
ARCHIVE = '03b1795720e60ee507cb4923b633b93d7533dfe151d45fc8ceb8e15642c1b2e8'
LOCK = '10b8ea13c1fd4526aca570dfa7fa5e6e064f69198ffca18a59cb349a5f17d50a'
BUILD_HELPERS = {
    '.github/scripts/current-arm/build.py': '48e9ff20de127b89be21bd6fe47b1db487b0598f5905b2acbb9ae5e75f94691a',
    '.github/scripts/matched-arm/build.py': 'e3c5c469d71babc30a1c8603a2422a2484c93cf134cd2c4963868ca05a728c6f',
    '.github/scripts/matched-arm/common.py': '73dff1daafe857a2df0f6ed3fae768a0e19f248a4077870ea9a8c0db63fd47e9',
    '.github/scripts/matched-arm/frozen.py': '3ee22d1d2e540b2ce6e4b98bc08134c3e14a3349559f34d5b2dee7ab9a57f0f2'}
REPOSITORY, REPOSITORY_ID = 'jonathanong/no-mistakes', 1235521884
TREES = {'base': 'b0c58d8ae0c29597aae546873927d6721abec6b8',
         'phases': 'ac757c6dd00fa82de24c1d96682a913af4a143bb'}
BINARIES = {'base': '1f95a3378d607060e4d416ed2da73fbcb053cba9385dca7c646eb28224df1b1e',
            'phases': '16a7e512821f35a2643efe69ab8584ec128b2b846cc60c3e8a8fc4bbc803368d'}
PROBE = '45f3c5bb35a722e59f6437fd5b3429aa6012cea3653746689db34b63fa8639fb'
FIELDS = ('lockSha256', 'rustc', 'cargo', 'features', 'profile', 'command',
          'buildJobs', 'incremental')
IMAGE = ('ubuntu24-arm64', '20261018.154.1')
RUSTC = ('rustc 1.99.1 (58cf8f9c3 2026-10-13)\nbinary: rustc\n'
        'commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67\n'
        'commit-date: 2026-10-13\nhost: aarch64-unknown-linux-gnu\n'
        'release: 1.99.1\nLLVM version: 23.1.1')
CARGO = 'cargo 1.99.1 (5f94df478 2026-08-27)'


def check_artifact_row(row, artifact_id, expected_digest, run_id, head_sha, name):
    run = row['workflow_run']
    assert row['id'] == artifact_id and row['name'] == name and not row['expired']
    assert row['digest'] == 'sha256:' + expected_digest
    assert run['id'] == run_id and run['head_sha'] == head_sha
    assert run['repository_id'] == REPOSITORY_ID
    assert run['head_repository_id'] == REPOSITORY_ID
    return {'id': row['id'], 'name': row['name'], 'digest': row['digest'],
            'expired': row['expired'], 'workflowRun': run}


def artifact_row(artifact_id, expected_digest, run_id, head_sha, name):
    request = urllib.request.Request(
        f'https://api.github.com/repos/{REPOSITORY}/actions/artifacts/{artifact_id}',
        headers={'Authorization': 'Bearer ' + os.environ['GITHUB_TOKEN'],
                 'Accept': 'application/vnd.github+json'})
    with urllib.request.urlopen(request, timeout=20) as response:
        row = json.load(response)
    return check_artifact_row(row, artifact_id, expected_digest, run_id, head_sha, name)


def stable_machine(machine):
    required = ('uname', 'affinity', 'imageOS', 'imageVersion', 'cpuinfo', 'meminfo')
    assert all(key in machine for key in required)
    uname = machine['uname']
    assert len(uname) == 5
    return {'uname': [uname[0], uname[2], uname[3], uname[4]],
            'affinity': machine['affinity'], 'imageOS': machine['imageOS'],
            'imageVersion': machine['imageVersion'], 'cpuinfo': machine['cpuinfo']}


def verify():
    assert os.uname().machine == 'aarch64'
    assert (os.environ['ImageOS'], os.environ['ImageVersion']) == IMAGE
    rows, proofs = {}, {}
    for role in SOURCES:
        folder = Path('cli') / role
        proof = json.loads((folder / 'provenance.json').read_text())
        assert proof['source'] == SOURCES[role] and proof['tree'] == TREES[role]
        assert not proof.get('frozenCliBuild')
        assert proof['lockSha256'] == LOCK
        assert proof['artifactHarnessCommit'] == HARNESS
        assert proof['binarySha256'] == BINARIES[role] == common.digest(folder / 'no-mistakes')
        assert proof['cpuProbeSha256'] == PROBE == common.digest(folder / 'cpu-probe')
        assert proof['rustc'] == RUSTC and proof['cargo'] == CARGO
        assert proof['features'] == 'default'
        assert proof['profile'] == {'lto': True, 'codegen-units': 1}
        assert proof['command'] == ['cargo', 'build', '--release', '--locked',
                                    '-p', 'no-mistakes', '--bin', 'no-mistakes']
        assert proof['buildJobs'] == 2 and proof['incremental'] is False
        machine = proof['machine']
        assert (machine['imageOS'], machine['imageVersion']) == IMAGE
        assert machine['uname'][4] == 'aarch64'
        stable_machine(machine)
        (folder / 'no-mistakes').chmod(0o755)
        (folder / 'cpu-probe').chmod(0o755)
        jobs = int(subprocess.check_output([str((folder / 'cpu-probe').resolve())]))
        assert jobs == 4
        proof['effectiveDefaultJobs'] = jobs
        proofs[role] = proof
    assert all(proofs['base'][key] == proofs['phases'][key] for key in FIELDS)
    assert proofs['base']['cpuProbeBuild'] == proofs['phases']['cpuProbeBuild']
    assert stable_machine(proofs['base']['machine']) == stable_machine(proofs['phases']['machine'])
    assert proofs['base']['binarySha256'] != proofs['phases']['binarySha256']
    harness = Path(__file__).resolve().parents[3]
    helper_hashes = {name: common.digest(harness / name) for name in BUILD_HELPERS}
    assert helper_hashes == BUILD_HELPERS
    manifest = harness / 'fixtures/performance/current-arm/retry/build-artifact-root-verified.json'
    artifact = json.loads(manifest.read_text())
    assert artifact['run'] == RUN and artifact['artifact'] == ARTIFACT
    assert artifact['archiveSha256'] == ARCHIVE and artifact['zipEntries'] == 12
    assert artifact['sources'] == SOURCES
    for role in SOURCES:
        for field in ('binarySha256', 'cpuProbeSha256', 'rustc', 'cargo', 'features',
                      'profile', 'buildJobs', 'incremental'):
            assert artifact['builds'][role][field] == proofs[role][field]
    measured_machine = common.machine()
    assert stable_machine(proofs['base']['machine']) == stable_machine(measured_machine)
    physical = common.workload(Path('fixture').resolve())
    consumer = json.loads(Path('prepared/fixture-provenance.json').read_text())
    assert physical == consumer
    return {'purpose': 'observable-capacity-only', 'eligibleForGate': False,
            'collectionCapSeconds': CAP_SECONDS, 'diagnostics': 'on', 'jobs': '1',
            'sourceProofs': proofs, 'builderMachine': proofs['base']['machine'],
            'measurementMachine': measured_machine, 'workload': physical,
            'cliArtifact': artifact_row(ARTIFACT, ARCHIVE, RUN, HARNESS, 'current-phase-clis'),
            'consumerArtifact': artifact_row(12158768554,
                '080223598499563b3f21bdd29e450536db72183c5ef65e3d3f221976273c6911',
                39189171362, '76b3a95fa47490371eeeaa9e0eb8794fd7a9ad98',
                'matched-consumer')}


def arm(role):
    output = Path('results')
    provenance = json.loads((output / 'provenance.json').read_text())
    root = Path('fixture').resolve()
    assert common.workload(root) == provenance['workload']
    rows = output / 'samples.json'
    data = json.loads(rows.read_text()) if rows.exists() else []
    assert role in ('base', 'phases') and all(row['arm'] != role for row in data)
    worker.CAP_SECONDS = CAP_SECONDS
    try:
        row = worker.sample((Path('cli') / role / 'no-mistakes').resolve(),
                            root, root / '.no-mistakes.yml',
                            '1', 'on', False, output, role, len(data), heartbeat=True)
        assert common.workload(root) == provenance['workload']
    except BaseException as error:
        tag = f'{len(data)}-{role}'
        stdout, stderr = output / (tag + '.json'), output / (tag + '.stderr')
        if 'row' in locals():
            row.update(eligible=False, status='postflight-failed', error=repr(error))
        else:
            row = {'arm': role, 'ordinal': len(data), 'eligible': False,
                   'status': 'interrupted-or-failed', 'error': repr(error),
                   'stdout': stdout.name, 'stderr': stderr.name,
                   'stdoutSha256': common.digest(stdout) if stdout.exists() else None,
                   'stderrSha256': common.digest(stderr) if stderr.exists() else None}
        common.save(output / f'failure-{role}.json', row)
        data.append(row)
        common.save(rows, data)
        common.save(output / 'feasibility.json', outcome(data))
        raise
    data.append(row)
    common.save(rows, data)
    common.save(output / 'feasibility.json', outcome(data))


def finalize():
    output = Path('results')
    rows = json.loads((output / 'samples.json').read_text()) if (output / 'samples.json').exists() else []
    status, parity = 'failed', {'parityVerified': False}
    try:
        assert [row['arm'] for row in rows] == ['base', 'phases']
        assert all(row['eligible'] for row in rows)
        provenance = json.loads((output / 'provenance.json').read_text())
        assert common.workload(Path('fixture').resolve()) == provenance['workload']
        parity = validate_parity(output, rows)
        status = 'passed'
    finally:
        result = outcome(rows, status)
        result['parityVerified'] = status == 'passed'
        result['capacityOnly'] = True
        common.save(output / 'parity.json', parity)
        common.save(output / 'feasibility.json', result)
    if status != 'passed':
        raise SystemExit(1)


if __name__ == '__main__':
    common.install_cancel_handler()
    Path('results').mkdir(exist_ok=True)
    try:
        {'verify': lambda: common.save('results/provenance.json', verify()),
         'base': lambda: arm('base'), 'candidate': lambda: arm('phases'),
         'finalize': finalize}[sys.argv[1]]()
    except BaseException as error:
        common.save('results/step-error.json', {'step': sys.argv[1], 'error': repr(error)})
        raise
