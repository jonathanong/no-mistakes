use super::*;

// This is the original f126 ownership algorithm, kept as a test-only oracle.
#[path = "receiver_traversal_oracle_tests.rs"]
mod oracle;

#[test]
fn fused_traversal_matches_original_deterministic_operation_traces() {
    let keys = [
        (0, "box".to_string()),
        (1, "box".to_string()),
        (0, "other".to_string()),
    ];
    let paths = [
        vec![],
        vec![None],
        vec![Some("get".to_string())],
        vec![Some("outer".to_string()), None],
    ];
    let receipts = [
        vec![],
        vec![None],
        vec![Some((30, None))],
        vec![Some((20, None))],
        vec![Some((10, Some(CallableId(7))))],
        vec![Some((30, None)), Some((40, None))],
        vec![None, Some((35, Some(CallableId(9))))],
    ];
    let mut seed = 2019_u64;
    for trace in 0..64 {
        let mut actual = state::TraversalState::default();
        let mut expected = oracle::TraversalState::default();
        for step in 0..256 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let key = &keys[((seed >> 32) as usize) % keys.len()];
            let path = &paths[((seed >> 40) as usize) % paths.len()];
            let reads = &receipts[((seed >> 48) as usize) % receipts.len()];
            if (seed >> 24).is_multiple_of(3) {
                assert_eq!(
                    actual.remove_active(key),
                    expected.active.remove(key),
                    "trace {trace} step {step} remove {key:?}"
                );
            } else {
                assert_eq!(
                    actual.enter(key, path, reads.clone()),
                    expected.enter(key, path, reads.clone()),
                    "trace {trace} step {step} enter {key:?}/{path:?}/{reads:?}"
                );
            }
            #[cfg(feature = "test-instrumentation")]
            assert_eq!(
                actual.heap_buffers(),
                expected.heap_buffers(),
                "trace {trace} step {step} heap work units"
            );
            for key in &keys {
                assert_eq!(
                    actual.contains_active(key),
                    expected.active.contains(key),
                    "trace {trace} step {step} active {key:?}"
                );
            }
        }
    }
}
