use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use serde_yaml::Value;
use std::path::{Path, PathBuf};

const OPTIONS: &str = r#"
uses:
  - ./.github/actions/setup-aws
  - aws-actions/configure-aws-credentials@
stepTimeoutMinutes: 2
nestedInput: action-timeout-s
nestedTimeoutSeconds: 90
forbidNestedInComposite: true
"#;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-action-timeout-pair/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, yaml: &str, rels: &[&str]) -> Vec<RuleFinding> {
    let files: Vec<PathBuf> = rels.iter().map(|rel| root.join(rel)).collect();
    check_with_files(root, &config(yaml), &files).unwrap()
}

fn parsed(rel: &str, source: &str, yaml: &str) -> Vec<RuleFinding> {
    let opts = compile_options(serde_yaml::from_str(yaml).unwrap());
    let value = serde_yaml::from_str(source).unwrap();
    scan::check_parsed(rel, source, &value, &opts)
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture("pass");
    assert!(run(
        &root,
        OPTIONS,
        &[
            ".github/workflows/ci.yml",
            ".github/actions/setup-aws/action.yml",
        ]
    )
    .is_empty());
}

#[test]
fn fail_step_requires_literal_timeout_minutes() {
    let findings = run(
        &fixture("fail-step"),
        OPTIONS,
        &[".github/workflows/ci.yml"],
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("timeout-minutes: 2"));
}

#[test]
fn fail_nested_rejects_quoted_nested_timeout() {
    let findings = run(
        &fixture("fail-nested"),
        OPTIONS,
        &[".github/workflows/ci.yml"],
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("action-timeout-s: 90"));
    assert!(findings[0].message.contains("bare number"));
}

#[test]
fn fail_composite_rejects_nested_matching_uses() {
    let findings = run(
        &fixture("fail-composite"),
        OPTIONS,
        &[".github/actions/other-composite/action.yml"],
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("nests a configured action"));
}

#[test]
fn empty_uses_is_silent() {
    assert!(run(
        &fixture("fail-step"),
        "uses: []\nstepTimeoutMinutes: 2\n",
        &[".github/workflows/ci.yml"],
    )
    .is_empty());
}

#[test]
fn invalid_yaml_is_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("ci.yml");
    std::fs::write(&path, "jobs: {broken: [{{").unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let opts = compile_options(serde_yaml::from_str(OPTIONS).unwrap());
    assert!(scan::check_file(tmp.path(), &path, &opts, &sources).is_empty());
}

#[test]
fn quoted_or_wrong_step_timeout_is_a_finding() {
    let source = r#"
jobs:
  deploy:
    steps:
      - name: local
        uses: ./.github/actions/setup-aws
        timeout-minutes: "2"
      - name: oversized
        uses: ./.github/actions/setup-aws
        timeout-minutes: 10
"#;
    let findings = parsed(".github/workflows/ci.yml", source, OPTIONS);
    assert_eq!(findings.len(), 2, "{findings:?}");
}

#[test]
fn direct_third_party_requires_nested_number() {
    let source = r#"
jobs:
  deploy:
    steps:
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
"#;
    let findings = parsed(".github/workflows/ci.yml", source, OPTIONS);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("action-timeout-s")),
        "{findings:?}"
    );
}

#[test]
fn prefix_match_is_case_insensitive_and_strips_slash() {
    let source = r#"
jobs:
  deploy:
    steps:
      - uses: AWS-Actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: 90
      - uses: ./.github/actions/setup-aws/
        timeout-minutes: 2
"#;
    assert!(parsed(".github/workflows/ci.yml", source, OPTIONS).is_empty());
}

#[test]
fn wrapper_composite_still_requires_nested_input() {
    let source = r#"
runs:
  using: composite
  steps:
    - name: inner
      uses: aws-actions/configure-aws-credentials@v4
"#;
    let findings = parsed(".github/actions/setup-aws/action.yml", source, OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("action-timeout-s"));
}

