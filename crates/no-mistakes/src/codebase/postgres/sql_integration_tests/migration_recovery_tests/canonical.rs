use super::fixture;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn canonical_failures_keep_schema_and_partial_dml_projections_distinct() {
    let path = fixture("recovered-and-failed-insert.sql");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    assert!(matches!(
        parsed[0].statement,
        sqlparser::ast::Statement::Insert(_)
    ));
    assert!(parsed[0].complete);
    assert!(parsed[0].recovery_failure.is_none());
    let canonical = crate::codebase::postgres::migration::recovery_failures(&parsed);
    assert_eq!(
        canonical
            .iter()
            .map(|failure| failure.line)
            .collect::<Vec<_>>(),
        [3, 4]
    );
    let direct = extract_sql_statement_facts(&source);
    assert_eq!(direct.parse_failures, canonical);
    assert_eq!(
        canonical
            .iter()
            .map(|failure| failure.schema_unanalyzable)
            .collect::<Vec<_>>(),
        [true, true]
    );
    let schema = crate::codebase::postgres::extract_migration_facts(&source);
    assert_eq!(schema.unanalyzable_statements, canonical);
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        path.parent().unwrap(),
        vec![path.clone()],
        Vec::new(),
        CheckFactPlan {
            postgres_schema: true,
            postgres_dml: true,
            postgres_sql_include: vec!["recovered-and-failed-insert.sql".into()],
            ..Default::default()
        },
        None,
        sources.clone(),
    );
    assert_eq!(
        facts.postgres_statements(&path, None).unwrap()[0].parse_failures,
        canonical
    );
    assert_eq!(
        facts
            .postgres_schema_file(&path)
            .unwrap()
            .unanalyzable_statements,
        schema.unanalyzable_statements
    );
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn canonical_failures_preserve_procedural_leaf_locations_and_order() {
    let source = std::fs::read_to_string(fixture("canonical-leaf-order.sql")).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    let failures = crate::codebase::postgres::migration::recovery_failures(&parsed);
    // BEGIN has its own chunk so the nested view is rejected after a complete parse.
    assert_eq!(
        failures
            .iter()
            .map(|failure| (
                failure.line,
                failure.column,
                failure.source_order.clone(),
                failure.insert_keyword_count,
            ))
            .collect::<Vec<_>>(),
        [(5, 5, vec![1, 1], 1), (9, 5, vec![1, 3, 1], 0)]
    );
    assert!(failures[0].head.starts_with("INSERT INTO"));
    assert!(failures[1].head.starts_with("CREATE RECURSIVE VIEW"));
    assert_eq!(
        crate::codebase::postgres::extract_migration_facts(&source).unanalyzable_statements,
        failures
    );
    assert_eq!(
        extract_sql_statement_facts(&source).parse_failures,
        failures
    );
}

#[test]
fn lexical_chunk_reparsing_replaces_only_the_outer_ordinal() {
    let source = std::fs::read_to_string(fixture("canonical-lexical-order.sql")).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    let failures = crate::codebase::postgres::migration::recovery_failures(&parsed);
    assert_eq!(
        failures
            .iter()
            .map(|failure| (failure.line, failure.source_order.clone()))
            .collect::<Vec<_>>(),
        [(4, vec![1, 1]), (7, vec![2])]
    );
}

#[test]
fn canonical_empty_and_lexically_invalid_sources_need_no_chunk_sidecar() {
    for name in ["canonical-empty.sql", "canonical-comments.sql"] {
        let source = std::fs::read_to_string(fixture(name)).unwrap();
        let parsed =
            crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
        assert!(crate::codebase::postgres::migration::recovery_failures(&parsed).is_empty());
        assert!(extract_sql_statement_facts(&source)
            .parse_failures
            .is_empty());
    }
    let source = std::fs::read_to_string(fixture("canonical-lexical-single.sql")).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    assert!(parsed[0].chunk_tokens.is_none());
    let failures = crate::codebase::postgres::migration::recovery_failures(&parsed);
    assert_eq!((failures[0].line, failures[0].column), (3, 5));
    assert_eq!(failures[0].source_order, [0]);
    assert!(failures[0].head.starts_with("SELECT 'unterminated"));
    assert_eq!(
        extract_sql_statement_facts(&source).parse_failures,
        failures
    );
}

#[test]
fn parse_failure_locations_account_for_recovered_inserts_before_unsafe_siblings() {
    let facts = extract_sql_statement_facts(
        &std::fs::read_to_string(fixture("recovered-and-failed-insert.sql")).unwrap(),
    );
    assert_eq!(
        facts
            .parse_failures
            .iter()
            .map(|failure| (
                failure.line,
                failure.insert_keyword_count,
                failure.recovered_insert_count
            ))
            .collect::<Vec<_>>(),
        [(3, 0, 0), (4, 1, 0)]
    );
    assert!(facts.parse_failures[0]
        .head
        .starts_with("CREATE RECURSIVE VIEW invalid_view"));
    assert!(facts.parse_failures[1]
        .head
        .starts_with("INSERT INTO events"));
    let facts = extract_sql_statement_facts(
        &std::fs::read_to_string(fixture("failed-trigger.sql")).unwrap(),
    );
    assert!(facts
        .parse_failures
        .iter()
        .all(|failure| failure.insert_keyword_count == 0));
}
