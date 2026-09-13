use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/workflow-topology-policy/fixture")
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
    check_with_files(root, &config(yaml), &[]).unwrap()
}

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
