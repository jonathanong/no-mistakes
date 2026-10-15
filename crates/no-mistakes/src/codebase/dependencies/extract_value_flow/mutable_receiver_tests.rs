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
        (32..=16 * 32).contains(&visits),
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

#[cfg(feature = "test-instrumentation")]
#[path = "mutable_receiver_scale_tests.rs"]
mod scale;

#[test]
#[cfg(feature = "test-instrumentation")]
fn mutable_root_target_decomposition_excludes_member_writes() {
    for (name, patterns) in [
        ("mutable-let-direct.mts", 0),
        ("mutable-let-pattern-rhs-rebind.mts", 1),
    ] {
        let source = mutable_receiver_source(name);
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
        assert!(facts.value_flow.reference_root_probes > 0);
        assert_eq!(
            facts.value_flow.receiver_root_target_decompositions, patterns,
            "{name}"
        );
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn terminal_receiver_histories_skip_successor_bookkeeping() {
    let source = mutable_receiver_source("terminal-receiver-history.mts");
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
    assert!(facts.value_flow.reference_terminal_paths > 0);
    assert_eq!(
        facts.value_flow.reference_successor_bookkeeping, 2,
        "only the two mutable containers require successor bookkeeping"
    );
    assert!(facts.value_flow.reference_path_visits <= 8);
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn imported_alias_successors_remain_active_with_an_unrelated_mutable_root() {
    let source = mutable_receiver_source("terminal-import-with-unrelated-root.mts");
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
    assert_eq!(facts.value_flow.reference_terminal_paths, 0);
    assert!(facts.value_flow.reference_successor_bookkeeping >= 2);
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
}

#[path = "mutable_receiver_version_tests.rs"]
mod versions;
