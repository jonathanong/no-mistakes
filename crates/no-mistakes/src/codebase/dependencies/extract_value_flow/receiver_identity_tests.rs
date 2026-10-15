use super::*;

#[test]
fn capture_suffixes_share_alias_chain_and_direct_writes_stay_cold() {
    for (case, nodes, logical, visits) in [
        ("global-capture-chain", 32, 528, None),
        ("global-direct-write", 0, 0, Some(0)),
        ("global-many-captures", 16, 16, Some(2063)),
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
        #[cfg(feature = "test-instrumentation")]
        {
            assert_eq!(
                facts.value_flow.reference_capture_indexes,
                usize::from(nodes > 0),
                "{case}"
            );
            assert_eq!(facts.value_flow.reference_capture_nodes, nodes, "{case}");
            assert_eq!(facts.value_flow.reference_phase_indexes, 0, "{case}");
            if let Some(visits) = visits {
                assert_eq!(facts.value_flow.reference_identity_visits, visits, "{case}");
            }
            assert_eq!(
                facts.value_flow.reference_capture_logical_tokens, logical,
                "{case}"
            );
        }
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = (facts, nodes, logical, visits);
    }
}

#[test]
fn capture_paths_compare_and_release_without_recursive_stack_use() {
    let reads = vec![Some((1, None)); 50_000];
    let mut first = capture_reads::CaptureInterner::default();
    let mut second = capture_reads::CaptureInterner::default();
    let left = first.capture(&reads);
    let right = second.capture(&reads);
    assert_eq!(left, right);
    drop(first);
    drop(second);
    drop(left);
    drop(right);
}

#[test]
fn inert_rhs_classification_stops_at_nested_assignment_boundaries() {
    let source = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-containers/global-rhs-stress.mts")).unwrap();
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
    #[cfg(feature = "test-instrumentation")]
    {
        assert_eq!(facts.value_flow.early_assignment_catalogs, 40);
        assert_eq!(facts.value_flow.reference_rhs_visits, 42);
        assert_eq!(facts.value_flow.reference_phase_indexes, 40);
        assert_eq!(facts.value_flow.reference_phase_nodes, 80);
        assert_eq!(facts.value_flow.reference_phase_path_tokens, 40);
        assert_eq!(facts.value_flow.reference_phase_pending_tokens, 0);
        assert_eq!(facts.value_flow.reference_phase_effects, 0);
        assert_eq!(facts.value_flow.reference_phase_queries, 0);
    }
    #[cfg(not(feature = "test-instrumentation"))]
    let _ = facts;
}
