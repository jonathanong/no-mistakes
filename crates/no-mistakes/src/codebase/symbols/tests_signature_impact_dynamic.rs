#[test]
fn signature_impact_keeps_dynamic_import_file_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| {
            entry["file"] == "dynamic-import-caller.mts" && entry.get("symbol").is_none()
        }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-barrel-caller.mts" && entry.get("symbol").is_none()
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-alias-caller.mts" && entry.get("symbol").is_none()
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-aliased-barrel-caller.mts"
            && entry.get("symbol").is_none()
    }));
    assert!(v["suggestedTests"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-caller.test.mts"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-unused.mts" && entry.get("symbol").is_none()
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-shadowed-member.mts" && entry.get("symbol").is_none()
    }));
    assert!(!v["suggestedTests"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-unused.test.mts"
    }));
    assert!(!v["suggestedTests"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "dynamic-import-test-unused.test.mts"
    }));
}
