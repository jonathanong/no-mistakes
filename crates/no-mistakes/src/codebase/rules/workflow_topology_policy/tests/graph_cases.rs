use super::*;

#[test]
fn required_direct_edge_is_enforced() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
requiredDirectEdges:
  - [".github/workflows/pipeline.yml#build", ".github/workflows/pipeline.yml#missing"]
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required direct edge missing")),
        "{findings:?}"
    );
}

#[test]
fn required_job_missing_and_forbidden_job_present() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
requiredJobs: [".github/workflows/pipeline.yml#ghost"]
forbiddenJobs: [".github/workflows/pipeline.yml#build"]
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required job missing")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("forbidden job present")),
        "{findings:?}"
    );
}

#[test]
fn forbidden_direct_and_required_transitive_edges() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
forbiddenDirectEdges:
  - [".github/workflows/pipeline.yml#build", ".github/workflows/pipeline.yml#test"]
requiredTransitiveEdges:
  - [".github/workflows/pipeline.yml#build", ".github/workflows/pipeline.yml#ghost"]
forbiddenTransitiveEdges:
  - [".github/workflows/pipeline.yml#build", ".github/workflows/pipeline.yml#deploy"]
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("forbidden direct edge present")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("required transitive edge missing")),
        "{findings:?}"
    );
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("forbidden transitive edge present")),
        "{findings:?}"
    );
}

#[test]
fn exact_fan_in_mismatch_and_missing_target() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
exactFanIns:
  ".github/workflows/pipeline.yml#deploy": [".github/workflows/pipeline.yml#build"]
  ".github/workflows/pipeline.yml#ghost": []
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("exact fan-in mismatch")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("exact fan-in target missing")),
        "{findings:?}"
    );
}

#[test]
fn exact_fan_in_matches() {
    let findings = run(
        &fixture("needs-basic"),
        r#"
exactFanIns:
  ".github/workflows/pipeline.yml#test": [".github/workflows/pipeline.yml#build"]
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}
