use super::*;

#[test]
fn suspensions_invalidate_actual_captures_without_allocating_on_cold_paths() {
    for (case, invalidated) in [
        ("global-after-capture-await", true),
        ("global-await-operand-capture", true),
        ("global-before-capture-await", false),
        ("global-deferred-await", false),
        ("global-after-capture-yield", true),
        ("global-yield-operand-capture", true),
        ("global-before-capture-yield", false),
        ("global-cold-suspension", false),
    ] {
        let source = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-containers")
            .join(format!("{case}.mts"))).unwrap();
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty(), "{case}");
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            true,
        );
        let invalid = facts
            .value_flow
            .receiver_mutations
            .as_ref()
            .is_some_and(|scopes| {
                scopes.values().any(|histories| {
                    histories.get("client").is_some_and(|history| {
                        history.assignment_invalidated(&["get"], source.len() as u32, None, false)
                    })
                })
            });
        assert_eq!(invalid, invalidated, "{case}: original imported receiver");
        #[cfg(feature = "test-instrumentation")]
        if case == "global-cold-suspension" {
            assert_eq!(facts.value_flow.reference_call_indexes, 0);
            assert_eq!(facts.value_flow.reference_capture_indexes, 0);
        } else {
            // Prove these negatives exercised a real capture and indexed events.
            assert_eq!(facts.value_flow.reference_capture_indexes, 1, "{case}");
            assert_eq!(facts.value_flow.reference_call_indexes, 1, "{case}");
            assert!(facts.value_flow.reference_call_entries <= 6, "{case}");
        }
    }
}
