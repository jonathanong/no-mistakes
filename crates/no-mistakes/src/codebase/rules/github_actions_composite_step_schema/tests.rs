use super::scan::{contains_mapping_key, mapping_key_line, starts_block_scalar, yaml_parse_line};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use serde_yaml::Value;
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-composite-step-schema/fixture")
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

fn default_config() -> NoMistakesConfig {
    config_with_options("{}")
}

fn action_path(root: &Path) -> PathBuf {
    root.join(".github/actions/my-action/action.yml")
}

fn run_on_files(root: &Path, files: &[PathBuf], yaml: &str) -> Vec<RuleFinding> {
    check_with_files(root, &config_with_options(yaml), files).unwrap()
}

fn write_action(root: &Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    path
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
    let tmp = tempfile::tempdir().unwrap();
    let workflow = write_action(
        tmp.path(),
        ".github/workflows/ci.yml",
        "jobs:\n  build:\n    steps:\n      - timeout-minutes: 5\n        run: echo\n",
    );
    let findings = run_on_files(tmp.path(), &[workflow], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn include_override_can_scan_root_action_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        "action.yaml",
        "runs:\n  using: composite\n  steps:\n    - id: only-id\n      timeout-minutes: 1\n      run: echo\n      shell: bash\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "include: [action.yaml]");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("step \"only-id\""),
        "{}",
        findings[0].message
    );
}

#[test]
fn extra_forbidden_keys_can_ban_an_allowed_key() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/env-step/action.yml",
        "runs:\n  using: composite\n  steps:\n    - uses: actions/checkout@main\n      env:\n        FOO: bar\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "extraForbiddenKeys: [env]");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("env"));
}

#[test]
fn allowed_keys_override_replaces_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/custom/action.yml",
        "runs:\n  using: composite\n  steps:\n    - name: Custom\n      timeout-minutes: 3\n      run: echo\n      shell: bash\n",
    );
    let findings = run_on_files(
        tmp.path(),
        &[path],
        "allowedKeys: [name, timeout-minutes, run, shell]",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn allowed_keys_override_flags_default_keys_that_were_dropped() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/custom/action.yml",
        "runs:\n  using: composite\n  steps:\n    - name: Custom\n      uses: actions/checkout@main\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "allowedKeys: [name]");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("uses"));
}

#[test]
fn label_falls_back_to_uses_then_step_number() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/labels/action.yml",
        "\
runs:
  using: composite
  steps:
    - uses: actions/checkout@main
      timeout-minutes: 2
    - run: echo unlabeled
      shell: bash
      timeout-minutes: 3
",
    );
    let findings = run_on_files(tmp.path(), &[path], "{}");
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
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/empty-name/action.yml",
        "runs:\n  using: composite\n  steps:\n    - name: \"  \"\n      id: fallback\n      timeout-minutes: 1\n      run: echo\n      shell: bash\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("step \"fallback\""),
        "{}",
        findings[0].message
    );
}

#[test]
fn disable_file_comment_skips_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/suppressed/action.yml",
        "# no-mistakes-disable-file github-actions-composite-step-schema\n\
         runs:\n  using: composite\n  steps:\n    - name: Hidden\n      timeout-minutes: 9\n      run: echo\n      shell: bash\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn disable_file_comment_skips_invalid_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/bad/action.yml",
        "# no-mistakes-disable-file github-actions-composite-step-schema\nruns: {using: composite, steps: [{{invalid}\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_or_non_array_steps_are_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = write_action(
        tmp.path(),
        ".github/actions/no-steps/action.yml",
        "name: No Steps\nruns:\n  using: composite\n",
    );
    let mapping = write_action(
        tmp.path(),
        ".github/actions/map-steps/action.yml",
        "runs:\n  using: composite\n  steps:\n    first:\n      run: echo\n",
    );
    let findings = run_on_files(tmp.path(), &[missing, mapping], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn scalar_and_missing_runs_are_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let scalar = write_action(
        tmp.path(),
        ".github/actions/scalar/action.yml",
        "just-a-scalar-value\n",
    );
    let no_runs = write_action(
        tmp.path(),
        ".github/actions/no-runs/action.yml",
        "name: Incomplete\ndescription: no runs key\n",
    );
    let node = write_action(
        tmp.path(),
        ".github/actions/js/action.yml",
        "runs:\n  using: node20\n  main: index.js\n  timeout-minutes: 5\n",
    );
    let findings = run_on_files(tmp.path(), &[scalar, no_runs, node], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn null_and_non_mapping_steps_are_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/null-step/action.yml",
        "\
runs:
  using: composite
  steps:
    - ~
    - echo this is a string step
    - name: Real Step
      run: echo hi
      shell: bash
",
    );
    let findings = run_on_files(tmp.path(), &[path], "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn non_string_mapping_keys_are_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/numeric-key/action.yml",
        "runs:\n  using: composite\n  steps:\n    - 1: ignored\n      run: echo\n      shell: bash\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "{}");
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
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/ok/action.yml",
        "runs:\n  using: composite\n  steps: []\n",
    );
    let err =
        check_with_files(tmp.path(), &config_with_options("include: ['[']"), &[path]).unwrap_err();
    assert!(err.to_string().contains("unclosed"), "{err:#}");
}

#[test]
fn using_must_be_the_string_composite() {
    let tmp = tempfile::tempdir().unwrap();
    let quoted = write_action(
        tmp.path(),
        ".github/actions/quoted/action.yml",
        "runs:\n  using: \"composite\"\n  steps:\n    - run: echo\n      shell: bash\n      timeout-minutes: 1\n",
    );
    let sequence = write_action(
        tmp.path(),
        ".github/actions/seq-using/action.yml",
        "runs:\n  using: [composite]\n  steps:\n    - run: echo\n      timeout-minutes: 1\n",
    );
    let findings = run_on_files(tmp.path(), &[quoted, sequence], "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].file.contains("quoted"), "{findings:?}");
}

#[test]
fn flow_style_timeout_still_flags_from_parsed_keys() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_action(
        tmp.path(),
        ".github/actions/flow/action.yml",
        "runs:\n  using: composite\n  steps:\n    - {name: Flow, timeout-minutes: 4, run: echo, shell: bash}\n",
    );
    let findings = run_on_files(tmp.path(), &[path], "{}");
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
