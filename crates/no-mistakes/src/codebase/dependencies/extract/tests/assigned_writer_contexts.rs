use super::*;
use oxc_parser::Parser;

#[test]
fn exact_writes_keep_mixed_and_single_writer_contexts_distinct() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/assigned-writer-contexts.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let writer = |name: &str| {
        facts
            .callable_scope_ids
            .iter()
            .find(|(_, scope)| scope == name)
            .unwrap()
            .0
    };
    let first = writer("writeFirst");
    let second = writer("writeSecond");
    let receivers = facts
        .value_flow
        .receiver_mutations
        .as_ref()
        .unwrap()
        .get(&0)
        .unwrap();
    let exact = receivers["primary"]
        .assigned
        .as_ref()
        .unwrap()
        .members
        .get("get")
        .unwrap();
    assert!(
        std::mem::size_of_val(exact) <= 16,
        "exact write summaries must not retain a heap-backed context set per property"
    );
    // Each writer still has another possible writer, even before its own write.
    for context in [None, Some(first), Some(second)] {
        assert!(receivers["primary"].assignment_invalidated(&["get"], 0, context, false));
        assert!(receivers["primary"].assignment_invalidated(&["other"], 0, context, false));
    }
    assert!(!receivers["session"].assignment_invalidated(&["info"], 0, Some(first), false));
    assert!(receivers["session"].assignment_invalidated(&["info"], 0, Some(first), true));
    assert!(receivers["session"].assignment_invalidated(&["info"], 0, Some(second), false));
    assert!(receivers["session"].assignment_invalidated(&["info"], u32::MAX, Some(first), false));
}
