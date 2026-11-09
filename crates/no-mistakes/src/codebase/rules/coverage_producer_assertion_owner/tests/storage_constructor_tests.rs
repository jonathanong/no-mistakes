#[test]
fn imported_storage_constructor_is_not_an_instance_callback_source() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join("review-storage-constructor.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
    let findings = report["rules"].as_array().unwrap();
    let targets = findings
        .iter()
        .filter_map(|finding| finding["target"].as_str())
        .collect::<Vec<_>>();
    for reported in ["staticRun", "staticExit", "unknownRun"] {
        assert!(targets.contains(&reported), "{report}");
    }
    for accepted in [
        "instanceRun",
        "instanceExit",
        "newInstanceRun",
        "typedInstanceRun",
    ] {
        assert!(!targets.contains(&accepted), "{report}");
    }
    for target in ["staticRun", "staticExit"] {
        let finding = findings
            .iter()
            .find(|finding| finding["target"] == target)
            .unwrap();
        assert!(
            finding["message"]
                .as_str()
                .unwrap()
                .contains("unanalyzable"),
            "{report}"
        );
    }
}

#[test]
fn storage_constructor_uncertainty_respects_ignore_policy() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join("review-storage-constructor-ignore.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 0, "{report}");
}
