use super::coverage_support::{fixture, run, topology_fixture};

#[test]
fn step_order_invalid_and_missing_by_each_selector() {
    let findings = run(
        &fixture("diamond"),
        r#"
stepOrders:
  - jobId: ".github/workflows/pipeline.yml#a"
    steps:
      - id: second
      - id: first
  - jobId: ".github/workflows/pipeline.yml#a"
    steps:
      - {}
      - {}
  - jobId: ".github/workflows/pipeline.yml#a"
    steps:
      - id: missing-id
  - jobId: ".github/workflows/pipeline.yml#a"
    steps:
      - name: Missing Name
  - jobId: ".github/workflows/pipeline.yml#a"
    steps:
      - uses: actions/missing@v1
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required step order invalid")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required ordered step missing")),
        "{findings:?}"
    );
}

#[test]
fn artifact_edges_present_absent_and_wrong_match_kind() {
    let findings = run(
        &topology_fixture("artifact-basic"),
        r#"
requiredArtifactEdges:
  - from: ".github/workflows/main.yml#first"
    to: ".github/workflows/main.yml#exact"
    name: build-linux
  - from: ".github/workflows/main.yml#first"
    to: ".github/workflows/main.yml#exact"
    name: build-linux
    match: exact
  - from: ".github/workflows/main.yml#first"
    to: ".github/workflows/main.yml#exact"
    name: build-linux
    match: nope
  - from: ".github/workflows/main.yml#first"
    to: ".github/workflows/main.yml#exact"
    name: missing-artifact
"#,
    );
    let missing: Vec<_> = findings
        .iter()
        .filter(|finding| finding.message.contains("required artifact edge missing"))
        .collect();
    assert_eq!(missing.len(), 2, "{findings:?}");
}
