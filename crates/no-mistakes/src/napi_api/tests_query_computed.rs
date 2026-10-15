// Included into napi_api::tests; shares the existing prepared public API helpers.

#[test]
fn effects_json_preserves_computed_key_frequency_guards() {
    for (name, expected) in [
        ("default-computed-arrow", 0),
        ("default-computed-function", 0),
        ("default-computed-method", 0),
        ("default-computed-class", 0),
        ("default-computed-parameter", 0),
        ("default-computed-key", 1),
        ("default-computed-class-key", 1),
        ("default-computed-restored", 1),
        ("module-computed-arrow", 0),
    ] {
        let options = json!({
            "root": fixture_root("inline-member-precision"),
            "kind": "cache", "entry": format!("src/{name}.mts")
        })
        .to_string();
        let output = effects_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        let sites = report["callSites"].as_array().unwrap();
        assert_eq!(sites.len(), expected, "{name}: {report}");
        assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
    }
}
