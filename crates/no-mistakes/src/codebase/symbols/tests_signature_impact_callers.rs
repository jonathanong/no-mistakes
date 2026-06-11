#[test]
fn signature_impact_includes_same_file_exported_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "utils.mts" && entry["symbol"] == "parseAndFormatDate"
    }));
}

#[test]
fn signature_impact_includes_private_file_callers_with_exports() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "private-caller-with-export.mts" && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_excludes_barrel_owner_files_from_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "date-barrel.mts" && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_includes_same_file_export_alias_callers() {
    let out = run_capture(impact_args("aliasedParseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "utils.mts" && entry["symbol"] == "formatAliasedDate"
    }));
}

#[test]
fn signature_impact_includes_namespace_private_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "private-namespace-caller-with-export.mts"
            && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_suggests_recovered_private_test_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["testCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "private-caller-with-export.test.mts" && entry.get("symbol").is_none()
    }));
    assert!(v["suggestedTests"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "private-caller-with-export.test.mts"
    }));
    assert!(!v["warnings"].as_array().unwrap().iter().any(|entry| {
        entry["type"] == "no-suggested-tests"
    }));
}
