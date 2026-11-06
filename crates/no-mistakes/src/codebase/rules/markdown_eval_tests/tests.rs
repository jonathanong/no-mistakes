use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

// Output cases moved to cli_mrf_snapshot_markdown_eval_tests. Keep missing-file
// skips and option defaults below.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/markdown-eval-tests/fixture")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("include: [\"**/*.test.mts\"]\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn unreadable_eval_candidates_are_skipped() {
    let root = fixture("fail");
    let files = vec![root.join("missing.test.mts")];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn option_defaults_are_empty() {
    let opts = Options::default();
    assert!(opts.include.is_empty());
    assert!(opts.allow.is_empty());
}

#[test]
fn unselected_and_unreadable_allow_entries_are_not_stale() {
    let root = fixture("fail");
    let mut config = config();
    config.rules[0].options = serde_yaml::from_str(
        "include: ['**/*.test.mts']\nallow: [missing.test.mts, excluded.test.mts]",
    )
    .unwrap();
    let findings = check_with_files(&root, &config, &[root.join("missing.test.mts")]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
