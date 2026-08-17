// ── EdgeKind::Selector / playwright selector edges ───────────────────────

fn intern() -> crate::codebase::analysis_session::PathInterner {
    crate::codebase::analysis_session::PathInterner::new()
}

fn collect_playwright_selector_edges(
    root: &Path,
    config_path: Option<&Path>,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
) -> Vec<Edge> {
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::from_paths(root, all_files);
    collect_playwright_selector_edges_with_graph(
        root,
        config_path,
        PlaywrightSelectorEdgeInputs {
            all_files,
            facts,
            partial_graph: None,
            graph_tsconfig: None,
            snapshot: &snapshot,
            prepared_settings: &[],
            interner: &intern(),
        },
    )
    .unwrap_or_default()
}

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

    let result = selector_dep_edge(&root, &edge, &intern()).unwrap();
    // test_file → app_file (mirrors TestOf direction so dependents_of(app_file) returns tests)
    assert_eq!(result.0, NodeId::file(p("/root/tests/e2e/nav.spec.ts")));
    assert_eq!(result.1, NodeId::file(p("/root/web/components/nav.tsx")));
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

    let result = selector_dep_edge(&root, &edge, &intern()).unwrap();
    // test_file → app_file (mirrors TestOf direction so dependents_of(app_file) returns tests)
    assert_eq!(result.0, NodeId::file(p("/root/tests/e2e/button.spec.ts")));
    assert_eq!(result.1, NodeId::file(p("/root/web/components/button.tsx")));
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
    assert!(selector_dep_edge(&root, &edge, &intern()).is_none());
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
        from == &NodeId::file(search_spec.clone())
            && to == &NodeId::file(search_bar.clone())
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
fn configured_selector_wrappers_create_only_the_declared_selector_edges() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/selector-wrappers"),
    );
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let edges = collect_playwright_selector_edges(&root, None, &all_files, None);
    let app_file = NodeId::file(root.join("web/page.tsx"));
    let test_file = NodeId::file(root.join("tests/page.spec.ts"));

    let selector_edges = edges
        .iter()
        .filter(|(from, to, kind)| {
            from == &test_file && to == &app_file && *kind == EdgeKind::Selector
        })
        .count();
    assert_eq!(selector_edges, 6, "{edges:#?}");
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
    let settings =
        crate::playwright::config::test_support::load_settings(&root, None, &[], None).unwrap();
    let playwright_configs = crate::playwright::playwright_config::load_many(
        &root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )
    .unwrap();
    let mut test_id_attributes_by_path = std::collections::HashMap::new();
    for test_file in
        crate::playwright::test_support::discover_test_files(&root, &settings, &playwright_configs)
            .unwrap()
    {
        let attributes = test_file.test_id_attributes();
        test_id_attributes_by_path.insert(test_file.path, attributes);
    }
    assert!(
        !test_id_attributes_by_path.is_empty(),
        "sanity check: fixture must have discoverable Playwright test files"
    );
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let playwright_plan = crate::codebase::check_facts::PlaywrightFactPlan::from_settings(
        &root,
        settings,
        test_id_attributes_by_path,
        false,
        &snapshot,
    )
    .unwrap();
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

#[test]
fn selector_analysis_reuses_matching_route_import_graph() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingFacts {
        facts: TsFactMap,
        graph_files: Vec<PathBuf>,
        lookups: AtomicUsize,
    }

    impl TsFactLookup for CountingFacts {
        fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
            self.lookups.fetch_add(1, Ordering::Relaxed);
            self.facts.get(path)
        }

        fn covers_ts_fact_plan(&self, required: TsFactPlan) -> bool {
            self.facts.plan().covers(required)
        }

        fn graph_files(&self) -> Option<&[PathBuf]> {
            Some(&self.graph_files)
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-selectors/selector-text-locator/fixture")
        .canonicalize()
        .expect("fixture root resolves");
    let settings = crate::playwright::config::test_support::load_settings(&root, None, &[], None)
        .expect("Playwright settings load");
    let tsconfig =
        crate::playwright::analysis::pipeline_text_test_support::load_route_import_tsconfig(
            &root, &settings,
        )
        .expect("route-import tsconfig loads");
    let graph_files = GraphFiles::discover(&root).all().to_vec();
    let facts = CountingFacts {
        facts: collect_ts_facts(&graph_files, TsFactPlan::imports()),
        graph_files: graph_files.clone(),
        lookups: AtomicUsize::new(0),
    };
    let graph = crate::playwright::analysis::pipeline_text_test_support::build_route_import_graph(
        &root,
        &settings,
        Some(&facts),
        None,
        &graph_files,
    )
    .expect("route-import graph builds");

    facts.lookups.store(0, Ordering::Relaxed);
    let matching = run_playwright_selector_analysis(
        &root,
        None,
        Some(&facts),
        Some(&graph),
        Some(&tsconfig),
        &graph_files,
    )
    .expect("selector analysis reuses matching graph");
    assert_eq!(facts.lookups.load(Ordering::Relaxed), 0);

    let mut mismatched_tsconfig = tsconfig.clone();
    mismatched_tsconfig.paths_dir = root.join("different-paths-root");
    let mismatched = run_playwright_selector_analysis(
        &root,
        None,
        Some(&facts),
        Some(&graph),
        Some(&mismatched_tsconfig),
        &graph_files,
    )
    .expect("selector analysis rebuilds mismatched graph");
    assert!(facts.lookups.load(Ordering::Relaxed) > 0);

    let matching_edges = selector_edges_from_analysis(&root, &graph_files, &matching, &intern());
    let mismatched_edges =
        selector_edges_from_analysis(&root, &graph_files, &mismatched, &intern());
    assert!(!matching_edges.is_empty());
    assert_eq!(matching_edges, mismatched_edges);
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

