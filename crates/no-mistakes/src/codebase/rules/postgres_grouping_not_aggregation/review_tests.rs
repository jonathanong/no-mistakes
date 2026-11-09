use super::analyze;
use crate::codebase::postgres::statements::SqlLocationDemand;
use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_policy, SqlFactSite, StatementPolicySources,
};
use crate::codebase::postgres::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, SchemaCatalog, SqlStatementFileFacts,
    TrustedSqlTag,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&crate::test_support::rule_fixture_root(
        super::RULE_ID,
    ))
}

fn catalog() -> SchemaCatalog {
    let root = fixture();
    let sources = super::super::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

fn assert_review_findings(records: &[SqlStatementFileFacts], kind: &str) {
    let catalog = catalog();
    let mut actual = Vec::new();
    for record in records {
        assert!(!record.parse_failed, "{record:?}");
        for (query_index, query) in record.grouping_queries.iter().enumerate() {
            for finding in analyze::query(&query.query, &catalog, &query.parameter_sites) {
                let select_index = query
                    .sites
                    .iter()
                    .position(|site| *site == finding.site)
                    .unwrap();
                let position = record
                    .variant_locations
                    .as_ref()
                    .unwrap()
                    .position(SqlFactSite::Grouping(query_index, select_index))
                    .unwrap();
                assert_eq!(finding.code, "aggregate-existence-test");
                assert!(
                    finding.message.contains("threshold IS NOT NULL"),
                    "{}",
                    finding.message
                );
                assert!(
                    finding.message.contains("preserve the original NOT EXISTS"),
                    "{}",
                    finding.message
                );
                actual.push((position.source_line, finding.code));
            }
        }
    }
    actual.sort();
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-grouping-not-aggregation/fixture/review/expected.json"
    )))
    .unwrap();
    assert_eq!(serde_json::to_value(actual).unwrap(), expected[kind]);
}

#[test]
fn raw_review_controls_only_report_guarded_absence_rewrites() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-grouping-not-aggregation/fixture/review/raw/review-controls.sql"));
    let facts = extract_sql_statement_facts_with_policy(
        sql,
        false,
        None,
        SqlLocationDemand::Full,
        StatementPolicySources {
            collect_grouping: true,
            ..Default::default()
        },
        false,
    );
    assert_review_findings(&[facts], "raw");
}

#[test]
fn embedded_review_controls_only_report_guarded_absence_rewrites() {
    let path = fixture().join("review/src/review-controls.ts");
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-grouping-not-aggregation/fixture/review/src/review-controls.ts"));
    let options = EmbeddedSqlOptions::configured("db", &["query".into()]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "db".into(),
            name: "sql".into(),
        },
    ]);
    let embedded = extract_embedded_sql_from_source(&path, source, &options);
    assert_eq!(embedded.calls.len(), 18);
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
    assert_review_findings(&records, "embedded");
}
