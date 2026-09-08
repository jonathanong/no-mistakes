use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn extract(name: &str) -> super::EmbeddedSqlFileFacts {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture");
    extract_embedded_sql_from_source(&fixture(name), &source, &EmbeddedSqlOptions::default())
}

#[test]
fn call_shadowed_by_a_destructured_outer_parameter_fails_closed() {
    let facts = extract("composed-chain-shadowed-outer-destructured-param.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_call_shadowed_by_its_own_destructured_parameter_fails_closed() {
    let facts = extract("composed-chain-shadowed-helper-destructured-param.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_call_shadowed_by_a_default_valued_array_element_past_a_rest_param_fails_closed() {
    let facts = extract("composed-chain-shadowed-array-rest-default-param.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_function_declaration_is_rejected() {
    let facts = extract("composed-chain-function-reassigned.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_via_var_initializer_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-via-var-initializer.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn top_level_binary_composition_of_two_trusted_placeholders_renumbers_sequentially() {
    let facts = extract("composed-append-binary-placeholders.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1 AND status = sql_placeholder_2")
    );
}

#[test]
fn same_file_function_binary_composition_of_two_trusted_placeholders_renumbers_sequentially() {
    let facts = extract("composed-chain-function-binary-placeholders.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1 AND status = sql_placeholder_2")
    );
}

#[test]
fn helper_return_tagged_by_a_parameter_named_sql_fails_closed() {
    let facts = extract("composed-chain-shadowed-sql-tag-param.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn local_tagged_by_a_parameter_named_sql_fails_closed() {
    let facts = extract("composed-chain-shadowed-sql-tag-local.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn nested_function_declaration_shadowing_a_top_level_helper_fails_closed() {
    let facts = extract("composed-chain-shadowed-nested-function-declaration.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_through_destructuring_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-destructured.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_tagged_by_a_captured_top_level_const_rebinding_sql_fails_closed() {
    let facts = extract("composed-chain-shadowed-captured-top-level-const.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn call_shadowed_by_a_catch_parameter_fails_closed() {
    let facts = extract("composed-chain-shadowed-catch-parameter.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn named_function_expression_self_binding_is_not_collected_under_its_const_name() {
    let facts = extract("composed-chain-shadowed-named-function-expression-self-binding.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_tagged_by_a_top_level_arrow_rebinding_sql_fails_closed() {
    let facts = extract("composed-chain-shadowed-top-level-arrow-rebinding-sql.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_return_tagged_by_a_parameter_named_sql_with_no_interpolation_fails_closed() {
    let facts = extract("composed-chain-shadowed-sql-tag-param-no-interpolation.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_via_for_of_loop_target_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-via-for-of.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn tag_use_of_a_named_function_expressions_own_self_binding_fails_closed() {
    let facts = extract("composed-chain-shadowed-named-function-expression-self-tag.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_tagged_by_an_imported_sql_binding_fails_closed() {
    let facts = extract("composed-chain-shadowed-imported-sql-tag.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}
