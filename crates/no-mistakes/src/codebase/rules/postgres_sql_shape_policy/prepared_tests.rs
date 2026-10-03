use super::*;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactMap, CheckFactPlan,
};
use std::sync::Arc;

#[test]
fn prepared_custom_executor_profiles_canonicalize_order_and_duplicates() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/prepared");
    let config = config_yaml("importSpecifier: '@custom/database'\nexecutorNames: [last, first, last]\nbannedShapes: [not-in-subquery]");
    let findings =
        check_with_files(&root, &config, &[root.join("src/custom-executors.ts")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 3]
    );
}

#[test]
fn missing_prepared_projections_and_bad_sql_globs_report_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/prepared"),
    );
    let files = [root.join("src/builders.ts")];
    let config = config_yaml("bannedShapes: [not-in-subquery]");
    assert!(
        check_with_files_sources_and_facts(&root, &config, &files, &CheckFactMap::default())
            .unwrap_err()
            .to_string()
            .contains("failed to collect PostgreSQL facts")
    );
    let sources = super::super::source_store_for_files(&files);
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files.to_vec(),
        vec![],
        CheckFactPlan {
            embedded_sql: true,
            embedded_sql_options: vec![EmbeddedSqlOptions::default()],
            ..Default::default()
        },
        None,
        Arc::clone(&sources),
    );
    assert!(
        check_with_files_sources_and_facts(&root, &config, &files, &facts)
            .unwrap_err()
            .to_string()
            .contains("prepared PostgreSQL facts are missing")
    );
    let bad = config_yaml("sqlInclude: ['[']");
    assert!(
        check_with_files_sources_and_facts(&root, &bad, &[], &CheckFactMap::default())
            .unwrap_err()
            .to_string()
            .contains("sqlInclude")
    );
}

fn config_yaml(yaml: &str) -> crate::config::v2::NoMistakesConfig {
    use crate::config::v2::schema::{RuleDef, RuleScope};
    crate::config::v2::NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn complete_from_less_builders_keep_limit_locations_and_suppressions() {
    let root = super::shape_tests::fixture("prepared");
    let config = config_yaml("bannedShapes: [literal-limit]");
    let files = [root.join("src/from-less-builders.ts")];
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [6, 10, 16, 20]
    );
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [6, 10]
    );
}
