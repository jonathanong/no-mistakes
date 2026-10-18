use super::*;

#[test]
fn step_order_reports_missing_steps() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#build"
    steps:
      - uses: actions/checkout@v4
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required ordered step missing")),
        "{findings:?}"
    );
}

#[test]
fn artifact_edge_and_step_order_job_missing() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
requiredArtifactEdges:
  - from: ".github/workflows/pipeline.yml#build"
    to: ".github/workflows/pipeline.yml#test"
    name: dist
    match: prefix
  - from: ".github/workflows/pipeline.yml#ghost"
    to: ".github/workflows/pipeline.yml#test"
    name: dist
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#ghost"
    steps:
      - name: build
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required artifact edge missing")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("step-order job missing")),
        "{findings:?}"
    );
}

#[test]
fn matching_step_order_passes_and_reversed_order_fails() {
    let ordered = fixture("ordered-steps");
    let passing = run(
        &ordered,
        r#"
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#build"
    steps:
      - id: first
      - id: second
"#,
    );
    assert!(passing.is_empty(), "{passing:?}");
    let reversed = run(
        &ordered,
        r#"
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#build"
    steps:
      - id: second
      - id: first
"#,
    );
    assert!(
        reversed
            .iter()
            .any(|finding| finding.message.contains("required step order invalid")),
        "{reversed:?}"
    );
}
