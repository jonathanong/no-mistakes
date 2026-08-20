use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/markdown-eval-tests/fixture")
            .join(name),
    )
}

fn config(allow: &[&str]) -> NoMistakesConfig {
    let allow_yaml = if allow.is_empty() {
        String::new()
    } else {
        let entries = allow
            .iter()
            .map(|path| format!("- {path}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("allow:\n{entries}\n")
    };
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!("include: [\"**/*.test.mts\"]\n{allow_yaml}"))
                .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, files: &[&str], allow: &[&str]) -> Vec<RuleFinding> {
    let files: Vec<PathBuf> = files.iter().map(|file| root.join(file)).collect();
    check_with_files(root, &config(allow), &files).unwrap()
}

#[test]
fn flags_markdown_eval_spawn_tests() {
    let root = fixture("fail");
    let findings = run(&root, &["slow.test.mts"], &[]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "slow.test.mts");
    assert!(findings[0].message.contains("eval"));
}

#[test]
fn ignores_spawn_free_markdown_readers() {
    let root = fixture("pass");
    assert!(run(&root, &["content.test.mts"], &[]).is_empty());
}

#[test]
fn exact_allowlist_skips_matching_files() {
    let root = fixture("allow");
    assert!(run(&root, &["slow.test.mts"], &["slow.test.mts"]).is_empty());
}

#[test]
fn stale_allowlist_entry_is_a_finding() {
    let root = fixture("pass");
    let findings = run(&root, &["content.test.mts"], &["missing.test.mts"]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("stale"));
}

#[test]
fn unreadable_eval_candidates_are_skipped() {
    let root = fixture("fail");
    let files = vec![root.join("missing.test.mts")];
    let findings = check_with_files(&root, &config(&[]), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn option_defaults_are_empty() {
    let opts = Options::default();
    assert!(opts.include.is_empty());
    assert!(opts.allow.is_empty());
}
