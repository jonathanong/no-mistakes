use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use serde_yaml::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-job-timeouts/fixture")
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

fn run(root: &Path, yaml: &str) -> Vec<RuleFinding> {
    let files = vec![root.join(".github/workflows/ci.yml")];
    check_with_files(root, &config(yaml), &files).unwrap()
}

#[test]
fn missing_timeout_is_a_finding() {
    let findings = run(&fixture("fail-missing"), "maxMinutes: 10");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("no timeout-minutes"));
}

#[test]
fn timeout_over_cap_is_a_finding() {
    let findings = run(&fixture("fail-cap"), "maxMinutes: 10");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("over the 10-minute cap"));
}

#[test]
fn literal_timeout_within_cap_passes() {
    assert!(run(&fixture("pass"), "maxMinutes: 10").is_empty());
}

#[test]
fn reusable_workflow_callers_are_skipped() {
    assert!(run(&fixture("reusable"), "maxMinutes: 10").is_empty());
}

#[test]
fn invalid_yaml_is_a_diagnostic() {
    let findings = run(&fixture("invalid"), "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("invalid YAML"));
}

#[test]
fn step_timeout_may_not_exceed_job_timeout() {
    let findings = run(
        &fixture("step-exceeds"),
        "maxMinutes: 10\nrejectStepExceedingJob: true\n",
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("exceeding its job timeout")),
        "{findings:?}"
    );
}

#[test]
fn allow_entry_raises_the_cap_for_one_job() {
    let findings = run(
        &fixture("allow"),
        r#"
maxMinutes: 10
allow:
  - job: ".github/workflows/ci.yml#coverage"
    maxMinutes: 20
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn allow_without_max_accepts_any_timeout() {
    let findings = run(
        &fixture("allow"),
        r#"
maxMinutes: 10
allow:
  - job: ".github/workflows/ci.yml#coverage"
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn allow_entry_still_fails_when_over_its_max() {
    let findings = run(
        &fixture("allow"),
        r#"
maxMinutes: 10
allow:
  - job: ".github/workflows/ci.yml#coverage"
    maxMinutes: 15
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("allowlisted max")),
        "{findings:?}"
    );
}

#[test]
fn stale_allow_entry_is_a_finding() {
    let findings = run(
        &fixture("pass"),
        r#"
maxMinutes: 10
allow:
  - job: ".github/workflows/ci.yml#missing"
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("stale")),
        "{findings:?}"
    );
}

#[test]
fn custom_include_still_scans_workflows() {
    assert!(run(
        &fixture("pass"),
        "include: [\".github/workflows/ci.yml\"]\nmaxMinutes: 10\n"
    )
    .is_empty());
}

#[test]
fn include_without_matches_is_silent() {
    assert!(run(&fixture("pass"), "include: [\"nope.yml\"]").is_empty());
}

#[test]
fn missing_max_minutes_does_not_cap() {
    assert!(run(&fixture("pass"), "{}").is_empty());
}

#[test]
fn disable_file_comment_skips_the_workflow() {
    assert!(run(&fixture("disable"), "maxMinutes: 10").is_empty());
}

#[test]
fn unreadable_workflow_is_skipped() {
    let root = fixture("pass");
    let path = root.join(".github/workflows/missing.yml");
    let sources = super::super::source_store_for_files(&[]);
    let opts = compile_options(Options::default());
    assert!(scan::check_file(&root, &path, &opts, &sources).is_empty());
}

#[test]
fn parsed_workflow_shapes_cover_job_and_step_edges() {
    let source = r#"
jobs:
  1:
    timeout-minutes: 5
  skipped: not-a-mapping
  bare:
    timeout-minutes: 10
  stringy:
    timeout-minutes: "8"
  bad:
    timeout-minutes: true
  stepsy:
    timeout-minutes: 10
    steps:
      - not-a-mapping
      - run: echo
      - id: weird
        timeout-minutes: nope
      - name: slow
        timeout-minutes: 20
"#;
    let opts = compile_options(
        serde_yaml::from_str("maxMinutes: 10\nrejectStepExceedingJob: true\n").unwrap(),
    );
    let value = serde_yaml::from_str(source).unwrap();
    let findings = scan::check_parsed(".github/workflows/ci.yml", source, &value, &opts);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("no supported literal upper bound")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("exceeding its job timeout")),
        "{findings:?}"
    );
}

#[test]
fn workflows_without_jobs_are_silent() {
    let opts = compile_options(Options::default());
    let source = "name: ci\n";
    let value = serde_yaml::from_str(source).unwrap();
    assert!(scan::check_parsed("ci.yml", source, &value, &opts).is_empty());
}

#[test]
fn timeout_minutes_accepts_strings_and_rejects_other_values() {
    assert_eq!(
        yaml::timeout_minutes(&Value::String(" 10 ".to_string())),
        Some(10)
    );
    assert_eq!(yaml::timeout_minutes(&Value::Bool(true)), None);
    assert_eq!(yaml::timeout_minutes(&Value::Number((-1i64).into())), None);
    assert_eq!(yaml::step_label(&serde_yaml::Mapping::new(), 2), "step #3");
    assert_eq!(yaml::key_line("name: ci\n", "missing"), 1);
    let allow = AllowEntry::default();
    assert_eq!(allow.clone().job, allow.job);
}
