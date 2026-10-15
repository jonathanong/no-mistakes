"""Read-only reviewed protocol integrity check; never dispatches a run."""
import hashlib
import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = Path('/tmp/no-mistakes-2060')

def sha(data): return hashlib.sha256(data).hexdigest()
def git(*args): return subprocess.check_output(['git', *args], cwd=REPO).strip()

def main():
    pair = json.loads((HERE/'pair.json').read_text())
    assert pair['status'] == 'REVIEWED AND FROZEN'
    assert pair['pr']['number'] == 2065 and pair['pr']['head_sha'] == pair['candidate']['revision']
    assert pair['candidate']['revision'] == 'd3729e21d9647a64f6c05442c56acb8880485abe'
    assert git('rev-parse', pair['candidate']['revision']+'^{tree}').decode() == pair['candidate']['tree']
    assert git('merge-base',pair['baseline']['revision'],pair['candidate']['revision']).decode() == pair['baseline']['revision']
    assert pair['workload_count'] == len(pair['workloads']) == 31
    assert len({w['name'] for w in pair['workloads']}) == 31
    assert pair['sampling']['long_orders'] == ['ABBA','BAAB'] and pair['sampling']['long_blocks_per_order'] == 4
    assert pair['sampling']['minimum_long_rows_per_eligible_workload'] == 32
    assert pair['features'] == ['default','mermaid-validation'] and pair['build_jobs'] == 2
    assert pair['toolchain'] == '1.99.1'
    assert pair['source_git_inventory_sha256'] == {
        'baseline':'debc1b4796489318bbd2c093962c8d8f3a22e168c7369afdb578639e483f72db',
        'candidate':'7d4b14ae4b6432569fd6a7e3b7444e15414e5c6b32a1c136feaa9ad04d9aa35b'}
    assert pair['pr']['required_successful_ci_run'] == 38788622449
    assert pair['pr']['four_codecov_check_runs'] == {
        'codecov/patch/rust':116611346964,'codecov/project/rust':116611336058,
        'codecov/patch/javascript':116609286515,'codecov/project/javascript':116609265819}
    policy=pair['semantic_policy']
    assert policy['native_report_count'] == 18
    assert len(policy['candidate_api_effect_expectations']) == 17
    assert sum(map(len,policy['candidate_api_effect_expectations'].values())) == policy['candidate_effect_count'] == 25
    assert 'receiver-order' in pair['inputs'] and 'receiver-order' in policy['untimed_graph_inputs']
    assert 'receiver-order' in policy['unchanged_graph_controls']
    assert 'layers-0-forward' not in policy['unchanged_graph_controls']
    assert len(pair['inputs']['receiver-order']['files']) == 41
    assert len(pair['inputs']['imported-utility-order']['files']) == 7
    assert 'imported-utility-order' in policy['untimed_graph_inputs']
    assert 'imported-utility-order:no-calls' in policy['untimed_graph_inputs']
    assert policy['custom_utility_api_effect_expectations'] == {
        'early.mts':[5],'late.mts':[4],'late-shapes.mts':[4,9],'shadows.mts':[]}
    assert policy['custom_utility_report_count'] == 5 and policy['custom_utility_effect_count'] == 4
    for workload in pair['workloads']:
        assert workload['input'] in pair['inputs']
        if workload['kind']=='probe': assert workload['mode'] in ('facts-imports','facts-values','graph','graph-no-calls')
        else: assert workload['kind']=='cli' and workload['command']
    for item in pair['inputs'].values():
        assert item['files']
        for row in item['files']:
            data=subprocess.check_output(['git','show',pair['candidate']['revision']+':'+row['path']],cwd=REPO)
            assert sha(data)==row['sha256'] and len(data)==row['bytes'],row['path']
    for section in ('reference_script_hashes','reference_support_hashes'):
        for name,expected in pair['template'][section].items():
            assert sha((HERE/name).read_bytes())==expected,name
    hashes=pair['semantic_policy']['report_hashes']
    assert set(hashes)=={'graph','cli','probe','napi'}
    assert set(hashes['graph'])==set(policy['untimed_graph_inputs'])
    assert set(hashes['cli'])=={w['name'] for w in pair['workloads'] if w['kind']=='cli'}
    assert set(hashes['probe'])=={w['name'] for w in pair['workloads'] if w['kind']=='probe'}
    assert set(hashes['napi'])=={'typed-projections','imported-utility-order'}
    assert policy['status'] == 'REVIEWED FULL REPORTS'
    assert pair['diagnostic_commit'] is None
    workflow=(HERE/'workflow-proposal.yml').read_text()
    assert 'diagnostic/typed-executor-native-2060-d372' in workflow
    assert "if: ${{ github.event_name == 'push' }}" in workflow
    assert "if: ${{ github.event_name == 'workflow_dispatch' }}" in workflow
    assert "if: ${{ github.event_name == 'workflow_dispatch' && always() }}" in workflow
    assert 'approved_diagnostic_commit:' in workflow
    assert 'APPROVED_DIAGNOSTIC_COMMIT: ${{ inputs.approved_diagnostic_commit }}' in workflow
    assert 'needs: [pair-preflight, production-build, layout-build]' in workflow
    assert "if: ${{ always() }}" in workflow
    assert 'BUILD_RESULT: ${{ needs.production-build.result }}' in workflow
    print(f"REVIEWED protocol integrity PASS: {len(pair['workloads'])} workloads, {sum(len(i['files']) for i in pair['inputs'].values())} exact Git input files; no dispatch or measurement performed")

if __name__=='__main__':main()
