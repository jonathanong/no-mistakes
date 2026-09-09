use super::{extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions};
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

fn assert_dynamic(name: &str) {
    let facts = extract(name);
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic, "{name}");
}

#[test]
fn rest_array_parameter_shadows_a_same_named_helper() {
    assert_dynamic("composed-chain-shadowed-rest-array-param.ts");
}

#[test]
fn imported_string_is_not_the_intrinsic_string_raw_tag() {
    assert_dynamic("composed-chain-shadowed-imported-string.ts");
}

#[test]
fn classic_for_initializer_shadows_a_helper_inside_the_loop() {
    let facts = extract("composed-chain-shadowed-classic-for-init.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn classic_for_const_initializer_does_not_reassign_the_helper_after_the_loop() {
    let facts = extract("composed-chain-classic-for-init-does-not-reassign-after.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 1"));
}

#[test]
fn classic_for_var_initializer_remains_visible_after_the_loop() {
    let facts = extract("composed-chain-classic-for-var-init-is-visible-after.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 1"));
}

#[test]
fn function_declaration_named_sql_is_not_the_trusted_tag() {
    assert_dynamic("composed-chain-shadowed-function-declaration-sql.ts");
}

#[test]
fn class_named_string_is_not_the_intrinsic_string_raw_tag() {
    assert_dynamic("composed-chain-shadowed-class-string.ts");
}

#[test]
fn nested_class_named_string_is_not_the_intrinsic_string_raw_tag() {
    assert_dynamic("composed-chain-shadowed-nested-class-string.ts");
}

#[test]
fn callable_string_rebinding_is_not_exempted_as_a_helper_shape() {
    assert_dynamic("composed-chain-shadowed-callable-string-rebinding.ts");
}

#[test]
fn parameter_assignment_does_not_reassign_an_unrelated_top_level_helper() {
    let facts = extract("composed-chain-param-assignment-does-not-reassign-helper.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 1"));
}

#[test]
fn parameter_var_declaration_does_not_reassign_an_unrelated_top_level_helper() {
    let facts = extract("composed-chain-param-var-does-not-reassign-helper.js");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 1"));
}

#[test]
fn function_declaration_named_sql_returning_env_is_not_the_trusted_tag() {
    assert_dynamic("composed-chain-shadowed-function-declaration-env-sql.ts");
}

#[test]
fn append_does_not_renumber_user_authored_placeholder_substrings() {
    let facts = extract("composed-chain-append-literal-placeholder-name.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT sql_placeholder_1 AS sql_placeholder_1")
    );
}
