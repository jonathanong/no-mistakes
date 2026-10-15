use super::*;

#[test]
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
    assert_eq!(facts.value_flow.receiver_only_nodes, 129);
    assert!(facts.value_flow.receiver_only_lookups <= 3 * 128);
    assert_eq!(facts.value_flow.receiver_only_visits, 128);
    assert_eq!(facts.value_flow.reference_state_heap_buffers, 0);
    assert_eq!(facts.value_flow.receiver_only_pending_spills, 0);
    // Early invalidation and post-RHS completion both rebind, but the name
    // needs storage only on the first event.
    assert_eq!(facts.value_flow.reference_rebind_calls, 256);
    assert_eq!(facts.value_flow.reference_rebind_name_allocations, 1);
    // Only the 128 member writers need phase/history metadata. The root
    // reassignments still invalidate bindings without creating writer receipts.
    assert_eq!(facts.value_flow.reference_metadata_captures, 128);
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn nested_class_patterns_bound_root_phase_walk_work() {
    for name in [
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        let source = mutable_receiver_source(name);
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            true,
        );
        let visits = facts.value_flow.receiver_only_phase_visits;
        assert!(
            visits > 0 && visits <= 32 * 16,
            "{name}: root phase visits {visits}"
        );
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn deferred_mutable_alias_diamond_retains_shared_traversal() {
    let source = mutable_receiver_source("mutable-pattern-deferred-diamond.mts");
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
    assert!(facts.value_flow.reference_state_heap_buffers > 0);
    assert!(
        facts.value_flow.reference_path_visits <= 16 * 32,
        "diamond projection visits {}",
        facts.value_flow.reference_path_visits
    );
}

#[test]
fn deferred_foreign_roots_do_not_rescan_future_origin_suffixes() {
    let mut counts = Vec::new();
    for writes in [32, 128] {
        let source =
            mutable_receiver_source(&format!("mutable-pattern-future-foreign-{writes}.mts"));
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
        counts.push(facts.value_flow.receiver_only_visits);
    }
    assert!(
        counts[1] <= counts[0] + 16 && counts[1] <= 16 * 4,
        "sixteen writes with32/128 future foreign leaves: {counts:?}"
    );
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn mutable_root_alternatives_keep_lazy_pending_overflow() {
    for name in [
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
    ] {
        let source = mutable_receiver_source(name);
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            true,
        );
        let receiver = if name.contains("foreign") {
            "replica"
        } else {
            "client"
        };
        let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0][receiver];
        assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
        assert!(facts.value_flow.receiver_only_pending_spills > 0, "{name}");
        assert!(facts.value_flow.receiver_only_visits > 1, "{name}");
    }
}
