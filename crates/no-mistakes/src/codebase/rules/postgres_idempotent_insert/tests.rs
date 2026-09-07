use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-idempotent-insert/fixture")
            .join(name),
    )
}

fn config_yaml(yaml: &str) -> NoMistakesConfig {
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

fn default_config() -> NoMistakesConfig {
    config_yaml("sqlInclude: [\"sql/**/*.sql\"]")
}

fn run(root: &Path) -> Vec<RuleFinding> {
    check_with_files(root, &default_config(), &[root.join("sql/001.sql")]).unwrap()
}

#[test]
fn flags_insert_without_conflict() {
    let findings = run(&fixture("fail"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("ON CONFLICT"), "{findings:?}");
}

#[test]
fn do_nothing_passes() {
    assert!(run(&fixture("pass-do-nothing")).is_empty());
}

#[test]
fn not_exists_passes() {
    assert!(run(&fixture("pass-not-exists")).is_empty());
}

#[test]
fn volatile_update_fails() {
    let findings = run(&fixture("fail-volatile"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("volatile"), "{findings:?}");
}

#[test]
fn compile_options_default_checks_on() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert!(compiled.check_convergence);
    assert!(compiled.check_volatility);
    assert!(compiled.check_arbiter);
    assert!(compiled.check_triggers);
    assert!(compiled.check_generated);
    assert!(compiled.scan_embedded);
    assert!(compiled.replay_safe.is_empty());
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("sql/disabled.sql");
    let mut findings =
        check_with_files(&root, &default_config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("sql/does-not-exist.sql");
    let error = check_with_files(&root, &default_config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect PostgreSQL"),
        "{error}"
    );
}

#[test]
fn invalid_include_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_yaml("include: ['[']"),
        &[root.join("sql/001.sql")],
    )
    .expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn rejects_unknown_unanalyzable_sql() {
    let error = compile_options(&Options {
        unanalyzable_sql: "fial".into(),
        ..Default::default()
    })
    .err()
    .expect("mode");
    assert!(error.to_string().contains("unanalyzableSql"), "{error}");
}

#[test]
fn psql_standalone_and_ignored_unparseable_inserts() {
    let root = fixture("fail");
    let psql = root.join("sql/001.psql");
    let findings = check_with_files(
        &root,
        &config_yaml("sqlInclude: [\"sql/**/*.psql\"]\nscanEmbedded: false"),
        std::slice::from_ref(&psql),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    let embedded = root.join("src/query.ts");
    let skipped = check_with_files(
        &root,
        &config_yaml("scanEmbedded: false"),
        std::slice::from_ref(&embedded),
    )
    .unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    let unparseable = root.join("sql/unparseable.sql");
    let ignored = check_with_files(
        &root,
        &config_yaml("sqlInclude: [\"sql/**/*.sql\"]\nunanalyzableSql: ignore"),
        std::slice::from_ref(&unparseable),
    )
    .unwrap();
    assert!(ignored.is_empty(), "{ignored:?}");
    let flagged =
        check_with_files(&root, &default_config(), std::slice::from_ref(&unparseable)).unwrap();
    assert!(
        flagged
            .iter()
            .any(|finding| finding.message.contains("replay-safe")
                || finding.message.contains("could not be proven")),
        "{flagged:?}"
    );
}
