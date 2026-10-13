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

fn run(scenario: &str, options: &str) -> Vec<RuleFinding> {
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
    check_with_files(&root, &config, &[root.join(".github/workflows/ci.yml")]).unwrap()
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
fn nested_list_items_do_not_shift_step_locations() {
    let findings = run("nested", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 13);
    assert!(run("nested-suppressed", "{}").is_empty());
}

#[test]
fn indentless_step_sequence_has_usable_locations() {
    let findings = run("indentless", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert!(run("indentless-suppressed", "{}").is_empty());
}

#[test]
fn parallel_group_checks_nested_executable_steps() {
    let findings = run("parallel", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 7);
    assert_eq!(
        findings[0].target.as_deref(),
        Some(".github/workflows/ci.yml#build#1/frontend")
    );
    assert!(run("parallel-suppressed", "{}").is_empty());
}

#[test]
fn nested_step_allow_uses_group_and_child_selectors() {
    let findings = run(
        "parallel",
        r#"
allow:
  - step: ".github/workflows/ci.yml#build#1/frontend"
    reason: "Covered by an external deadline."
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn job_key_comments_keep_step_locations_and_suppression() {
    let findings = run("commented-job", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("commented-job-suppressed", "{}").is_empty());
}

#[test]
fn workflow_dispatch_input_named_like_job_does_not_shift_step_location() {
    let findings = run("duplicate-job-name", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert!(run("duplicate-job-name-suppressed", "{}").is_empty());
}

#[test]
fn anchored_job_has_usable_step_location() {
    let findings = run("anchored-job", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("anchored-job-suppressed", "{}").is_empty());
}

#[test]
fn aliased_job_reports_inherited_steps_on_alias_line() {
    let findings = run("aliased-job", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert!(run("aliased-job-suppressed", "{}").is_empty());
}

#[test]
fn quoted_and_anchored_mapping_keys_keep_nested_locations() {
    let findings = run("decorated-keys", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert!(run("decorated-keys-suppressed", "{}").is_empty());
}

#[test]
fn flow_style_steps_use_their_declaration_line() {
    let findings = run("flow-steps", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 5);
    assert!(run("flow-steps-suppressed", "{}").is_empty());
}

#[test]
fn anchored_flow_style_steps_use_declaration_line() {
    let findings = run("anchored-flow-steps", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 5);
    assert!(run("anchored-flow-steps-suppressed", "{}").is_empty());
}

#[test]
fn flow_style_parallel_and_jobs_use_declaration_lines() {
    for (scenario, line) in [("flow-parallel", 6), ("flow-jobs", 2)] {
        let findings = run(scenario, "{}");
        assert_eq!(findings.len(), 1, "{scenario}: {findings:?}");
        assert_eq!(findings[0].line, line);
        assert!(run(&format!("{scenario}-suppressed"), "{}").is_empty());
    }
}

#[test]
fn indented_root_mapping_keeps_step_location() {
    let findings = run("indented-root", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("indented-root-suppressed", "{}").is_empty());
}

#[test]
fn matrix_steps_do_not_shift_executable_step_location() {
    let findings = run("matrix-steps", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert!(run("matrix-steps-suppressed", "{}").is_empty());
}

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
fn unmatched_include_is_silent() {
    assert!(run("missing", "include: ['other/*.yml']").is_empty());
}
