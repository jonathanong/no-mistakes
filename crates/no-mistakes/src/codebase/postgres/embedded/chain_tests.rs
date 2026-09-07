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
