#[test]
fn same_root_check_playwright_and_graph_reports_share_one_parse_pass() {
    let source = parser_fixture("playwright");
    let fixture = no_mistakes::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = [
        parse_json(crate::check_json_impl(crate::options::test_json_arg(json!({ "root": root }).to_string())).unwrap()),
        parse_json(
            crate::playwright_check_json_impl(
                crate::options::test_json_arg(json!({ "root": root, "assertUniqueHtmlIds": true }).to_string(),)
            )
            .unwrap(),
        ),
        parse_json(
            crate::dependencies_json_impl(
                crate::options::test_json_arg(json!({
                    "root": root,
                    "files": ["app/page.tsx"],
                    "relationships": ["import"]
                })
                .to_string(),)
            )
            .unwrap(),
        ),
    ];

    no_mistakes::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        crate::options::test_json_arg(json!({
            "root": root,
            "reports": [
                { "type": "check" },
                { "type": "playwrightCheck", "assertUniqueHtmlIds": true },
                {
                    "type": "dependencies",
                    "files": ["app/page.tsx"],
                    "relationships": ["import"]
                }
            ]
        })
        .to_string(),)
    )
    .unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);

    assert_eq!(report_results(&output), standalone);
    assert_each_indexable_file_parsed_once(&root, &counts);
}

#[test]
fn prepared_react_mixed_views_match_standalone_and_parse_once() {
    let source = category_fixture("react-traits-usages", "basic");
    let fixture = no_mistakes::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    // Keep the intentionally malformed junk fixture outside the explicit React target set.
    let targets = ["app/components/**/*.tsx"];
    let standalone = [
        parse_json(
            crate::react_analyze_json_impl(
                crate::options::test_json_arg(json!({ "root": root, "targets": targets }).to_string(),)
            )
            .unwrap(),
        ),
        parse_json(
            crate::react_check_json_impl(
                crate::options::test_json_arg(json!({ "root": root, "targets": targets }).to_string(),)
            )
            .unwrap(),
        ),
        parse_json(
            crate::react_usages_json_impl(
                crate::options::test_json_arg(json!({
                    "root": root,
                    "targets": targets,
                    "target": "app/components/button.tsx#Button",
                    "include": "stories,tests,props"
                })
                .to_string(),)
            )
            .unwrap(),
        ),
    ];

    no_mistakes::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        crate::options::test_json_arg(json!({
            "root": root,
            "reports": [
                { "type": "reactAnalyze", "targets": targets },
                { "type": "reactCheck", "targets": targets },
                {
                    "type": "reactUsages",
                    "targets": targets,
                    "target": "app/components/button.tsx#Button",
                    "include": "stories,tests,props"
                }
            ]
        })
        .to_string(),)
    )
    .unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);

    assert_eq!(report_results(&output), standalone);
    assert_each_indexable_file_parsed_once(&root, &counts);
}

#[test]
fn prepared_server_mixed_views_match_standalone() {
    let root = analysis_fixture("routes/good");
    let standalone = [
        parse_json(
            crate::server_routes_json_impl(crate::options::test_json_arg(json!({ "root": root }).to_string())).unwrap(),
        ),
        parse_json(
            crate::server_contracts_json_impl(crate::options::test_json_arg(json!({ "root": root }).to_string()))
                .unwrap(),
        ),
    ];

    let output = analyze_project_json_impl(
        crate::options::test_json_arg(json!({
            "root": root,
            "reports": [
                { "type": "serverRoutes" },
                { "type": "serverContracts" }
            ]
        })
        .to_string(),)
    )
    .unwrap();

    assert_eq!(report_results(&output), standalone);
}