#[test]
fn forbid_nested_false_allows_other_composites() {
    let source = r#"
runs:
  using: composite
  steps:
    - uses: ./.github/actions/setup-aws
"#;
    let yaml = r#"
uses:
  - ./.github/actions/setup-aws
forbidNestedInComposite: false
"#;
    assert!(parsed(".github/actions/other/action.yml", source, yaml).is_empty());
}

#[test]
fn run_text_and_unrelated_uses_are_ignored() {
    let source = r#"
jobs:
  deploy:
    steps:
      - run: echo aws-actions/configure-aws-credentials
      - uses: actions/checkout@v4
      - ~
      - name: ok
        uses: ./.github/actions/setup-aws
        timeout-minutes: 2
"#;
    assert!(parsed(".github/workflows/ci.yml", source, OPTIONS).is_empty());
}

#[test]
fn disable_file_comment_skips() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(".github/workflows/ci.yml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        "# no-mistakes-disable-file github-actions-action-timeout-pair\njobs:\n  deploy:\n    steps:\n      - uses: ./.github/actions/setup-aws\n",
    )
    .unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let opts = compile_options(serde_yaml::from_str(OPTIONS).unwrap());
    assert!(scan::check_file(tmp.path(), &path, &opts, &sources).is_empty());
}

#[test]
fn unreadable_and_include_misses_are_silent() {
    let root = fixture("pass");
    let missing = root.join(".github/workflows/missing.yml");
    let sources = super::super::source_store_for_files(&[]);
    let opts = compile_options(serde_yaml::from_str(OPTIONS).unwrap());
    assert!(scan::check_file(&root, &missing, &opts, &sources).is_empty());
    assert!(run(
        &root,
        "include: [\"nope.yml\"]\nuses: [\"./.github/actions/setup-aws\"]\n",
        &[".github/workflows/ci.yml"]
    )
    .is_empty());
}

#[test]
fn scalar_jobs_and_non_composite_are_silent() {
    assert!(parsed("ci.yml", "name: ci\n", OPTIONS).is_empty());
    assert!(parsed(
        ".github/actions/docker/action.yml",
        "runs:\n  using: docker\n  steps:\n    - uses: ./.github/actions/setup-aws\n",
        OPTIONS
    )
    .is_empty());
}

#[test]
fn yaml_helpers_cover_literals_and_labels() {
    assert_eq!(yaml::literal_u64(&Value::Bool(true)), None);
    assert_eq!(yaml::literal_u64(&Value::Number((-1i64).into())), None);
    assert_eq!(yaml::literal_u64(&Value::Number(2u64.into())), Some(2));
    assert_eq!(yaml::normalize_uses(" ./path/ "), "./path");
    assert_eq!(yaml::step_label(&serde_yaml::Mapping::new(), 2), "step #3");
    assert_eq!(yaml::key_line("name: ci\n", "missing"), 1);
    assert_eq!(yaml::yaml_got(None), "null");
    assert_eq!(yaml::yaml_got(Some(&Value::Bool(true))), "true");
    assert_eq!(yaml::yaml_got(Some(&Value::Number(90u64.into()))), "90");
    assert_eq!(
        yaml::yaml_got(Some(&Value::String("90".to_string()))),
        "\"90\""
    );
    assert_eq!(
        yaml::yaml_got(Some(&Value::Sequence(Vec::new()))),
        "non-literal"
    );
    assert!(Options::default().forbid_nested_in_composite);
    assert!(Options::default().uses.is_empty());
}

#[test]
fn missing_timeout_options_skip_those_checks() {
    let source = r#"
jobs:
  1:
    steps:
      - uses: ./.github/actions/setup-aws
  skipped: not-a-mapping
  nosteps:
    runs-on: ubuntu-slim
  ok:
    steps:
      - not-a-mapping
      - uses: aws-actions/configure-aws-credentials@v4
"#;
    let yaml = r#"
uses:
  - ./.github/actions/setup-aws
  - aws-actions/configure-aws-credentials@
"#;
    assert!(parsed("ci.yml", source, yaml).is_empty());
}
