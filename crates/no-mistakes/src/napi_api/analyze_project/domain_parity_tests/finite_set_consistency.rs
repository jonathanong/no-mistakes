#[test]
fn finite_set_call_literals_match_standalone_and_parse_only_the_call_source_once() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../fixtures/rules/finite-set-consistency/call-literals/napi-parity",
        ),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let options = json!({ "root": root, "config": ".no-mistakes.yml" });
    let standalone = parse_json(crate::napi_api::check_json_impl(options.to_string()).unwrap());

    crate::ast::begin_parse_count(&root);
    let aggregate = parse_json(
        analyze_project_json_impl(
            json!({
                "root": root,
                "config": ".no-mistakes.yml",
                "reports": [{ "type": "check", "id": "check" }]
            })
            .to_string(),
        )
        .unwrap(),
    );
    let counts = crate::ast::finish_parse_count(&root);
    let result = &aggregate["reports"][0]["result"];

    assert_eq!(result, &standalone);
    assert!(result["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["target"] == "reconcileRuntimeGenerations"
    }));
    assert_eq!(counts.get(&root.join("schedules.mts")), Some(&1));
    assert_eq!(counts.len(), 1, "{counts:#?}");
}

#[test]
fn supplemental_skipped_call_sources_preserve_check_scope_and_parse_once() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped",
        ),
    );
    let standalone = parse_json(
        crate::napi_api::check_json_impl(
            json!({ "root": root, "config": ".no-mistakes.yml" }).to_string(),
        )
        .unwrap(),
    );

    crate::ast::begin_parse_count(&root);
    let aggregate = parse_json(
        analyze_project_json_impl(
            json!({
                "root": root,
                "config": ".no-mistakes.yml",
                "reports": [{ "type": "check", "id": "check" }]
            })
            .to_string(),
        )
        .unwrap(),
    );
    let counts = crate::ast::finish_parse_count(&root);
    let result = &aggregate["reports"][0]["result"];

    assert_eq!(result, &standalone);
    assert!(result["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "finite-set-consistency" && finding["target"] == "missing"
    }));
    // The generated call source is available to finite-set extraction only;
    // it must not become a test file for the unrelated dynamic-import rule.
    assert!(result["rules"].as_array().unwrap().iter().all(|finding| {
        finding["rule"] != "test-no-unmocked-dynamic-imports"
    }));
    assert_eq!(
        counts.get(&root.join("generated/schedules.test.mts")),
        Some(&1),
        "supplemental call sources should share the request parse session: {counts:#?}"
    );
}
