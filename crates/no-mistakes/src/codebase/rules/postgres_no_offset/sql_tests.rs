use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

// Output cases moved to cli_postgres_snapshot_no_offset. Keep the invalid glob,
// the pre-suppression count, and idempotent scans below.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-offset/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn messages(root: &str, yaml: &str, files: &[&str]) -> Vec<String> {
    let root = fixture(root);
    let paths: Vec<PathBuf> = files.iter().map(|file| root.join(file)).collect();
    check_with_files(&root, &config(yaml), &paths)
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

const SQL: &str = "sqlInclude: [\"db/**/*.sql\"]\n";

#[test]
fn invalid_sql_include_glob_errors() {
    let root = fixture("fail-sql");
    let error = check_with_files(
        &root,
        &config("sqlInclude: ['[']\n"),
        &[root.join("db/views.sql")],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn suppression_directives_hide_sql_offsets() {
    let root = fixture("suppress-sql");
    let files: Vec<PathBuf> = ["next-line.sql", "line.sql", "file.sql"]
        .into_iter()
        .map(|name| root.join("db").join(name))
        .collect();
    let mut findings = check_with_files(&root, &config(SQL), &files).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn repeated_sql_scans_match() {
    let files = ["db/views.sql"];
    assert_eq!(
        messages("fail-sql", SQL, &files),
        messages("fail-sql", SQL, &files)
    );
}
