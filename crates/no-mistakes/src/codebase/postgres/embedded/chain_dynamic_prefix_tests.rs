use super::{
    extract_embedded_sql_from_source, EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlOptions,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn extract(name: &str) -> EmbeddedSqlFileFacts {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture");
    extract_embedded_sql_from_source(&fixture(name), &source, &EmbeddedSqlOptions::default())
}

#[test]
fn opaque_chain_append_recovers_a_same_file_helper_prefix() {
    let facts = extract("composed-chain-append-helper-spread.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics")
    );
}

#[test]
fn opaque_chain_append_rejects_a_shadowed_helper_prefix() {
    let facts = extract("composed-chain-append-shadowed-helper-spread.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[0].sql_text, None);
}

#[test]
fn non_append_member_call_does_not_recover_a_dynamic_prefix() {
    let facts = extract("composed-chain-non-append.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[0].sql_text, None);
}
