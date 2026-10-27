use super::scan::source_lines::{contains_mapping_key, starts_block_scalar};
use super::scan::{mapping_key_line, yaml_parse_line};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub(super) fn fixture(path: &str) -> PathBuf {
    crate::test_support::rule_fixture_root("github-actions-composite-step-schema").join(path)
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

fn run_on_files(root: &Path, files: &[PathBuf], yaml: &str) -> Vec<RuleFinding> {
    check_with_files(root, &config_with_options(yaml), files).unwrap()
}

fn case_root(case: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&fixture(&format!("cases/{case}")))
}

// Output cases moved to cli_gha_snapshot_composite_step_schema.

#[test]
fn check_entry_point_discovers_fail_fixture() {
    let root = fixture("fail");
    let findings = check(&root, &default_config()).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.target.as_deref() == Some("timeout-minutes")),
        "{findings:?}"
    );
}

// Output cases moved to cli_gha_snapshot_composite_step_schema.

#[test]
fn unreadable_file_produces_no_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(".github/actions/missing/action.yml");
    let findings = run_on_files(tmp.path(), &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn invalid_include_glob_is_an_error() {
    let root = case_root("invalid-include-glob");
    let path = root.join(".github/actions/ok/action.yml");
    let err = check_with_files(&root, &config_with_options("include: ['[']"), &[path]).unwrap_err();
    assert!(err.to_string().contains("unclosed"), "{err:#}");
}

// Output cases moved to cli_gha_snapshot_composite_step_schema.

#[test]
fn mapping_key_line_falls_back_when_key_text_is_absent() {
    assert_eq!(mapping_key_line("name: none\n", "timeout-minutes", 0), 1);
    assert!(!contains_mapping_key(
        "description: mention timeout-minutes here",
        "timeout-minutes"
    ));
    assert!(contains_mapping_key(
        "      timeout-minutes: 5",
        "timeout-minutes"
    ));
    assert!(!contains_mapping_key(
        "      my-timeout-minutes: 5",
        "timeout-minutes"
    ));
    assert!(!contains_mapping_key(
        "      timeout-minutes-extra: 5",
        "timeout-minutes"
    ));
    assert!(contains_mapping_key(
        "      timeout-minutes : 5",
        "timeout-minutes"
    ));
}

#[test]
fn mapping_key_line_skips_block_scalar_bodies() {
    let source = "\
description: |
  timeout-minutes:
    documented
timeout-minutes: 5
";
    assert_eq!(mapping_key_line(source, "timeout-minutes", 0), 4);
    let folded = "\
description: >
  timeout-minutes: documented
timeout-minutes: 7
";
    assert_eq!(mapping_key_line(folded, "timeout-minutes", 0), 3);
    assert!(starts_block_scalar("description: |-"));
    assert!(starts_block_scalar("description: |+2"));
    assert!(starts_block_scalar("description: >-"));
    assert!(!starts_block_scalar("timeout-minutes: 5"));
    assert!(!starts_block_scalar("url: https://example.com"));
}

#[test]
fn yaml_parse_line_uses_location_or_defaults_to_one() {
    let located = serde_yaml::from_str::<Value>("[unclosed").unwrap_err();
    assert!(yaml_parse_line(&located) >= 1);
}

#[test]
fn comment_only_timeout_minutes_is_not_a_mapping_key() {
    assert!(!contains_mapping_key(
        "      # timeout-minutes: 5",
        "timeout-minutes"
    ));
}
