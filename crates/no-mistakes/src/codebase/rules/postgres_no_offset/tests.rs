use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/postgres-no-offset")
}

fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("fixture").join(scenario)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn default_config() -> NoMistakesConfig {
    config_with_options("{}")
}

fn ts_file(root: &Path) -> PathBuf {
    root.join("src/query.ts")
}

fn findings_for(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    let file = ts_file(&root);
    check_with_files(&root, &default_config(), &[file]).unwrap()
}

#[test]
fn fail_fixture_reports_offset() {
    let findings = findings_for("fail");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, "src/query.ts");
    assert!(findings[0].line > 0);
    assert_eq!(findings[0].target.as_deref(), Some("offset"));
    assert!(findings[0].message.contains("OFFSET"), "{findings:#?}");
}

#[test]
fn interpolated_offset_is_reported() {
    let findings = findings_for("fail-placeholder");
    assert_eq!(findings.len(), 1, "{findings:#?}");
}

#[test]
fn limit_without_offset_is_clean() {
    assert!(findings_for("pass-limit").is_empty());
}

#[test]
fn offset_in_string_literal_is_clean() {
    assert!(findings_for("pass-prose").is_empty());
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("src/disabled.ts");
    let mut findings =
        check_with_files(&root, &default_config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn include_and_exclude_globs_filter_files() {
    let root = fixture("fail");
    let files = vec![ts_file(&root)];
    let excluded = check_with_files(
        &root,
        &config_with_options("exclude: ['src/query.ts']"),
        &files,
    )
    .unwrap();
    assert!(excluded.is_empty(), "{excluded:#?}");

    let included = check_with_files(
        &root,
        &config_with_options("include: ['src/query.ts']"),
        &files,
    )
    .unwrap();
    assert_eq!(included.len(), 1);

    let missed = check_with_files(
        &root,
        &config_with_options("include: ['src/missing.ts']"),
        &files,
    )
    .unwrap();
    assert!(missed.is_empty(), "{missed:#?}");
}

#[test]
fn custom_executor_and_specifier_are_required_to_match() {
    let root = fixture("fail");
    let files = vec![ts_file(&root)];
    let other = check_with_files(
        &root,
        &config_with_options("importSpecifier: '@other/db'\nexecutorNames: [run]"),
        &files,
    )
    .unwrap();
    assert!(other.is_empty(), "{other:#?}");
}

#[test]
fn invalid_include_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_with_options("include: ['[']"),
        &[ts_file(&root)],
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
        &[ts_file(&root)],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("src/does-not-exist.ts");
    let error = check_with_files(&root, &default_config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect embedded SQL"),
        "{error}"
    );
}

#[test]
fn missing_sql_text_is_ignored() {
    let call = crate::codebase::postgres::EmbeddedSqlCall {
        line: 1,
        callee: "query".to_string(),
        sql_text: None,
    };
    assert!(super::scan::findings_for_call("src/query.ts", &call).is_empty());
}

#[test]
fn unparseable_sql_is_ignored() {
    let call = crate::codebase::postgres::EmbeddedSqlCall {
        line: 1,
        callee: "query".to_string(),
        sql_text: Some("SELECT id FROM posts OFFSET".to_string()),
    };
    assert!(super::scan::findings_for_call("src/query.ts", &call).is_empty());
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
