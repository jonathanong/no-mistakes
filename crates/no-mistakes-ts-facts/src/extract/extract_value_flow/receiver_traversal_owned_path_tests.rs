use super::*;

use super::super::super::oracle;

#[test]
fn owned_paths_preserve_buffers_and_empty_dynamic_distinction() {
    let key = (0, "box".to_string());
    for path in [vec![], vec![None], vec![Some("client".to_string())]] {
        let original = path.clone();
        let pointer = path.first().and_then(Option::as_ref).map(|s| s.as_ptr());
        let mut state = TraversalState::default();
        let (reads, selected) = state.enter_owned_path(&key, path, vec![None]).unwrap();
        assert_eq!(reads, vec![None]);
        assert_eq!(selected.as_ref(), original);
        assert!(matches!(selected, Cow::Borrowed(_)));
        assert_eq!(
            selected
                .first()
                .and_then(Option::as_ref)
                .map(|s| s.as_ptr()),
            pointer
        );
        drop(selected);
        assert!(state.remove_active(&key));
    }
}

#[test]
fn owned_paths_match_original_operation_traces() {
    let keys = [
        (0, "box".to_string()),
        (1, "box".to_string()),
        (0, "other".to_string()),
    ];
    let paths = [
        vec![],
        vec![None],
        vec![Some("client".to_string())],
        vec![Some("outer".to_string()), None],
    ];
    let receipts = [
        vec![],
        vec![None],
        vec![Some((30, None))],
        vec![Some((20, None))],
        vec![Some((10, Some(CallableId(7))))],
        vec![None, Some((35, None))],
    ];
    let mut seed = 2019_u64;
    for _ in 0..64 {
        let mut actual = TraversalState::default();
        let mut expected = oracle::TraversalState::default();
        for _ in 0..256 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let key = &keys[((seed >> 32) as usize) % keys.len()];
            let path = &paths[((seed >> 40) as usize) % paths.len()];
            let reads = &receipts[((seed >> 48) as usize) % receipts.len()];
            if (seed >> 24).is_multiple_of(3) {
                assert_eq!(actual.remove_active(key), expected.active.remove(key));
            } else {
                let result = actual.enter_owned_path(key, path.clone(), reads.clone());
                let expected = expected.enter(key, path, reads.clone());
                match result {
                    Some((reads, selected)) => {
                        assert_eq!(Some(reads), expected);
                        assert_eq!(selected.as_ref(), path);
                    }
                    None => assert!(expected.is_none()),
                }
            }
            #[cfg(feature = "test-instrumentation")]
            assert_eq!(actual.heap_buffers(), expected.heap_buffers());
            for key in &keys {
                assert_eq!(actual.contains_active(key), expected.active.contains(key));
            }
        }
    }
}

#[cfg(feature = "test-instrumentation")]
#[path = "receiver_traversal_owned_facts_tests.rs"]
mod facts;
