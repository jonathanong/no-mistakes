use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-step-timeouts/fixture")
            .join(scenario),
    )
}

fn run_files(scenario: &str, options: &str, files: &[&str]) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = files.iter().map(|file| root.join(file)).collect::<Vec<_>>();
    check_with_files(&root, &config, &files).unwrap()
}

fn run(scenario: &str, options: &str) -> Vec<RuleFinding> {
    run_files(scenario, options, &[".github/workflows/ci.yml"])
}

#[test]
fn requires_run_and_uses_step_timeouts() {
    let findings = run("missing", "{}");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some(".github/workflows/ci.yml#build#checkout")));
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some(".github/workflows/ci.yml#build#2")));
    assert!(findings.iter().all(|f| f.line > 1));
}

#[test]
fn blocking_wait_steps_require_bounded_timeouts() {
    let missing = run("wait-missing", "{}");
    assert_eq!(missing.len(), 2, "{missing:?}");
    assert_eq!(missing.iter().map(|f| f.line).collect::<Vec<_>>(), [10, 11]);
    assert!(run("wait-valid", "{}").is_empty());
    let invalid = run("wait-invalid", "{}");
    assert_eq!(invalid.len(), 2, "{invalid:?}");
}

#[test]
fn literals_within_cap_pass_and_reusable_callers_are_skipped() {
    assert!(run("pass", "maxMinutes: 30").is_empty());
}

#[test]
fn invalid_zero_expression_and_over_cap_are_findings() {
    let findings = run("invalid", "maxMinutes: 10");
    assert_eq!(findings.len(), 5, "{findings:?}");
    assert!(findings
        .iter()
        .any(|f| f.message.contains("expressions need a reasoned allow")));
    assert!(findings
        .iter()
        .any(|f| f.message.contains("over the 10-minute cap")));
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some(".github/workflows/ci.yml#build#quoted")));
}

#[test]
fn github_ceiling_applies_even_with_a_higher_configured_cap() {
    let findings = run("invalid", "maxMinutes: 999");
    assert!(findings
        .iter()
        .any(|f| f.message.contains("360-minute cap")));
}

#[test]
fn reasoned_allow_accepts_expression_and_can_raise_cap() {
    let findings = run(
        "allowed",
        r#"
maxMinutes: 10
allow:
  - step: ".github/workflows/ci.yml#build#dynamic"
    reason: "The matrix bounds this value."
  - step: ".github/workflows/ci.yml#build#2"
    reason: "Slow external service."
    maxMinutes: 20
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn allow_selector_preserves_hash_in_workflow_filename() {
    let findings = run_files(
        "hash-workflow",
        r#"
allow:
  - step: ".github/workflows/release#nightly.yml#build#dynamic"
    reason: "Matrix value is bounded."
"#,
        &[".github/workflows/release#nightly.yml"],
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn blank_and_stale_allow_entries_are_findings() {
    let findings = run(
        "allowed",
        r#"
allow:
  - step: ".github/workflows/ci.yml#build#dynamic"
    reason: " "
  - step: ".github/workflows/ci.yml#build#absent"
    reason: "No longer needed."
"#,
    );
    assert!(findings
        .iter()
        .any(|f| f.message.contains("needs a reason")));
    assert!(findings.iter().any(|f| f.message.contains("stale")));
}

#[test]
fn allow_for_literal_within_default_cap_is_stale() {
    let findings = run(
        "pass",
        r#"
maxMinutes: 30
allow:
  - step: ".github/workflows/ci.yml#build#checkout"
    reason: "No longer needed."
    maxMinutes: 40
"#,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("stale"));
}

#[test]
fn lower_allow_cap_is_active_without_a_stale_finding() {
    let findings = run(
        "lower-cap",
        r#"
maxMinutes: 30
allow:
  - step: ".github/workflows/ci.yml#build#slow"
    reason: "This step has a stricter deadline."
    maxMinutes: 10
"#,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("over the 10-minute cap"));
}

#[test]
fn allow_does_not_accept_invalid_literal_timeouts() {
    let findings = run(
        "invalid",
        r#"
allow:
  - step: ".github/workflows/ci.yml#build#zero"
    reason: "Test deliberate invalid input."
  - step: ".github/workflows/ci.yml#build#bool"
    reason: "Test deliberate invalid input."
  - step: ".github/workflows/ci.yml#build#quoted"
    reason: "Test deliberate invalid input."
"#,
    );
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some(".github/workflows/ci.yml#build#zero")));
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some(".github/workflows/ci.yml#build#bool")));
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some(".github/workflows/ci.yml#build#quoted")));
}

#[test]
fn duplicate_allow_entries_are_findings() {
    let findings = run(
        "allowed",
        r#"
allow:
  - step: ".github/workflows/ci.yml#build#dynamic"
    reason: "First."
  - step: ".github/workflows/ci.yml#build#dynamic"
    reason: "Second."
"#,
    );
    assert!(findings.iter().any(|f| f.message.contains("duplicate")));
}

#[test]
fn incomplete_allow_selector_has_one_malformed_finding() {
    let findings = run(
        "pass",
        r#"
allow:
  - step: ".github/workflows/ci.yml#build"
    reason: "Missing step selector."
"#,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("malformed"));
    assert!(!findings[0].message.contains("stale"));
}

mod locations;

#[test]
fn disable_file_and_line_suppressions_work() {
    assert!(run("disable-file", "{}").is_empty());
    assert!(run("disable-line", "{}").is_empty());
}

#[test]
fn deferred_suppression_retains_findings_for_audit_reports() {
    for scenario in [
        "disable-file",
        "disable-line",
        "parallel-suppressed",
        "commented-job-suppressed",
    ] {
        let root = fixture(scenario);
        let files = [root.join(".github/workflows/ci.yml")];
        let sources = super::super::source_store_for_files(&files);
        let config = NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.into(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str("{}").unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let findings =
            check_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
        assert_eq!(findings.len(), 1, "{scenario}: {findings:?}");
    }
}

#[test]
fn invalid_yaml_reports_a_diagnostic() {
    let findings = run("bad-yaml", "{}");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("invalid YAML"));
}

#[test]
fn invalid_yaml_still_reports_duplicate_allow_entries() {
    let findings = run(
        "bad-yaml",
        r#"
allow:
  - step: ".github/workflows/ci.yml#build#checkout"
    reason: "Former exception."
  - step: ".github/workflows/ci.yml#build#checkout"
    reason: "Duplicate former exception."
"#,
    );
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("invalid YAML")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("duplicate")));
}

#[test]
fn unmatched_include_is_silent() {
    assert!(run("missing", "include: ['other/*.yml']").is_empty());
}
