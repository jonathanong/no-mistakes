// ── EdgeKind::Selector / playwright selector edges ───────────────────────

#[test]
fn selector_dep_edge_maps_selector_edge_to_dep_graph_edge() {
    use crate::playwright::analysis::types::Edge as PwEdge;
    use std::sync::Arc;

    let root = p("/root");
    let app_file = Arc::new("web/components/nav.tsx".to_string());
    let test_file = Arc::new("tests/e2e/nav.spec.ts".to_string());
    let edge = PwEdge::Selector {
        test_file: test_file.clone(),
        test_name: None,
        describe_path: Arc::new(vec![]),
        app_file: app_file.clone(),
        attribute: "data-pw".to_string(),
        value: "nav-btn".to_string(),
        selector: "getByTestId('nav-btn')".to_string(),
        line: 5,
    };

    let result = selector_dep_edge(&root, &edge).unwrap();
    // test_file → app_file (mirrors TestOf direction so dependents_of(app_file) returns tests)
    assert_eq!(result.0, NodeId::File(p("/root/tests/e2e/nav.spec.ts")));
    assert_eq!(result.1, NodeId::File(p("/root/web/components/nav.tsx")));
    assert_eq!(result.2, EdgeKind::Selector);
}

#[test]
fn selector_dep_edge_maps_locator_text_edge_to_dep_graph_edge() {
    use crate::playwright::analysis::types::{Edge as PwEdge, SelectorRef};
    use std::sync::Arc;

    let root = p("/root");
    let app_file = Arc::new("web/components/button.tsx".to_string());
    let test_file = Arc::new("tests/e2e/button.spec.ts".to_string());
    let edge = PwEdge::LocatorText {
        test_file: test_file.clone(),
        test_name: None,
        describe_path: Arc::new(vec![]),
        app_file: app_file.clone(),
        locator_kind: "getByRole".to_string(),
        role: Some("button".to_string()),
        text: "Save".to_string(),
        locator: "getByRole('button', { name: 'Save' })".to_string(),
        selector_refs: vec![SelectorRef {
            attribute: "data-pw".to_string(),
            value: "save-btn".to_string(),
        }],
        reasons: vec![],
        line: 10,
    };

    let result = selector_dep_edge(&root, &edge).unwrap();
    // test_file → app_file (mirrors TestOf direction so dependents_of(app_file) returns tests)
    assert_eq!(result.0, NodeId::File(p("/root/tests/e2e/button.spec.ts")));
    assert_eq!(result.1, NodeId::File(p("/root/web/components/button.tsx")));
    assert_eq!(result.2, EdgeKind::Selector);
}

#[test]
fn selector_dep_edge_returns_none_for_route_edge() {
    use crate::playwright::analysis::types::Edge as PwEdge;
    use std::sync::Arc;

    let root = p("/root");
    let edge = PwEdge::Route {
        test_file: Arc::new("tests/e2e/nav.spec.ts".to_string()),
        test_name: None,
        describe_path: Arc::new(vec![]),
        route_file: Arc::new("web/app/page.tsx".to_string()),
        route: Arc::new("/".to_string()),
        url: Arc::new("http://localhost/".to_string()),
        hook: false,
        line: 1,
    };
    assert!(selector_dep_edge(&root, &edge).is_none());
}

#[test]
fn collect_playwright_selector_edges_returns_empty_without_playwright_config() {
    // A fixture with no playwright config should return empty without panicking.
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("simple"));
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let edges = collect_playwright_selector_edges(&root, &all_files);
    // No playwright config → error → empty vec (graceful fallback).
    assert!(edges.is_empty());
}

#[test]
fn collect_playwright_selector_edges_returns_edges_for_route_group_fixture() {
    // The playwright-coverage-route-group fixture has data-pw attributes and
    // getByTestId calls; selector edges should connect components to their tests.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/playwright-coverage-route-group/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let edges = collect_playwright_selector_edges(&root, &all_files);
    assert!(
        !edges.is_empty(),
        "expected selector edges from playwright-coverage-route-group fixture"
    );
    // search-bar.tsx is only reachable via selector edges (not imported anywhere).
    let search_bar = root.join("web/components/search-bar.tsx");
    let search_spec = root.join("tests/e2e/search-bar.spec.ts");
    let has_edge = edges.iter().any(|(from, to, kind)| {
        from == &NodeId::File(search_spec.clone())
            && to == &NodeId::File(search_bar.clone())
            && *kind == EdgeKind::Selector
    });
    assert!(
        has_edge,
        "expected selector edge from search-bar.spec.ts → search-bar.tsx"
    );
}

#[test]
fn collect_playwright_selector_edges_returns_edges_for_fixture_with_selectors() {
    // Use the nextjs-selectors/selector-covered fixture which has data-testid
    // attributes in app files and getByTestId calls in its spec file.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-selectors/selector-covered/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let edges = collect_playwright_selector_edges(&root, &all_files);
    assert!(
        !edges.is_empty(),
        "expected selector edges from nextjs-selectors/selector-covered fixture"
    );
    assert!(
        edges.iter().all(|(_, _, kind)| *kind == EdgeKind::Selector),
        "all edges produced must have EdgeKind::Selector"
    );
}

#[test]
fn graph_build_plan_playwright_selectors_enabled_in_all() {
    let plan = GraphBuildPlan::all();
    assert!(plan.playwright_selectors);
}

#[test]
fn graph_build_plan_playwright_selectors_from_allowed() {
    let allowed: HashSet<EdgeKind> = [EdgeKind::Selector].into();
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    assert!(plan.playwright_selectors);
    assert!(!plan.playwright_routes);
    assert!(!plan.imports);
}

#[test]
fn graph_build_plan_playwright_selectors_not_set_by_default() {
    let plan = GraphBuildPlan::default();
    assert!(!plan.playwright_selectors);
}
