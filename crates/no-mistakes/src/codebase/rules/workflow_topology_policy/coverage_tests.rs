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

fn topology_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/workflow-topology")
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
        findings.iter().any(|finding| finding
            .message
            .contains("required transitive edge missing")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("required direct edge missing")),
        "{findings:?}"
    );
}
