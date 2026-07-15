#[test]
fn playwright_check_napi_parses_each_source_file_once() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/playwright"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();

    crate::ast::begin_parse_count(&root);
    let output = playwright_check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = [
        root.join("app/Widget.tsx"),
        root.join("app/page.tsx"),
        root.join("playwright.config.ts"),
        root.join("playwright.helper.ts"),
        root.join("tests/home.spec.ts"),
    ];

    assert_eq!(report["summary"]["totalRoutes"], 1);
    assert_eq!(counts.len(), expected.len(), "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    for file in expected {
        assert_eq!(counts.get(&file), Some(&1), "{counts:?}");
    }
}

#[test]
fn playwright_wrapper_edges_have_cli_napi_and_analyze_project_parity() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/selector-wrappers"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let check = playwright_check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let check_value: serde_json::Value = serde_json::from_str(&check).unwrap();
    assert_eq!(check_value["summary"]["coveredSelectors"], 6);
    assert_eq!(check_value["summary"]["uncoveredSelectors"], 6);
    assert_eq!(
        check_value["selectors"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|selector| selector["covered"] == true)
            .count(),
        6
    );
    let mode = check_value["selectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|selector| selector["value"] == "mode")
        .unwrap();
    assert!(mode.get("helperReferences").is_none());

    let standalone = playwright_edges_json_impl(json!({ "root": root }).to_string()).unwrap();
    let standalone_value: serde_json::Value = serde_json::from_str(&standalone).unwrap();
    let selectors = standalone_value["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["kind"] == "selector")
        .map(|edge| edge["selector"].as_str().unwrap())
        .collect::<Vec<_>>();

    for covered in [
        "aside-button",
        "default-button",
        "namespace-button",
        "namespace-native-name",
        "package-import-button",
        "workspace-export-button",
    ] {
        assert!(selectors.iter().any(|value| value.contains(covered)));
    }
    for uncovered in [
        "shadowed-button",
        "unconfigured-button",
        "ambiguous-button",
        "ambiguous-namespace-button",
        "recognized-missing-button",
    ] {
        assert!(!selectors.iter().any(|value| value.contains(uncovered)));
    }

    let request = json!({
            "root": root,
            "reports": [
                { "type": "playwrightCheck" },
                { "type": "playwrightEdges" }
            ]
        })
        .to_string();
    crate::ast::begin_parse_count(&root);
    let batched = super::analyze_project::analyze_project_json_impl(request).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let expected = [
        root.join("playwright.config.ts"),
        root.join("packages/locators/src/aside.ts"),
        root.join("tests/default-locator.ts"),
        root.join("tests/helpers.ts"),
        root.join("tests/namespace-locators.ts"),
        root.join("tests/page.spec.ts"),
        root.join("tests/unconfigured.ts"),
        root.join("web/page.tsx"),
    ];
    assert_eq!(counts.len(), expected.len(), "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    for file in expected {
        assert_eq!(counts.get(&file), Some(&1), "{counts:?}");
    }
    let batched: serde_json::Value = serde_json::from_str(&batched).unwrap();
    assert_eq!(batched["reports"][0]["result"], check_value);
    assert_eq!(batched["reports"][1]["result"], standalone_value);
}

#[test]
fn playwright_wrapper_check_parses_each_requested_source_once() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/selector-wrappers"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);
    playwright_check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(
        counts.get(&root.join("web/page.tsx")),
        Some(&1),
        "{counts:?}"
    );
    assert_eq!(
        counts.get(&root.join("tests/page.spec.ts")),
        Some(&1),
        "{counts:?}"
    );
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
}
