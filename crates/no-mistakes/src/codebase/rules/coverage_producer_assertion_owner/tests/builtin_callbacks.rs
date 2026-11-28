#[test]
fn builtins_execute_exact_callback_bindings_and_keep_uncertainty_controls() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = serde_json::json!({
        "root": root, "config": root.join("continuations.yml")
    });
    let report: serde_json::Value =
        serde_json::from_str(&crate::test_support::check_json(input).unwrap()).unwrap();
    let findings = report["rules"].as_array().unwrap();
    let targets: Vec<_> = findings
        .iter()
        .map(|f| f["target"].as_str().unwrap())
        .collect();
    assert_eq!(
        targets,
        [
            "leaves",
            "unresolved",
            "unresolvedCall",
            "storedMap",
            "storedArray",
            "unknownReceiver",
            "collision",
            "registered",
            "callbackNoOwner",
            "spreadContinuation",
            "emptyNumbers",
            "emptyTexts",
            "foreignRun",
            "shadowed",
            "promiseType",
            "arrayType",
            "storageType",
            "numericMap",
            "plainRun",
            "mapThen",
            "stringFinally",
            "conditionalCallback",
            "unknownCallback",
            "absentCallback",
            "nonCallableCallback",
            "replaceLeaf",
            "emptyArray",
            "emptyAlias",
            "emptyChain",
            "arrayJoinMap",
            "promiseSliceFinally",
            "arrayIteratorMap",
            "factoryValue",
            "repeatedNoOwner",
        ],
        "{report}"
    );
    assert!(findings[0].get("causes").is_none(), "{report}");
    let registered = findings
        .iter()
        .find(|f| f["target"] == "registered")
        .unwrap();
    assert!(registered.get("causes").is_none(), "{report}");
    let no_owner = findings
        .iter()
        .find(|f| f["target"] == "callbackNoOwner")
        .unwrap();
    assert!(no_owner.get("causes").is_none(), "{report}");
    for target in [
        "absentCallback",
        "nonCallableCallback",
        "replaceLeaf",
        "emptyArray",
        "emptyAlias",
        "emptyChain",
        "emptyNumbers",
        "emptyTexts",
        "factoryValue",
        "repeatedNoOwner",
    ] {
        let finding = findings
            .iter()
            .find(|finding| finding["target"] == target)
            .unwrap();
        assert!(finding.get("causes").is_none(), "{report}");
    }
    for finding in findings.iter().filter(|f| {
        !matches!(
            f["target"].as_str(),
            Some(
                "leaves"
                    | "registered"
                    | "callbackNoOwner"
                    | "absentCallback"
                    | "nonCallableCallback"
                    | "replaceLeaf"
                    | "emptyArray"
                    | "emptyAlias"
                    | "emptyChain"
                    | "emptyNumbers"
                    | "emptyTexts"
                    | "factoryValue"
                    | "repeatedNoOwner"
            )
        )
    }) {
        assert!(
            finding["message"]
                .as_str()
                .unwrap()
                .contains("unanalyzable"),
            "{report}"
        );
        assert!(
            !finding["causes"].as_array().unwrap().is_empty(),
            "{report}"
        );
    }
    let collision = findings
        .iter()
        .find(|f| f["target"] == "collision")
        .unwrap();
    assert_eq!(collision["causes"][0]["callee"], "<unknown>.finally");
    assert_eq!(
        collision["causes"][0]["detail"],
        "built-in callback `fn` is not statically resolved"
    );
}
