use super::*;

#[test]
#[cfg(feature = "test-instrumentation")]
fn single_version_receiver_writes_do_not_allocate_version_tables() {
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
    assert!(facts.value_flow.reference_successor_bookkeeping >= 128);
    assert_eq!(facts.value_flow.reference_version_table_capacity, 0);
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn distinct_receiver_versions_keep_separate_traversal_states() {
    let source = mutable_receiver_source("mutable-let-self-copy.mts");
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
    assert!(facts.value_flow.reference_version_table_capacity > 0);
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
}
