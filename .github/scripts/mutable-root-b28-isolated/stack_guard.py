"""Pinned stack identity permits the unchanged lower layer to merge."""
def verify_stack(spec, observations):
    parent, candidate = observations['parent'], observations['pr']
    baseline = spec['baseline_head']
    assert parent['head']['sha'] == baseline, 'parent head changed'
    assert observations['baseline_commit']['tree']['sha'] == spec['baseline_tree'], 'parent tree changed'
    assert parent['state'] == 'open' or parent.get('merged') is True, 'parent closed unmerged'
    assert candidate['state'] == 'open' and candidate['draft'] is True, 'candidate must remain open draft'
    assert candidate['head']['sha'] == spec['candidate_head'], 'candidate head changed'
    main = observations['main']['object']['sha']
    merged_main = (parent.get('merged') is True and main == parent.get('merge_commit_sha')
                   and observations['main_commit']['tree']['sha'] == spec['baseline_tree']
                   and observations['main_commit']['parents'][0]['sha'] == spec['main'])
    assert main == spec['main'] or merged_main, 'unrelated main advance'
    base = candidate['base']
    pinned = base['ref'] == spec['baseline_branch'] and base['sha'] == baseline
    retargeted = merged_main and base['ref'] == 'main' and base['sha'] == main
    assert pinned or retargeted, 'candidate base changed'
