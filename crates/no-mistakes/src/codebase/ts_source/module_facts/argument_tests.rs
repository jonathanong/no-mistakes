use super::*;
use arguments::ArgumentValue;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn read(name: &str, requested: bool) -> TypeScriptModuleFacts {
    let path =
        crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates").join(name);
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    test_helpers::extract_with_arguments(&parsed.program, requested)
}

#[test]
fn exported_aggregates_escape_but_primitive_alias_remains_known() {
    let facts = read("escape.mjs", true);
    let calls = facts.call_arguments.unwrap();
    assert_eq!(calls.len(), 1);
    let values = &calls.values().next().unwrap().values;
    assert_eq!(values[0], ArgumentValue::Scalar(serde_json::json!("node")));
    assert_eq!(values[1], ArgumentValue::Unknown);
    assert_eq!(values[2], ArgumentValue::Unknown);
}

#[test]
fn destructured_symbols_never_project_the_entire_initializer() {
    let calls = read("destructuring.mjs", true).call_arguments.unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls.values().next().unwrap().values,
        vec![ArgumentValue::Unknown; 2]
    );
}

#[test]
fn popular_alias_budget_is_explicit_unknown_and_demand_is_optional() {
    let facts = read("popular-alias.mjs", true);
    let calls = facts.call_arguments.unwrap();
    assert_eq!(calls.len(), 300);
    assert!(calls
        .values()
        .all(|v| v.values == vec![ArgumentValue::Unknown]));
    assert!(read("popular-alias.mjs", false).call_arguments.is_none());
}

#[test]
fn aggregate_lifetime_rejects_repeated_carriers_but_accepts_fresh_local() {
    let calls = read("repeated.mjs", true).call_arguments.unwrap();
    let mut values: Vec<_> = calls.iter().collect();
    values.sort_by_key(|(offset, _)| **offset);
    assert_eq!(values.len(), 4);
    assert_eq!(values[0].1.values, vec![ArgumentValue::Unknown]);
    assert_eq!(values[1].1.values, vec![ArgumentValue::Unknown]);
    assert!(matches!(values[2].1.values[0], ArgumentValue::Array(_)));
    assert_eq!(values[3].1.values, vec![ArgumentValue::Unknown]);
}

#[test]
fn export_alias_computed_key_and_argument_membership_bounds_fail_closed() {
    let calls = read("export-alias.mjs", true).call_arguments.unwrap();
    let mut values: Vec<_> = calls.iter().collect();
    values.sort_by_key(|(offset, _)| **offset);
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].1.values, vec![ArgumentValue::Unknown]);
    assert_eq!(
        values[1].1.values,
        vec![ArgumentValue::Scalar(serde_json::json!("node"))]
    );
    let calls = read("computed-keys.mjs", true).call_arguments.unwrap();
    let mut values: Vec<_> = calls.iter().collect();
    values.sort_by_key(|(offset, _)| **offset);
    assert_eq!(values.len(), 4);
    for index in [0, 1, 3] {
        assert_eq!(values[index].1.values, vec![ArgumentValue::Unknown]);
    }
    assert!(matches!(values[2].1.values[0], ArgumentValue::Object(_)));
    let calls = read("membership-bound.mjs", true).call_arguments.unwrap();
    let values = calls.values().next().unwrap();
    assert_eq!(values.values.len(), 32);
    assert_eq!(values.values[0], ArgumentValue::Unknown);
}

#[test]
fn invalid_semantic_identity_does_not_publish_literal_argument_proofs() {
    let facts = read("semantic-invalid.mjs", true);
    assert!(!facts.diagnostics.is_empty());
    assert!(facts.call_arguments.is_none());
}
