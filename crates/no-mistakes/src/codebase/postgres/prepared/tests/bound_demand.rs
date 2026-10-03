use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlStatementFileFacts};
use std::{path::PathBuf, sync::Arc};

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/fixture/prepared-demand"),
    )
}

fn without_bounds(facts: &[SqlStatementFileFacts]) -> Vec<SqlStatementFileFacts> {
    facts
        .iter()
        .cloned()
        .map(|mut fact| {
            fact.bounds.clear();
            fact
        })
        .collect()
}

#[test]
fn bound_demand_is_additive_for_sql_embedded_and_fragment_projections() {
    let root = root();
    let sql = root.join("queries.sql");
    let ts = root.join("queries.ts");
    let files = vec![sql.clone(), ts.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let profile = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    let mut plan = CheckFactPlan {
        postgres_dml: true,
        postgres_fragments: true,
        postgres_sql_include: vec!["**/*.sql".into()],
        embedded_sql: true,
        embedded_sql_options: vec![profile.clone()],
        ..Default::default()
    };
    assert!(!plan.postgres_bounds);
    crate::ast::begin_parse_count(&root);
    let baseline = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files.clone(),
        vec![],
        plan.clone(),
        None,
        Arc::clone(&sources),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&ts), Some(&1), "{counts:#?}");
    let baseline_sql = baseline.postgres_statements(&sql, None).unwrap();
    let baseline_ts = baseline.postgres_statements(&ts, Some(&profile)).unwrap();
    assert_eq!(baseline_sql.len(), 1);
    assert_eq!(baseline_ts.len(), 1);
    assert!(baseline_sql[0].bounds.is_empty());
    assert!(baseline_ts[0].bounds.is_empty());
    assert_eq!(sources.physical_read_count(), 2);

    // Unioning a separate consumer's demand must preserve the bound projection.
    plan.include(CheckFactPlan {
        postgres_bounds: true,
        ..Default::default()
    });
    plan.include(CheckFactPlan::default());
    assert!(plan.postgres_bounds);
    crate::ast::begin_parse_count(&root);
    let enabled = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files,
        vec![],
        plan,
        None,
        Arc::clone(&sources),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&ts), Some(&1), "{counts:#?}");
    let enabled_sql = enabled.postgres_statements(&sql, None).unwrap();
    let enabled_ts = enabled.postgres_statements(&ts, Some(&profile)).unwrap();
    assert_eq!(enabled_sql[0].bounds.len(), 3);
    assert_eq!(enabled_ts[0].bounds.len(), 1);
    assert_eq!(without_bounds(enabled_sql), baseline_sql);
    assert_eq!(without_bounds(enabled_ts), baseline_ts);
    assert_eq!(sources.physical_read_count(), 2);
    assert!(std::ptr::eq(
        enabled_sql,
        enabled.postgres_statements(&sql, None).unwrap()
    ));

    let baseline_fragments = baseline.postgres_fragments(&ts, &profile).unwrap();
    let enabled_fragments = enabled.postgres_fragments(&ts, &profile).unwrap();
    assert_eq!(baseline_fragments.len(), 1);
    assert_eq!(enabled_fragments.len(), 1);
    assert!(enabled_fragments[0].statements.bounds.is_empty());
    assert_eq!(
        enabled_fragments[0].statements.as_ref(),
        baseline_fragments[0].statements.as_ref()
    );
}

#[test]
fn optional_extraction_preserves_every_existing_statement_field() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/prepared-demand/queries.sql"
    ));
    let enabled = crate::codebase::postgres::extract_sql_statement_facts(sql);
    let baseline =
        crate::codebase::postgres::statements::extract_sql_statement_facts_with_bounds(sql, false);
    assert_eq!(enabled.bounds.len(), 3);
    assert!(baseline.bounds.is_empty());
    assert_eq!(without_bounds(&[enabled]), [baseline]);
}
