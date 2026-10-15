use super::*;
use oxc_parser::Parser;

#[test]
fn synthetic_default_owner_is_distinct_from_nested_runtime_callables() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/inline-member-precision/fixture/src/default-snapshot-nested.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let synthetic = CallableId(source.find("export default").unwrap() as u32);
    let arrows = facts
        .value_flow
        .calls
        .iter()
        .filter_map(|call| match call.callee {
            ValueExpression::Callable(id) => Some((call, id)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(arrows.len(), 2);
    let (outer, outer_id) = arrows
        .iter()
        .find(|(call, _)| call.context == Some(synthetic))
        .unwrap();
    assert!(outer.once);
    assert!(outer_id.0 > synthetic.0);
    let (inner, inner_id) = arrows
        .iter()
        .find(|(call, _)| call.context == Some(*outer_id))
        .unwrap();
    assert!(!inner.once);
    assert!(inner_id.0 > outer_id.0);
}

#[test]
fn computed_callable_guards_restore_module_frequency_before_the_next_key() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/inline-member-precision/fixture/src/default-computed-restored.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let frequency = facts
        .value_flow
        .calls
        .iter()
        .filter(|call| matches!(call.callee, ValueExpression::Callable(_)))
        .map(|call| call.once)
        .collect::<Vec<_>>();
    assert_eq!(frequency, [false, true, false, true, false, true]);
}
