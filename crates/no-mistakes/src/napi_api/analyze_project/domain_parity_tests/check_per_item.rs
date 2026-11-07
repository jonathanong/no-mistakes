#[test]
fn per_item_check_matches_standalone_and_preserves_batched_callsite_fields() {
    let saved = repo_fixture(&["test-cases", "rules", "query-reached-per-item", "fixture"]);
    let materialized = crate::test_support::materialize_saved_fixture(&saved);
    let root = materialized.path().canonicalize().unwrap();
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root }).to_string(),
        ))
        .unwrap(),
    );
    assert!(!standalone["rules"].as_array().unwrap().is_empty());
    crate::ast::begin_parse_count(&root);
    let checked = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "reports": [
            { "type": "check", "id": "check" }
        ] })
            .to_string(),
        ))
        .unwrap(),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(checked["reports"][0]["result"], standalone);
    assert_eq!(
        counts.get(&root.join("src/entry.mts")),
        Some(&1),
        "{counts:#?}"
    );
    assert_eq!(
        counts.get(&root.join("src/helpers.mts")),
        Some(&1),
        "{counts:#?}"
    );
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    // The ordinary call-sites query retains its existing symbol projection.
    // Additive output parity is separate from the new check's parse ownership.
    let combined = parse_json(analyze_project_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "root": root, "reports": [
            { "type": "check", "id": "check" },
            { "type": "callSites", "id": "calls", "file": "src/helpers.mts", "exportName": "lookup" }
        ] }).to_string()),
    ).unwrap());
    assert_eq!(combined["reports"][0]["result"], standalone);
    let baseline = parse_json(analyze_project_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "root": root, "reports": [
            { "type": "callSites", "id": "calls", "file": "src/helpers.mts", "exportName": "lookup" }
        ] }).to_string()),
    ).unwrap());
    assert_eq!(combined["reports"][1], baseline["reports"][0]);
}

#[test]
fn bounded_per_item_shapes_match_prepared_checks_with_one_source_pass() {
    for shape in [
        "counter",
        "tuple-enum",
        "chunk",
        "keyset",
        "closed-keys",
        "distinct-calls",
        "dependent-chain",
    ] {
        let saved = repo_fixture(&[
            "test-cases",
            "rules",
            "query-reached-per-item",
            "bounded-shapes",
            shape,
        ]);
        let materialized = crate::test_support::materialize_saved_fixture(&saved);
        let root = materialized.path().canonicalize().unwrap();
        let standalone = parse_json(
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root }).to_string(),
            ))
            .unwrap(),
        );
        assert!(
            !standalone["rules"].as_array().unwrap().is_empty(),
            "{shape}"
        );
        crate::ast::begin_parse_count(&root);
        let prepared = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root, "reports": [
                { "type": "check", "id": "enabled" }
            ] })
                .to_string(),
            ))
            .unwrap(),
        );
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(prepared["reports"][0]["result"], standalone, "{shape}");
        assert_eq!(
            counts.get(&root.join("entry.mts")),
            Some(&1),
            "{shape}: {counts:#?}"
        );
        assert!(
            counts.values().all(|count| *count == 1),
            "{shape}: {counts:#?}"
        );
        let combined = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root, "reports": [
                { "type": "check", "id": "enabled" },
                { "type": "callSites", "id": "calls", "file": "entry.mts", "exportName": "bounded" }
            ] })
                .to_string(),
            ))
            .unwrap(),
        );
        assert_eq!(combined["reports"][0]["result"], standalone, "{shape}");
        let calls = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root, "reports": [
                { "type": "callSites", "id": "calls", "file": "entry.mts", "exportName": "bounded" }
            ] })
                .to_string(),
            ))
            .unwrap(),
        );
        assert_eq!(combined["reports"][1], calls["reports"][0], "{shape}");
    }
}
