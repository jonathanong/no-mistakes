use super::*;

#[test]
fn ambiguous_active_versions_keep_their_exit_ownership() {
    let mut traversal = ReceiverTraversal::default();
    let key = (0, "client".to_string());
    let path = [Some("get".to_string())];
    let mut origins = mutable_origins::ReceiverBindingOrigins::default();
    origins.record((0, "client"), (1, None), ValueExpression::Unknown, true);
    let first = origins.version(&key, (1, None)).unwrap();
    assert!(!traversal.exit_single_version(&key));
    assert!(traversal
        .enter(&key, &path, vec![Some((30, None))], None)
        .is_some());
    assert!(traversal
        .enter(&key, &path, vec![Some((30, None))], Some(first))
        .is_some());
    // The ordinary and versioned entries may share a name. Neither owns a
    // fast exit until the original coordinate has disambiguated the other.
    assert!(!traversal.exit_single_version(&key));
    assert!(traversal
        .enter(&key, &path, vec![None], Some(first))
        .is_none());
    traversal.exit(&key, None);
    assert!(traversal.exit_single_version(&key));
    assert!(!traversal.exit_single_version(&key));
    assert!(traversal
        .enter(&key, &path, vec![None], Some(first))
        .is_some());
    origins.record((0, "client"), (2, None), ValueExpression::Undefined, true);
    let second = origins.version(&key, (2, None)).unwrap();
    assert!(traversal
        .enter(&key, &path, vec![None], Some(second))
        .is_some());
    assert!(!traversal.exit_single_version(&key));
    traversal.exit(&key, Some(second));
    // Retained overflow versions require exact lookup even after an exit.
    assert!(!traversal.exit_single_version(&key));
    traversal.exit(&key, Some(first));
    assert!(traversal
        .enter(&key, &path, vec![None], Some(first))
        .is_none());
}

#[test]
fn active_binding_cycles_are_independent_of_visited_paths() {
    let mut traversal = ReceiverTraversal::default();
    let key = (0, "box".to_string());
    let get = [Some("get".to_string())];
    let set = [Some("set".to_string())];
    let reads = vec![Some((30, None))];
    assert_eq!(
        traversal.enter_ordinary(&key, &get, reads.clone()),
        Some(reads.clone())
    );
    // A second member path cannot bypass a still-active binding cycle.
    assert_eq!(traversal.enter_ordinary(&key, &set, reads.clone()), None);
    traversal.exit_ordinary(&key);
    // Once exited, the distinct path must not be mistaken for the first visit.
    assert_eq!(
        traversal.enter_ordinary(&key, &set, reads.clone()),
        Some(reads.clone())
    );
    traversal.exit_ordinary(&key);
    assert_eq!(traversal.enter_ordinary(&key, &get, reads), None);
    assert_eq!(
        traversal.enter_ordinary(&key, &get, vec![Some((20, None))]),
        Some(vec![Some((20, None))])
    );
    traversal.exit_ordinary(&key);
    assert_eq!(
        traversal.enter_ordinary(&key, &get, vec![None]),
        Some(vec![None])
    );
    traversal.exit_ordinary(&key);
    assert_eq!(
        traversal.enter_ordinary(&key, &get, vec![Some((10, None))]),
        None
    );
}

#[test]
fn exiting_inline_active_key_keeps_other_active_bindings() {
    let mut traversal = ReceiverTraversal::default();
    let first = (0, "first".to_string());
    let other = (0, "other".to_string());
    let third = (0, "third".to_string());
    let path = [Some("get".to_string())];
    assert!(traversal
        .enter_ordinary(&first, &path, vec![None])
        .is_some());
    assert!(traversal
        .enter_ordinary(&other, &path, vec![None])
        .is_some());
    traversal.exit_ordinary(&first);
    assert!(traversal
        .enter_ordinary(&third, &path, vec![None])
        .is_some());
    assert!(traversal
        .enter_ordinary(&other, &path, vec![None])
        .is_none());
    traversal.exit_ordinary(&other);
    traversal.exit_ordinary(&third);
    assert!(traversal
        .enter_ordinary(&other, &[None], vec![None])
        .is_some());
}
