#[test]
fn signature_impact_treats_aliased_local_barrels_as_exports() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "aliased-local-date-barrel.mts" && entry["symbol"] == "parse"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "aliased-local-date-barrel.mts" && entry["symbol"] == "parse"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "aliased-local-barrel-consumer.mts"
            && entry["symbol"] == "parseAliasedDate"
    }));
}

#[test]
fn signature_impact_treats_same_name_local_barrels_as_exports() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "same-name-local-date-barrel.mts" && entry["symbol"] == "parseDate"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "same-name-local-date-barrel.mts" && entry["symbol"] == "parseDate"
    }));
}
