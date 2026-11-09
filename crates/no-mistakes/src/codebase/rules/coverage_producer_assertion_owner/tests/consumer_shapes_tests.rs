#[test]
fn untyped_known_method_leaves_preserve_typed_and_callback_uncertainty() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join("consumer-shapes.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert!(
        !findings
            .iter()
            .any(|finding| finding["target"] == "arrayFindOwner"),
        "{report}"
    );
    for target in [
        "untypedNumber",
        "untypedJoin",
        "untypedHas",
        "unknownNumber",
        "anyNumber",
        "localUnknown",
        "resultUnknown",
        "resultUnknownFactory",
        "arrayFindNoOwner",
        "setHas",
        "unknownCast",
        "anyCast",
        "unknownCastBinding",
    ] {
        let finding = findings
            .iter()
            .find(|finding| finding["target"] == target)
            .unwrap_or_else(|| panic!("missing {target}: {report}"));
        assert!(finding["causes"].is_null(), "{target}: {report}");
    }
    for target in [
        "untypedCustom",
        "typedWrong",
        "customSink",
        "wrongLiteral",
        "unknownFind",
        "castedBinding",
        "directCasted",
        "directAssertion",
    ] {
        let finding = findings
            .iter()
            .find(|finding| finding["target"] == target)
            .unwrap_or_else(|| panic!("missing {target}: {report}"));
        assert!(
            finding["message"]
                .as_str()
                .unwrap()
                .contains("unanalyzable"),
            "{target}: {report}"
        );
    }
    let dispatches = findings
        .iter()
        .map(|finding| finding["causes"].as_array().map_or(0, Vec::len))
        .sum::<usize>();
    assert_eq!(
        dispatches, 8,
        "rejected 73e1dd54 reports 19 unresolved dispatches on this saved fixture: {report}"
    );
}
