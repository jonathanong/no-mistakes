"""Static fail-closed guard for the proposed #2022 native source pair.

This module never builds or samples. A reviewed driver must call check() before
each stage, then perform that stage's separately reviewed operations.
"""
import hashlib
import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def git(repo, *args):
    return subprocess.check_output(['git', *args], cwd=repo, text=True).strip()


def check(spec, repo, stage, diagnostic_head=None, approved_commit=None):
    assert stage in {'pair-preflight', 'production-build', 'layout-build',
                     'semantic-preflight', 'napi-parity', 'sample', 'summary'}
    assert spec['status'] == 'REVIEWED AND FROZEN', 'draft experiment cannot execute'
    assert isinstance(spec['pr']['number'], int) and spec['pr']['number'] > 0
    assert spec['pr']['head_sha'] == spec['candidate']['pr_head']
    assert spec['candidate']['revision'] == spec['candidate']['proposed_revision']
    assert spec['candidate']['tree'] == git(repo, 'rev-parse', spec['candidate']['revision'] + '^{tree}')
    assert spec['baseline']['tree'] == git(repo, 'rev-parse', spec['baseline']['revision'] + '^{tree}')
    assert git(repo, 'show', '-s', '--format=%P', spec['candidate']['revision']).split() == [
        spec['baseline']['revision'], spec['pr']['head_sha']
    ]
    assert git(repo, 'merge-base', spec['baseline']['revision'], spec['candidate']['revision']) == spec['baseline']['revision']
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
        assert git(repo, 'show', '-s', '--format=%P', diagnostic_head).split() == [spec['diagnostic_parent']]
        assert spec['diagnostic_parent'] == spec['pr']['head_sha']
        changes = git(repo, 'diff', '--name-status', diagnostic_head + '^', diagnostic_head).splitlines()
        assert changes and all(line.startswith('A\t.github/scripts/repeated-target-native-1971-next/')
                               or line == 'A\t.github/workflows/repeated-target-native-1971-next.yml'
                               for line in changes)
    for input_ in spec['inputs'].values():
        assert input_['files']
        for row in input_['files']:
            data = subprocess.check_output(['git', 'show', spec['candidate']['revision'] + ':' + row['path']], cwd=repo)
            assert hashlib.sha256(data).hexdigest() == row['sha256']
            assert len(data) == row['bytes']
    return True


if __name__ == '__main__':
    import sys
    spec = json.loads((HERE / 'pair.json').read_text())
    check(spec, Path(sys.argv[1]), sys.argv[2],
          sys.argv[3] if len(sys.argv) > 3 else None,
          sys.argv[4] if len(sys.argv) > 4 else None)
