use super::*;

#[test]
fn analyze_project_integration_discovery_preserves_check_and_graph_batch_parity() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/discovery-scope/fixture"),
    );
    let result = analyze_project_check_result(&root);
    assert_eq!(result, standalone_check_result(&root));
    let output = analyze_project_value_impl(json!({
        "root": root,
        "config": ".no-mistakes.yml",
        "reports": [
            { "type": "check" },
            {
                "type": "dependents",
                "files": ["helpers/dependency.ts"],
                "relationships": ["import"]
            }
        ]
    }))
    .unwrap();
    let batch: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(result, batch["reports"][0]["result"]);
    assert!(!batch["reports"][1]["result"]["files"]
        .as_array()
        .unwrap()
        .is_empty());
    let findings = result["integration"].as_array().unwrap();
    assert_eq!(findings.len(), 7, "{result:#}");
    for name in [
        "bridge",
        "collision before",
        "collision call",
        "collision export",
    ] {
        assert!(
            findings.iter().any(|finding| finding["testName"] == name),
            "{name}: {result:#}"
        );
    }
    for name in ["ambient imports", "collision after"] {
        assert!(
            !findings.iter().any(|finding| finding["testName"] == name),
            "{name}: {result:#}"
        );
    }
}
