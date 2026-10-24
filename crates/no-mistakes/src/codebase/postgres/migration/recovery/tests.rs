use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::{extract_sql_statement_facts, statements::SqlFactSite};
use std::path::PathBuf;
fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-parse-failure-locations/fixture")
            .join(name),
    )
}
#[test]
fn parse_failure_locations_match_direct_and_prepared_facts_without_rereading() {
    let path = fixture("failures.sql");
    let root = path.parent().unwrap();
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        root,
        vec![path.clone()],
        Vec::new(),
        CheckFactPlan {
            postgres_schema: true,
            postgres_dml: true,
            postgres_bounds: true,
            postgres_sql_include: vec!["failures.sql".into()],
            ..Default::default()
        },
        None,
        sources.clone(),
    );
    let prepared = &facts.postgres_statements(&path, None).unwrap()[0];
    let source = sources.read_path(&path).unwrap();
    let direct = extract_sql_statement_facts(&source);
    assert_eq!(prepared.parse_failures, direct.parse_failures);
    assert_eq!(
        prepared.parse_failure_sites().collect::<Vec<_>>(),
        [
            (SqlFactSite::ParseFailure(0), 3),
            (SqlFactSite::ParseFailure(1), 6)
        ]
    );
    assert!(prepared
        .parse_failures
        .iter()
        .all(|failure| failure.column == 1 && failure.insert_keyword_count == 0));
    assert_eq!(prepared.bounds, direct.bounds);
    assert_eq!(
        facts
            .postgres_schema_file(&path)
            .unwrap()
            .unanalyzable_statements,
        direct.parse_failures,
    );
    assert_eq!(sources.physical_read_count(), 1);
}
#[test]
fn parse_failure_locations_rebase_through_the_existing_embedded_provenance() {
    let path = fixture("failures.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("db", &["query".into()]),
    );
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts_with_demand(
        &embedded,
        true,
        false,
        crate::codebase::postgres::statement_facts::SqlLocationDemand::Full,
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(
        facts[0]
            .parse_failures
            .iter()
            .map(|failure| failure.line)
            .collect::<Vec<_>>(),
        [4, 7]
    );
    let locations = facts[0].variant_locations.as_ref().unwrap();
    for (index, line) in [4, 7].into_iter().enumerate() {
        assert_eq!(
            locations
                .position(SqlFactSite::ParseFailure(index))
                .unwrap()
                .source_line,
            line
        );
    }
}
#[test]
fn parse_failure_locations_keep_insert_relevance_per_failed_chunk() {
    let facts = extract_sql_statement_facts(
        &std::fs::read_to_string(fixture("insert-failures.sql")).unwrap(),
    );
    assert_eq!(
        facts
            .parse_failures
            .iter()
            .map(|failure| (failure.line, failure.insert_keyword_count))
            .collect::<Vec<_>>(),
        [(3, 1), (4, 0), (6, 1)]
    );
    let facts = extract_sql_statement_facts(
        &std::fs::read_to_string(fixture("valid-siblings.sql")).unwrap(),
    );
    let stars: Vec<_> = facts
        .selects
        .iter()
        .flat_map(|select| &select.star_projections)
        .map(|star| star.line)
        .collect();
    assert_eq!(stars, [4, 6]);
}

#[test]
fn parse_failure_locations_retain_a_lexically_invalid_later_statement() {
    let source = std::fs::read_to_string(fixture("lexical-failure.sql")).unwrap();
    let facts = extract_sql_statement_facts(&source);
    assert_eq!(
        facts
            .parse_failure_sites()
            .map(|(_, line)| line)
            .collect::<Vec<_>>(),
        [3]
    );
    assert!(facts
        .parse_failures
        .iter()
        .all(|failure| failure.insert_keyword_count == 0));
}

#[test]
fn parse_failure_locations_preserve_columns_and_composed_lexical_origins() {
    for (name, column) in [("lexical-same-line.sql", 11), ("lexical-unicode.sql", 13)] {
        let facts = extract_sql_statement_facts(&std::fs::read_to_string(fixture(name)).unwrap());
        assert_eq!(
            facts
                .parse_failures
                .iter()
                .map(|failure| (failure.line, failure.column))
                .collect::<Vec<_>>(),
            [(1, column)]
        );
    }
    let path = fixture("lexical-composed.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("db", &["query".into()]),
    );
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts_with_demand(
        &embedded,
        true,
        false,
        crate::codebase::postgres::statement_facts::SqlLocationDemand::Full,
    );
    assert_eq!(
        facts[0]
            .parse_failures
            .iter()
            .map(|failure| failure.line)
            .collect::<Vec<_>>(),
        [3]
    );
    let at = facts[0]
        .variant_locations
        .as_ref()
        .unwrap()
        .position(SqlFactSite::ParseFailure(0))
        .unwrap();
    assert_eq!((at.sql_line, at.sql_column, at.source_line), (1, 11, 3));
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
        [(1, 1, 1), (3, 0, 0), (4, 1, 0)]
    );
    let facts = extract_sql_statement_facts(
        &std::fs::read_to_string(fixture("failed-trigger.sql")).unwrap(),
    );
    assert!(facts
        .parse_failures
        .iter()
        .all(|failure| failure.insert_keyword_count == 0));
}

#[test]
fn canonical_failures_keep_schema_and_partial_dml_projections_distinct() {
    let path = fixture("recovered-and-failed-insert.sql");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    let canonical = super::recovery_failures(&parsed);
    let direct = extract_sql_statement_facts(&source);
    assert_eq!(direct.parse_failures, canonical);
    assert_eq!(
        canonical
            .iter()
            .map(|failure| failure.schema_unanalyzable)
            .collect::<Vec<_>>(),
        [false, true, true]
    );
    let schema = crate::codebase::postgres::extract_migration_facts(&source);
    assert_eq!(schema.unanalyzable_statements, canonical[1..]);
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
    let failures = super::recovery_failures(&parsed);
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
    assert!(failures[1].head.starts_with("CREATE VIEW"));
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
    let failures = super::recovery_failures(&parsed);
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
        assert!(super::recovery_failures(&parsed).is_empty());
        assert!(extract_sql_statement_facts(&source)
            .parse_failures
            .is_empty());
    }
    let source = std::fs::read_to_string(fixture("canonical-lexical-single.sql")).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    assert!(parsed[0].chunk_tokens.is_none());
    let failures = super::recovery_failures(&parsed);
    assert_eq!((failures[0].line, failures[0].column), (3, 5));
    assert_eq!(failures[0].source_order, [0]);
    assert!(failures[0].head.starts_with("SELECT 'unterminated"));
    assert_eq!(
        extract_sql_statement_facts(&source).parse_failures,
        failures
    );
}

#[test]
fn lexical_body_failures_share_canonical_records_without_duplicate_projection_sites() {
    let path = crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/procedural/migrations/lexical-command-error.sql"));
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed =
        crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(&source);
    let canonical = super::recovery_failures(&parsed);
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
