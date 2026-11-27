use super::*;

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/identifier-provenance-demand.mts",
    )
}

#[test]
fn ordinary_value_flow_keeps_scalar_assignment_indexes_cold() {
    let path = fixture();
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    for identifier_provenance in [false, true] {
        let mut collector = ImportCollector {
            collect_value_flow: true,
            collect_identifier_provenance: identifier_provenance,
            local_stack: vec![fx_set()],
            type_local_stack: vec![fx_set()],
            lexical_scope_ids: vec![0],
            next_lexical_scope_id: 1,
            ..Default::default()
        };
        collector.lexical_scope_parents.insert(0, None);
        crate::extract::predeclare_program_value_bindings(&mut collector, &parsed.program);
        collector.visit_program(&parsed.program);
        assert_eq!(
            collector
                .identifier_initializers
                .values()
                .map(|names| names.len())
                .sum::<usize>(),
            if identifier_provenance { 2 } else { 0 },
        );
        assert_eq!(
            collector
                .value_flow
                .identifier_writes
                .iter()
                .filter(|write| write.assigned.is_some())
                .count(),
            usize::from(identifier_provenance),
        );
        assert!(!collector.value_flow.bindings.is_empty());
    }
}

#[test]
fn direct_value_flow_extraction_retains_scalar_assignment_compatibility() {
    let path = fixture();
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
    let facts = crate::extract::extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert_eq!(
        facts
            .value_flow
            .identifier_writes
            .iter()
            .filter(|write| write.assigned.is_some())
            .count(),
        1
    );
}
