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
fn fluent_append_chain_used_as_initializer_is_composed() {
    let facts = extract("composed-chain-init.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1 AND active")
    );
}

#[test]
fn append_of_a_call_to_a_same_file_builder_is_composed() {
    let facts = extract("composed-append-call-arg.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE post_id")
    );
}

#[test]
fn call_to_a_same_file_function_returning_a_chain_is_composed() {
    let facts = extract("composed-chain-function-call.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}

#[test]
fn parameter_used_outside_a_placeholder_position_fails_closed() {
    let facts = extract("composed-append-call-arg-param.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn call_to_an_external_function_fails_closed() {
    let facts = extract("composed-append-call-arg-external.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn chain_append_of_a_static_template_is_composed() {
    let facts = extract("composed-chain-append-template.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}

#[test]
fn chain_append_of_a_trusted_tagged_template_is_composed() {
    let facts = extract("composed-chain-append-tagged-trusted.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics AND active")
    );
}

#[test]
fn chain_append_of_an_untrusted_interpolating_tag_fails_closed() {
    let facts = extract("composed-chain-append-tagged-untrusted.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn chain_append_of_a_binary_composition_is_composed() {
    let facts = extract("composed-chain-append-binary.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE active")
    );
}

#[test]
fn chain_append_of_a_spread_argument_fails_closed() {
    let facts = extract("composed-chain-append-spread.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn call_to_a_self_recursive_same_file_function_fails_closed() {
    let facts = extract("composed-chain-function-recursive.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_call_shadowed_by_its_own_parameter_fails_closed() {
    let facts = extract("composed-chain-shadowed-helper-param.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn call_shadowed_by_an_enclosing_parameter_fails_closed() {
    let facts = extract("composed-chain-shadowed-outer-param.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn helper_chain_beyond_the_depth_budget_fails_closed() {
    let facts = extract("composed-chain-function-deep.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn fluent_append_of_two_trusted_placeholders_renumbers_sequentially() {
    let facts = extract("composed-chain-append-tagged-placeholders.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1 AND status = sql_placeholder_2")
    );
}

#[test]
fn statement_level_append_of_two_trusted_placeholders_renumbers_sequentially() {
    let facts = extract("composed-append-tagged-placeholders.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1 AND status = sql_placeholder_2")
    );
}

#[test]
fn async_helper_is_rejected() {
    let facts = extract("composed-chain-function-async.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn generator_helper_is_rejected() {
    let facts = extract("composed-chain-function-generator.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn const_bound_function_expression_helper_is_composed() {
    let facts = extract("composed-chain-const-function-expression.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}

#[test]
fn const_bound_block_bodied_arrow_helper_is_composed() {
    let facts = extract("composed-chain-const-arrow.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}

#[test]
fn exported_const_bound_arrow_helper_is_composed() {
    let facts = extract("composed-chain-const-arrow-exported.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}

#[test]
fn expression_bodied_arrow_helper_is_not_collected() {
    let facts = extract("composed-chain-const-arrow-expression-body.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn async_const_bound_arrow_helper_is_rejected() {
    let facts = extract("composed-chain-const-arrow-async.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn exported_non_function_declaration_is_ignored_by_helper_collection() {
    let facts = extract("composed-chain-export-class-ignored.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
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
