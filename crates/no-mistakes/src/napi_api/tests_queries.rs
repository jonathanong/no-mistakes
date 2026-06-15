use super::queries::{
    call_sites_json_impl, dead_exports_json_impl, exports_of_json_impl, importers_json_impl,
    resolve_check_json_impl,
};

#[test]
fn importers_json_lists_direct_importers() {
    let options = json!({ "file": "util.ts", "root": fixture_root("queries") }).to_string();
    let output = importers_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["dependentsCount"], 3);
}

#[test]
fn exports_of_json_skips_importers_when_requested() {
    let options = json!({
        "file": "util.ts",
        "root": fixture_root("queries"),
        "noImporters": true,
    })
    .to_string();
    let output = exports_of_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["exports"][0]["importers"], json!([]));
}

#[test]
fn dead_exports_json_flags_dead() {
    let options = json!({ "file": "util.ts", "root": fixture_root("queries") }).to_string();
    let output = dead_exports_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["anyDead"], true);
}

#[test]
fn call_sites_json_reports_sites() {
    let options = json!({
        "file": "util.ts",
        "exportName": "used",
        "root": fixture_root("queries"),
    })
    .to_string();
    let output = call_sites_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["callSites"].as_array().unwrap().len(), 4);
}

#[test]
fn resolve_check_json_reports_unresolved() {
    let options = json!({ "file": "broken.ts", "root": fixture_root("queries") }).to_string();
    let output = resolve_check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["allResolve"], false);
}

#[test]
fn query_impls_require_inputs() {
    let missing_file = importers_json_impl(json!({}).to_string()).unwrap_err();
    assert!(missing_file.reason.contains("file is required"));

    let missing_export =
        call_sites_json_impl(json!({ "file": "util.ts" }).to_string()).unwrap_err();
    assert!(missing_export.reason.contains("exportName is required"));
}
