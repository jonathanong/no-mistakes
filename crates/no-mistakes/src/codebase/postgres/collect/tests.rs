use super::{
    collect_postgres_facts, collect_schema_facts, extract_embedded_sql_facts, extract_schema_facts,
};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresFactError, PostgresSchemaOptions};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts")
}

fn schema_path(name: &str) -> PathBuf {
    fixture_root().join("schema").join(name)
}

fn embedded_path(name: &str) -> PathBuf {
    fixture_root().join("embedded").join(name)
}

fn store(paths: &[PathBuf]) -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(paths)))
}

#[test]
fn extract_schema_facts_reads_through_source_store() {
    let generated = schema_path("generated-column.sql");
    let table_pk = schema_path("table-level-pk.sql");
    let sources = store(&[generated.clone(), table_pk.clone()]);
    let facts = extract_schema_facts(
        Path::new("/repo"),
        &sources,
        &[table_pk.clone(), generated.clone()],
    )
    .expect("schema facts");
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].path, generated);
    assert_eq!(facts[0].tables[0].table_name, "MixedCase");
    assert_eq!(facts[1].tables[0].table_name, "table_level_pk");
}

#[test]
fn collect_schema_facts_honors_include_globs() {
    let root = fixture_root();
    let generated = schema_path("generated-column.sql");
    let notes = schema_path("notes.txt");
    let sources = store(&[generated.clone(), notes.clone()]);
    let facts = collect_schema_facts(
        &root,
        &sources,
        &[generated.clone(), notes],
        &PostgresSchemaOptions::default(),
    )
    .expect("filtered");
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].path, generated);
}

#[test]
fn collect_schema_facts_custom_glob_matches_basename() {
    let root = fixture_root();
    let generated = schema_path("generated-column.sql");
    let sources = store(std::slice::from_ref(&generated));
    let options = PostgresSchemaOptions {
        sql_include: vec!["generated-column.sql".to_string()],
    };
    let facts =
        collect_schema_facts(&root, &sources, std::slice::from_ref(&generated), &options).unwrap();
    assert_eq!(facts.len(), 1);
}

#[test]
fn invalid_sql_include_glob_returns_error() {
    let error = collect_schema_facts(
        Path::new("/repo"),
        &store(&[]),
        &[],
        &PostgresSchemaOptions {
            sql_include: vec!["[".to_string()],
        },
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid sqlInclude"));
    assert!(error.path.is_none());
}

#[test]
fn extract_schema_facts_skips_unparseable_sql() {
    let path = schema_path("invalid.sql");
    let sources = store(std::slice::from_ref(&path));
    let facts = extract_schema_facts(Path::new("/repo"), &sources, std::slice::from_ref(&path))
        .expect("skip unparseable");
    assert_eq!(facts.len(), 1);
    assert!(facts[0].tables.is_empty());
}

#[test]
fn extract_schema_facts_reads_create_table_from_mixed_migrations() {
    let path = schema_path("mixed-do-block.sql");
    let sources = store(std::slice::from_ref(&path));
    let facts = extract_schema_facts(Path::new("/repo"), &sources, std::slice::from_ref(&path))
        .expect("mixed");
    assert_eq!(facts[0].tables.len(), 1);
    assert_eq!(facts[0].tables[0].table_name, "items");
    assert!(facts[0].tables[0]
        .columns
        .iter()
        .any(|column| column.name == "created_at" && column.is_generated));
}

#[test]
fn extract_schema_facts_reports_read_errors() {
    let missing = fixture_root().join("schema/does-not-exist.sql");
    let sources = store(&[]);
    let error = extract_schema_facts(Path::new("/repo"), &sources, std::slice::from_ref(&missing))
        .unwrap_err();
    assert_eq!(error.path.as_deref(), Some(missing.as_path()));
    assert!(error.to_string().contains("failed to read"));
}

#[test]
fn extract_embedded_sql_facts_reads_through_source_store() {
    let tagged = embedded_path("tagged-template.ts");
    let sources = store(std::slice::from_ref(&tagged));
    let facts = extract_embedded_sql_facts(
        Path::new("/repo"),
        &sources,
        std::slice::from_ref(&tagged),
        &EmbeddedSqlOptions::default(),
    )
    .unwrap();
    assert_eq!(facts[0].path, tagged);
    assert_eq!(facts[0].executor_bindings, ["query"]);
}

#[test]
fn collect_postgres_facts_respects_plan_flags() {
    let sql = schema_path("generated-column.sql");
    let ts = embedded_path("string-literal.ts");
    let sources = store(&[sql.clone(), ts.clone()]);
    let files = vec![sql, ts];
    let empty = collect_postgres_facts(
        &fixture_root(),
        &sources,
        &files,
        &CheckFactPlan::default(),
        &PostgresSchemaOptions::default(),
        &EmbeddedSqlOptions::default(),
    )
    .unwrap();
    assert!(empty.schema.is_empty());
    assert!(empty.embedded.is_empty());

    let both = collect_postgres_facts(
        &fixture_root(),
        &sources,
        &files,
        &CheckFactPlan {
            postgres_schema: true,
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &PostgresSchemaOptions::default(),
        &EmbeddedSqlOptions::default(),
    )
    .unwrap();
    assert_eq!(both.schema.len(), 1);
    assert_eq!(both.embedded.len(), 1);
}

#[test]
fn fact_error_display_without_path() {
    let error = PostgresFactError::message("nope");
    assert_eq!(error.to_string(), "nope");
    assert!(error.path.is_none());
}

#[test]
fn default_schema_options_include_sql_files() {
    assert_eq!(PostgresSchemaOptions::default().sql_include, ["**/*.sql"]);
}

#[test]
fn compile_sql_include_rejects_invalid_globs() {
    let error = super::compile_sql_include(&["[".into()]).unwrap_err();
    assert!(error.to_string().contains("invalid sqlInclude"));
}
