use super::substitute_target_columns;
use std::collections::BTreeMap;

fn projections() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("key".to_string(), "input.source_key".to_string()),
        ("other".to_string(), "input.other_key".to_string()),
    ])
}

#[test]
fn substitutes_nested_unary_cast_and_binary_expressions() {
    let projections = projections();

    assert!(substitute_target_columns("((+key)::text)", &projections).is_some());
    assert!(substitute_target_columns("key + 1", &projections).is_some());
}

#[test]
fn substitutes_distinctness_expressions_on_both_sides() {
    let projections = projections();

    assert!(substitute_target_columns("key IS DISTINCT FROM other", &projections).is_some());
    assert!(substitute_target_columns("key IS NOT DISTINCT FROM other", &projections).is_some());
}

#[test]
fn accepts_literal_and_function_expression_leaves() {
    let projections = projections();

    assert!(substitute_target_columns("DATE '2026-09-07'", &projections).is_some());
    assert!(substitute_target_columns("lower(key)", &projections).is_some());
}

#[test]
fn rejects_missing_projections_and_unsupported_expression_shapes() {
    let projections = projections();

    assert!(substitute_target_columns("missing", &projections).is_none());
    assert!(substitute_target_columns("key BETWEEN 1 AND 2", &projections).is_none());
    assert!(substitute_target_columns("count(*)", &projections).is_none());
}
