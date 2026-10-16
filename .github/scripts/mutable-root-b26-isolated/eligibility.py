"""One current-state guard at declared stage boundaries; never a polling loop."""
import json
from supervise import command


def verify(spec, output, deadline, stage):
    prefix = 'eligibility-' + stage
    urls = {'main': 'git/ref/heads/main', 'pr': 'pulls/2054',
            'ci': 'actions/runs/38943552393',
            'checks': 'commits/' + spec['candidate_head'] + '/check-runs?per_page=100'}
    results = {}
    for key, endpoint in urls.items():
        label = prefix + '-' + key
        command(['gh', 'api', 'repos/jonathanong/no-mistakes/' + endpoint], label, output, deadline, 30)
        results[key] = json.loads((output / (label + '.stdout')).read_text())
    assert results['main']['object']['sha'] == spec['main']
    pr = results['pr']
    assert pr['state'] == 'open' and pr['draft'] is True and pr['head']['sha'] == spec['candidate_head']
    ci = results['ci']
    assert ci['head_sha'] == spec['candidate_head'] and ci['status'] == 'completed' and ci['conclusion'] == 'success'
    checks = results['checks']
    assert checks['total_count'] <= 100
    for name in ['codecov/patch/rust', 'codecov/project/rust', 'codecov/patch/javascript', 'codecov/project/javascript']:
        found = [c for c in checks['check_runs'] if c['name'] == name]
        assert len(found) == 1 and found[0]['status'] == 'completed' and found[0]['conclusion'] == 'success'
