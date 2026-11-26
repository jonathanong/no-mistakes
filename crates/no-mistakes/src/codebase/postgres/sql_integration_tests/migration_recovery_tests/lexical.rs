use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::extract_sql_statement_facts;
use std::path::PathBuf;

#[test]
fn lexical_body_failures_share_canonical_records_without_duplicate_projection_sites() {
    let path = crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/procedural/migrations/lexical-command-error.sql"));
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    let canonical = crate::codebase::postgres::migration::recovery_failures(&parsed);
    // Both prior control-only chunks and prior recovered SQL must retain the lexical leaf.
    assert_eq!(
        canonical
            .iter()
            .map(|failure| (
                failure.line,
                failure.schema_unanalyzable,
                failure.statement_unanalyzable,
            ))
            .collect::<Vec<_>>(),
        [
            (2, true, false),
            (7, false, true),
            (10, true, true),
            (13, true, false),
            (15, false, true),
            (16, false, true),
            (19, true, true)
        ]
    );
    let statements = extract_sql_statement_facts(&source);
    // The recovered BEGIN/CREATE chunk keeps its existing separate partial-recovery site.
    assert_eq!(
        statements
            .parse_failure_sites()
            .map(|(_, line)| line)
            .collect::<Vec<_>>(),
        [7, 10, 15, 16, 19]
    );
    let schema = crate::codebase::postgres::extract_migration_facts(&source);
    assert_eq!(
        schema
            .unanalyzable_statements
            .iter()
            .map(|failure| failure.line)
            .collect::<Vec<_>>(),
        [2, 10, 13, 19]
    );
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let prepared = collect_check_facts_with_graph_files_playwright_and_sources(
        path.parent().unwrap(),
        vec![path.clone()],
        Vec::new(),
        CheckFactPlan {
            postgres_schema: true,
            postgres_dml: true,
            postgres_sql_include: vec!["lexical-command-error.sql".into()],
            ..Default::default()
        },
        None,
        sources.clone(),
    );
    assert_eq!(
        prepared.postgres_statements(&path, None).unwrap()[0].parse_failures,
        statements.parse_failures
    );
    assert_eq!(
        prepared
            .postgres_schema_file(&path)
            .unwrap()
            .unanalyzable_statements,
        schema.unanalyzable_statements
    );
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn recursive_heads_preserve_canonical_failures_and_valid_sibling_asts() {
    let path = crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
        .join("recursive-heads/migrations/001.sql");
    let source = std::fs::read_to_string(path).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    let canonical = crate::codebase::postgres::migration::recovery_failures(&parsed);
    assert_eq!(
        crate::codebase::postgres::extract_migration_facts(&source).unanalyzable_statements,
        canonical
            .iter()
            .filter(|failure| failure.schema_unanalyzable)
            .cloned()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        extract_sql_statement_facts(&source).parse_failures,
        canonical
    );
    // Invalid siblings must not change valid recursive views or create failures for them.
    assert_eq!(
        parsed
            .iter()
            .filter(|located| matches!(&located.statement,
            sqlparser::ast::Statement::CreateView(view)
                if view.query.with.as_ref().is_some_and(|with| with.recursive)))
            .count(),
        2
    );
}
