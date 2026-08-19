use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
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
