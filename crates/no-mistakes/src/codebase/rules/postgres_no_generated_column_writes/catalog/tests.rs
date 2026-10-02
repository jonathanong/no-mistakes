use super::*;
use crate::codebase::postgres::{SqlColumnMetadata, SqlCreateTableMetadata, SqlSchemaFileFacts};
use std::path::PathBuf;

fn generated_col(name: &str) -> SqlColumnMetadata {
    SqlColumnMetadata {
        name: name.to_string(),
        type_name: None,
        constraints: Vec::new(),
        is_primary_key: false,
        is_generated: true,
        generated_expression: None,
        generated_function: None,
        generated_function_arg_columns: Vec::new(),
        generated_source_columns: Vec::new(),
    }
}

fn plain_col(name: &str) -> SqlColumnMetadata {
    SqlColumnMetadata {
        name: name.to_string(),
        type_name: None,
        constraints: Vec::new(),
        is_primary_key: false,
        is_generated: false,
        generated_expression: None,
        generated_function: None,
        generated_function_arg_columns: Vec::new(),
        generated_source_columns: Vec::new(),
    }
}

#[test]
fn collects_generated_columns_and_extra_tables() {
    let schema = [SqlSchemaFileFacts {
        path: PathBuf::from("schema.sql"),
        tables: vec![SqlCreateTableMetadata {
            table_name: "items".to_string(),
            columns: vec![plain_col("id"), generated_col("created_at")],
        }],
        ..Default::default()
    }];
    let extra = [ExtraGeneratedColumn {
        table: "votes".to_string(),
        column: "created_at".to_string(),
    }];
    let catalog = catalog_from_tables(&live_tables(&schema), &extra);
    assert!(catalog.get("items").is_some_and(|table| {
        table.generated.contains("created_at") && table.column_order.is_some()
    }));
    assert!(catalog
        .get("votes")
        .is_some_and(|table| table.column_order.is_none()));
}

#[test]
fn ignores_tables_without_generated_columns_and_blank_extras() {
    let schema = [SqlSchemaFileFacts {
        path: PathBuf::from("schema.sql"),
        tables: vec![SqlCreateTableMetadata {
            table_name: "plain".to_string(),
            columns: vec![plain_col("id")],
        }],
        ..Default::default()
    }];
    let extra = [
        ExtraGeneratedColumn {
            table: String::new(),
            column: "created_at".to_string(),
        },
        ExtraGeneratedColumn {
            table: "votes".to_string(),
            column: String::new(),
        },
    ];
    assert!(catalog_from_tables(&live_tables(&schema), &extra).is_empty());
}

#[test]
fn stale_extras_that_duplicate_schema_generated_columns() {
    let schema = [SqlSchemaFileFacts {
        path: PathBuf::from("schema.sql"),
        tables: vec![SqlCreateTableMetadata {
            table_name: "items".to_string(),
            columns: vec![plain_col("id"), generated_col("created_at")],
        }],
        ..Default::default()
    }];
    let extra = [ExtraGeneratedColumn {
        table: "items".to_string(),
        column: "created_at".to_string(),
    }];
    let findings = stale_extra_findings_from_tables(&live_tables(&schema), &extra);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("items.created_at"));
}

#[test]
fn legacy_supplied_facts_keep_alter_column_support() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/alter-trigger",
    );
    let sql = std::fs::read_to_string(root.join("schema.sql")).unwrap();
    let mut facts = crate::codebase::postgres::extract_migration_facts(&sql);
    // Programmatic Rust callers may still populate the original aggregate fields.
    facts.table_events.clear();
    facts.table_events_collected = false;
    let catalog = trigger_catalog_from_tables(&live_tables(&[facts]), &["updated_at".into()]);
    assert!(catalog
        .get("orders")
        .is_some_and(|table| table.generated.contains("updated_at")));
    assert!(catalog
        .get("external_orders")
        .is_some_and(|table| table.column_order.is_none()));
}
