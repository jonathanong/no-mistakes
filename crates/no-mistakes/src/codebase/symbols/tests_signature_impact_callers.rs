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
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "aliased-local-date-barrel.mts" && entry.get("symbol").is_none()
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

#[test]
fn signature_impact_ignores_import_only_files_without_symbol_usage() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "unused-import.mts" && entry.get("symbol").is_none()
    }));
}

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
}

#[test]
fn signature_impact_keeps_require_file_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "require-caller.mts" && entry.get("symbol").is_none()
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "require-barrel-caller.mts" && entry.get("symbol").is_none()
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "require-alias-caller.mts" && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_does_not_treat_alias_export_name_as_local() {
    let out = run_capture(impact_file_args("aliased-shadow.mts", "parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "aliased-shadow.mts" && entry["symbol"] == "formatShadowDate"
    }));
}

#[test]
fn signature_impact_keeps_private_callers_in_mixed_reexport_files() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "mixed-date-barrel.mts" && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_recovers_workspace_import_private_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "workspace-private-caller.mts" && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_recovers_namespace_reexport_private_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-date-barrel.mts" && entry["symbol"] == "dates"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-barrel-caller.mts" && entry.get("symbol").is_none()
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-barrel-unused-caller.mts"
    }));
}

#[test]
fn signature_impact_does_not_reclassify_excluded_tests_as_production() {
    let mut args = impact_args("parseDate", Format::Json);
    args.config = Some(PathBuf::from("exclude-other-test.no-mistakes.yml"));
    let out = run_capture(args);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "excluded-private-caller.test.mts"
    }));
    assert!(!v["testCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "excluded-private-caller.test.mts"
    }));
}
