use super::*;

// Output cases moved to cli_gha_snapshot_step_timeouts.

#[test]
fn removed_workflow_file_keeps_allow_audits_active() {
    let options = r#"
allow:
  - step: ".github/workflows/deleted.yml#build#checkout"
    reason: "Former exception."
  - step: ".github/workflows/deleted.yml#build#checkout"
    reason: "Duplicate former exception."
"#;
    // `check` discovers ci.yml. An empty file list is not a CLI invocation.
    assert_eq!(run_files("deleted-workflow", options, &[]).len(), 2);
}

#[test]
fn missing_workflow_source_is_skipped() {
    assert!(run_files("pass", "{}", &[".github/workflows/missing.yml"]).is_empty());
}
