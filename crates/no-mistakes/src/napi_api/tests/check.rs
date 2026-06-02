use super::*;
use serde_json::json;

#[test]
fn check_json_returns_global_check_report() {
    let options = json!({
        "root": fixture_root("unique-exports-basic"),
        "config": ".no-mistakes.yml",
        "tsconfig": "tsconfig.json"
    })
    .to_string();
    let output = check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["codebase"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "unique-exports" && finding["exportName"] == "shared"
    }));
    assert!(value["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn check_json_returns_warnings_for_skipped_configured_check() {
    let options = json!({
        "root": fixture_root("test-no-unmocked-dynamic-imports-unknown-vitest-project"),
    })
    .to_string();
    let output = check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["warnings"].as_array().unwrap().iter().any(|warning| {
        warning
            .as_str()
            .is_some_and(|warning| warning.contains("unknown vitest project web"))
    }));
    assert_eq!(value["rules"].as_array().map(Vec::len), Some(0));
}
