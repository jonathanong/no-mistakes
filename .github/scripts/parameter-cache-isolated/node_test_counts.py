import re
def require_tests(stdout, expected):
    pairs=re.findall(r'^(?:#|ℹ) (tests|pass|fail|skipped|cancelled) (\d+)$',stdout,re.MULTILINE)
    counts=dict(pairs)
    assert len(pairs)==len(counts)==5
    assert counts=={'tests':str(expected),'pass':str(expected),'fail':'0','skipped':'0','cancelled':'0'},counts
    return counts
