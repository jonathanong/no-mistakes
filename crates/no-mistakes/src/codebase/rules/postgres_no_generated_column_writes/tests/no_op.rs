use super::*;

// Output cases moved to cli_postgres_snapshot_generated_column_writes.

#[test]
fn collected_empty_event_stream_does_not_fall_back_to_dormant_schema_metadata() {
    let root = unit_fixture("table-event-forms");
    let source = std::fs::read_to_string(root.join("dormant-only.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_migration_facts(&source);
    assert!(facts.table_events_collected);
    assert!(facts.table_events.is_empty());
    assert!(
        !facts.tables.is_empty(),
        "Policy metadata must still include dormant DDL"
    );
}
