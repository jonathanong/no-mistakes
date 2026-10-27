use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

// Output cases moved to cli_postgres_snapshot_no_offset. Keep invalid options,
// read errors, compile_options, and the pre-suppression count below.

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/postgres-no-offset")
}

pub(super) fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("fixture").join(scenario)
}

pub(super) fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
        ..Default::default()
    });
    config
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("src/disabled.ts");
    let mut findings = check_with_files(
        &root,
        &config_with_options("{}"),
        std::slice::from_ref(&file),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn invalid_include_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_with_options("include: ['[']"),
        &[root.join("src/query.ts")],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn invalid_exclude_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_with_options("exclude: ['[']"),
        &[root.join("src/query.ts")],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("src/does-not-exist.ts");
    let error = check_with_files(&root, &config_with_options("{}"), &[missing]).expect_err("read");
    assert!(error.to_string().contains("failed to read"), "{error}");
}

#[test]
fn compile_options_honor_overrides() {
    let compiled = compile_options(&Options {
        import_specifier: "@other/db".to_string(),
        executor_names: vec!["run".to_string()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
}

#[test]
fn compile_options_fill_defaults() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert_eq!(
        compiled.embedded.import_specifier,
        EmbeddedSqlOptions::default().import_specifier
    );
    assert_eq!(
        compiled.embedded.executor_names,
        EmbeddedSqlOptions::default().executor_names
    );
}
