use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-require-fk-on-delete/fixture")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("sqlInclude: [\"migrations/**/*.sql\"]").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path) -> Vec<RuleFinding> {
    let files = vec![root.join("migrations/001.sql")];
    check_with_files(root, &config(), &files).unwrap()
}

#[test]
fn flags_omitted_on_delete() {
    let findings = run(&fixture("fail"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("children.parent_id"));
    assert!(findings[0].message.contains("ON DELETE"), "{findings:?}");
}

#[test]
fn flags_no_action() {
    let findings = run(&fixture("fail-no-action"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("children.parent_id"));
}

#[test]
fn flags_alter_table_fk_without_on_delete() {
    let findings = run(&fixture("fail-alter"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("children.parent_id"));
}

#[test]
fn flags_omitted_on_delete_inside_do_block() {
    let findings = run(&fixture("fail-do-block"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("children.parent_id"));
}

#[test]
fn explicit_on_delete_is_clean() {
    assert!(run(&fixture("pass")).is_empty());
}

#[test]
fn option_defaults_use_schema_sql_include() {
    let compiled = compile_options(&Options::default());
    assert_eq!(
        compiled.schema.sql_include,
        crate::codebase::postgres::PostgresSchemaOptions::default().sql_include
    );
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("migrations/does-not-exist.sql");
    let error = check_with_files(&root, &config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect PostgreSQL"),
        "{error}"
    );
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("migrations/disabled.sql");
    let mut findings = check_with_files(&root, &config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}
