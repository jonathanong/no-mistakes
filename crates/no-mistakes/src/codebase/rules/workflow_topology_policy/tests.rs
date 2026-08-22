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
