use super::*;

// Output cases moved to cli_gha_snapshot_step_timeouts.

#[test]
fn removed_workflow_file_reports_duplicates_without_false_stale_entries() {
    let options = r#"
allow:
  - step: ".github/workflows/deleted.yml#build#checkout"
    reason: "Former exception."
  - step: ".github/workflows/deleted.yml#build#checkout"
    reason: "Duplicate former exception."
"#;
    // `check` discovers ci.yml. An empty file list is not a CLI invocation.
    let findings = run_files("deleted-workflow", options, &[]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("duplicate"));
}

#[test]
fn missing_workflow_source_is_skipped() {
    assert!(run_files("pass", "{}", &[".github/workflows/missing.yml"]).is_empty());
}
