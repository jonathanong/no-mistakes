#[test]
fn playwright_and_symbols_share_full_config_facts_with_standalone_parity() {
    let source = parser_fixture("playwright");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = [
        parse_json(
            crate::napi_api::playwright_tests_json_impl(
                json!({ "root": root, "files": ["app/page.tsx"] }).to_string(),
            )
            .unwrap(),
        ),
        parse_json(
            crate::napi_api::symbols_json_impl(
                json!({ "root": root, "files": ["playwright.config.ts"] }).to_string(),
            )
            .unwrap(),
        ),
    ];
    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                { "type": "playwrightTests", "files": ["app/page.tsx"] },
                { "type": "symbols", "files": ["playwright.config.ts"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(report_results(&output), standalone);
    assert_eq!(standalone[1]["files"][0]["exports"][0]["line"], 3);
    assert_eq!(counts.get(&root.join("playwright.config.ts")), Some(&1));
    assert_eq!(counts.get(&root.join("playwright.helper.ts")), Some(&1));
}

#[test]
fn check_playwright_and_symbols_keep_config_symbols_and_check_output() {
    let source = parser_fixture("playwright");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = [
        parse_json(
            crate::napi_api::check_json_impl(
                json!({ "root": root, "config": "isolation.no-mistakes.yml" }).to_string(),
            )
            .unwrap(),
        ),
        parse_json(
            crate::napi_api::playwright_tests_json_impl(
                json!({
                    "root": root,
                    "files": ["app/page.tsx"],
                    "playwrightConfig": ["ignored/playwright.ignored.config.ts"]
                })
                .to_string(),
            )
            .unwrap(),
        ),
        parse_json(
            crate::napi_api::symbols_json_impl(
                json!({
                    "root": root,
                    "files": ["ignored/playwright.ignored.config.ts"]
                })
                .to_string(),
            )
            .unwrap(),
        ),
    ];
    let widened = parse_json(
        crate::napi_api::check_json_impl(
            json!({
                "root": root.join("ignored"),
                "config": root.join("isolation.no-mistakes.yml")
            })
            .to_string(),
        )
        .unwrap(),
    );

    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "config": "isolation.no-mistakes.yml",
            "reports": [
                { "type": "check" },
                {
                    "type": "playwrightTests",
                    "files": ["app/page.tsx"],
                    "playwrightConfig": ["ignored/playwright.ignored.config.ts"]
                },
                {
                    "type": "symbols",
                    "files": ["ignored/playwright.ignored.config.ts"]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(report_results(&output), standalone);
    assert!(standalone[0]["codebase"].as_array().unwrap().is_empty());
    assert!(
        widened["codebase"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule"] == "unique-exports"),
        "the ignored duplicate must be a meaningful check violation: {widened:#}"
    );
    assert!(standalone[2]["files"][0]["exports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["name"] == "default" && entry["line"] == 5));
    assert_eq!(
        counts.get(&root.join("ignored/playwright.ignored.config.ts")),
        Some(&1)
    );
    assert_eq!(counts.get(&root.join("playwright.helper.ts")), Some(&1));
}
