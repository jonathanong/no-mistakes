use super::*;

#[test]
#[cfg(feature = "test-instrumentation")]
fn nested_patterns_share_one_catalog_across_same_and_distinct_receivers() {
    for (mode, entries) in [("same", 1), ("members", 64), ("receivers", 64)] {
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../test-cases/codebase-analysis/import-facts/fixture/destructuring-phase-{mode}.mts"));
        let source = std::fs::read_to_string(fixture).unwrap();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{mode}: {:?}",
            parsed.diagnostics
        );
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            true,
        );
        assert_eq!(facts.value_flow.early_assignment_catalogs, 1, "{mode}");
        assert_eq!(facts.value_flow.early_assignment_phase_writes, 64, "{mode}");
        assert_eq!(facts.value_flow.early_assignment_scope_receipts, 64, "{mode}");
        assert_eq!(facts.value_flow.early_assignment_segments, 128, "{mode}");
        assert_eq!(
            facts.value_flow.early_assignment_normalization_visits, entries,
            "{mode}"
        );
        // Count the retained entries across the entire file, not just one member.
        let histories = facts.value_flow.receiver_mutations.as_ref().unwrap();
        let total = histories
            .values()
            .flat_map(|names| names.values())
            .map(|history| history.early_phase_entries())
            .sum::<usize>();
        assert_eq!(total, entries, "{mode}");
        for index in 0..64 {
            let receiver = if mode == "receivers" {
                format!("r{index}")
            } else {
                "primary".into()
            };
            let member = if mode == "members" {
                format!("p{index}")
            } else {
                "get".into()
            };
            let marker = format!("consume({receiver})");
            let offset = source.rfind(&marker).unwrap() as u32;
            assert!(
                histories[&0][&receiver].assignment_invalidated(&[&member], offset, None, false),
                "{mode} {index}"
            );
        }
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn nested_lhs_rhs_and_deferred_contexts_partition_catalogs_and_read_boundaries() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture/src/destructuring-nested.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert_eq!(facts.value_flow.early_assignment_catalogs, 6);
    assert_eq!(facts.value_flow.early_assignment_phase_writes, 8);
    assert_eq!(facts.value_flow.early_assignment_normalization_visits, 8);
    assert!(facts.value_flow.early_assignment_segments <= 48);
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    for (receiver, marker, expected) in [
        ("inner", "nestedRhs(inner, outer)", false),
        ("outer", "nestedRhs(inner, outer)", true),
        ("inner", "afterInner(inner)", true),
        ("rhsNested", "source(rhsNested)", false),
        ("rhsNested", "afterNested(rhsNested)", true),
        ("root", "afterRoot(root)", true),
        ("updated", "afterUpdate(updated)", true),
    ] {
        let offset = source.find(marker).unwrap() as u32;
        // Both call acquisition (start) and completion (closing parenthesis)
        // must map to the same phase unless an inner assignment intervenes.
        for offset in [offset, offset + marker.len() as u32 - 1] {
            assert_eq!(
                histories[receiver].assignment_invalidated(&["get"], offset, None, false),
                expected,
                "{receiver} {marker} at {offset}"
            );
        }
    }
}
