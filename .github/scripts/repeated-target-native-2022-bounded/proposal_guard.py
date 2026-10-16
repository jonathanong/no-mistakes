"""Static fail-closed guard for the proposed #2022 native source pair.

This module never builds or samples. A reviewed driver must call check() before
each stage, then perform that stage's separately reviewed operations.
"""
import hashlib
import json
import subprocess
import io
import tarfile
from pathlib import Path

from postgres_source_control import validate_pins

HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def git(repo, *args):
    from phase_common import run
    return run(['git', *args], cwd=repo, timeout=90).decode().strip()


def check(spec, repo, stage, diagnostic_head=None, approved_commit=None):
    assert stage in {'pair-preflight', 'production-build', 'layout-build',
                     'semantic-preflight', 'napi-parity', 'sample', 'summary'}
    assert spec['status'] == 'REVIEWED AND FROZEN', 'draft experiment cannot execute'
    from cohorts import validate
    validate(spec)
    # Pin shape is mandatory before source I/O or any separately gated build.
    facades = spec['semantic_policy'].get('napi_facade_sha256')
    assert isinstance(facades, dict) and set(facades) == {'index.js', 'index.mjs'}, 'missing or malformed N-API facade pins'
    assert all(isinstance(value, str) and len(value) == 64
               and all(char in '0123456789abcdef' for char in value)
               for value in facades.values()), 'missing or malformed N-API facade pins'
    control = spec['semantic_policy'].get('conditional_executor_control')
    assert isinstance(control, dict) and set(control) == {'fixture', 'script', 'expected_tests_per_arm'}, 'missing or malformed conditional executor control pins'
    assert control['expected_tests_per_arm'] == 1, 'conditional executor control test count changed'
    expected_control_paths = {'fixture': 'fixtures/postgres-facts/executed-source/conditional-executors.ts',
                              'script': 'packages/no-mistakes/scripts/postgres-conditional-executors-api.test.js'}
    for key in ('fixture', 'script'):
        row = control[key]
        assert isinstance(row, dict) and set(row) == {'path', 'sha256', 'source_revision'}, 'missing or malformed conditional executor control pins'
        assert isinstance(row['sha256'], str) and len(row['sha256']) == 64 and all(c in '0123456789abcdef' for c in row['sha256']), 'missing or malformed conditional executor control pins'
        assert row['path'] == expected_control_paths[key], 'conditional executor control path changed'
        assert row['source_revision'] == spec['baseline']['revision'], 'conditional executor source revision changed'
    validate_pins(spec)
    assert isinstance(spec['pr']['number'], int) and spec['pr']['number'] > 0
    assert spec['pr']['head_sha'] == spec['candidate']['pr_head']
    assert spec['candidate']['revision'] == spec['candidate']['proposed_revision']
    assert spec['candidate']['tree'] == git(repo, 'rev-parse', spec['candidate']['revision'] + '^{tree}')
    assert spec['baseline']['tree'] == git(repo, 'rev-parse', spec['baseline']['revision'] + '^{tree}')
    assert git(repo, 'show', '-s', '--format=%P', spec['candidate']['revision']).split() == [
        spec['baseline']['revision'], spec['pr']['head_sha']
    ]
    assert git(repo, 'merge-base', spec['baseline']['revision'], spec['candidate']['revision']) == spec['baseline']['revision']
    assert git(repo, 'merge-base', spec['baseline']['revision'], spec['pr']['head_sha']) == spec['baseline']['revision']
    assert spec['candidate']['tree'] == git(repo, 'rev-parse', spec['pr']['head_sha'] + '^{tree}')
    assert spec['semantic_policy']['status'] == 'REVIEWED FULL REPORTS'
    assert spec['semantic_policy']['report_hashes']
    assert spec['source_git_inventory_sha256']['baseline']
    assert spec['source_git_inventory_sha256']['candidate']
    assert spec['diagnostic_branch']
    assert spec['pr']['required_successful_ci_run']
    assert len(spec['pr']['four_codecov_check_runs']) == 4
    if diagnostic_head is not None:
        assert approved_commit and len(approved_commit) == 40
        assert all(c in '0123456789abcdef' for c in approved_commit)
        assert diagnostic_head == approved_commit
        assert spec['diagnostic_parent'] == spec['pr']['head_sha']
        assert git(repo, 'merge-base', spec['diagnostic_parent'], diagnostic_head) == spec['diagnostic_parent']
        changes = git(repo, 'diff', '--name-status', spec['diagnostic_parent'], diagnostic_head).splitlines()
        assert changes and all(line.startswith('A\t.github/scripts/repeated-target-native-2022-bounded/')
                               or line == 'A\t.github/workflows/repeated-target-native-1971-next.yml'
                               for line in changes)
    wanted={row['path']:row for input_ in spec['inputs'].values()for row in input_['files']}
    assert all(input_['files']for input_ in spec['inputs'].values())
    if wanted:
        from phase_common import run
        data=run(['git','archive','--format=tar',spec['candidate']['revision'],*sorted(wanted)],cwd=repo,timeout=90)
        observed={}
        with tarfile.open(fileobj=io.BytesIO(data),mode='r:')as archive:
            for member in archive:
                if member.name not in wanted:continue
                assert member.isfile()
                raw=archive.extractfile(member).read()
                observed[member.name]={'path':member.name,'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'lines':len(raw.splitlines())}
        assert observed==wanted
    return True


if __name__ == '__main__':
    import sys
    spec = json.loads((HERE / 'pair.json').read_text())
    check(spec, Path(sys.argv[1]), sys.argv[2],
          sys.argv[3] if len(sys.argv) > 3 else None,
          sys.argv[4] if len(sys.argv) > 4 else None)
