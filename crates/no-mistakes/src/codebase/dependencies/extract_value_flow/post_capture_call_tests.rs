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

#[cfg(feature = "test-instrumentation")]
#[test]
fn decorator_applications_record_indexed_receiver_barriers() {
    for (case, invalidated, entries) in [
        ("global-decorator-application", true, 3),
        ("global-decorator-late-key", false, 4),
        ("global-decorator-anonymous-late-key", false, 4),
        ("global-decorator-nested-class", true, 3),
        ("global-decorator-static-block", true, 4),
        ("global-decorator-static-field", true, 4),
        ("global-decorator-accessor", true, 3),
        ("global-decorator-anonymous", true, 3),
        ("global-decorator-anonymous-late-member", true, 4),
        ("global-decorator-member-static", true, 4),
        ("global-decorator-before-capture", false, 2),
        ("global-decorator-class-static", false, 3),
        ("global-decorator-computed-key", false, 3),
        ("global-decorator-copied-snapshot", true, 3),
        ("global-decorator-deferred", false, 3),
        ("global-decorator-factory", true, 4),
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
            "{case}"
        );
        // Negatives must retain real captures and the application event, rather
        // than pass because an unrelated history gap already dropped precision.
        assert!(facts.value_flow.reference_capture_nodes > 0, "{case}");
        assert_eq!(facts.value_flow.reference_capture_indexes, 1, "{case}");
        assert_eq!(facts.value_flow.reference_call_indexes, 1, "{case}");
        assert_eq!(facts.value_flow.reference_call_entries, entries, "{case}");
    }
}
