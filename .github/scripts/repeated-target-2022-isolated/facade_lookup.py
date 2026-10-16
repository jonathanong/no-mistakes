"""Adapt only facade Git lookup to the public tree-equivalent baseline commit."""
BASELINE_PRODUCER = 'c896db3775411e9b7867260db420a73965b91a9e'
PUBLIC_BASELINE = 'b9476931aeecc3e24f43a276bf1c6a2088bebe3b'


def with_public_baseline(original, spec, proof):
    assert proof['baseline_actual_producer'] == BASELINE_PRODUCER
    assert proof['baseline_prior_main'] == PUBLIC_BASELINE
    assert proof['baseline_prior_tree_equal'] is True
    assert spec['baseline']['revision'] == BASELINE_PRODUCER
    # The unmodified checker still verifies BOTH pinned live facades against
    # BOTH Git blobs. Only its baseline lookup ID changes, never a producer.
    spec['baseline']['revision'] = PUBLIC_BASELINE
    try:
        return original()
    finally:
        spec['baseline']['revision'] = BASELINE_PRODUCER
