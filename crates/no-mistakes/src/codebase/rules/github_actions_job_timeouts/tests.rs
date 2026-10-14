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

#[test]
fn step_sum_bound_is_opt_in() {
    assert!(run(&fixture("step-sum"), "{}").is_empty());
    assert!(run(&fixture("step-sum"), "rejectJobAtOrAboveStepSum: false").is_empty());
}

#[test]
fn step_sum_rejects_equal_and_larger_job_deadlines_only_with_complete_bounds() {
    let findings = run(&fixture("step-sum"), "rejectJobAtOrAboveStepSum: true");
    let mut targets: Vec<_> = findings
        .iter()
        .map(|finding| finding.target.as_deref().unwrap())
        .collect();
    targets.sort_unstable();
    assert_eq!(
        targets,
        ["above", "conditional-matrix-composite", "equal"],
        "{findings:?}"
    );
    for finding in findings {
        let timeout = if finding.target.as_deref() == Some("above") {
            11
        } else {
            10
        };
        assert!(finding
            .message
            .contains(&format!("timeout-minutes is {timeout},")));
        assert!(
            finding.message.contains("step timeout sum of 10"),
            "{finding:?}"
        );
        assert!(finding.message.contains("largest step timeout: 6"));
        assert!(finding.message.contains("choose a job timeout below"));
        assert!(finding.message.contains("tighter total runtime bound"));
    }
}

#[test]
fn allow_entries_do_not_bypass_step_sum_bound() {
    for entry in ["", "    maxMinutes: 30\n"] {
        let findings = run(&fixture("step-sum"), &format!(
            "rejectJobAtOrAboveStepSum: true\nmaxMinutes: 10\nallow:\n  - job: '.github/workflows/ci.yml#above'\n{entry}"
        ));
        let above: Vec<_> = findings
            .iter()
            .filter(|finding| finding.target.as_deref() == Some("above"))
            .collect();
        assert_eq!(above.len(), 1, "{findings:?}");
        assert!(above[0].message.contains("step timeout sum of 10"));
    }
}

#[test]
fn step_sum_respects_file_and_line_suppressions() {
    for name in ["step-sum-disabled", "step-sum-suppressed"] {
        assert!(
            run(&fixture(name), "rejectJobAtOrAboveStepSum: true").is_empty(),
            "{name}"
        );
    }
}

#[test]
fn step_sum_option_composes_with_step_exceeding_job_check() {
    let options = "rejectJobAtOrAboveStepSum: true\nrejectStepExceedingJob: true";
    let findings = run(&fixture("step-sum"), options);
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("step timeout sum"))
            .count(),
        3
    );
    let findings = run(&fixture("step-exceeds"), options);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("exceeding its job timeout"));
}

#[test]
fn quoted_job_keys_preserve_job_locations_and_line_suppression() {
    let findings = run(
        &fixture("step-sum-quoted"),
        "rejectJobAtOrAboveStepSum: true",
    );
    let mut locations: Vec<_> = findings
        .iter()
        .map(|finding| (finding.target.as_deref().unwrap(), finding.line))
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [("anchored", 58), ("visible#colon:job", 51)],
        "{findings:?}"
    );
    // Existing job diagnostics share these locations, including quoted aliases.
    let findings = run(&fixture("step-sum-quoted"), "maxMinutes: 9");
    let mut locations: Vec<_> = findings
        .iter()
        .map(|finding| (finding.target.as_deref().unwrap(), finding.line))
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [("anchored", 58), ("visible#colon:job", 51)],
        "{findings:?}"
    );
}

#[test]
fn job_locations_support_indented_root_and_inline_mapping() {
    let options = "rejectJobAtOrAboveStepSum: true";
    let findings = run(&fixture("step-sum-indented"), options);
    let mut locations: Vec<_> = findings
        .iter()
        .map(|finding| (finding.target.as_deref().unwrap(), finding.line))
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [("anchored", 60), ("visible#colon:job", 53)],
        "{findings:?}"
    );
    let findings = run(&fixture("step-sum-inline"), options);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 3);
}

#[test]
fn non_string_explicit_keys_do_not_steal_real_job_locations() {
    let findings = run(
        &fixture("step-sum-explicit"),
        "rejectJobAtOrAboveStepSum: true",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("real#job"));
    assert_eq!(findings[0].line, 18);
}

#[test]
fn explicit_scalar_job_keys_preserve_locations_and_suppression() {
    for options in ["rejectJobAtOrAboveStepSum: true", "maxMinutes: 9"] {
        let findings = run(&fixture("step-sum-explicit-scalar"), options);
        let mut locations: Vec<_> = findings
            .iter()
            .map(|finding| (finding.target.as_deref().unwrap(), finding.line))
            .collect();
        locations.sort_unstable();
        assert_eq!(
            locations,
            [("following", 57), ("normal", 46), ("visible#colon:job", 38)],
            "{findings:?}"
        );
    }
}
