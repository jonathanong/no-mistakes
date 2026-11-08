#[test]
fn node_check_returns_the_same_cause_details_for_findings_advisories_and_failures() {
    let root =
        crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-uncertainty");
    for (config, field, expected) in [
        ("dispatch", "rules", "callee `dispatch`"),
        ("advise", "advisories", "callee `dispatch`"),
        (
            "depth",
            "rules",
            "maxDepth=1; last module src/helper.mts#helper",
        ),
        (
            "nodes",
            "rules",
            "maxNodes=1; last module src/producer.mts#bounded",
        ),
        ("source", "incomplete", "source failure in src/broken.mts"),
    ] {
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": root, "config": root.join(format!("{config}.yml"))
        }));
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        let message = report[field][0]["message"].as_str().unwrap();
        assert!(message.contains(expected), "{config}: {report}");
    }
}
