use super::*;

#[test]
fn phase_writes_reuse_lexical_ownership_from_the_normal_visitor() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture/src/destructuring-scopes.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program, &source, false, false, true,
    );
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    for (receiver, marker, invalidated) in [
        ("api", "scopeApi(api)", false),
        ("block", "scopeBlock(block)", false),
        ("computedScope", "scopeComputed(computedScope)", false),
        ("updatedScope", "scopeUpdated(updatedScope)", false),
        ("matchingScope", "scopeMatching(matchingScope)", true),
    ] {
        let offset = source.find(marker).unwrap() as u32;
        assert_eq!(histories[receiver].assignment_invalidated(&["get"], offset, None, false),
            invalidated, "{receiver} at {marker}");
        assert!(histories[receiver].assignment_invalidated(&["get"], source.len() as u32, None, false));
    }
}
