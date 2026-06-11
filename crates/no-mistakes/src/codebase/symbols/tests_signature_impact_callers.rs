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

#[test]
fn signature_impact_includes_same_file_default_export_local_callers() {
    let out = run_capture(impact_file_args("default-utils.mts", "default", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "default-utils.mts" && entry["symbol"] == "formatDefaultDate"
    }));
}

#[test]
fn signature_impact_reports_recovered_default_callers_as_default() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "default-wrapper.mts" && entry["symbol"] == "default"
    }));
}

#[test]
fn signature_impact_recovers_private_callers_imported_through_barrels() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "private-barrel-caller-with-export.mts" && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_recovers_private_callers_imported_through_aliased_barrels() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "private-aliased-barrel-caller-with-export.mts"
            && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_suggests_tests_for_recovered_production_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["suggestedTests"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "private-barrel-caller-with-export.test.mts"
    }));
}
