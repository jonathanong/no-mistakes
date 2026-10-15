// Included in `napi_api::tests`; fixtures distinguish ownership from execution.
#[test]
fn effects_json_preserves_default_export_execution_frequency() {
    for (name, expected) in [
        ("default-snapshot-owned", 1),
        ("default-object-snapshot-owned", 1),
        ("default-wrapped-snapshot-owned", 1),
        ("default-restored-snapshot-owned", 1),
        ("default-deferred-arrow", 0),
        ("default-deferred-function", 0),
        ("default-deferred-method", 0),
        ("default-deferred-field", 0),
        ("default-deferred-accessor", 0),
        ("default-deferred-nested", 0),
        ("default-snapshot-write-first", 0),
        ("default-live-receiver", 0),
        ("default-snapshot-async", 0),
        ("default-snapshot-captured", 1),
        ("default-receiver-captured", 0),
        ("default-deferred-computed-method", 0),
        ("default-deferred-computed-function", 0),
        ("default-deferred-computed-arrow", 0),
        ("default-deferred-computed-parameter", 0),
        ("default-deferred-computed-arrow-parameter", 0),
        ("default-deferred-computed-class-method", 0),
        ("default-deferred-computed-getter", 0),
        ("default-deferred-computed-setter", 0),
        ("default-computed-key-snapshot-owned", 1),
        ("default-class-key-snapshot-owned", 1),
    ] {
        let options = json!({
            "root": fixture_root("inline-member-precision"),
            "kind": "cache", "entry": format!("src/{name}.mts")
        }).to_string();
        let output = effects_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        let sites = report["callSites"].as_array().unwrap();
        assert_eq!(sites.len(), expected, "{name}: {report}");
        assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
    }
}
