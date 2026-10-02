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
