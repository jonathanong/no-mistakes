use super::*;

fn mutable_receiver_source(name: &str) -> String {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    std::fs::read_to_string(root.join("src/nested-receiver-containers").join(name)).unwrap()
}

#[test]
fn mutable_container_references_stay_receiver_only_and_imports_only_stays_cold() {
    let source = mutable_receiver_source("mutable-let-guarded.mts");
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    for enabled in [false, true] {
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            enabled,
        );
        assert!(!facts
            .value_flow
            .bindings
            .iter()
            .any(|binding| binding.name == "box"));
        if enabled {
            let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
            assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
            #[cfg(feature = "test-instrumentation")]
            {
                assert_eq!(facts.value_flow.receiver_only_origins, 1);
                assert!(facts.value_flow.reference_path_visits <= 8);
            }
        } else {
            assert!(facts.value_flow.receiver_mutations.is_none());
            #[cfg(feature = "test-instrumentation")]
            {
                assert_eq!(facts.value_flow.receiver_only_origins, 0);
                assert_eq!(facts.value_flow.reference_capture_indexes, 0);
            }
        }
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn mutable_root_snapshots_bound_origin_projection_work() {
    let source = mutable_receiver_source("mutable-root-scale.mts");
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
    assert!(!facts
        .value_flow
        .bindings
        .iter()
        .any(|binding| binding.name == "box"));
    assert_eq!(facts.value_flow.receiver_only_origins, 129);
    assert_eq!(facts.value_flow.receiver_only_nodes, 257);
    assert!(facts.value_flow.receiver_only_lookups <= 3 * 128);
    assert_eq!(facts.value_flow.receiver_only_visits, 128);
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn nested_mutable_patterns_bound_rhs_phase_walk_work() {
    let source = mutable_receiver_source("mutable-pattern-phase-scale.mts");
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
    let visits = facts.value_flow.receiver_only_phase_visits;
    assert!(
        visits >= 32 && visits <= 16 * 32,
        "root RHS visits: {visits}"
    );
    let imports = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        false,
    );
    assert_eq!(imports.value_flow.receiver_only_phase_visits, 0);
}

#[test]
fn receiver_traversal_revisits_only_for_weaker_acquisition_evidence() {
    let mut traversal = receiver_traversal::ReceiverTraversal::default();
    let key = (0, "client".to_string());
    let path = vec![Some("get".to_string())];
    let late = vec![Some((30, None))];
    assert_eq!(
        traversal.enter(&key, &path, late.clone(), None),
        Some(late.clone())
    );
    // An active receiver stops cycles even when a second route is weaker.
    assert_eq!(traversal.enter(&key, &path, vec![None], None), None);
    traversal.exit(&key, None);
    let early = vec![Some((10, None))];
    assert_eq!(
        traversal.enter(&key, &path, early.clone(), None),
        Some(early)
    );
    traversal.exit(&key, None);
    assert_eq!(traversal.enter(&key, &path, late, None), None);
    // Ambiguous acquisition is weaker than either exact cutoff and must revisit.
    assert_eq!(
        traversal.enter(&key, &path, vec![None], None),
        Some(vec![None])
    );
    traversal.exit(&key, None);
    assert_eq!(
        traversal.enter(&key, &path, vec![Some((1, None))], None),
        None
    );
}
