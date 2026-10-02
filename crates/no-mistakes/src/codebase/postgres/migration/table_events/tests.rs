use super::*;

fn fixture(file: &str) -> String {
    std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/table-event-forms").join(file)).unwrap()
}

#[test]
fn ordered_events_keep_quoted_names_function_arguments_and_table_primary_keys() {
    let facts = super::super::extract_migration_facts(&fixture("schema.sql"));
    assert_eq!(facts.table_events.len(), 6, "{facts:?}");
    let SqlTableSchemaEvent::Create {
        table,
        unqualified_table,
        columns,
        ..
    } = &facts.table_events[0]
    else {
        panic!("create")
    };
    assert_eq!(table, "public.quoted.table");
    assert_eq!(unqualified_table, "quoted.table");
    assert!(columns[0].is_primary_key);
    assert_eq!(columns[2].generated_function_arg_columns, ["Input", "id"]);
    assert!(facts
        .table_events
        .windows(2)
        .all(|pair| pair[0].source_order() <= pair[1].source_order()));
}

#[test]
fn malformed_ddl_positions_are_tolerant_and_do_not_invent_names() {
    let mut positions = Positions::new(&fixture("invalid.sql"));
    assert_eq!(positions.take("CREATE", "absent"), [usize::MAX]);
    assert_eq!(positions.take("ALTER", "foo"), [1]);
    assert_eq!(positions.take("DROP", "not an identifier"), [usize::MAX]);
}

#[test]
fn live_events_exclude_dormant_and_conditional_ddl_but_preserve_policy_facts() {
    let facts = super::super::extract_migration_facts(&fixture("execution.sql"));
    let events: Vec<_> = facts
        .table_events
        .iter()
        .map(|event| match event {
            SqlTableSchemaEvent::Create { table, .. } => format!("create:{table}"),
            SqlTableSchemaEvent::AddColumn { column, .. } => format!("add:{}", column.name),
            SqlTableSchemaEvent::Drop { table, .. } => format!("drop:{table}"),
        })
        .collect();
    assert_eq!(
        events,
        [
            "create:orders",
            "create:immediate",
            "create:immediate_exec",
            "create:immediate_quoted",
            "add:actual_column"
        ]
    );
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "dormant_generated"));
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "conditional_created"));
    assert!(facts
        .add_columns
        .iter()
        .any(|column| column.column_name == "dormant_column"));
}

#[test]
fn transaction_projection_keeps_only_committed_table_events() {
    let facts = super::super::extract_migration_facts(&fixture("transactions.sql"));
    let events: Vec<_> = facts
        .table_events
        .iter()
        .map(|event| match event {
            SqlTableSchemaEvent::Create {
                relation_key,
                temporary,
                ..
            } => format!("create:{relation_key}:temporary={temporary}"),
            SqlTableSchemaEvent::AddColumn { relation_key, .. } => {
                format!("add:{relation_key}")
            }
            SqlTableSchemaEvent::Drop { relation_key, .. } => format!("drop:{relation_key}"),
        })
        .collect();
    assert_eq!(
        events,
        [
            "create:orders:temporary=false",
            "create:committed_start:temporary=false",
            "create:committed_end:temporary=false",
            "create:rollback_prepared_is_not_rollback:temporary=false",
            "create:savepoint_kept:temporary=false",
            "create:after_rollback_to:temporary=false",
            "create:after_rollback_without_keyword:temporary=false",
            "create:temporary_kept:temporary=true",
            "create:plpgsql_block_kept:temporary=false",
            "create:committed_before_chain:temporary=false",
            "create:committed_after_rollback_chain:temporary=false",
            "create:committed_before_work_chain:temporary=false",
        ]
    );
    // Projection is deliberately limited to live catalog events. Broad policy
    // facts still describe source text even when its transaction rolls back.
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "rolled_back_create"));
    assert!(facts
        .add_columns
        .iter()
        .any(|column| column.column_name == "rolled_back_column"));
    assert!(facts
        .dropped_tables
        .iter()
        .any(|drop| drop.name == "orders"));
}
