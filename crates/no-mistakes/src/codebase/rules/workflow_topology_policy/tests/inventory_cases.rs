use super::*;

#[test]
fn matching_inventory_and_edges_pass() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
jobInventory:
  .github/workflows/pipeline.yml: [build, test, deploy]
requiredDirectEdges:
  - [".github/workflows/pipeline.yml#build", ".github/workflows/pipeline.yml#test"]
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn inventory_mismatch_stale_and_missing() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
jobInventory:
  .github/workflows/pipeline.yml: [build]
  .github/workflows/missing.yml: [job]
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("job inventory mismatch")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("workflow inventory stale")),
        "{findings:?}"
    );
}

#[test]
fn inventory_missing_forbidden_missing_jobs_and_present_transitive() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
jobInventory:
  .github/workflows/missing.yml: [job]
forbiddenDirectEdges:
  - [".github/workflows/pipeline.yml#ghost", ".github/workflows/pipeline.yml#build"]
requiredTransitiveEdges:
  - [".github/workflows/pipeline.yml#build", ".github/workflows/pipeline.yml#deploy"]
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("workflow inventory missing")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("workflow inventory stale")),
        "{findings:?}"
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.message.contains("forbidden direct edge present")),
        "{findings:?}"
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.message.contains("required transitive edge missing")),
        "{findings:?}"
    );
}
