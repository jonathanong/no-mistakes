#[test]
fn signature_impact_classifies_value_alias_exports_as_exports() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "value-alias-date-barrel.mts" && entry["symbol"] == "parse"
    }));
    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-value-alias-date-barrel.mts" && entry["symbol"] == "parse"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "value-alias-date-barrel.mts"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-value-alias-date-barrel.mts"
    }));
}
