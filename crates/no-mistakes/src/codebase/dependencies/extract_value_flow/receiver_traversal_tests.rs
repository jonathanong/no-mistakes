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
