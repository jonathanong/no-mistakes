#[test]
fn build_tests_report_produces_entries_with_routes_and_fetch_apis() {
    let root = std::path::Path::new("/repo");
    let edges = vec![
        Edge::Route {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: Some(std::sync::Arc::new("visits home".to_string())),
            describe_path: std::sync::Arc::new(vec!["Suite".to_string()]),
            route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            route: std::sync::Arc::new("/".to_string()),
            url: std::sync::Arc::new("/".to_string()),
            hook: false,
            line: 1,
        },
        Edge::Fetch {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: Some(std::sync::Arc::new("visits home".to_string())),
            describe_path: std::sync::Arc::new(vec!["Suite".to_string()]),
            route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            route: std::sync::Arc::new("/".to_string()),
            method: "GET".to_string(),
            path: "/api/health".to_string(),
            side: "server".to_string(),
            cached: false,
        },
        Edge::LocatorText {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: Some(std::sync::Arc::new("visits home".to_string())),
            describe_path: std::sync::Arc::new(vec!["Suite".to_string()]),
            app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            locator_kind: "text".to_string(),
            role: None,
            text: "Save".to_string(),
            locator: "getByText(Save)".to_string(),
            test_id_attributes: vec!["data-testid".to_string()],
            selector_refs: vec![],
            reasons: vec!["route-signal".to_string()],
            line: 5,
        },
    ];
    let report = build_tests_report(&edges, &[], root);
    assert_eq!(report.tests.len(), 1);
    assert_eq!(report.tests[0].name.as_deref(), Some("visits home"));
    assert_eq!(report.tests[0].describe_path, vec!["Suite".to_string()]);
    assert!(report.tests[0].routes.contains(&"/".to_string()));
    assert!(report.tests[0]
        .fetch_apis
        .contains(&"GET /api/health".to_string()));
    assert!(report.tests[0]
        .locator_texts
        .contains(&"text: Save".to_string()));
}

#[test]
fn build_tests_report_groups_selector_edges_by_attribute() {
    let root = std::path::Path::new("/repo");
    let edges = vec![
        Edge::Selector {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: Some(std::sync::Arc::new("visits home".to_string())),
            describe_path: std::sync::Arc::new(vec![]),
            app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            attribute: "id".to_string(),
            value: "main-nav".to_string(),
            selector: "#main-nav".to_string(),
            line: 1,
        },
        Edge::Selector {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: Some(std::sync::Arc::new("visits home".to_string())),
            describe_path: std::sync::Arc::new(vec![]),
            app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            attribute: "data-testid".to_string(),
            value: "save".to_string(),
            selector: "getByTestId(save)".to_string(),
            line: 1,
        },
    ];
    let report = build_tests_report(&edges, &[], root);
    assert_eq!(report.tests.len(), 1);
    assert!(report.tests[0].html_ids.contains(&"main-nav".to_string()));
    assert!(report.tests[0].test_ids.contains(&"save".to_string()));
}

#[test]
fn build_tests_report_with_absolute_file_path_filter() {
    let root = std::path::Path::new("/repo");
    let edges = vec![Edge::Route {
        test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
        test_name: Some(std::sync::Arc::new("visits home".to_string())),
        describe_path: std::sync::Arc::new(vec![]),
        route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
        route: std::sync::Arc::new("/".to_string()),
        url: std::sync::Arc::new("/".to_string()),
        hook: false,
        line: 1,
    }];
    // Pass an absolute path as the file filter — exercises the absolute branch in input_file()
    let abs_filter = std::path::PathBuf::from("/repo/tests/e2e/app.spec.ts");
    let report = build_tests_report(&edges, &[abs_filter], root);
    assert_eq!(report.tests.len(), 1);
    assert_eq!(report.tests[0].name.as_deref(), Some("visits home"));
}

#[test]
fn build_tests_report_filters_locator_text_edges_by_file() {
    let root = std::path::Path::new("/repo");
    let edges = vec![Edge::LocatorText {
        test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
        test_name: Some(std::sync::Arc::new("visits home".to_string())),
        describe_path: std::sync::Arc::new(vec![]),
        app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
        locator_kind: "text".to_string(),
        role: None,
        text: "Save".to_string(),
        locator: "getByText(Save)".to_string(),
        test_id_attributes: vec!["data-testid".to_string()],
        selector_refs: vec![],
        reasons: vec!["route-signal".to_string()],
        line: 5,
    }];
    let report = build_tests_report(
        &edges,
        &[std::path::PathBuf::from("tests/e2e/other.spec.ts")],
        root,
    );
    assert!(report.tests.is_empty());
}

#[test]
fn report_json_surfaces_project_selection_errors() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let error = report_json(
        PlaywrightReportKind::Check,
        PlaywrightReportOptions {
            project: Some("missing-project".to_string()),
            ..report_options(root)
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("missing-project"));
}

#[test]
fn report_json_related_requires_files_before_settings() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let error = report_json(
        PlaywrightReportKind::Related,
        PlaywrightReportOptions {
            project: Some("missing-project".to_string()),
            ..report_options(root)
        },
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "files must contain at least one file");
}

#[test]
fn report_json_accepts_project_selection_before_analysis() {
    let root = fixture_path(&["integration-tests", "basic"]);
    let error = report_json(
        PlaywrightReportKind::Check,
        PlaywrightReportOptions {
            project: Some("pw-unit".to_string()),
            ..report_options(root)
        },
    )
    .unwrap_err();

    assert!(
        error.to_string().contains("no Next.js page routes found"),
        "{error:#}"
    );
}

#[test]
fn report_json_surfaces_analysis_errors() {
    let root = fixture_path(&["ast-snippets", "main", "invalid-test-source"]);
    let error = report_json(PlaywrightReportKind::Check, report_options(root)).unwrap_err();

    assert!(error.to_string().contains("no Next.js page routes found"));
}
