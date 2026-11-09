#[test]
fn json_callbacks_can_reach_assertion_owner() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join("json-reviver.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
    let findings = report["rules"].as_array().unwrap();
    let targets = findings
        .iter()
        .filter_map(|finding| finding["target"].as_str())
        .collect::<Vec<_>>();
    for accepted in ["reviverOwner", "inlineReviverOwner", "stringifyOwner"] {
        assert!(!targets.contains(&accepted), "{report}");
    }
    for rejected in [
        "reviverNoOwner",
        "absentReviver",
        "literalReviver",
        "unknownReviver",
        "stringifyUnknown",
        "stringifyArray",
        "stringifyAbsent",
    ] {
        assert!(targets.contains(&rejected), "{report}");
    }
    let unknown = findings
        .iter()
        .find(|finding| finding["target"] == "unknownReviver")
        .unwrap();
    assert!(
        unknown["message"]
            .as_str()
            .unwrap()
            .contains("unanalyzable"),
        "{report}"
    );
    let unknown = findings
        .iter()
        .find(|finding| finding["target"] == "stringifyUnknown")
        .unwrap();
    assert!(
        unknown["message"]
            .as_str()
            .unwrap()
            .contains("unanalyzable"),
        "{report}"
    );
}
