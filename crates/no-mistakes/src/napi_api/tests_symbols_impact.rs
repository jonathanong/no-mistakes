#[test]
fn symbols_json_returns_signature_impact_report() {
    let options = json!({
        "root": fixture_root("tests-impact-symbol"),
        "files": ["utils.mts"],
        "mode": "signature-impact",
        "symbol": "parseDate"
    })
    .to_string();

    let output = symbols_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["symbol"], "parseDate");
    assert!(value["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "other.mts" && entry["symbol"] == "parse" }));
    assert!(value["suggestedTests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "other.test.mts" }));
}
