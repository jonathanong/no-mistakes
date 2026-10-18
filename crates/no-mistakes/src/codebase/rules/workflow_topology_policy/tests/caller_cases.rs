use super::*;

#[test]
fn caller_allowlist_stale_for_non_callable() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
exactCallerJobs:
  .github/workflows/missing.yml: []
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("caller allowlist stale")),
        "{findings:?}"
    );
}

#[test]
fn lock_intent_missing_and_stale_reason() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
unlockedWorkflowReasons:
  .github/workflows/missing.yml: " "
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("lock intent missing")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("unlocked reason stale")),
        "{findings:?}"
    );
}

#[test]
fn matching_artifact_and_caller_allowlist_on_repository_fixture() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/workflow-topology/repository"),
    );
    let matching = run(
        &root,
        r#"
requiredArtifactEdges:
  - from: ".github/workflows/ci.yml#build"
    to: ".github/workflows/ci.yml#consume"
    name: build-ubuntu-latest
    match: pattern
exactCallerJobs:
  .github/workflows/reusable.yml: [".github/workflows/ci.yml#reusable"]
"#,
    );
    assert!(
        !matching
            .iter()
            .any(|finding| finding.message.contains("required artifact edge missing")),
        "{matching:?}"
    );
    let missing_allowlist = run(
        &root,
        r#"
exactCallerJobs:
  .github/workflows/ci.yml: []
"#,
    );
    assert!(
        missing_allowlist
            .iter()
            .any(|finding| finding.message.contains("caller allowlist missing")),
        "{missing_allowlist:?}"
    );
}

#[test]
fn unlocked_empty_reason_caller_mismatch_and_step_labels() {
    let empty_reason = run(
        &fixture("needs-basic"),
        r#"
unlockedWorkflowReasons:
  .github/workflows/pipeline.yml: ""
"#,
    );
    assert!(
        empty_reason
            .iter()
            .any(|finding| finding.message.contains("unlocked reason empty")),
        "{empty_reason:?}"
    );
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/workflow-topology/repository"),
    );
    let mismatch = run(
        &root,
        r#"
exactCallerJobs:
  .github/workflows/reusable.yml: [".github/workflows/ci.yml#missing"]
unlockedWorkflowReasons:
  .github/workflows/ci.yml: locked
requiredArtifactEdges:
  - from: ".github/workflows/ci.yml#build"
    to: ".github/workflows/ci.yml#consume"
    name: build-ubuntu-latest
"#,
    );
    assert!(
        mismatch
            .iter()
            .any(|finding| finding.message.contains("caller allowlist mismatch")),
        "{mismatch:?}"
    );
    assert!(
        mismatch
            .iter()
            .any(|finding| finding.message.contains("unlocked reason stale")),
        "{mismatch:?}"
    );
    let ordered = fixture("ordered-steps");
    let labeled = run(
        &ordered,
        r#"
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#build"
    steps:
      - uses: actions/checkout@v4
      - name: second
"#,
    );
    assert!(labeled.is_empty(), "{labeled:?}");
    let unlabeled = run(
        &ordered,
        r#"
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#empty"
    steps:
      - {}
"#,
    );
    assert!(
        unlabeled
            .iter()
            .any(|finding| finding.message.contains("required ordered step missing")),
        "{unlabeled:?}"
    );
}
