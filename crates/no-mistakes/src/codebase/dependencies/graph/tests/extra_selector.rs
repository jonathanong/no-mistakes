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
        test_id_attributes: vec!["data-pw".to_string()],
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
    let edges = collect_playwright_selector_edges(&root, &all_files, None);
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
    let edges = collect_playwright_selector_edges(&root, &all_files, None);
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
    let edges = collect_playwright_selector_edges(&root, &all_files, None);
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
fn collect_playwright_selector_edges_filters_to_all_files_set() {
    // Passing an empty all_files set should produce no edges even when the
    // analysis finds matches, because the file-set filter drops them.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/playwright-coverage-route-group/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    // Pass an empty file list — all candidate edge endpoints are outside the set.
    let edges = collect_playwright_selector_edges(&root, &[], None);
    assert!(
        edges.is_empty(),
        "edges outside all_files set must be filtered out, got: {edges:?}"
    );
}

/// Regression test: `collect_playwright_selector_edges` must produce the same
/// edges whether or not it's handed already-collected Playwright facts. The
/// facts-aware path (`analyze_test_occurrences`, reusing cached URLs/
/// selectors/text-locators/helper-references) exists specifically so a
/// `DepGraph` build sharing a `CheckFactMap` (e.g. `check`'s `forbidden-
/// dependencies` rule) doesn't re-parse and re-analyze every Playwright test
/// file from scratch — a real, measured cost on large repos. This proves that
/// reuse path is wired correctly and doesn't silently drop or duplicate edges.
#[test]
fn collect_playwright_selector_edges_matches_with_and_without_shared_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/playwright-coverage-route-group/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    // Build the PlaywrightFactPlan directly from Playwright *settings* (which
    // this fixture has) rather than via `playwright::rules::fact_plan`, which
    // additionally requires a Playwright *rule* to be configured — an
    // unrelated, orthogonal gate this fixture intentionally leaves unset.
    let settings = crate::playwright::config::load_settings(&root, None, &[], None).unwrap();
    let selector_regexes = std::sync::Arc::new(
        crate::playwright::selectors::compile_selector_regexes_with_html_ids(
            &settings.selector_attributes,
            &settings.component_selector_attributes,
            settings.html_ids,
        ),
    );
    let playwright_configs = crate::playwright::playwright_config::load_many(
        &root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )
    .unwrap();
    let mut test_id_attributes_by_path = std::collections::HashMap::new();
    for test_file in
        crate::playwright::analysis::discover::discover_test_files(&root, &settings, &playwright_configs)
            .unwrap()
    {
        let attributes = test_file.test_id_attributes();
        test_id_attributes_by_path.insert(test_file.path, attributes);
    }
    assert!(
        !test_id_attributes_by_path.is_empty(),
        "sanity check: fixture must have discoverable Playwright test files"
    );
    let playwright_plan = crate::codebase::check_facts::PlaywrightFactPlan {
        navigation_helpers: settings.navigation_helpers.clone(),
        selector_regexes,
        test_id_attributes_by_path: std::sync::Arc::new(test_id_attributes_by_path),
    };
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts_with_playwright(
        &root,
        all_files.clone(),
        crate::codebase::check_facts::CheckFactPlan::default(),
        Some(playwright_plan),
    );

    let mut edges_without_facts = collect_playwright_selector_edges(&root, &all_files, None);
    let mut edges_with_facts =
        collect_playwright_selector_edges(&root, &all_files, Some(&facts));
    edges_without_facts.sort();
    edges_with_facts.sort();

    assert!(
        !edges_without_facts.is_empty(),
        "sanity check: fixture must produce selector edges"
    );
    assert_eq!(
        edges_without_facts, edges_with_facts,
        "reusing shared Playwright facts must not change which edges are produced"
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
