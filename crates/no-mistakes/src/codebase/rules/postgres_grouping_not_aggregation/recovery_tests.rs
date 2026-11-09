use super::analyze;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::statements::{
    SqlFactSite, SqlLocationDemand, StatementPolicySources,
};
use crate::codebase::postgres::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, SchemaCatalog, SqlStatementFileFacts,
    TrustedSqlTag,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-grouping-not-aggregation/fixture")
}

fn assert_complete_sibling_findings(records: &[SqlStatementFileFacts], kind: &str) {
    let root = fixture();
    let sources = super::super::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let mut lines = Vec::new();
    assert_eq!(records.len(), 1);
    for record in records {
        assert!(record.parse_failed);
        assert_eq!(record.grouping_queries.len(), 2);
        for (query_index, query) in record.grouping_queries.iter().enumerate() {
            for finding in analyze::query(&query.query, &catalog, &query.parameter_sites) {
                assert_eq!(finding.code, "distinct-over-unique-key");
                let select_index = query
                    .sites
                    .iter()
                    .position(|site| *site == finding.site)
                    .unwrap();
                let line = record
                    .variant_locations
                    .as_ref()
                    .and_then(|locations| {
                        locations.position(SqlFactSite::Grouping(query_index, select_index))
                    })
                    .map_or(finding.site.0, |position| position.source_line);
                lines.push(line);
            }
        }
    }
    lines.sort();
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-grouping-not-aggregation/fixture/recovery/expected.json"
    )))
    .unwrap();
    assert_eq!(serde_json::to_value(lines).unwrap(), expected[kind]);
}

#[test]
fn native_recovery_reports_only_complete_siblings() {
    let root = fixture();
    let path = root.join("recovery/raw/recovery-controls.sql");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
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
        sources,
    );
    assert_complete_sibling_findings(facts.postgres_statements(&path, None).unwrap(), "raw");
}

#[test]
fn embedded_recovery_reports_only_complete_siblings_at_physical_lines() {
    let path = fixture().join("recovery/src/recovery-controls.ts");
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-grouping-not-aggregation/fixture/recovery/src/recovery-controls.ts"));
    let options = EmbeddedSqlOptions::configured("db", &["query".into()]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "db".into(),
            name: "sql".into(),
        },
    ]);
    let embedded = extract_embedded_sql_from_source(&path, source, &options);
    assert_eq!(embedded.calls.len(), 1);
    let records = crate::codebase::postgres::tests::embedded_call_facts_with_policy(
        &embedded,
        false,
        false,
        SqlLocationDemand::Full,
        StatementPolicySources {
            collect_grouping: true,
            ..Default::default()
        },
    );
    assert_complete_sibling_findings(&records, "embedded");
}
