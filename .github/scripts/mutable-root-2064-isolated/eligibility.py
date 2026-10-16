"""One current-state guard at declared stage boundaries; never a polling loop."""
import json
from supervise import command


def verify(spec, output, deadline, stage):
    prefix = 'eligibility-' + stage
    urls = {'main': 'git/ref/heads/main', 'pr': 'pulls/' + str(spec['pr_number']),
            'ci': 'actions/runs/' + str(spec['ci_run']),
            'checks': 'commits/' + spec['candidate_head'] + '/check-runs?per_page=100'}
    results = {}
    for key, endpoint in urls.items():
        label = prefix + '-' + key
        command(['gh', 'api', 'repos/jonathanong/no-mistakes/' + endpoint], label, output, deadline, 30)
        results[key] = json.loads((output / (label + '.stdout')).read_text())
    # The immutable scientific baseline may merge while the child is measured.
    # Main is recorded, but eligibility follows the exact parent source pair.
    parent_label = prefix + '-parent'
    command(['gh', 'api', 'repos/jonathanong/no-mistakes/pulls/2054'], parent_label, output, deadline, 30)
    parent = json.loads((output / (parent_label + '.stdout')).read_text())
    assert parent['head']['sha'] == spec['actual_producer_baseline']
    baseline_label = prefix + '-baseline-commit'
    command(['gh', 'api', 'repos/jonathanong/no-mistakes/commits/' + spec['actual_producer_baseline']], baseline_label, output, deadline, 30)
    baseline = json.loads((output / (baseline_label + '.stdout')).read_text())
    assert baseline['commit']['tree']['sha'] == spec['baseline_tree']
    if parent['state'] == 'closed':
        assert parent['merged'] is True and parent['merge_commit_sha']
        merge_label = prefix + '-parent-merge'
        command(['gh', 'api', 'repos/jonathanong/no-mistakes/commits/' + parent['merge_commit_sha']], merge_label, output, deadline, 30)
        merge = json.loads((output / (merge_label + '.stdout')).read_text())
        assert merge['commit']['tree']['sha'] == spec['baseline_tree']
    else:
        assert parent['state'] == 'open'
    pr = results['pr']
    assert pr['state'] == 'open' and pr['draft'] is True and pr['head']['sha'] == spec['candidate_head']
    ci = results['ci']
    assert ci['head_sha'] == spec['candidate_head'] and ci['status'] == 'completed' and ci['conclusion'] == 'success'
    checks = results['checks']
    assert checks['total_count'] <= 100
    for name in ['codecov/patch/rust', 'codecov/project/rust', 'codecov/patch/javascript', 'codecov/project/javascript']:
        found = [c for c in checks['check_runs'] if c['name'] == name]
        assert len(found) == 1 and found[0]['status'] == 'completed' and found[0]['conclusion'] == 'success'
