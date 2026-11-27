use super::*;

#[test]
#[cfg(feature = "test-instrumentation")]
fn saved_root_writes_keep_first_receipts_paths_and_active_keys_inline() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    let source =
        std::fs::read_to_string(root.join("src/nested-receiver-containers/mutable-root-scale.mts"))
            .unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    state::FIRST_ALLOCATIONS.with(|counts| counts.set([0; 3]));
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    // These count the three actual clone/allocation sites; retaining one
    // owned key and member string is intentional and not counted as eliminated.
    assert_eq!(state::FIRST_ALLOCATIONS.with(|counts| counts.get()), [0; 3]);
    assert_eq!(facts.value_flow.receiver_only_visits, 128);
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
}

#[test]
fn traversal_storage_layout_is_reported_from_actual_private_types() {
    let bytes = std::mem::size_of::<state::TraversalState>();
    eprintln!("ACTUAL_TRAVERSAL_STATE_BYTES {bytes}");
    assert!(bytes <= 144, "original traversal state used 144 bytes");
}

#[test]
fn inline_receipts_preserve_empty_unknown_zip_and_context_identity() {
    let key = (0, "box".to_string());
    let mut empty = state::TraversalState::default();
    assert_eq!(empty.enter(&key, &[], vec![]), Some(vec![]));
    assert!(empty.remove_active(&key));
    // An empty stored receipt must not grow when a later arrival is longer.
    assert_eq!(empty.enter(&key, &[], vec![None]), None);
    let mut unknown = state::TraversalState::default();
    assert_eq!(unknown.enter(&key, &[None], vec![None]), Some(vec![None]));
    assert!(unknown.remove_active(&key));
    assert_eq!(unknown.enter(&key, &[None], vec![Some((10, None))]), None);
    let mut known = state::TraversalState::default();
    let path = [Some("client".to_string())];
    assert_eq!(
        known.enter(&key, &path, vec![Some((30, None))]),
        Some(vec![Some((30, None))])
    );
    assert!(known.remove_active(&key));
    assert_eq!(known.enter(&key, &path, vec![]), None);
    assert_eq!(
        known.enter(&key, &path, vec![Some((20, None)), None]),
        Some(vec![Some((20, None))])
    );
    assert!(known.remove_active(&key));
    assert_eq!(
        known.enter(&key, &path, vec![Some((10, Some(CallableId(7))))]),
        Some(vec![None])
    );
    assert!(known.remove_active(&key));
    assert!(!known.remove_active(&key));
    assert_eq!(known.enter(&key, &path, vec![None]), None);
}

#[test]
fn heap_receipts_preserve_length_order_and_inactive_promotion() {
    let key = (0, "box".to_string());
    let other = (0, "other".to_string());
    let path = [Some("outer".to_string()), None];
    let reads = vec![Some((30, None)), Some((40, None))];
    let mut traversal = state::TraversalState::default();
    assert_eq!(traversal.enter(&key, &path, reads.clone()), Some(reads));
    assert!(traversal.remove_active(&key));
    assert_eq!(
        traversal.enter(&key, &path, vec![Some((20, None))]),
        Some(vec![Some((20, None)), Some((40, None))])
    );
    assert!(traversal.remove_active(&key));
    assert_eq!(traversal.enter(&other, &[], vec![]), Some(vec![]));
    assert!(!traversal.remove_active(&key));
    // Promotion keeps both old receipt positions, even after a short arrival.
    assert_eq!(
        traversal.enter(&key, &path, vec![None, Some((35, None))]),
        Some(vec![None, Some((35, None))])
    );
    assert!(traversal.remove_active(&other));
    assert!(traversal.remove_active(&key));
    assert_eq!(
        traversal.enter(&key, &path, vec![None, Some((35, None))]),
        None
    );
}

#[test]
fn first_promotion_keeps_active_binding_and_distinguishes_dynamic_paths() {
    let first = (0, "first".to_string());
    let other = (0, "other".to_string());
    let mut traversal = state::TraversalState::default();
    assert_eq!(
        traversal.enter(&first, &[None], vec![None]),
        Some(vec![None])
    );
    // A different path for the active binding must reject before promotion.
    assert_eq!(traversal.enter(&first, &[], vec![]), None);
    assert!(matches!(
        traversal,
        state::TraversalState::FirstInline { .. }
    ));
    assert_eq!(traversal.enter(&other, &[], vec![]), Some(vec![]));
    assert!(traversal.contains_active(&first));
    assert!(traversal.contains_active(&other));
    assert_eq!(traversal.enter(&first, &[], vec![]), None);
    assert!(traversal.remove_active(&first));
    assert_eq!(traversal.enter(&first, &[], vec![]), Some(vec![]));
    assert!(traversal.remove_active(&other));
    assert!(traversal.remove_active(&first));
    assert_eq!(traversal.enter(&first, &[None], vec![None]), None);
}
