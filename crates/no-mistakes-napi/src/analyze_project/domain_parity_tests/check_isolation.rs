#[test]
fn supplemental_report_roots_cannot_bridge_primary_check_traversals() {
    let fixture = no_mistakes::test_support::materialize_gitignore_fixture(
        "analyze-project-check-supplemental-isolation",
    );
    let root = no_mistakes::codebase::ts_resolver::normalize_path(fixture.path());
    let standalone = parse_json(
        crate::check_json_impl(
            crate::options::test_json_arg(json!({ "root": root, "config": ".no-mistakes.yml" }).to_string(),)
        )
        .unwrap(),
    );

    no_mistakes::ast::begin_parse_count(&root);
    let aggregate = parse_json(
        analyze_project_json_impl(
            crate::options::test_json_arg(json!({
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
            .to_string(),)
        )
        .unwrap(),
    );
    let counts = no_mistakes::ast::finish_parse_count(&root);

    assert_eq!(aggregate["reports"][0]["result"], standalone);
    assert!(standalone["rules"].as_array().unwrap().iter().all(|f| {
        f["rule"] != "forbidden-dependencies" && f["rule"] != "test-no-unmocked-dynamic-imports"
    }));
    // The ignored file is parsed for its explicit report only; it must not
    // become an intermediate node in either check traversal.
    assert_eq!(counts.get(&root.join("ignored/bridge.ts")), Some(&1));
}
