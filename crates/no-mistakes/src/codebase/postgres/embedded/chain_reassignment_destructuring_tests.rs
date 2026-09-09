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
fn reassigned_via_var_declared_for_of_loop_target_is_rejected_after_the_loop() {
    let facts = extract("composed-chain-function-reassigned-via-var-for-of-declared.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_through_array_destructuring_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-array-destructured.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_through_array_rest_destructuring_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-array-rest-destructured.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_through_object_rest_destructuring_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-object-rest-destructured.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_through_renamed_object_property_destructuring_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-object-renamed-destructured.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn reassigned_through_a_defaulted_destructuring_target_is_rejected() {
    let facts = extract("composed-chain-function-reassigned-default-destructured.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn a_member_expression_assignment_does_not_count_as_reassignment() {
    let facts = extract("composed-chain-function-member-assignment-does-not-reassign.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics")
    );
}
