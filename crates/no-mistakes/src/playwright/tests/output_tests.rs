use crate::playwright::analysis::output::{
    build_related_report, print_coverage_text, print_edges_text,
};
use crate::playwright::analysis::tests_report::{build_tests_report, print_tests_text};
use crate::playwright::analysis::tests_report_types::{TestEntry, TestsReport};
use crate::playwright::analysis::types::{
    CoverageReport, CoverageRoute, CoverageSelector, DuplicateSelector, Edge, EdgeReport,
    SelectorHelperReference, SelectorRef, Summary,
};
use crate::playwright::test_support::fixture_path;
use crate::playwright::{report_json, PlaywrightReportKind, PlaywrightReportOptions};
use std::path::PathBuf;

fn report_options(root: PathBuf) -> PlaywrightReportOptions {
    PlaywrightReportOptions {
        root,
        config: None,
        playwright_config: Vec::new(),
        project: None,
        files: Vec::new(),
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: false,
        assert_unique_html_ids: false,
    }
}

fn parser_count_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/playwright"),
    )
}

fn assert_parser_count_fixture_parsed_once(
    root: &std::path::Path,
    counts: &std::collections::HashMap<PathBuf, usize>,
) {
    let expected = [
        root.join("app/Widget.tsx"),
        root.join("app/page.tsx"),
        root.join("playwright.config.ts"),
        root.join("playwright.helper.ts"),
        root.join("tests/home.spec.ts"),
    ];
    assert_eq!(counts.len(), expected.len(), "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    for file in expected {
        assert_eq!(counts.get(&file), Some(&1), "{counts:?}");
    }
}

#[test]
fn standalone_playwright_report_parses_each_source_file_once() {
    let source = parser_count_fixture();
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();

    crate::ast::begin_parse_count(&root);
    let output = report_json(PlaywrightReportKind::Check, report_options(root.clone())).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(report["summary"]["totalRoutes"], 1);
    assert_eq!(report["summary"]["uncoveredRoutes"], 0);
    assert_parser_count_fixture_parsed_once(&root, &counts);
}

#[test]
fn standalone_playwright_report_releases_cached_asts_before_serialization_returns() {
    let source = parser_count_fixture();
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();

    crate::ast::with_request_parse_cache(|| {
        let output =
            report_json(PlaywrightReportKind::Check, report_options(root.clone())).unwrap();

        assert!(!output.is_empty());
        assert_eq!(crate::ast::tests::request_parse_cache_len(), 0);
    });
}

#[test]
fn text_printers_cover_routes_and_selectors() {
    let coverage = CoverageReport {
        summary: Summary {
            total_routes: 1,
            covered_routes: 0,
            uncovered_routes: 1,
            total_selectors: 1,
            covered_selectors: 0,
            uncovered_selectors: 1,
            duplicate_selectors: 1,
            total_fetch_apis: 0,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 0,
        },
        routes: vec![CoverageRoute {
            route: "/missing".to_string(),
            file: "web/app/missing/page.tsx".to_string(),
            covered: false,
            tests: vec![],
            tests_detail: vec![],
            urls: vec![],
        }],
        selectors: vec![CoverageSelector {
            attribute: "data-testid".to_string(),
            value: "missing".to_string(),
            file: "web/app/page.tsx".to_string(),
            covered: false,
            unsupported_dynamic: false,
            tests: vec![],
            tests_detail: vec![],
            selectors: vec![],
            helper_references: vec![SelectorHelperReference {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                line: 4,
                call: "getAsideLocator(...)".to_string(),
            }],
        }],
        duplicate_selectors: vec![DuplicateSelector {
            attribute: "data-testid".to_string(),
            value: "missing".to_string(),
            file: "web/app/other.tsx".to_string(),
        }],
        fetch_apis: vec![],
    };
    print_coverage_text(&coverage);

    let edges = EdgeReport {
        edges: vec![
            Edge::Route {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                test_name: None,
                describe_path: std::sync::Arc::new(vec![]),
                route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
                route: std::sync::Arc::new("/".to_string()),
                url: std::sync::Arc::new("/".to_string()),
                hook: false,
                line: 1,
            },
            Edge::Selector {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                test_name: None,
                describe_path: std::sync::Arc::new(vec![]),
                app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
                attribute: "data-testid".to_string(),
                value: "save".to_string(),
                selector: "getByTestId(save)".to_string(),
                line: 1,
            },
            Edge::LocatorText {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                test_name: None,
                describe_path: std::sync::Arc::new(vec![]),
                app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
                locator_kind: "role".to_string(),
                role: Some("button".to_string()),
                text: "Save".to_string(),
                locator: "getByRole(button, name: Save)".to_string(),
                test_id_attributes: vec!["data-testid".to_string()],
                selector_refs: vec![SelectorRef {
                    attribute: "data-testid".to_string(),
                    value: "save".to_string(),
                }],
                reasons: vec!["route-signal".to_string()],
                line: 2,
            },
        ],
    };
    print_edges_text(&edges);
}

