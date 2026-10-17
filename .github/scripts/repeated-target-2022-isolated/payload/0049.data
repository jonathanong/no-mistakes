"""Static draft guard; final freeze requires new source and semantic reviews."""
import json
from pathlib import Path
import yaml

HERE = Path(__file__).resolve().parent
OLD = Path('/tmp/nm2022-v4-corrected-proposal/.github/scripts/repeated-target-native-1971')


def review():
    pair = json.loads((HERE / 'pair.json').read_text())
    historical = json.loads((OLD / 'pair.json').read_text())
    assert pair['status'].startswith('UNREVIEWED DRAFT')
    assert pair['pr']['number'] == 2022
    assert pair['pr']['head_sha'] == pair['candidate']['pr_head'] == pair['diagnostic_parent']
    assert pair['pr']['base_branch'] == 'main' and pair['pr']['draft'] is True
    assert pair['baseline']['revision'] == pair['baseline']['required_remote_head'] == '63760fa2a58b931498b9609afdc749dde12063aa'
    assert pair['candidate']['revision'] == pair['candidate']['proposed_revision'] == 'f6eafbf01485e66c99aa8d6ef7bb594d9d27e2b1'
    assert pair['candidate']['merge_parents'] == [pair['baseline']['revision'], pair['pr']['head_sha']]
    assert pair['source_git_inventory_sha256']['baseline'] and pair['source_git_inventory_sha256']['candidate']
    assert pair['pr']['required_successful_ci_run'] == 38809097005
    assert pair['pr']['four_codecov_check_runs'] == {
        'codecov/patch/rust': 116681517694,
        'codecov/project/rust': 116681510341,
        'codecov/patch/javascript': 116679111078,
        'codecov/project/javascript': 116679096645,
    }
    assert pair['semantic_policy']['status'] == 'UNREVIEWED FULL REPORTS'
    assert {kind: len(entries) for kind, entries in pair['semantic_policy']['report_hashes'].items()} == {
        'graph': 8, 'cli': 6, 'probe': 10, 'napi': 3,
    }
    assert pair['semantic_policy']['collection_provenance']['complete_manifest_sha256']
    assert pair['workloads'] == historical['workloads']
    assert pair['workload_count'] == len(pair['workloads']) == 16
    assert pair['sampling']['long_orders'] == ['ABBA', 'BAAB']
    assert pair['sampling']['long_blocks_per_order'] == historical['blocks_each_order']
    assert pair['sampling']['metrics'] == ['cpu_s', 'user_s', 'system_s', 'wall_s', 'rss_kib']
    assert set(pair['inputs']) == set(historical['inputs'])
    workflow = yaml.safe_load((HERE / 'workflow-proposal.yml').read_text())
    triggers = workflow.get('on', workflow.get(True))  # PyYAML's YAML 1.1 bool alias.
    assert triggers['push']['branches'] == [pair['diagnostic_branch']]
    assert workflow['jobs']['registration']['if'] == "${{ github.event_name == 'push' }}"
    assert workflow['jobs']['pair-preflight']['if'] == "${{ github.event_name == 'workflow_dispatch' }}"
    assert workflow['jobs']['measurement']['if'] == "${{ github.event_name == 'workflow_dispatch' && always() }}"
    assert set(workflow['jobs']['measurement']['needs']) == {'pair-preflight', 'production-build', 'layout-build'}
    for job in ('pair-preflight', 'production-build', 'layout-build', 'measurement'):
        checkout = [step for step in workflow['jobs'][job]['steps']
                    if step.get('uses', '').startswith('actions/checkout@')]
        assert len(checkout) == 1 and checkout[0]['with']['fetch-depth'] == 0
    return {'status': 'DRAFT FAIL CLOSED', 'workloads': 16, 'inputs': 12,
            'stages': list(workflow['jobs']), 'source_pair_pinned': True}


if __name__ == '__main__':
    print(json.dumps(review(), sort_keys=True))
