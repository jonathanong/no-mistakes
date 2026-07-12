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
    let edges = collect_playwright_selector_edges(&root, None, &all_files, None);
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
    let edges = collect_playwright_selector_edges(&root, None, &all_files, None);
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
    let edges = collect_playwright_selector_edges(&root, None, &all_files, None);
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
    let edges = collect_playwright_selector_edges(&root, None, &[], None);
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

    let mut edges_without_facts = collect_playwright_selector_edges(&root, None, &all_files, None);
    let mut edges_with_facts =
        collect_playwright_selector_edges(&root, None, &all_files, Some(&facts));
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

/// Regression test: `collect_playwright_selector_edges` must resolve Playwright
/// settings from the given `config_path`, not silently fall back to
/// default-discovery. The fixture's default-discovered `.no-mistakes.yml`
/// configures `data-testid` as the only test-id attribute, which does not
/// match the app file's `data-pw` attribute, so scanning without an explicit
/// config finds no selector edges. `custom.no-mistakes.yml` configures
/// `data-pw` instead — passing it as `config_path` must produce the edge that
/// default-discovery misses; if `config_path` were ignored (as it was before
/// this fix), both scans would return the same empty result.
#[test]
fn collect_playwright_selector_edges_uses_explicit_config_path_not_default_discovery() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/playwright-config-path-selector-scan/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);

    let edges_default = collect_playwright_selector_edges(&root, None, &all_files, None);
    assert!(
        edges_default.is_empty(),
        "sanity check: default-discovered config (data-testid) should not match the fixture's data-pw attribute, got: {edges_default:?}"
    );

    let custom_config = root.join("custom.no-mistakes.yml");
    let edges_custom =
        collect_playwright_selector_edges(&root, Some(&custom_config), &all_files, None);
    assert!(
        !edges_custom.is_empty(),
        "expected selector edges when passing the explicit --config path (data-pw)"
    );
}

// ── shared app-selector-occurrences cache (CheckFactMap) ─────────────────

/// Regression test: `CheckFactMap::get_or_compute_app_selector_occurrences`
/// must call `compute` at most once per distinct `scan_html_ids` key — this
/// is what actually makes `no-mistakes check` dedupe the app-wide selector
/// scan across `playwright::rules::check_with_facts` and
/// `forbidden_dependencies`'s `DepGraph` build (previously each paid the
/// full scan independently). Asserting on the *returned value* alone
/// wouldn't prove this — a non-caching implementation returns the same
/// value too, just by recomputing it; asserting on the call count does.
#[test]
fn get_or_compute_app_selector_occurrences_caches_per_scan_html_ids_key() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::playwright::selectors::AppSelector;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = CheckFactMap::default();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<Vec<AppSelector>> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    };

    let first = facts
        .get_or_compute_app_selector_occurrences(false, &compute)
        .unwrap();
    let second = facts
        .get_or_compute_app_selector_occurrences(false, &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a second call with the same scan_html_ids key must reuse the cached result, not recompute"
    );
    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "cached calls must return the same Arc allocation, not merely an equal value"
    );

    facts
        .get_or_compute_app_selector_occurrences(true, &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "a different scan_html_ids key is a real input to the scan (see doc comment) and must recompute"
    );
}

/// Regression test: `get_or_compute_route_reachable_files` — the cache behind
/// this session's largest measured win (~8s per call on a real monorepo,
/// dropping to ~0 on the second call) — must call `compute` at most once,
/// with no key needed (unlike `app_selector_occurrences`, this scan has no
/// caller-varying input; see the trait doc comment).
#[test]
fn get_or_compute_route_reachable_files_caches_across_calls() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = CheckFactMap::default();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };

    let first = facts.get_or_compute_route_reachable_files(&compute).unwrap();
    let second = facts.get_or_compute_route_reachable_files(&compute).unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a second call must reuse the cached result, not recompute the reachability scan"
    );
    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "cached calls must return the same Arc allocation, not merely an equal value"
    );
}

/// Regression test: `get_or_compute_playwright_routes` and
/// `get_or_compute_app_text_targets` — the two smaller keyless caches added
/// alongside `route_reachable_files` — must each call `compute` at most once.
#[test]
fn get_or_compute_routes_and_app_text_targets_cache_across_calls() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = CheckFactMap::default();

    let route_calls = AtomicUsize::new(0);
    let compute_routes = || -> Vec<crate::routes::Route> {
        route_calls.fetch_add(1, Ordering::SeqCst);
        Vec::new()
    };
    facts.get_or_compute_playwright_routes(&compute_routes);
    facts.get_or_compute_playwright_routes(&compute_routes);
    assert_eq!(
        route_calls.load(Ordering::SeqCst),
        1,
        "a second call must reuse the cached routes, not recompute"
    );

    let text_target_calls = AtomicUsize::new(0);
    let compute_text_targets = || -> anyhow::Result<_> {
        text_target_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    };
    facts
        .get_or_compute_app_text_targets(&compute_text_targets)
        .unwrap();
    facts
        .get_or_compute_app_text_targets(&compute_text_targets)
        .unwrap();
    assert_eq!(
        text_target_calls.load(Ordering::SeqCst),
        1,
        "a second call must reuse the cached app text targets, not recompute"
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
