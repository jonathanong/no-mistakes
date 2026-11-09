#[test]
fn builtins_execute_exact_callback_bindings_and_keep_uncertainty_controls() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join("continuations.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
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

#[cfg(feature = "test-instrumentation")]
#[test]
fn builtin_ownership_does_not_change_ordinary_query_reports() {
    let saved =
        crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let materialized = crate::test_support::materialize_saved_fixture(&saved);
    let root = materialized.path();
    let query = |config: &str| {
        crate::ast::begin_parse_count(root);
        let input = serde_json::json!({"root":root,"config":root.join(config),"reports":[
            {"type":"dependents","files":["src/owner.mts"],"relationships":["call"]},
            {"type":"callSites","file":"src/owner.mts","exportName":"owner"},
            {"type":"check"}
        ]});
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::analyze_project_json_impl(input).unwrap())
                .unwrap();
        let parses = crate::ast::finish_parse_count(root);
        assert!(!parses.is_empty(), "{report}");
        (report["reports"].as_array().unwrap()[..2].to_vec(), parses)
    };
    assert_eq!(query("disabled.yml"), query("continuations.yml"));
}

#[test]
fn native_queries_expose_builtin_callback_ownership_edges() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = serde_json::json!({
        "root":root,
        "files":["src/continuations.mts#promiseImported"],
        "relationships":["assertion-owner"],"includeSymbols":true
    });
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::dependencies_json_impl(input).unwrap()).unwrap();
    assert!(
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["file"] == "src/owner.mts"
                && node["symbol"] == "owner"
                && node["via"] == serde_json::json!(["assertion-owner"])),
        "{report}"
    );
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn builtin_ownership_projection_reuses_prepared_sources() {
    let saved =
        crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let materialized = crate::test_support::materialize_saved_fixture(&saved);
    let root = materialized.path();
    crate::ast::begin_parse_count(root);
    let input = serde_json::json!({"root":root,"config":root.join("continuations.yml"),"reports":[
        {"type":"dependents","files":["src/owner.mts"],"relationships":["call"]},
        {"type":"check"}
    ]});
    crate::napi_api::analyze_project_json_impl(input).unwrap();
    let parses = crate::ast::finish_parse_count(root);
    assert!(!parses.is_empty());
    assert!(parses.values().all(|count| *count == 1), "{parses:?}");
}
