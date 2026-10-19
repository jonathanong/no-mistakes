"""Fail-closed contracts for reusing only the freshly pinned CLI pair."""
from datetime import datetime, timezone
import re

from inputs import BUILD_FIELDS, SOURCES
from common import assert_same_machine_identity

BUILDER_RUN = 39293039433
BUILDER_SHA = '26c28f246c03ce7f235b7d1d8765e96a11005aed'
REPOSITORY_ID = 1235521884
REPOSITORY = 'jonathanong/no-mistakes'
ARTIFACT_NAME = 'current-verdict-arm-clis'
WORKFLOW = '.github/workflows/current-verdict-ordering-arm.yml'
RECHECK_JOB_MINUTES = 270


def sha(value):
    assert isinstance(value, str) and re.fullmatch('[0-9a-f]{64}', value)
    return value


def validate_manifest(manifest):
    assert manifest['schema'] == 'nm2238-recheck-artifact-v1'
    assert manifest['builderRun'] == BUILDER_RUN
    assert manifest['builderCommit'] == BUILDER_SHA
    assert manifest['repositoryId'] == REPOSITORY_ID
    assert manifest['sources'] == SOURCES
    assert manifest['artifactName'] == ARTIFACT_NAME
    assert type(manifest['artifactId']) is int and manifest['artifactId'] > 0
    sha(manifest['artifactZipSha256'])
    assert set(manifest['sourceTrees']) == set(SOURCES)
    assert set(manifest['build']) == set(BUILD_FIELDS)
    assert manifest['build']['features'] == 'default'
    assert manifest['build']['profile'] == {'lto': True, 'codegen-units': 1}
    assert manifest['build']['command'] == ['cargo', 'build', '--release', '--locked',
                                           '-p', 'no-mistakes', '--bin', 'no-mistakes']
    assert manifest['build']['buildJobs'] == 2
    assert manifest['build']['incremental'] is False
    sha(manifest['build']['lockSha256'])
    assert manifest['build']['rustc'] and manifest['build']['cargo']
    assert isinstance(manifest['files'], dict) and manifest['files']
    for path, digest in manifest['files'].items():
        assert path.startswith(('base/', 'candidate/'))
        sha(digest)
    for role in SOURCES:
        assert re.fullmatch('[0-9a-f]{40}', manifest['sourceTrees'][role])
        for filename in ('no-mistakes', 'cpu-probe', 'source.json', 'provenance.json'):
            assert f'{role}/{filename}' in manifest['files']


def validate_metadata(row, artifact_id, name, digest, run, head):
    assert row['id'] == artifact_id and row['name'] == name
    assert row['digest'] == 'sha256:' + sha(digest)
    assert row['expired'] is False
    assert datetime.fromisoformat(row['expires_at'].replace('Z', '+00:00')) > datetime.now(timezone.utc)
    origin = row['workflow_run']
    assert origin['id'] == run and origin['head_sha'] == head
    assert origin['repository_id'] == origin['head_repository_id'] == REPOSITORY_ID


def validate_run(run, jobs):
    assert run['id'] == BUILDER_RUN and run['head_sha'] == BUILDER_SHA
    assert run['repository']['id'] == run['head_repository']['id'] == REPOSITORY_ID
    assert run['path'] == WORKFLOW and run['event'] == 'push'
    assert run['head_branch'] == 'eval/2238-current-arm'
    assert jobs['total_count'] <= 100
    build = [job for job in jobs['jobs'] if job['name'] == 'Fresh matched production CLI pair']
    assert len(build) == 1 and build[0]['status'] == 'completed'
    assert build[0]['conclusion'] == 'success'


def validate_builds(manifest, proofs, initial):
    for role, proof in proofs.items():
        assert proof['source'] == SOURCES[role]
        assert proof['tree'] == manifest['sourceTrees'][role]
        assert proof['artifactHarnessCommit'] == BUILDER_SHA
        assert not proof.get('frozenCliBuild')
        assert all(proof[field] == manifest['build'][field] for field in BUILD_FIELDS)
        assert all(initial[role][key] == proof[key] for key in initial[role])
        for filename, field in [('no-mistakes', 'binarySha256'), ('cpu-probe', 'cpuProbeSha256')]:
            assert proof[field] == manifest['files'][f'{role}/{filename}']
    assert_same_machine_identity(proofs['base']['machine'], proofs['candidate']['machine'])
    assert proofs['base']['cpuProbeBuild'] == proofs['candidate']['cpuProbeBuild']
    assert proofs['base']['cpuProbeSha256'] == proofs['candidate']['cpuProbeSha256']
    assert proofs['base']['binarySha256'] != proofs['candidate']['binarySha256']
