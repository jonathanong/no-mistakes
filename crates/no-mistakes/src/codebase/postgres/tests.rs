use super::{
    extract_create_table_metadata, extract_dml_write_targets, extract_embedded_sql_from_source,
    extract_locking_select_metadata, extract_migration_facts, EmbeddedSqlOptions,
};
use crate::codebase::check_facts::CheckFactPlan;

#[test]
fn check_fact_plan_include_merges_postgres_flags() {
    let mut plan = CheckFactPlan::default();
    assert!(!plan.postgres_schema);
    assert!(!plan.embedded_sql);
    plan.include(CheckFactPlan {
        postgres_schema: true,
        ..CheckFactPlan::default()
    });
    assert!(plan.postgres_schema);
    assert!(!plan.embedded_sql);
    plan.include(CheckFactPlan {
        embedded_sql: true,
        ..CheckFactPlan::default()
    });
    assert!(plan.postgres_schema);
    assert!(plan.embedded_sql);
    plan.include(CheckFactPlan::default());
    assert!(plan.postgres_schema);
    assert!(plan.embedded_sql);
}

#[test]
fn public_extractors_are_callable_from_the_module_root() {
    let tables = extract_create_table_metadata("CREATE TABLE t (id int);");
    assert_eq!(tables[0].table_name, "t");
    let schema = extract_migration_facts("CREATE TABLE t (id int PRIMARY KEY);");
    assert_eq!(schema.indexes[0].leading_column.as_deref(), Some("id"));
    let facts = extract_embedded_sql_from_source(
        std::path::Path::new("root.ts"),
        "import { query } from '@data-stores/psql'\nquery('SELECT 1')\n",
        &EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 1"));
    assert_eq!(
        extract_dml_write_targets("UPDATE items SET note = 1"),
        ["items"]
    );
    let locks =
        extract_locking_select_metadata("SELECT * FROM t WHERE id = ANY($1) FOR UPDATE").unwrap();
    assert!(locks[0].has_multi_row_predicate);
}
