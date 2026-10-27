use super::*;

// Output cases moved to cli_postgres_snapshot_generated_column_writes.

#[test]
fn unresolved_format_relations_remain_broad_facts_but_not_live_events() {
    let root = unit_fixture("format-identities");
    let sql = std::fs::read_to_string(root.join("schema.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_migration_facts(&sql);
    assert!(facts.tables.iter().any(|table| {
        table.table_name == "dynamic_identifier"
            && table
                .columns
                .iter()
                .any(|column| column.name == "formatted_create")
    }));
    assert!(facts.add_columns.iter().any(|column| {
        column.table_name == "dynamic_value" && column.column_name == "formatted_alter"
    }));
    assert!(facts
        .dropped_tables
        .iter()
        .any(|table| table.name == "dynamic_identifier"));
    assert!(facts
        .dropped_tables
        .iter()
        .any(|table| table.name == "dynamic_value"));
    assert!(facts.table_events.iter().all(|event| match event {
        crate::codebase::postgres::SqlTableSchemaEvent::Drop { .. } => false,
        crate::codebase::postgres::SqlTableSchemaEvent::AddColumn { table, .. } => {
            table != "dynamic_value"
        }
        _ => true,
    }));
    assert_eq!(
        facts
            .table_events
            .iter()
            .filter(|event| matches!(
                event,
                crate::codebase::postgres::SqlTableSchemaEvent::Create { table, .. }
                    if table == "dynamic_identifier"
            ))
            .count(),
        1
    );
}

#[test]
fn conditional_variable_assignments_do_not_change_definite_live_history() {
    let root = unit_fixture("conditional-assignment");
    let sql = std::fs::read_to_string(root.join("schema.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_migration_facts(&sql);
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "dormant_fact"));
    assert!(facts
        .dropped_tables
        .iter()
        .any(|drop| drop.name == "orders"));
    assert!(facts.table_events.iter().all(|event| !matches!(
        event,
        crate::codebase::postgres::SqlTableSchemaEvent::Drop { .. }
    )));
}
