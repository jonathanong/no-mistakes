use super::*;
use serde_yaml::Value;
use std::path::PathBuf;

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-job-timeouts/fixture")
            .join(name),
    )
}

// Output cases moved to cli_gha_snapshot_job_timeouts.

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
fn document_location_parse_failure_returns_no_partial_index() {
    for name in ["invalid", "step-sum-empty"] {
        let source =
            std::fs::read_to_string(fixture(name).join(".github/workflows/ci.yml")).unwrap();
        let locations = super::job_lines::collect(&source);
        assert_eq!(locations.line("jobs"), 1);
    }
    let source =
        std::fs::read_to_string(fixture("step-sum-scalar-jobs").join(".github/workflows/ci.yml"))
            .unwrap();
    assert_eq!(super::job_lines::collect(&source).line("missing"), 2);
}
