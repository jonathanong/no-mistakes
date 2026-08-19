use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-fk-index/fixture")
            .join(name),
    )
}

fn config(extra: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                "sqlInclude: [\"migrations/**/*.sql\"]\n{extra}"
            ))
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, extra: &str) -> Vec<RuleFinding> {
    let files = vec![root.join("migrations/001.sql")];
    check_with_files(root, &config(extra), &files).unwrap()
}

#[test]
fn flags_foreign_key_without_leading_index() {
    let findings = run(&fixture("fail"), "");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("comments.post_id"));
    assert!(findings[0].message.contains("btree/hash"));
}

#[test]
fn accepts_btree_leading_index() {
    assert!(run(&fixture("pass"), "").is_empty());
}

#[test]
fn same_line_directive_exempts_the_fk() {
    assert!(run(&fixture("directive"), "").is_empty());
}

#[test]
fn allowed_column_exempts_the_fk() {
    assert!(run(&fixture("allow"), "allowedColumns: [comments.post_id]\n").is_empty());
}

#[test]
fn stale_allowed_column_is_a_finding() {
    let findings = run(&fixture("stale"), "allowedColumns: [comments.missing_id]\n");
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("stale")));
}

#[test]
fn gin_index_does_not_cover_equality_lookups() {
    let findings = run(&fixture("gin"), "");
    assert_eq!(findings.len(), 1, "{findings:?}");
}
