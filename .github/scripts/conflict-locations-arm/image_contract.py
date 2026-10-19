"""Disclose the known runner rollout while preserving each matched comparison."""
IMAGES = {'20261012.149.1', '20261018.154.1'}


def image_provenance(builder, measurement):
    # Both binaries share one builder; all BCCB arms share one measurement VM.
    # Equality is enforced within those boundaries, not across the two stages.
    for machine in (builder, measurement):
        assert machine['imageOS'] == 'ubuntu24-arm64'
        assert machine['uname'][-1] == 'aarch64'
        assert machine['imageVersion'] in IMAGES
    return {'builder': builder['imageVersion'],
            'measurement': measurement['imageVersion'],
            'versionsMatch': builder['imageVersion'] == measurement['imageVersion'],
            'comparisonBoundary': 'BCCB arms on one VM; both binaries from one builder'}
