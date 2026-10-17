"""One current-state guard at declared stage boundaries; never a polling loop."""
import json
from supervise import command
from stack_guard import verify_stack


def verify(spec, output, deadline, stage):
    prefix = 'eligibility-' + stage
    urls = {'main': 'git/ref/heads/main', 'pr': 'pulls/2054',
            'parent': 'pulls/2022',
            'ci': 'actions/runs/' + str(spec['candidate_ci_run_id']),
            'checks': 'commits/' + spec['candidate_head'] + '/check-runs?per_page=100'}
    results = {}
    for key, endpoint in urls.items():
        label = prefix + '-' + key
        command(['gh', 'api', 'repos/jonathanong/no-mistakes/' + endpoint], label, output, deadline, 30)
        results[key] = json.loads((output / (label + '.stdout')).read_text())
    for key, revision in [('baseline_commit', spec['baseline_head']),
                          ('main_commit', results['main']['object']['sha'])]:
        label = prefix + '-' + key
        command(['gh', 'api', 'repos/jonathanong/no-mistakes/git/commits/' + revision],
                label, output, deadline, 30)
        results[key] = json.loads((output / (label + '.stdout')).read_text())
    verify_stack(spec, results)
    ci = results['ci']
    assert ci['head_sha'] == spec['candidate_head'] and ci['status'] == 'completed' and ci['conclusion'] == 'success'
    checks = results['checks']
    assert checks['total_count'] <= 100
    for name in ['codecov/patch/rust', 'codecov/project/rust', 'codecov/patch/javascript', 'codecov/project/javascript']:
        found = [c for c in checks['check_runs'] if c['name'] == name]
        assert len(found) == 1 and found[0]['status'] == 'completed' and found[0]['conclusion'] == 'success'