#[test]
fn text_printer_covers_fetch_edges() {
    let edges = EdgeReport {
        edges: vec![Edge::Fetch {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: Some(std::sync::Arc::new("visits home".to_string())),
            describe_path: std::sync::Arc::new(vec!["Suite".to_string()]),
            route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            route: std::sync::Arc::new("/".to_string()),
            method: "GET".to_string(),
            path: "/api/health".to_string(),
            side: "server".to_string(),
            cached: false,
        }],
    };
    print_edges_text(&edges);
}

#[test]
fn related_report_includes_fetch_apis() {
    let root = std::path::Path::new("/repo");
    let edges = vec![
        Edge::Route {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: None,
            describe_path: std::sync::Arc::new(vec![]),
            route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            route: std::sync::Arc::new("/".to_string()),
            url: std::sync::Arc::new("/".to_string()),
            hook: false,
            line: 1,
        },
        Edge::Fetch {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: None,
            describe_path: std::sync::Arc::new(vec![]),
            route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            route: std::sync::Arc::new("/".to_string()),
            method: "GET".to_string(),
            path: "/api/health".to_string(),
            side: "server".to_string(),
            cached: false,
        },
        Edge::LocatorText {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            test_name: None,
            describe_path: std::sync::Arc::new(vec![]),
            app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            locator_kind: "text".to_string(),
            role: None,
            text: "Save".to_string(),
            locator: "getByText(Save)".to_string(),
            test_id_attributes: vec!["data-testid".to_string()],
            selector_refs: vec![],
            reasons: vec!["route-signal".to_string()],
            line: 3,
        },
    ];
    let related = build_related_report(root, &edges, &[PathBuf::from("/repo/web/app/page.tsx")]);
    assert!(related.tests.contains(&"tests/e2e/app.spec.ts".to_string()));
    assert!(related.fetch_apis.contains(&"GET /api/health".to_string()));
}

#[test]
fn print_tests_text_covers_html_ids() {
    let report = TestsReport {
        tests: vec![TestEntry {
            file: "tests/e2e/app.spec.ts".to_string(),
            name: Some("visits home".to_string()),
            describe_path: vec![],
            test_ids: vec![],
            html_ids: vec!["main-nav".to_string()],
            routes: vec![],
            fetch_apis: vec![],
            locator_texts: vec![],
        }],
    };
    print_tests_text(&report);
}

#[test]
fn print_tests_text_with_describe_path_and_unnamed_entry() {
    let report = TestsReport {
        tests: vec![
            TestEntry {
                file: "tests/e2e/app.spec.ts".to_string(),
                name: Some("my test".to_string()),
                describe_path: vec!["Suite".to_string(), "Nested".to_string()],
                test_ids: vec![],
                html_ids: vec![],
                routes: vec!["/".to_string()],
                fetch_apis: vec!["GET /api/data".to_string()],
                locator_texts: vec!["role: Save".to_string()],
            },
            TestEntry {
                file: "tests/e2e/app.spec.ts".to_string(),
                name: None,
                describe_path: vec![],
                test_ids: vec![],
                html_ids: vec![],
                routes: vec![],
                fetch_apis: vec![],
                locator_texts: vec![],
            },
        ],
    };
    print_tests_text(&report);
}

