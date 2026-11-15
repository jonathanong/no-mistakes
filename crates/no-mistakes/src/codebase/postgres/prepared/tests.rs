use super::*;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture/deferred"),
    )
}

#[test]
fn projections_are_prepared_once_and_borrowed_for_repeated_consumers() {
    let root = root();
    let sql = root.join("sql/locations.sql");
    let ts = root.join("queries.ts");
    let profiles = vec![EmbeddedSqlOptions::configured("@db", &[])];
    let sources = crate::codebase::rules::source_store_for_files(&[sql.clone(), ts.clone()]);
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            vec![sql.clone(), ts.clone()],
            Vec::new(),
            CheckFactPlan {
                postgres_schema: true,
                postgres_dml: true,
                postgres_sql_include: vec!["**/*.sql".into()],
                embedded_sql: true,
                embedded_sql_options: profiles.clone(),
                ..Default::default()
            },
            None,
            sources,
        );
    assert!(std::ptr::eq(
        facts.postgres_statements(&sql, None).unwrap(),
        facts.postgres_statements(&sql, None).unwrap()
    ));
    assert_eq!(
        facts.postgres_statements(&sql, None).unwrap()[0]
            .selects
            .len(),
        2
    );
    assert_eq!(
        facts
            .postgres_statements(&ts, Some(&profiles[0]))
            .unwrap()
            .len(),
        4
    );
    assert!(std::ptr::eq(
        facts.postgres_schema_file(&sql).unwrap(),
        facts.postgres_schema_file(&sql).unwrap()
    ));
    assert!(facts
        .postgres_schema_file(&root.join("missing.sql"))
        .unwrap_err()
        .to_string()
        .contains("prepared PostgreSQL facts are missing"));
    assert!(facts
        .postgres_statements(
            &ts,
            Some(&EmbeddedSqlOptions::configured("@example/db", &[]))
        )
        .is_err());
    let custom = crate::codebase::postgres::postgres_sql_paths(
        &root,
        &[sql],
        &PostgresSchemaOptions {
            sql_include: vec!["sql/*.sql".into()],
        },
    )
    .unwrap();
    assert_eq!(custom.len(), 1);
}

#[test]
fn request_preparation_preserves_io_failures_and_demand() {
    let root = root();
    let missing = root.join("absent.sql");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&missing));
    let plan = CheckFactPlan {
        postgres_schema: true,
        postgres_dml: true,
        postgres_sql_include: vec!["**/*.sql".into()],
        ..Default::default()
    };
    let facts = prepare(
        &root,
        std::slice::from_ref(&missing),
        &sources,
        &plan,
        &CheckFactMap::default(),
    );
    assert!(facts
        .schema(&missing)
        .unwrap_err()
        .to_string()
        .contains("failed to collect PostgreSQL facts"));
    assert!(facts.statements(&missing, None).is_err());
    let empty = prepare(
        &root,
        &[missing],
        &sources,
        &CheckFactPlan::default(),
        &CheckFactMap::default(),
    );
    assert!(empty.schema.is_empty());
    assert!(empty.statements.is_empty());
}

#[test]
fn missing_embedded_projections_and_independent_sql_demands_are_recorded() {
    let root = root();
    let sql = root.join("sql/schema.sql");
    let ts = root.join("queries.ts");
    let sources = crate::codebase::rules::source_store_for_files(&[sql.clone(), ts.clone()]);
    let schema = prepare(
        &root,
        std::slice::from_ref(&sql),
        &sources,
        &CheckFactPlan {
            postgres_schema: true,
            postgres_sql_include: vec!["**/*.sql".into()],
            ..Default::default()
        },
        &CheckFactMap::default(),
    );
    assert_eq!(
        schema.schema(&sql).unwrap().tables[0].table_name,
        "snapshot"
    );
    assert!(schema.statements.is_empty());
    let dml = prepare(
        &root,
        &[sql.clone(), ts.clone()],
        &sources,
        &CheckFactPlan {
            postgres_dml: true,
            postgres_fragments: true,
            postgres_sql_include: vec!["**/*.sql".into()],
            embedded_sql_options: vec![EmbeddedSqlOptions::configured("@example/db", &[])],
            ..Default::default()
        },
        &CheckFactMap::default(),
    );
    assert!(dml.schema.is_empty());
    assert!(dml
        .fragments(&ts, &EmbeddedSqlOptions::configured("@example/db", &[]))
        .is_err());
    assert!(dml.statements(&sql, None).is_ok());
    assert!(dml
        .statements(
            &ts,
            Some(&EmbeddedSqlOptions::configured("@example/db", &[]))
        )
        .unwrap_err()
        .to_string()
        .contains("prepared facts are missing"));
}

#[test]
fn prepared_schema_decodes_invalid_utf8_and_keeps_keys_and_indexes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules");
    let fk = root.join("postgres-fk-index/invalid-utf8/migrations/001.sql");
    let redundant = root.join("postgres-redundant-index/invalid-utf8/migrations/001.sql");
    let files = vec![fk.clone(), redundant.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = prepare(
        &root,
        &files,
        &sources,
        &CheckFactPlan {
            postgres_schema: true,
            postgres_sql_include: vec!["**/*.sql".into()],
            ..Default::default()
        },
        &CheckFactMap::default(),
    );
    let fk_facts = facts.schema(&fk).unwrap();
    assert!(fk_facts.unanalyzable_statements.is_empty(), "{fk_facts:?}");
    assert!(fk_facts
        .foreign_keys
        .iter()
        .any(|key| { key.table_name == "comments" && key.column_names == ["post_id"] }));
    let index_facts = facts.schema(&redundant).unwrap();
    assert!(
        index_facts.unanalyzable_statements.is_empty(),
        "{index_facts:?}"
    );
    let names: Vec<_> = index_facts
        .indexes
        .iter()
        .filter_map(|index| index.name.as_deref())
        .collect();
    assert!(names.contains(&"idx_events__topic_id"), "{names:?}");
    assert!(
        names.contains(&"idx_events__topic_id__created_at"),
        "{names:?}"
    );
}

mod bound_demand;
mod variant_demand;

mod bind_column_uses;
mod copy_recovery;

mod conflict_functions;

mod conflict_location_demand;

mod scan_compatibility;
