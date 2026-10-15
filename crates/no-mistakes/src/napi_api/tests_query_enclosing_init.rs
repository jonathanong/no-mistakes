#[test]
fn effects_json_preserves_enclosing_initializer_reachability() {
    let options = json!({
        "root": fixture_root("value-flow-receiver-evaluation-order"),
        "kind": "cache",
        "entry": "src/enclosing-tdz.mts"
    })
    .to_string();
    let output = effects_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let sites = report["callSites"].as_array().unwrap();
    assert_eq!(sites.len(), 5, "{report}");
    assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
}
