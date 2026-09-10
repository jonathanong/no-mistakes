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

#[test]
fn same_file_multistatement_helper_passed_to_write_is_composed() {
    let facts = extract("composed-helper-multistatement.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(facts.calls[0].callee, "write");
    let sql = facts.calls[0].sql_text.as_deref().unwrap();
    assert!(sql.starts_with("UPDATE activitypub_inbox_deliveries"));
    assert!(sql.contains("AND created_at < sql_placeholder_"));
}

#[test]
fn same_file_helper_building_complete_with_update_is_composed() {
    let facts = extract("composed-helper-with-update.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    let sql = facts.calls[0].sql_text.as_deref().unwrap();
    assert!(sql.contains("WITH stale AS"));
    assert!(sql.contains("UPDATE items SET gone = true"));
}

#[test]
fn same_file_helper_building_complete_with_insert_is_composed() {
    let facts = extract("composed-helper-with-insert.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    let sql = facts.calls[0].sql_text.as_deref().unwrap();
    assert!(sql.contains("WITH input AS"));
    assert!(sql.contains("INSERT INTO items (id)"));
    assert!(sql.contains("ON CONFLICT (id) DO NOTHING"));
}

#[test]
fn incomplete_with_prefix_plus_opaque_append_fails_closed() {
    let facts = extract("composed-append-incomplete-with.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[0].sql_text, None);
}
