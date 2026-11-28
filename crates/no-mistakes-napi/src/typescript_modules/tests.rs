use super::*;
#[test]
fn selected_module_binding_serializes_the_public_report_and_rejects_missing_files() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/typescript-module-facts");
    let json = analyze_typescript_modules_json_impl(
        serde_json::json!({"root": root, "files": ["bindings.ts"]}),
    )
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(report["modules"][0]["complete"], true);
    assert_eq!(report["modules"][0]["imports"].as_array().unwrap().len(), 4);
    let gaps = analyze_typescript_modules_json_impl(serde_json::json!({"root": root, "files": ["dynamic.ts", "semantic-invalid.ts", "missing.ts", "invalid.ts", "unsupported.txt"]})).unwrap();
    let gaps: serde_json::Value = serde_json::from_str(&gaps).unwrap();
    assert!(gaps["modules"]
        .as_array()
        .unwrap()
        .iter()
        .all(|module| module["complete"] == false));
    assert!(analyze_typescript_modules_json_impl(serde_json::json!({})).is_err());
    assert!(analyze_typescript_modules_json_impl(
        serde_json::json!({"root": root.join("bindings.ts"), "files": []})
    )
    .is_err());
    assert!(analyze_typescript_modules_json_impl(
        serde_json::json!({"files": [root.join("empty.ts")]})
    )
    .is_ok());
    assert!(analyze_typescript_modules_json_impl(serde_json::json!({"files": []})).is_ok());
}
