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
fn a_let_declared_chain_call_result_is_not_trusted_as_composed() {
    let facts = extract("composed-chain-function-let-declared-is-not-trusted.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn binary_composition_with_an_untrusted_interpolating_tag_operand_is_rejected() {
    let facts = extract("composed-binary-untrusted-interpolating-tag-operand.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn a_tag_spelled_as_a_static_member_other_than_raw_is_untrusted() {
    let facts = extract("composed-tag-static-member-not-raw-is-untrusted.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics")
    );
}

#[test]
fn a_raw_tag_on_a_nested_member_object_is_untrusted() {
    let facts = extract("composed-tag-raw-on-nested-member-object-is-untrusted.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics")
    );
}
