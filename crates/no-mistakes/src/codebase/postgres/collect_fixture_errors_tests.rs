use super::{
    collect_postgres_facts, collect_schema_facts, extract_embedded_sql_facts, extract_schema_facts,
    EmbeddedSqlOptions, PostgresFactError, PostgresSchemaOptions,
};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts/collect-errors")
}
fn store(paths: &[PathBuf]) -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(paths)))
}
fn options(name: &str) -> PostgresSchemaOptions {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SavedOptions {
        sql_include: Vec<String>,
    }
    let value: SavedOptions = serde_json::from_str(
        &std::fs::read_to_string(root().join(format!("{name}.json"))).unwrap(),
    )
    .unwrap();
    PostgresSchemaOptions {
        sql_include: value.sql_include,
    }
}
fn error<T>(value: Result<T, PostgresFactError>) -> PostgresFactError {
    match value {
        Ok(_) => panic!("expected the actual source/option error"),
        Err(error) => error,
    }
}
fn assert_read_error(error: PostgresFactError, path: &Path) {
    assert_eq!(error.path.as_deref(), Some(path));
    assert!(error.message.starts_with("failed to read:"), "{error}");
}

#[test]
fn collectors_report_real_schema_and_embedded_read_failures_with_the_source_path() {
    let root = root();
    let sql = root.join("unreadable.sql");
    let mts = root.join("unreadable.mts");
    let sources = store(&[sql.clone(), mts.clone()]);
    assert_read_error(
        error(extract_schema_facts(
            &root,
            &sources,
            std::slice::from_ref(&sql),
        )),
        &sql,
    );
    let embedded = EmbeddedSqlOptions::configured("@example/db", &[]);
    assert_read_error(
        error(extract_embedded_sql_facts(
            &root,
            &sources,
            std::slice::from_ref(&mts),
            &embedded,
        )),
        &mts,
    );
    assert_read_error(
        error(collect_postgres_facts(
            &root,
            &sources,
            std::slice::from_ref(&sql),
            &CheckFactPlan {
                postgres_schema: true,
                ..Default::default()
            },
            &options("valid"),
            &embedded,
        )),
        &sql,
    );
    assert_read_error(
        error(collect_postgres_facts(
            &root,
            &sources,
            std::slice::from_ref(&mts),
            &CheckFactPlan {
                embedded_sql: true,
                ..Default::default()
            },
            &options("valid"),
            &embedded,
        )),
        &mts,
    );
}

#[test]
fn invalid_include_options_are_pathless_and_disabled_selection_performs_no_reads() {
    let root = root();
    let path = root.join("unreadable.sql");
    let sources = store(std::slice::from_ref(&path));
    let paths = [path];
    let failure = error(collect_schema_facts(
        &root,
        &sources,
        &paths,
        &options("invalid"),
    ));
    assert_eq!(failure.path, None);
    assert!(failure.message.contains("invalid sqlInclude glob"));
    assert!(
        collect_schema_facts(&root, &sources, &paths, &options("disabled"))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn combined_fact_collection_keeps_schema_embedded_sql_and_statement_outputs() {
    let root = root();
    let sql = root.join("valid.sql");
    let mts = root.join("query.mts");
    let paths = [sql.clone(), mts.clone()];
    let sources = store(&paths);
    let facts = collect_postgres_facts(
        &root,
        &sources,
        &paths,
        &CheckFactPlan {
            postgres_schema: true,
            embedded_sql: true,
            postgres_dml: true,
            ..Default::default()
        },
        &options("valid"),
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
    .unwrap();
    assert_eq!(facts.schema.len(), 1);
    assert_eq!(facts.schema[0].path, sql);
    assert_eq!(facts.embedded.len(), 1);
    assert_eq!(facts.embedded[0].path, mts);
    assert_eq!(
        facts.embedded[0].calls[0].sql_text.as_deref(),
        Some("SELECT id FROM accounts")
    );
    assert!(!facts.statements.is_empty());
}
