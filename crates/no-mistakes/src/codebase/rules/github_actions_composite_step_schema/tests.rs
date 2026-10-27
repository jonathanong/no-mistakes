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

fn action_path(root: &Path) -> PathBuf {
    root.join(".github/actions/my-action/action.yml")
}

fn run_on_files(root: &Path, files: &[PathBuf], yaml: &str) -> Vec<RuleFinding> {
    check_with_files(root, &config_with_options(yaml), files).unwrap()
}

fn case_root(case: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&fixture(&format!("cases/{case}")))
}

#[test]
fn fail_fixture_flags_timeout_minutes() {
    let root = fixture("fail");
    let files = vec![action_path(&root)];
    let findings = run_on_files(&root, &files, "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, ".github/actions/my-action/action.yml");
    assert_eq!(findings[0].target.as_deref(), Some("timeout-minutes"));
    assert!(
        findings[0]
            .message
            .contains("composite action step \"Do something\" sets \"timeout-minutes\""),
        "{}",
        findings[0].message
    );
    assert!(
        findings[0]
            .message
            .contains("set it on the calling workflow step instead"),
        "{}",
        findings[0].message
    );
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture("pass");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn description_fixture_does_not_flag_prose() {
    let root = fixture("description");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn block_scalar_fixture_does_not_flag_documented_key() {
    let root = fixture("block-scalar");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn block_scalar_and_key_fixture_flags_the_real_key_line() {
    let root = fixture("block-scalar-and-key");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("timeout-minutes"));
    assert_eq!(findings[0].line, 9);
}

#[test]
fn disable_next_line_fixture_skips_findings() {
    let root = fixture("disable-next-line");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn disable_line_fixture_skips_findings() {
    let root = fixture("disable-line");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn docker_fixture_does_not_flag_timeout_minutes() {
    let root = fixture("docker");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn invalid_fixture_emits_yaml_diagnostic() {
    let root = fixture("invalid");
    let findings = run_on_files(&root, &[action_path(&root)], "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("invalid YAML"),
        "{}",
        findings[0].message
    );
}

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

#[test]
fn default_include_skips_workflow_yaml() {
    let root = case_root("default-include-skips-workflow");
    let workflow = root.join(".github/workflows/ci.yml");
    let findings = run_on_files(&root, &[workflow], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn include_override_can_scan_root_action_yaml() {
    let root = case_root("include-override-root-action");
    let path = root.join("action.yaml");
    let findings = run_on_files(&root, &[path], "include: [action.yaml]");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("step \"only-id\""),
        "{}",
        findings[0].message
    );
}

#[test]
fn extra_forbidden_keys_can_ban_an_allowed_key() {
    let root = case_root("extra-forbidden-env");
    let path = root.join(".github/actions/env-step/action.yml");
    let findings = run_on_files(&root, &[path], "extraForbiddenKeys: [env]");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("env"));
}

#[test]
fn allowed_keys_override_replaces_defaults() {
    let root = case_root("allowed-keys-keeps-timeout");
    let path = root.join(".github/actions/custom/action.yml");
    let findings = run_on_files(
        &root,
        &[path],
        "allowedKeys: [name, timeout-minutes, run, shell]",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn allowed_keys_override_flags_default_keys_that_were_dropped() {
    let root = case_root("allowed-keys-drops-uses");
    let path = root.join(".github/actions/custom/action.yml");
    let findings = run_on_files(&root, &[path], "allowedKeys: [name]");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("uses"));
}

#[test]
fn label_falls_back_to_uses_then_step_number() {
    let root = case_root("label-fallback");
    let path = root.join(".github/actions/labels/action.yml");
    let findings = run_on_files(&root, &[path], "{}");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(
        findings[0]
            .message
            .contains("step \"actions/checkout@main\""),
        "{}",
        findings[0].message
    );
    assert!(
        findings[1].message.contains("step \"step #2\""),
        "{}",
        findings[1].message
    );
}

#[test]
fn empty_name_falls_back_to_id() {
    let root = case_root("empty-name-fallback");
    let path = root.join(".github/actions/empty-name/action.yml");
    let findings = run_on_files(&root, &[path], "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("step \"fallback\""),
        "{}",
        findings[0].message
    );
}

#[test]
fn disable_file_comment_skips_findings() {
    let root = case_root("disable-file");
    let path = root.join(".github/actions/suppressed/action.yml");
    let findings = run_on_files(&root, &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn disable_file_comment_skips_invalid_yaml() {
    let root = case_root("disable-file-invalid");
    let path = root.join(".github/actions/bad/action.yml");
    let findings = run_on_files(&root, &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_or_non_array_steps_are_skipped() {
    let root = case_root("missing-or-non-array-steps");
    let missing = root.join(".github/actions/no-steps/action.yml");
    let mapping = root.join(".github/actions/map-steps/action.yml");
    let findings = run_on_files(&root, &[missing, mapping], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn scalar_and_missing_runs_are_skipped() {
    let root = case_root("scalar-and-missing-runs");
    let scalar = root.join(".github/actions/scalar/action.yml");
    let no_runs = root.join(".github/actions/no-runs/action.yml");
    let node = root.join(".github/actions/js/action.yml");
    let findings = run_on_files(&root, &[scalar, no_runs, node], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn null_and_non_mapping_steps_are_skipped() {
    let root = case_root("null-and-non-mapping-steps");
    let path = root.join(".github/actions/null-step/action.yml");
    let findings = run_on_files(&root, &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn non_string_mapping_keys_are_ignored() {
    let root = case_root("numeric-key");
    let path = root.join(".github/actions/numeric-key/action.yml");
    let findings = run_on_files(&root, &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

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

#[test]
fn using_must_be_the_string_composite() {
    let root = case_root("using-must-be-composite");
    let quoted = root.join(".github/actions/quoted/action.yml");
    let sequence = root.join(".github/actions/seq-using/action.yml");
    let findings = run_on_files(&root, &[quoted, sequence], "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].file.contains("quoted"), "{findings:?}");
}

#[test]
fn flow_style_timeout_still_flags_from_parsed_keys() {
    let root = case_root("flow-style-timeout");
    let path = root.join(".github/actions/flow/action.yml");
    let findings = run_on_files(&root, &[path], "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("timeout-minutes"));
}

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
