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

#[test]
fn call_repetition_belongs_to_execution_not_callable_creation() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture/src/created-callable-facts.mts",
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
    for (callee, repeated) in [
        ("parameterCall", false),
        ("bodyCall", false),
        ("nestedLoopCall", true),
        ("nestedBodyCall", false),
        ("bodyAfterCall", false),
        ("tagBody", false),
        ("constructBody", false),
        ("arrowParameterCall", false),
        ("arrowBodyCall", false),
        ("objectKeyCall", true),
        // Unknown keys have no entered callable owner: retain the legacy guard.
        ("objectBodyCall", true),
        ("objectNamedBodyCall", false),
        ("decoratorCall", true),
        ("classKeyCall", true),
        ("classBodyCall", true),
        ("classNamedBodyCall", false),
        ("overloadBodyCall", false),
        ("classOverloadBodyCall", false),
        ("defaultBodyCall", false),
        ("defaultLoopCall", true),
        ("outsideKeyCall", false),
        // The named-class specialized path has no extra legacy depth here;
        // preserving zero does not prove that an unqualified body executes once.
        ("outsideUnknownBodyCall", false),
        ("fieldCall", true),
        ("fieldArrowCall", false),
        ("staticCall", true),
        ("staticBlockCall", true),
        ("loopAfterCall", true),
        // Named-class instance fields use their class context rather than an
        // extra legacy repeat-depth increment; preserve that authored metadata.
        ("outsideFieldCall", false),
        ("outsideArrowBodyCall", false),
        ("outsideStaticCall", false),
        ("outsideStaticBlockCall", false),
        ("moduleAfterCall", false),
    ] {
        let calls: Vec<_> = facts
            .value_flow
            .calls
            .iter()
            .filter(|call| call.source_callee == callee)
            .collect();
        assert!(!calls.is_empty(), "saved execution marker {callee}");
        assert!(
            calls.iter().all(|call| call.repeated == repeated),
            "{callee}: {calls:?}"
        );
    }
}
