#[test]
fn signature_impact_recovers_namespace_reexport_private_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-date-barrel.mts" && entry["symbol"] == "dates"
    }));
    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-outer-through-reexport-date-barrel.mts"
            && entry["symbol"] == "dates"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-barrel-caller.mts" && entry.get("symbol").is_none()
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-outer-through-reexport-caller.mts"
            && entry.get("symbol").is_none()
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-barrel-unused-caller.mts"
    }));
}

#[test]
fn signature_impact_recovers_default_namespace_private_callers() {
    let out = run_capture(impact_file_args("default-source.mts", "default", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-default-source.mts" && entry["symbol"] == "dates"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-default-caller.mts" && entry.get("symbol").is_none()
    }));
}

#[test]
fn signature_impact_recovers_local_namespace_export_private_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-local-date-barrel.mts" && entry["symbol"] == "dates"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-local-barrel-caller.mts" && entry.get("symbol").is_none()
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-local-barrel-unused-caller.mts"
    }));
}
