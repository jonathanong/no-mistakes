use super::fixture;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::{extract_sql_statement_facts, statements::SqlFactSite};

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
