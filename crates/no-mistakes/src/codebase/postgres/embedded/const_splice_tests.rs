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
    let calls = read_calls("const-splice-member-write.ts");
    assert_eq!(calls.len(), 2, "{calls:?}");
    for call in &calls {
        assert!(call.is_unanalyzable(), "{call:?}");
        let sql = call.sql_text.as_deref().unwrap_or("");
        assert!(
            sql.contains("sql_placeholder_1"),
            "placeholder text preserved: {sql}"
        );
        assert!(!sql.contains("users"), "{sql}");
        assert!(!sql.contains('\0'), "{sql}");
    }
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

#[test]
fn empty_template_object_member_is_inlined() {
    let calls = read_calls("const-splice-member-empty-template.ts");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("SELECT id FROM users WHERE account_id = $1")
    );
}

#[test]
fn string_and_empty_template_pieces_are_inlined() {
    assert_inlined("const-splice-string-piece.ts", EmbeddedSqlKind::Inline);
    assert_inlined("const-splice-empty-piece.ts", EmbeddedSqlKind::Inline);
}

#[test]
fn nested_template_piece_is_inlined() {
    assert_inlined("const-splice-nested-template.ts", EmbeddedSqlKind::Inline);
}

#[test]
fn missing_object_member_stays_unknown() {
    assert_unknown("const-splice-missing-member.ts");
}

#[test]
fn non_string_const_template_piece_stays_unknown() {
    assert_unknown("const-splice-non-string-piece.ts");
}

#[test]
fn nested_sql_const_splice_keeps_calls_and_aligned_origins() {
    let calls = read_calls("const-splice-origins-nested-sql.ts");
    assert_eq!(calls.len(), 2, "{calls:?}");
    let mut sqls = Vec::new();
    for call in &calls {
        assert!(
            !call.is_unanalyzable(),
            "call dropped or left unanalyzable: {call:?}"
        );
        assert!(
            !call.variants.is_empty(),
            "analyzable call has no variant: {call:?}"
        );
        for variant in &call.variants {
            assert_eq!(
                variant.sql_source_offsets.len(),
                variant.sql_text.len(),
                "{}",
                variant.sql_text
            );
            sqls.push(variant.sql_text.clone());
        }
    }
    sqls.sort();
    assert_eq!(
        sqls,
        vec![
            "SELECT id, name, created_at FROM t WHERE id = sql_placeholder_1".to_string(),
            "SELECT id, name, created_at, updated_at, status, owner_id FROM t WHERE id = sql_placeholder_1 AND other = sql_placeholder_2".to_string(),
        ]
    );
}

#[path = "const_splice_review_tests.rs"]
mod review;
