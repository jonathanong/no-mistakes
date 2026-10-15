use super::*;

#[test]
fn post_capture_calls_are_shared_deduplicated_and_context_qualified() {
    for (case, invalidated) in [
        ("global-after-capture-call", true),
        ("global-call-argument-capture", true),
        ("global-after-capture-new", true),
        ("global-after-capture-tagged", true),
        ("global-before-capture-call", false),
        ("global-after-capture-deferred-call", false),
    ] {
        let source = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-containers")
            .join(format!("{case}.mts"))).unwrap();
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
        assert_eq!(
            histories.get("client").is_some_and(|history| {
                history.assignment_invalidated(&["get"], source.len() as u32, None, false)
            }),
            invalidated,
            "{case}: original imported receiver"
        );
        #[cfg(feature = "test-instrumentation")]
        {
            assert_eq!(facts.value_flow.reference_call_indexes, 1, "{case}");
            if invalidated {
                // One intervening opaque invocation, the helper get, and the
                // final helper invocation. PRE/POST must not duplicate entries.
                assert_eq!(facts.value_flow.reference_call_entries, 3, "{case}");
            } else {
                assert!(facts.value_flow.reference_call_entries <= 3, "{case}");
            }
        }
    }
}
