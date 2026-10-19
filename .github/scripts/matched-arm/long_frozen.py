"""Fail closed on the four artifacts from the audited 39189171362 ARM run."""
import json
import os
from pathlib import Path
import urllib.request

from common import SOURCES, digest, save, workload

RUN = 39189171362
HEAD = '76b3a95fa47490371eeeaa9e0eb8794fd7a9ad98'
REPOSITORY = 'jonathanong/no-mistakes'
REPOSITORY_ID = 1235521884
IMAGE = ('ubuntu24-arm64', '20261012.149.1')
PROBE = '45f3c5bb35a722e59f6437fd5b3429aa6012cea3653746689db34b63fa8639fb'
ARTIFACTS = {
    'matched-cli-base': (12158853321, 'b5df55f3dc761cc6acf1775c9fba9773c696ab7dcf95112e0087dbff641318d4'),
    'matched-cli-phases': (12159575789, '7f7f393164ea89e866b31dad63c220145cd061b8b119f4921e06abd57990e427'),
    'matched-cli-verdicts': (12159229842, '1d01722f1f1341465085ea9761a6f7fd45c78f11dd681df8f4e1597b5641e77a'),
    'matched-consumer': (12158768554, '080223598499563b3f21bdd29e450536db72183c5ef65e3d3f221976273c6911'),
}
BINARIES = {
    'base': '82e318de143c852f46f32b16fa60d908fdc32c011418ab266e84fe622f3bfe53',
    'phases': 'fc3173c7b49f4d430bff7ec947873575413eade92e948260d050069f5b8b2151',
    'verdicts': '98962e5fb403db98693821067854ef8bf31e2a0e7e19025f4b0eb79b0e24dc8b',
}
BUILD_FIELDS = ('lockSha256', 'rustc', 'cargo', 'features', 'profile',
                'command', 'buildJobs', 'incremental')


def verify_artifact(name, row):
    artifact_id, archive_sha = ARTIFACTS[name]
    origin = row['workflow_run']
    assert row['id'] == artifact_id and row['name'] == name
    assert row['digest'] == 'sha256:' + archive_sha and not row['expired']
    assert origin['id'] == RUN and origin['head_sha'] == HEAD
    assert origin['repository_id'] == REPOSITORY_ID
    assert origin['head_repository_id'] == REPOSITORY_ID


def artifact_metadata(name):
    artifact_id, _ = ARTIFACTS[name]
    request = urllib.request.Request(
        f'https://api.github.com/repos/{REPOSITORY}/actions/artifacts/{artifact_id}',
        headers={'Authorization': 'Bearer ' + os.environ['GITHUB_TOKEN'],
                 'Accept': 'application/vnd.github+json'},
    )
    with urllib.request.urlopen(request, timeout=20) as response:
        row = json.load(response)
    verify_artifact(name, row)
    return {'id': row['id'], 'name': row['name'], 'digest': row['digest'],
            'workflowRun': row['workflow_run']}


def verify_local():
    assert (os.environ.get('ImageOS'), os.environ.get('ImageVersion')) == IMAGE
    assert os.uname().machine == 'aarch64'
    proofs = {}
    for role, expected_sha in BINARIES.items():
        folder = Path('cli') / ('matched-cli-' + role)
        proof = json.loads((folder / 'provenance.json').read_text())
        assert proof['source'] == SOURCES[role]
        assert proof['binarySha256'] == expected_sha == digest(folder / 'no-mistakes')
        assert proof['cpuProbeSha256'] == PROBE == digest(folder / 'cpu-probe')
        assert (proof['machine']['imageOS'], proof['machine']['imageVersion']) == IMAGE
        assert proof['artifactHarnessCommit'] == HEAD
        proofs[role] = proof
    reference = proofs['base']
    assert reference['features'] == 'default'
    assert reference['profile'] == {'lto': True, 'codegen-units': 1}
    assert reference['command'] == ['cargo', 'build', '--release', '--locked',
                                    '-p', 'no-mistakes', '--bin', 'no-mistakes']
    assert reference['buildJobs'] == 2 and reference['incremental'] is False
    for role in ('phases', 'verdicts'):
        assert all(proofs[role][field] == reference[field] for field in BUILD_FIELDS)
    expected = json.loads(Path('prepared/fixture-provenance.json').read_text())
    assert workload(Path('fixture').resolve()) == expected
    return {'artifacts': [artifact_metadata(name) for name in ARTIFACTS],
            'sources': {role: proof['source'] for role, proof in proofs.items()},
            'binarySha256': BINARIES, 'cpuProbeSha256': PROBE,
            'consumer': expected, 'image': IMAGE}


if __name__ == '__main__':
    Path('results').mkdir(exist_ok=True)
    save('results/frozen-artifacts.json', verify_local())
