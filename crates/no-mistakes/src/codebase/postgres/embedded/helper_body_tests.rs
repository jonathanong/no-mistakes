use super::{extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions};
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

#[test]
fn typed_parameter_builder_recovers_only_straight_line_static_appends() {
    let facts = extract("composed-helper-parameter.ts");
    assert_eq!(facts.calls.len(), 18);
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("/* good */ SELECT id FROM items WHERE id = sql_placeholder_1")
    );
    assert_eq!(facts.calls[1].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[1].sql_text.as_deref(),
        Some("/* offset */ SELECT id FROM items OFFSET 2")
    );
    for call in &facts.calls[2..12] {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic);
    }
    assert_eq!(
        facts.calls[12].sql_text.as_deref(),
        Some("/* timed */ SELECT id FROM items WHERE id = sql_placeholder_1")
    );
    assert_eq!(
        facts.calls[13].sql_text.as_deref(),
        Some("/* timed */ SELECT OFFSET 1 id FROM items WHERE id = sql_placeholder_1")
    );
    assert_eq!(facts.calls[14].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[15].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[16].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        facts.calls[17].sql_text.as_deref(),
        Some("/* return */ SELECT id FROM items")
    );
}
