use super::super::super::tests::fixture;

#[test]
fn manually_constructed_schema_facts_keep_legacy_table_compatibility() {
    let root = fixture("review-followups");
    let text = std::fs::read_to_string(root.join("sql/schema.sql")).unwrap();
    let mut file = crate::codebase::postgres::extract_migration_facts(&text);
    file.table_events.clear();
    file.table_events_collected = false;
    let live = super::live_columns(&[&file]);
    assert!(live.contains_key("orders"));
}

#[test]
fn conditional_alter_does_not_add_columns_to_absent_tables() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/conditional-alter",
    );
    let text = std::fs::read_to_string(root.join("schema.sql")).unwrap();
    let file = crate::codebase::postgres::extract_migration_facts(&text);
    let live = super::live_columns(&[&file]);
    assert!(!live.contains_key("absent"));
    assert!(live["orders"]["computed"].is_generated);
    assert!(!live["orders"]["updated_at"].is_generated);
}