#[test]
fn edge_report_json_schema_is_stable_with_arc_fields() {
    let report = EdgeReport {
        edges: vec![
            Edge::Route {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                test_name: None,
                describe_path: std::sync::Arc::new(vec![]),
                route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
                route: std::sync::Arc::new("/".to_string()),
                url: std::sync::Arc::new("/api/health".to_string()),
                hook: false,
                line: 1,
            },
            Edge::Selector {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                test_name: Some(std::sync::Arc::new("visits home".to_string())),
                describe_path: std::sync::Arc::new(vec!["Suite".to_string()]),
                app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
                attribute: "data-testid".to_string(),
                value: "save".to_string(),
                selector: "getByTestId(save)".to_string(),
                line: 1,
            },
            Edge::Fetch {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                test_name: Some(std::sync::Arc::new("loads home".to_string())),
                describe_path: std::sync::Arc::new(vec![]),
                route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
                route: std::sync::Arc::new("/".to_string()),
                method: "GET".to_string(),
                path: "/api/health".to_string(),
                side: "server".to_string(),
                cached: false,
            },
            Edge::LocatorText {
                test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
                test_name: Some(std::sync::Arc::new("loads home".to_string())),
                describe_path: std::sync::Arc::new(vec![]),
                app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
                locator_kind: "role".to_string(),
                role: Some("button".to_string()),
                text: "Save".to_string(),
                locator: "getByRole(button, name: Save)".to_string(),
                test_id_attributes: vec!["data-testid".to_string()],
                selector_refs: vec![SelectorRef {
                    attribute: "data-testid".to_string(),
                    value: "save".to_string(),
                }],
                reasons: vec!["route-signal".to_string()],
                line: 4,
            },
        ],
    };

    let value = serde_json::to_value(report).unwrap();
    let edges = value["edges"].as_array().unwrap();

    let route = &edges[0];
    assert_eq!(route["kind"], "route");
    assert_eq!(route["testFile"], "tests/e2e/app.spec.ts");
    assert_eq!(route["routeFile"], "web/app/page.tsx");
    assert_eq!(route["route"], "/");
    assert_eq!(route["url"], "/api/health");
    assert_eq!(route["hook"], false);
    assert_eq!(route["line"], 1);
    assert!(!route.as_object().unwrap().contains_key("testName"));
    assert!(!route.as_object().unwrap().contains_key("describePath"));

    let selector = &edges[1];
    assert_eq!(selector["kind"], "selector");
    assert_eq!(selector["testFile"], "tests/e2e/app.spec.ts");
    assert_eq!(selector["testName"], "visits home");
    assert_eq!(selector["describePath"], serde_json::json!(["Suite"]));
    assert_eq!(selector["appFile"], "web/app/page.tsx");
    assert_eq!(selector["line"], 1);

    let fetch = &edges[2];
    assert_eq!(fetch["kind"], "fetch");
    assert_eq!(fetch["testFile"], "tests/e2e/app.spec.ts");
    assert_eq!(fetch["testName"], "loads home");
    assert!(!fetch.as_object().unwrap().contains_key("describePath"));
    assert_eq!(fetch["routeFile"], "web/app/page.tsx");
    assert_eq!(fetch["route"], "/");
    assert_eq!(fetch["method"], "GET");
    assert_eq!(fetch["path"], "/api/health");
    assert_eq!(fetch["side"], "server");
    assert!(fetch["cached"].as_bool().is_some_and(|cached| !cached));

    let locator_text = &edges[3];
    assert_eq!(locator_text["kind"], "locatorText");
    assert_eq!(locator_text["locatorKind"], "role");
    assert_eq!(locator_text["role"], "button");
    assert_eq!(locator_text["text"], "Save");
    assert_eq!(locator_text["reasons"], serde_json::json!(["route-signal"]));
    assert_eq!(locator_text["line"], 4);
}

include!("output_report_tests.rs");
