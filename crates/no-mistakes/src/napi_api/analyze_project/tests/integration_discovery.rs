use super::*;

#[test]
fn analyze_project_integration_discovery_preserves_cli_parity_and_helper_fallbacks() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/discovery-scope/fixture"),
    );
    let result = analyze_project_check_result(&root);
    assert_eq!(result, standalone_check_result(&root));
    let findings = result["integration"].as_array().unwrap();
    assert_eq!(findings.len(), 7, "{result:#}");
    assert!(findings
        .iter()
        .any(|finding| finding["testName"] == "bridge"));
    assert!(!findings
        .iter()
        .any(|finding| finding["testName"] == "ambient imports"));
}
