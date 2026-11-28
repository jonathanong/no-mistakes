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

#[test]
fn scan_reports_eval_findings_suppresses_allowed_and_flags_stale_allow_entries() {
    let fail_root = fixture("fail");
    let eval_file = crate::codebase::ts_source::discover_files(&fail_root, &[])
        .into_iter()
        .find(|p| p.to_string_lossy().ends_with(".test.mts"))
        .unwrap();
    let rel = relative_slash_path(&fail_root, &eval_file);
    let findings =
        check_with_files(&fail_root, &config(), std::slice::from_ref(&eval_file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");

    let mut allowed = config();
    allowed.rules[0].options =
        serde_yaml::from_str(&format!("include: ['**/*.test.mts']\nallow: ['{rel}']")).unwrap();
    assert!(check_with_files(&fail_root, &allowed, &[eval_file])
        .unwrap()
        .is_empty());

    let pass_root = fixture("pass");
    let pass_file = crate::codebase::ts_source::discover_files(&pass_root, &[])
        .into_iter()
        .find(|p| p.to_string_lossy().ends_with(".test.mts"))
        .unwrap();
    let pass_rel = relative_slash_path(&pass_root, &pass_file);
    let mut stale = config();
    stale.rules[0].options = serde_yaml::from_str(&format!(
        "include: ['**/*.test.mts']\nallow: ['{pass_rel}']"
    ))
    .unwrap();
    let stale_findings = check_with_files(&pass_root, &stale, &[pass_file]).unwrap();
    assert_eq!(stale_findings.len(), 1, "{stale_findings:?}");
    assert!(stale_findings[0]
        .message
        .contains("stale markdown-eval-tests allowlist entry"));
}
