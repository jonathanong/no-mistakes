use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn check_file(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    uses_re: &Regex,
    sha_re: &Regex,
    version_re: &Regex,
) -> Vec<RuleFinding> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    check_file_with_sources(
        path,
        root,
        exclude_set,
        uses_re,
        sha_re,
        version_re,
        &sources,
    )
}

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-pinned-hash/fixture")
        .join(path)
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

fn case_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("github-actions-pinned-hash")
            .join("cases")
            .join(name),
    )
}

// Output cases moved to cli_gha_snapshot_pinned_hash.

#[test]
fn non_workflow_yaml_not_checked() {
    let root = case_root("non-workflow-yaml");
    let path = root.join("config.yml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(&path, &root, &exclude_set, &uses_re, &sha_re, &version_re);
    assert!(
        findings.is_empty(),
        "non-workflow yaml should not be checked"
    );
}

#[test]
fn multiple_violations_reported() {
    // Keep valid YAML here; malformed-workflow uncertainty has its own saved regression.
    let root = crate::test_support::rule_fixture_root("file-input-analysis");
    let path = root.join(".github/workflows/multiple-unpinned.yml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let findings = check_file(
        &path,
        &root,
        &build_exclude_globset(&[]),
        &uses_re,
        &sha_re,
        &version_re,
    );
    assert_eq!(findings.len(), 2, "expected 2 violations: {findings:?}");
}

#[test]
fn check_entry_point_uses_discovery() {
    // smoke-test the public `check()` entry point; uses the pass fixture
    // which enables the rule and has only pinned actions
    let root = fixture("pass");
    let config = config_with_options("{}");
    let findings = super::check(&root, &config).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn non_yaml_extension_not_checked() {
    // A file with a non-yml/yaml extension in .github/workflows/ must be skipped
    let root = case_root("non-yaml-extension");
    let path = root.join(".github/workflows/ci.sh");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(&path, &root, &exclude_set, &uses_re, &sha_re, &version_re);
    assert!(
        findings.is_empty(),
        "non-yaml extension should not be checked"
    );
}

#[test]
fn composite_action_yaml_extension_checked() {
    // .github/actions/**/action.yaml (note: .yaml not .yml) must be checked
    let root = case_root("composite-action-yaml");
    let path = root.join(".github/actions/my-action/action.yaml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(&path, &root, &exclude_set, &uses_re, &sha_re, &version_re);
    assert_eq!(findings.len(), 1, "composite action.yaml must be checked");
}

#[test]
fn unreadable_file_reports_unanalyzable_input() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(".github/workflows/missing.yml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(
        &path,
        tmp.path(),
        &exclude_set,
        &uses_re,
        &sha_re,
        &version_re,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn block_header_accepts_one_chomp_or_indent() {
    assert!(yaml_uses::is_block_header("|+2"));
    assert!(yaml_uses::is_block_header("|2+"));
    assert!(yaml_uses::is_block_header("|-"));
    assert!(!yaml_uses::is_block_header("|not"));
    assert!(!yaml_uses::is_block_header("|12"));
    assert!(!yaml_uses::is_block_header("|+-"));
    assert!(!yaml_uses::is_block_header("plain"));
}
