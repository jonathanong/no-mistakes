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
