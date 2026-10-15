use super::*;

#[test]
fn guarded_reference_work_uses_one_shared_index_and_no_cold_maps() {
    assert_eq!(
        std::mem::size_of::<Option<(std::num::NonZeroU32, u32)>>(),
        8
    );
    for (case, guarded) in [("guarded-many-writes", true), ("unguarded-root", false)] {
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
        let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["box"];
        let events = history
            .assigned
            .as_ref()
            .unwrap()
            .replacements
            .as_ref()
            .unwrap();
        if guarded {
            let regions = events.regions.as_ref().unwrap();
            assert_eq!(regions.len(), 1);
            assert_eq!(regions.values().next().unwrap()["client"].len(), 128);
            #[cfg(feature = "test-instrumentation")]
            {
                assert_eq!(events.region_record_attempts, 128);
                assert_eq!(events.region_queries.get(), 128);
            }
        } else {
            assert!(events.regions.is_none());
            #[cfg(feature = "test-instrumentation")]
            assert_eq!(events.region_record_attempts, 0);
        }
    }
}

#[test]
fn guarded_proof_and_import_histories_use_supported_immutable_bindings() {
    for (case, cut) in [
        ("guarded-arm", true),
        ("guarded-opposite", false),
        ("guarded-after", false),
        ("guarded-child", false),
        ("guarded-ancestor-rhs", false),
        ("guarded-ancestor-conditional", false),
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
        let invalidated = histories.get("client").is_some_and(|history| {
            history.assignment_invalidated(&["get"], source.len() as u32, None, false)
        });
        assert_eq!(
            invalidated, !cut,
            "{case}: original import mutation history"
        );
        if cut {
            let offset = source.find("box.client.get =").unwrap() as u32;
            let history = &histories["box"];
            let events = history
                .assigned
                .as_ref()
                .unwrap()
                .replacements
                .as_ref()
                .unwrap();
            let region = *events.regions.as_ref().unwrap().keys().next().unwrap();
            let path = [Some("client".into())];
            let reads = [Some((offset, None))];
            assert!(!history.reference_may_be_replaced(&path, &reads, None, None, None));
            assert!(history.reference_may_be_replaced(&path, &reads, Some(region), None, None));
        }
        #[cfg(feature = "test-instrumentation")]
        if case.starts_with("guarded-ancestor-") {
            assert_eq!(facts.value_flow.receiver_region_breaks, 1, "{case}");
        }
    }
}
