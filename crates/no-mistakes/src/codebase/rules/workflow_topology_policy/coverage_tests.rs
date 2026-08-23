use super::coverage_support::{fixture, run, topology_fixture};

#[test]
fn workflow_inventory_missing_when_actual_path_is_unlisted() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
jobInventory:
  .github/workflows/other.yml: [job]
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("workflow inventory missing")),
        "{findings:?}"
    );
}

#[test]
fn forbidden_edges_are_silent_when_an_endpoint_is_missing() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
forbiddenDirectEdges:
  - [".github/workflows/pipeline.yml#ghost", ".github/workflows/pipeline.yml#build"]
forbiddenTransitiveEdges:
  - [".github/workflows/pipeline.yml#build", ".github/workflows/pipeline.yml#ghost"]
"#,
    );
    assert!(
        findings.iter().all(|finding| {
            !finding.message.contains("forbidden direct edge")
                && !finding.message.contains("forbidden transitive edge")
        }),
        "{findings:?}"
    );
}

#[test]
fn diamond_covers_transitive_revisit_and_valid_step_order() {
    let findings = run(
        &fixture("diamond"),
        r#"
requiredTransitiveEdges:
  - [".github/workflows/pipeline.yml#a", ".github/workflows/pipeline.yml#d"]
forbiddenDirectEdges:
  - [".github/workflows/pipeline.yml#a", ".github/workflows/pipeline.yml#d"]
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#a"
    steps:
      - id: first
        uses: actions/checkout@v4
        name: First
      - id: second
        name: Second
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn caller_allowlist_missing_and_mismatch() {
    let findings = run(
        &topology_fixture("reusable-calls"),
        r#"
exactCallerJobs:
  .github/workflows/reusable-callee.yml: []
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("caller allowlist missing")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("caller allowlist mismatch")),
        "{findings:?}"
    );
}

#[test]
fn unlocked_reason_stale_and_empty_on_locked_workflow() {
    let findings = run(
        &topology_fixture("concurrency"),
        r#"
unlockedWorkflowReasons:
  .github/workflows/pipeline.yml: "   "
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("unlocked reason stale")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("unlocked reason empty")),
        "{findings:?}"
    );
}

#[test]
fn self_cycle_job_is_skipped_in_transitive_walk() {
    let findings = run(
        &topology_fixture("job-cycle"),
        r#"
requiredTransitiveEdges:
  - [".github/workflows/pipeline.yml#self", ".github/workflows/pipeline.yml#a"]
requiredDirectEdges:
  - [".github/workflows/pipeline.yml#a", ".github/workflows/pipeline.yml#b"]
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required transitive edge missing")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("required direct edge missing")),
        "{findings:?}"
    );
}
