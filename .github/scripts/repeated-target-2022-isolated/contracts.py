"""Fail-closed pure boundary checks, independent of measurement execution."""
import hashlib
import json
from pathlib import Path


def approved(plan, context):
    assert plan['status'] == 'REVIEWED_ISOLATED_EIGHT_COHORTS'
    assert plan['execution_approval']
    assert context['GITHUB_REPOSITORY'] == 'jonathanong/no-mistakes'
    assert context['APPROVED_SHA'] == context['GITHUB_SHA']


def transfer(stage, assets, context):
    proof = json.loads((stage / 'transport.json').read_text())
    assert proof['complete'] is True and proof['producer'] == 'native-download'
    assert len(proof['assets']) == len(assets) == 8
    assert len({r['name'] for r in assets}) == 8
    assert len({r['asset_id'] for r in proof['assets']}) == 8
    for key, env in [('repository', 'GITHUB_REPOSITORY'), ('run_id', 'GITHUB_RUN_ID'), ('run_attempt', 'GITHUB_RUN_ATTEMPT'), ('workflow_sha', 'GITHUB_SHA')]:
        assert proof[key] == context[env]
    for expected in assets:
        matches = [r for r in proof['assets'] if r['name'] == expected['name']]
        assert len(matches) == 1
        row = matches[0]
        path = stage / expected['name']
        assert path.parent == stage and not path.is_symlink() and path.is_file()
        assert row['asset_id'] == expected['id'] and row['exit'] == 0
        assert row['bytes'] == path.stat().st_size == expected['size']
        with path.open('rb') as stream:
            actual = hashlib.file_digest(stream, 'sha256').hexdigest()
        assert row['sha256'] == actual == expected['sha256']
    return proof


def layouts(plan):
    gate = plan['layout_gate']
    assert gate['status'] == 'PASS exact current compiler layouts'
    for key in ['proof', 'root_review']:
        pin = gate[key]
        assert hashlib.sha256(Path(pin['path']).read_bytes()).hexdigest() == pin['sha256']
    proof = json.loads(Path(gate['proof']['path']).read_text())
    assert proof['status'] == 'PASS'
    assert set(proof['variants']) == set(gate['required'])
    types = ['ImportCollector', 'ValueFlowFacts', 'ImportFacts', 'TsFileFacts', 'ValueCall', 'ResolvedCallSite']
    for variant in proof['variants'].values():
        assert [r['type'] for r in variant] == types
        assert all(type(r['size']) is int and r['size'] > 0 and r['align'] == 8 for r in variant)
    return 24


def asset_role(row):
    # Producer asset names are provenance; the declared role selects its use.
    assert row['arm'] in ('baseline', 'candidate')
    assert row['file'] in ('graph-probe', 'timing-probe', 'cli', 'addon.node')
    assert row['name'] and row['name'] not in ('.', '..')
    assert Path(row['name']).name == row['name']
    return row['arm'], row['file']
