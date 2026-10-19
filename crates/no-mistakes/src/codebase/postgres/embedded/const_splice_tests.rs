use super::{
    extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlOptions,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn extract(name: &str) -> super::EmbeddedSqlFileFacts {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture");
    extract_embedded_sql_from_source(
        &fixture(name),
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
}

fn read_calls(name: &str) -> Vec<EmbeddedSqlCall> {
    extract(name)
        .calls
        .into_iter()
        .filter(|call| call.callee == "read")
        .collect()
}

fn assert_inlined(name: &str, kind: EmbeddedSqlKind) {
    let calls = read_calls(name);
    assert_eq!(calls.len(), 1, "{name}");
    assert_eq!(calls[0].kind, kind, "{name}");
    assert!(!calls[0].is_unanalyzable(), "{name}");
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("SELECT id FROM orders WHERE account_id = $1"),
        "{name}"
    );
}

fn assert_unknown(name: &str) {
    let calls = read_calls(name);
    assert!(!calls.is_empty(), "{name}");
    for call in &calls {
        assert!(call.is_unanalyzable(), "{name}: {call:?}");
        assert_ne!(
            call.sql_text.as_deref(),
            Some("SELECT id FROM users WHERE account_id = $1"),
            "{name}"
        );
        assert_ne!(
            call.sql_text.as_deref(),
            Some("SELECT id FROM orders WHERE account_id = $1"),
            "{name}"
        );
    }
}

#[test]
fn local_const_is_inlined_into_the_template() {
    assert_inlined("const-splice-local.ts", EmbeddedSqlKind::Inline);
}

#[test]
fn as_const_scalar_is_inlined_into_the_template() {
    assert_inlined("const-splice-as-const.ts", EmbeddedSqlKind::Inline);
}

#[test]
fn empty_template_const_is_inlined_into_the_template() {
    assert_inlined("const-splice-empty-template.ts", EmbeddedSqlKind::Inline);
}

#[test]
fn unwritten_object_member_is_inlined_into_the_template() {
    let calls = read_calls("const-splice-member-template.ts");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("SELECT id FROM users WHERE account_id = $1")
    );
}

#[test]
fn unwritten_object_member_appended_to_a_builder_is_composed() {
    let calls = read_calls("const-splice-member-append.ts");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("SELECT id FROM users WHERE account_id = $1")
    );
}

#[test]
fn member_write_before_or_after_the_read_stays_unknown() {
    assert_unknown("const-splice-member-write.ts");
}

#[test]
fn escaped_object_member_stays_unknown() {
    assert_unknown("const-splice-member-escape.ts");
}

#[test]
fn let_binding_stays_unknown() {
    assert_unknown("const-splice-let.ts");
}

#[test]
fn spread_object_member_stays_unknown() {
    assert_unknown("const-splice-spread.ts");
}

#[test]
fn computed_member_write_stays_unknown() {
    assert_unknown("const-splice-computed-write.ts");
}
