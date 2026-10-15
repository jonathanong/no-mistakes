"""Exact stacked source pair and one-shot remote check for the ARM diagnostic."""
import hashlib
import json
import os
import subprocess
from pathlib import Path

from common import require
from proposal_guard import check, git

HERE = Path(__file__).resolve().parent
SPEC = json.loads((HERE / 'pair.json').read_text())
ROOT = HERE.parents[2] if len(HERE.parents) > 2 else Path('/tmp/no-mistakes-2060')


def remote_snapshot():
    """One remote snapshot per stage; no CI polling or waiting."""
    def api(path):
        return json.loads(subprocess.check_output(['gh', 'api', path], cwd=ROOT, text=True))
    parent = api('repos/jonathanong/no-mistakes/pulls/2022')
    pr = api('repos/jonathanong/no-mistakes/pulls/' + str(SPEC['pr']['number']))
    require(parent['head']['sha'] == SPEC['baseline']['required_remote_head'], 'stacked_parent_head_changed')
    require(pr['head']['sha'] == SPEC['pr']['head_sha'] and pr['head']['sha'] == SPEC['candidate']['revision'], 'candidate_remote_head_changed')
    require(pr['base']['ref'] == SPEC['pr']['base_branch'] and pr['draft'] is True, 'candidate_base_or_draft_changed')
    run = api('repos/jonathanong/no-mistakes/actions/runs/' + str(SPEC['pr']['required_successful_ci_run']))
    require(run['head_sha'] == SPEC['candidate']['revision'] and run['conclusion'] == 'success', 'exact_head_ci_not_successful')
    pinned = SPEC['pr']['four_codecov_check_runs']
    names = ['codecov/' + kind + '/' + language for kind in ('patch', 'project') for language in ('rust', 'javascript')]
    require(set(pinned) == set(names), 'four_codecov_pin_set_changed')
    for name, check_id in pinned.items():
        row = api('repos/jonathanong/no-mistakes/check-runs/' + str(check_id))
        require(row['id'] == check_id and row['name'] == name
                and row['head_sha'] == SPEC['candidate']['revision']
                and row['status'] == 'completed' and row['conclusion'] == 'success',
                'pinned_codecov_check_not_successful', check=name)
    return {'parent_head': parent['head']['sha'], 'candidate_head': pr['head']['sha'], 'candidate_base':pr['base']['ref'], 'draft':pr['draft'], 'ci_run':run['id'], 'codecov':pinned}


def current_pair(_checkout=None):
    snap = remote_snapshot()
    return snap, SPEC['baseline']['revision']


def source_inventory(source):
    """Compare every Rust/Cargo source working byte with its Git blob, not only HEAD."""
    source = Path(source)
    files = git(source, 'ls-files').splitlines()
    selected = sorted(p for p in files if p in ('Cargo.toml', 'Cargo.lock', 'crates/no-mistakes/Cargo.toml')
                      or p.startswith('crates/no-mistakes/') and p.endswith('.rs'))
    rows = []
    for name in selected:
        work = (source / name).read_bytes()
        blob = subprocess.check_output(['git', 'show', 'HEAD:' + name], cwd=source)
        require(work == blob, 'source_file_differs_from_git', path=name)
        rows.append({'path':name,'sha256':hashlib.sha256(work).hexdigest(),'bytes':len(work)})
    proof = hashlib.sha256(json.dumps(rows,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    return rows, proof


def reviewed_stage(stage, diagnostic=True):
    head = git(ROOT, 'rev-parse', 'HEAD') if diagnostic else None
    approved_commit = os.environ.get('APPROVED_DIAGNOSTIC_COMMIT')
    check(SPEC, ROOT, stage, head, approved_commit)
    require(not git(ROOT, 'status', '--porcelain'), 'dirty_diagnostic_checkout')
    manifest=json.loads((HERE/'proposal-file-hashes.json').read_text())
    for name, expected in manifest['files'].items():
        path=HERE/name
        require(path.is_file() and not path.is_symlink()
                and hashlib.sha256(path.read_bytes()).hexdigest()==expected,
                'reviewed_driver_file_changed',file=name)
    tracked=set(git(ROOT,'ls-files','--','.github/scripts/typed-executor-native-2060').splitlines())
    expected_tracked={'.github/scripts/typed-executor-native-2060/'+name for name in manifest['files']}
    expected_tracked.add('.github/scripts/typed-executor-native-2060/proposal-file-hashes.json')
    require(tracked==expected_tracked,'reviewed_driver_file_set_changed')
    require(os.environ.get('GITHUB_REF') == 'refs/heads/' + SPEC['diagnostic_branch'], 'wrong_branch')
    require(os.environ.get('GITHUB_SHA') == approved_commit, 'wrong_workflow_commit')
    return remote_snapshot()
