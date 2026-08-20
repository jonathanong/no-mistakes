use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-constraint-validate/fixture")
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
fn flags_not_valid_without_validate() {
    let findings = run(&fixture("fail-missing"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("VALIDATE CONSTRAINT"));
}

#[test]
fn flags_validate_without_not_valid() {
    let findings = run(&fixture("fail-orphan"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("NOT VALID"));
}

#[test]
fn paired_not_valid_and_validate_pass() {
    assert!(run(&fixture("pass")).is_empty());
}

#[test]
fn paired_not_valid_inside_do_block_passes() {
    assert!(run(&fixture("pass-do-block")).is_empty());
}

#[test]
fn flags_not_valid_inside_do_block_without_validate() {
    let findings = run(&fixture("fail-do-missing"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("VALIDATE CONSTRAINT"));
}

#[test]
fn option_defaults_use_schema_sql_include() {
    let compiled = compile_options(&Options::default());
    assert_eq!(
        compiled.schema.sql_include,
        crate::codebase::postgres::PostgresSchemaOptions::default().sql_include
    );
}
