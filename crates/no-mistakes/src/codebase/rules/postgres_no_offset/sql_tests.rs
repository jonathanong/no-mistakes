use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

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
            options: serde_yaml::from_str(yaml).unwrap(),
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
fn sql_views_and_queries_report_each_offset() {
    let body = messages("fail-sql", SQL, &["db/views.sql"]).join("\n");
    assert!(
        body.contains("OFFSET 0 used as an optimizer fence; use a MATERIALIZED CTE (WITH x AS MATERIALIZED (...))"),
        "{body}"
    );
    assert!(body.contains("do not use SQL OFFSET"), "{body}");
}

#[test]
fn materialized_cte_is_quiet() {
    assert!(messages("pass-sql", SQL, &["db/views.sql"]).is_empty());
}

#[test]
fn default_sql_include_leaves_sql_unscanned() {
    let body = messages("pass-default", "{}", &["db/views.sql"]);
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn sql_outside_sql_include_is_unscanned() {
    let body = messages(
        "skip-sql",
        "sqlInclude: [\"db/views/**/*.sql\"]\n",
        &["db/other.sql"],
    );
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn rows_placeholder_and_two_offsets_are_reported() {
    let body = messages("edges", SQL, &["db/edges.sql"]).join("\n");
    assert!(body.contains("optimizer fence"), "{body}");
    assert!(body.contains("do not use SQL OFFSET"), "{body}");
    assert_eq!(messages("edges", SQL, &["db/edges.sql"]).len(), 4, "{body}");
}

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
fn embedded_offset_zero_uses_the_fence_message() {
    let root = fixture("review-followups");
    let findings = check_with_files(&root, &config("{}"), &[root.join("src/zero.ts")]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("optimizer fence"));
    assert_eq!(findings[0].line, 4);
}

#[test]
fn a_missing_sql_file_is_skipped() {
    let root = fixture("fail-sql");
    let findings = check_with_files(&root, &config(SQL), &[root.join("db/missing.sql")]).unwrap();
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
