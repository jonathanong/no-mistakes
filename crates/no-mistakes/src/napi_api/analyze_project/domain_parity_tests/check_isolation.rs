#[test]
fn supplemental_report_roots_cannot_bridge_primary_check_traversals() {
    let fixture = crate::test_support::materialize_gitignore_fixture(
        "analyze-project-check-supplemental-isolation",
    );
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
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
                "reports": [
                    { "type": "check" },
                    {
                        "type": "dependencies",
                        "files": ["ignored/bridge.ts"],
                        "relationships": ["import"]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap(),
    );
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(aggregate["reports"][0]["result"], standalone);
    assert!(standalone["rules"].as_array().unwrap().iter().all(|finding| {
        finding["rule"] != "forbidden-dependencies"
            && finding["rule"] != "test-no-unmocked-dynamic-imports"
    }));
    // The ignored file is parsed for its explicit report only; it must not
    // become an intermediate node in either check traversal.
    assert_eq!(counts.get(&root.join("ignored/bridge.ts")), Some(&1));
}
