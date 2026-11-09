use super::*;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::statements::StatementPolicySources;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-grouping-not-aggregation/fixture")
}

fn sites(facts: &SqlStatementFileFacts) -> Vec<usize> {
    let mut lines = facts
        .grouping_queries
        .iter()
        .flat_map(|query| query.sites.iter().map(|site| site.0))
        .collect::<Vec<_>>();
    lines.sort();
    lines
}

#[test]
fn grouping_retains_complete_siblings_around_an_unsupported_chunk() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-grouping-not-aggregation/fixture/recovery/raw/recovery-controls.sql"));
    let normalized = crate::codebase::postgres::parse::PreparedSql::new(source);
    let recovered = crate::codebase::postgres::parse::parse_postgres_sql_lenient_with_sources(
        source,
        normalized.normalized(),
    );
    assert!(
        recovered.iter().any(|statement| !statement.complete
            && statement.recovery_failure.is_some()
            && matches!(
                &statement.statement,
                sqlparser::ast::Statement::Discard { .. }
            )),
        "{recovered:?}"
    );
    let facts = extract_sql_statement_facts_with_policy(
        source,
        false,
        None,
        SqlLocationDemand::Full,
        StatementPolicySources {
            collect_grouping: true,
            ..Default::default()
        },
        false,
    );
    assert!(facts.parse_failed);
    assert_eq!(sites(&facts), [2, 6]);
    assert_eq!(facts.grouping_queries.len(), 2);
}

#[test]
fn native_preparation_keeps_per_statement_grouping_completeness_without_rereading() {
    let root = fixture();
    let path = root.join("recovery/raw/recovery-controls.sql");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        vec![path.clone()],
        Vec::new(),
        CheckFactPlan {
            postgres_dml: true,
            postgres_grouping: true,
            postgres_sql_include: vec!["recovery/raw/**/*.sql".into()],
            ..Default::default()
        },
        None,
        sources.clone(),
    );
    let records = facts.postgres_statements(&path, None).unwrap();
    assert_eq!(records.len(), 1);
    assert!(records[0].parse_failed);
    assert_eq!(sites(&records[0]), [2, 6]);
    assert_eq!(records[0].grouping_queries.len(), 2);
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn borrowed_completeness_gates_query_roots_without_changing_other_facts() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/completeness.sql"
    ));
    let complete: Vec<bool> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/completeness.json"
    )))
    .unwrap();
    let prepared = crate::codebase::postgres::parse::PreparedSql::new(sql);
    let statements = prepared.parse_policy().unwrap();
    assert_eq!(statements.len(), complete.len());
    let project = |grouping_complete: Option<&[bool]>| {
        extract_from_parsed_with_recovered_placeholders(
            sql,
            &prepared,
            &statements,
            true,
            false,
            None,
            StatementPolicySources {
                collect_grouping: true,
                grouping_complete,
                ..Default::default()
            },
        )
    };
    let mut ungated = project(None);
    let mut gated = project(Some(&complete));
    assert_eq!(sites(&ungated), [1, 2, 3]);
    assert_eq!(sites(&gated), [1, 3]);
    assert_eq!(gated.grouping_queries.len(), 2);
    assert!(gated.parse_failed);
    gated.grouping_queries.clear();
    ungated.grouping_queries.clear();
    assert_eq!(gated, ungated);
}
