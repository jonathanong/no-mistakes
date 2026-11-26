use super::*;

#[test]
fn replacement_patterns_keep_the_selected_rhs_receiver_identity() {
    for name in [
        "assignment-rhs-pattern.mts",
        "assignment-rhs-pattern-order.mts",
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
        let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
        assert!(
            histories["replica"].assignment_invalidated(&["get"], source.len() as u32, None, false),
            "{name}"
        );
        assert!(
            !histories.get("primary").is_some_and(|history| {
                history.assignment_invalidated(&["get"], source.len() as u32, None, false)
            }),
            "{name}: the replaced root is untouched"
        );
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn replacement_rhs_prefix_queries_do_not_scan_history() {
    let mut observations = Vec::new();
    let mut receipts = Vec::new();
    for size in [64, 256] {
        let source = mutable_receiver_source(&format!("assignment-rhs-history-{size}.mts"));
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
        assert!(facts.value_flow.reference_rhs_origin_queries > 0);
        observations.push(facts.value_flow.reference_rhs_origin_queries);
        receipts.push(facts.value_flow.reference_rhs_receipt_queries);
    }
    // The same sixteen descendant writes query ordered prefixes, independent
    // of how many earlier RHS replacement events those prefixes contain.
    assert_eq!(observations[0], observations[1]);
    assert_eq!(receipts[0], receipts[1]);
    assert!(receipts[0] > 0 && receipts[0] <= 16 * 6, "{receipts:?}");
    // At most four selected-prefix queries and three destination-parent
    // queries per physical descendant writer, regardless of history length.
    assert!(observations[0] <= 16 * 7, "{observations:?}");
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn replacement_rhs_canonical_destinations_share_diamond_arrivals() {
    for depth in [4, 8] {
        let source = mutable_receiver_source(&format!("assignment-rhs-diamond-{depth}.mts"));
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
        assert!(facts.value_flow.reference_rhs_destinations > 0);
        assert!(
            facts.value_flow.reference_rhs_destinations <= 8,
            "depth {depth}: {}",
            facts.value_flow.reference_rhs_destinations
        );
        let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["primary"];
        assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn replacement_rhs_repeated_possible_identity_shares_projection() {
    for size in [64, 256] {
        let source =
            mutable_receiver_source(&format!("assignment-rhs-conditional-history-{size}.mts"));
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
        for name in ["primary", "replica"] {
            let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0][name];
            assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
        }
        // Each conditional store needs its one-entry construction buffer; the
        // final write should not enqueue the same immutable RHS alias N times.
        assert!(
            facts.value_flow.reference_pending_capacity <= size + 32,
            "size {size}: pending {}",
            facts.value_flow.reference_pending_capacity
        );
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn rebound_parent_identity_barriers_require_existing_acquisition_demand() {
    for (name, expected) in [
        ("assignment-rhs-preindex-rebound-parent.mts", 1),
        ("assignment-rhs-rebound-parent-conditional.mts", 1),
        ("assignment-rhs-rebound-parent-unknown.mts", 0),
        ("assignment-rhs-rebound-parent-scope.mts", 1),
        ("assignment-rhs-barrier-cold.mts", 0),
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
        assert_eq!(facts.value_flow.reference_rhs_barriers, expected, "{name}");
        if name == "assignment-rhs-barrier-cold.mts" {
            assert_eq!(facts.value_flow.reference_rhs_destinations, 0);
        }
    }
}
