// Included into `napi_api::tests` with the shared fixture and JSON helpers.

#[test]
fn effects_json_distinguishes_loop_setup_from_repeated_positions() {
    for (name, expected) in [
        ("loop-setup-annex.cjs", 0),
        ("loop-setup-async.mts", 0),
        ("loop-setup-for-declaration.mts", 1),
        ("loop-setup-for-expression.mts", 1),
        ("loop-setup-for-update.mts", 0),
        ("loop-setup-function.mts", 0),
        ("loop-setup-in-body.mts", 0),
        ("loop-setup-in-default.mts", 0),
        ("loop-setup-in-rhs.mts", 1),
        ("loop-setup-in-target.mts", 0),
        ("loop-setup-nested-for.mts", 0),
        ("loop-setup-nested-in.mts", 0),
        ("loop-setup-nested-of.mts", 0),
        ("loop-setup-of-body.mts", 0),
        ("loop-setup-of-default.mts", 0),
        ("loop-setup-of-rhs.mts", 1),
        ("loop-setup-of-target.mts", 0),
    ] {
        let options = json!({
            "root": fixture_root("inline-member-precision"),
            "kind": "cache", "entry": format!("src/{name}")
        }).to_string();
        let output = effects_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        let sites = report["callSites"].as_array().unwrap();
        assert_eq!(sites.len(), expected, "{name}: {report}");
        assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
    }
}
