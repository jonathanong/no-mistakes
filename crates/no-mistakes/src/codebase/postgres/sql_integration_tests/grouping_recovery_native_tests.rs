use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::statements::SqlStatementFileFacts;
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
