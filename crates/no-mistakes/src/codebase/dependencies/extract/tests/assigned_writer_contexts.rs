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

#[test]
fn destructuring_exact_histories_separate_lhs_phases_from_rhs_reads() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture/src/destructuring.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    for (receiver, member, marker, expected) in [
        ("primary", "get", "after(primary)", true),
        ("replica", "get", "before(replica)", false),
        ("scoped", "get", "afterRhs(scoped)", true),
        ("scoped", "get", "rhs(scoped)", false),
        ("session", "info", "untouched(session)", false),
        ("nested", "get", "const saved = nested.get", false),
        ("local", "get", "copiedAfter(local.get)", true),
        ("standalone", "get", "nestedAfter(standalone)", true),
        ("standalone", "get", "enclosingDefault(standalone)", false),
        ("reversed", "get", "outerAfter(reversed)", true),
        ("arrayClient", "get", "arrayAfter(arrayClient)", true),
        ("computed", "get", "computedAfter(computed)", true),
        ("dynamic", "get", "dynamicAfter(dynamic)", true),
        ("ownDefault", "get", "ownBefore(ownDefault)", false),
        ("overlap", "get", "overlappingAfter(overlap)", true),
    ] {
        let offset = source.find(marker).unwrap() as u32;
        assert_eq!(
            histories[receiver].assignment_invalidated(&[member], offset, None, false),
            expected,
            "{receiver}.{member} at {marker}"
        );
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn destructuring_ranges_are_recorded_once_and_shared_across_aliases() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/destructuring-shared-history.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.errors.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert_eq!(facts.value_flow.early_assignment_range_records, 128);
    assert_eq!(facts.value_flow.early_assignment_stack_growths, 128);
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    let history = &histories["primary"];
    assert_eq!(
        history
            .assigned
            .as_ref()
            .unwrap()
            .early
            .as_ref()
            .unwrap()
            .range_count(),
        128
    );
    for index in 0..128 {
        assert!(std::sync::Arc::ptr_eq(
            history,
            &histories[&format!("alias{index}")]
        ));
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn non_receiver_destructuring_keeps_early_range_storage_cold() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/destructuring-no-receiver.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.errors.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert_eq!(facts.value_flow.early_assignment_stack_growths, 0);
    assert_eq!(facts.value_flow.early_assignment_range_records, 0);
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["primary"];
    assert!(history.assigned.as_ref().unwrap().early.is_none());
}
