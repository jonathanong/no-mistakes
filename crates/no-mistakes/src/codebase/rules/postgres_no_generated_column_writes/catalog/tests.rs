use super::*;
use crate::codebase::postgres::{SqlColumnMetadata, SqlCreateTableMetadata};
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
    let catalog = catalog_from_facts(&schema, &extra);
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
    assert!(catalog_from_facts(&schema, &extra).is_empty());
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
    let findings = stale_extra_findings(&schema, &extra);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("items.created_at"));
}
