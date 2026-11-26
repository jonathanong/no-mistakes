use super::ci_topology_json_impl;
use serde_json::json;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/workflow-topology/concurrency")
}

#[test]
fn mermaid_matches_the_saved_cli_diagram() {
    let root = fixture();
    let output = ci_topology_json_impl(json!({"root": root, "format": "mermaid"})).unwrap();
    let diagram: String = serde_json::from_str(&output).unwrap();
    assert_eq!(
        diagram,
        std::fs::read_to_string(root.join("expected.mmd")).unwrap()
    );
}

#[test]
fn mermaid_rejects_graph_diagnostics_while_json_keeps_them() {
    let root = fixture();
    let input = json!({"root": root, "workflows": ["missing.yml"]});
    let output = ci_topology_json_impl(input.clone()).unwrap();
    let graph: serde_json::Value = serde_json::from_str(&output).unwrap();
    let diagnostic = graph["diagnostics"][0]["message"].as_str().unwrap();
    let mut mermaid = input;
    mermaid["format"] = "mermaid".into();
    let error = ci_topology_json_impl(mermaid).unwrap_err();
    assert!(error.reason.contains(diagnostic));
}
