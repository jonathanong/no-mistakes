use super::*;

#[test]
fn import_exclusions_preserve_syntactic_calls_and_merge_nested_guards() {
    let source = include_str!("../../../../../../test-cases/codebase-analysis/graph-import-main/fixture/main.ts");
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(Path::new("main.ts"), &allocator, source, SourceType::ts());
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let inactive = |offset| facts.import_inactive_call_ranges.iter()
        .any(|(start, end)| *start <= offset && offset < *end);
    for callee in ["plain", "strict", "conjunction", "command", "namedCallback"] {
        assert!(facts.function_calls.iter().any(|call|
            call.caller_id.is_none() && call.callee == callee && inactive(call.offset)), "{callee}");
    }
    for callee in ["alternate", "negated", "after"] {
        assert!(facts.function_calls.iter().any(|call|
            call.callee == callee && !inactive(call.offset)), "{callee}");
    }
    assert!(facts.function_calls.iter().any(|call| call.is_callback
        && call.caller_id.is_none() && call.callee.starts_with("<anonymous:") && inactive(call.offset)));
    assert!(facts.import_inactive_call_ranges.windows(2).all(|pair| pair[0].1 < pair[1].0));
    assert!(facts.function_calls.iter().any(|call| call.callee == "commandEffect" && call.caller_id.is_some()));
}
