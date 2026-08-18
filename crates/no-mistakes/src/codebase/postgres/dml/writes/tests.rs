use super::{
    find_generated_column_writes, GeneratedColumnWrite, GeneratedTable, GeneratedTableColumns,
};

fn catalog() -> GeneratedTableColumns {
    let mut tables = GeneratedTableColumns::default();
    tables.insert_table(GeneratedTable {
        name: "items".to_string(),
        generated: ["created_at".to_string()].into_iter().collect(),
        column_order: Some(vec![
            "id".to_string(),
            "created_at".to_string(),
            "note".to_string(),
        ]),
    });
    tables
}

fn columns(sql: &str) -> Vec<String> {
    find_generated_column_writes(sql, &catalog())
        .into_iter()
        .map(|write| write.column)
        .collect()
}

#[test]
fn flags_each_supported_dml_shape() {
    assert_eq!(
        columns("UPDATE items SET created_at = now()"),
        ["created_at"]
    );
    assert_eq!(
        columns("INSERT INTO items (id, created_at) VALUES (1, now())"),
        ["created_at"]
    );
    assert_eq!(
        columns("INSERT INTO items VALUES (1, now(), 'n')"),
        ["created_at"]
    );
    assert_eq!(
        columns(
            "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET created_at = now()"
        ),
        ["created_at"]
    );
    assert_eq!(
        columns(
            "MERGE INTO items t USING s ON t.id = s.id \
             WHEN MATCHED THEN UPDATE SET created_at = now() \
             WHEN NOT MATCHED THEN INSERT (id, created_at) VALUES (s.id, now())"
        ),
        ["created_at"]
    );
}

#[test]
fn skips_source_column_writes_and_unrelated_tables() {
    assert!(columns("UPDATE items SET note = 'ok'").is_empty());
    assert!(columns("INSERT INTO items (id, note) VALUES (1, 'ok')").is_empty());
    assert!(columns("UPDATE other SET created_at = now()").is_empty());
}

#[test]
fn columnless_insert_requires_a_positional_hit() {
    assert!(columns("INSERT INTO items VALUES (1)").is_empty());
    assert_eq!(
        columns("INSERT INTO items SELECT 1, now(), 'n'"),
        ["created_at"]
    );
    assert_eq!(
        columns("INSERT INTO items SELECT * FROM staging"),
        ["created_at"]
    );
}

#[test]
fn extra_table_without_order_skips_positional_inserts() {
    let mut tables = GeneratedTableColumns::default();
    tables.insert_table(GeneratedTable {
        name: "votes".to_string(),
        generated: ["created_at".to_string()].into_iter().collect(),
        column_order: None,
    });
    assert!(
        find_generated_column_writes("INSERT INTO votes VALUES (1, now())", &tables).is_empty()
    );
    assert_eq!(
        find_generated_column_writes("UPDATE votes SET created_at = now()", &tables),
        [GeneratedColumnWrite {
            table: "votes".to_string(),
            column: "created_at".to_string(),
        }]
    );
}

#[test]
fn unparseable_sql_and_empty_catalog_are_silent() {
    assert!(find_generated_column_writes("UPDATE items SET", &catalog()).is_empty());
    assert!(find_generated_column_writes(
        "UPDATE items SET created_at = now()",
        &GeneratedTableColumns::default()
    )
    .is_empty());
}

#[test]
fn flags_tuple_assignments_merge_row_and_set_operations() {
    assert_eq!(
        columns("UPDATE items SET (created_at, note) = (now(), 'n')"),
        ["created_at"]
    );
    assert!(columns("MERGE INTO items t USING s ON true WHEN MATCHED THEN DELETE").is_empty());
    assert_eq!(
        columns("MERGE INTO items t USING s ON true WHEN NOT MATCHED THEN INSERT VALUES (s.id, now(), 'n')"),
        ["created_at"]
    );
    let items = catalog();
    let items = items.get("items").expect("items");
    assert_eq!(
        super::update::merge_insert_width(&sqlparser::ast::MergeInsertKind::Row, items),
        Some(3)
    );
    assert_eq!(
        columns("INSERT INTO items SELECT 1, now() UNION ALL SELECT 2, now()"),
        ["created_at"]
    );
    assert!(columns("INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING").is_empty());
    assert_eq!(
        columns("INSERT INTO items SELECT q.* FROM staging q"),
        ["created_at"]
    );
    assert!(columns("INSERT INTO items DEFAULT VALUES; CREATE TABLE extra (id int);").is_empty());
    assert!(columns(
        "UPDATE items SET note = 'x'; UPDATE mystery SET created_at = now(); MERGE INTO other t USING s ON true WHEN MATCHED THEN UPDATE SET created_at = now(); INSERT INTO elsewhere (created_at) VALUES (now());"
    )
    .is_empty());
    assert_eq!(
        columns("INSERT INTO items (SELECT 1, now(), 'n')"),
        ["created_at"]
    );
    assert!(columns(
        "UPDATE items SET note = 'x'; MERGE INTO (SELECT * FROM items) t USING s ON true WHEN MATCHED THEN UPDATE SET created_at = now();"
    )
    .is_empty());
}

#[test]
fn insert_into_query_target_and_derived_update_are_skipped() {
    use sqlparser::ast::{Statement, TableObject};

    let Statement::Insert(mut insert) =
        crate::codebase::postgres::parse_postgres_sql("INSERT INTO items VALUES (1, now(), 'n')")
            .unwrap()
            .pop()
            .expect("insert")
    else {
        panic!("expected insert");
    };
    let Statement::Query(query) = crate::codebase::postgres::parse_postgres_sql("SELECT 1")
        .unwrap()
        .pop()
        .expect("query")
    else {
        panic!("expected query");
    };
    insert.table = TableObject::TableQuery(query);
    let mut writes = Vec::new();
    super::insert::collect_insert_writes(&insert, &catalog(), &mut writes);
    assert!(writes.is_empty(), "{writes:?}");
    assert!(columns("UPDATE (SELECT * FROM items) SET created_at = now()").is_empty());
}

#[test]
fn merging_catalog_entries_fills_missing_column_order() {
    let mut tables = GeneratedTableColumns::default();
    tables.insert_table(GeneratedTable {
        name: "items".to_string(),
        generated: ["created_at".to_string()].into_iter().collect(),
        column_order: None,
    });
    tables.insert_table(GeneratedTable {
        name: "items".to_string(),
        generated: ["created_at".to_string()].into_iter().collect(),
        column_order: Some(vec!["id".to_string(), "created_at".to_string()]),
    });
    assert_eq!(
        tables.get("items").expect("items").column_order.as_deref(),
        Some(["id".to_string(), "created_at".to_string()].as_slice())
    );
}

#[test]
fn merging_catalog_entries_keeps_schema_order() {
    let mut tables = catalog();
    tables.insert_table(GeneratedTable {
        name: "ITEMS".to_string(),
        generated: ["other_gen".to_string()].into_iter().collect(),
        column_order: None,
    });
    let items = tables.get("Items").expect("merged");
    assert!(items.generated.contains("created_at"));
    assert!(items.generated.contains("other_gen"));
    assert_eq!(
        items.column_order.as_deref(),
        Some(
            [
                "id".to_string(),
                "created_at".to_string(),
                "note".to_string()
            ]
            .as_slice()
        )
    );
}
