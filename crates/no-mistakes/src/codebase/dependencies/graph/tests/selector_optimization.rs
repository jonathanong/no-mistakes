use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn repository_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies")
            .join(name)
            .join("fixture"),
    )
}

fn assert_selector_edge(edges: &[Edge], root: &Path, app_file: &str) {
    assert!(edges.iter().any(|(from, to, kind)| {
        from == &NodeId::File(root.join("tests/e2e/app.spec.ts"))
            && to == &NodeId::File(root.join(app_file))
            && *kind == EdgeKind::Selector
    }));
}

fn assert_graph_selector_edge(graph: &DepGraph, root: &Path, app_file: &str) {
    let allowed = HashSet::from([EdgeKind::Selector]);
    let dependencies = graph.deps_of(
        &[NodeId::File(root.join("tests/e2e/app.spec.ts"))],
        None,
        Some(&allowed),
    );
    assert!(dependencies
        .iter()
        .any(|entry| entry.node == NodeId::File(root.join(app_file))));
}

#[test]
fn text_locator_edges_match_without_facts_and_with_default_or_sparse_facts() {
    use crate::codebase::check_facts::CheckFactMap;

    let root = repository_fixture("selector-text-sparse-universe");
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let default_facts = CheckFactMap::default();
    let mut sparse_facts = CheckFactMap {
        files: vec![root.join("web/app/components/discuss-button.tsx")],
        ..CheckFactMap::default()
    };
    sparse_facts.ts.insert(
        root.join("web/app/components/discuss-button.tsx"),
        crate::codebase::check_facts::CheckFileFacts::default(),
    );

    assert!(default_facts.graph_files().is_none());
    assert!(sparse_facts.graph_files().is_none());

    let analysis = run_playwright_selector_analysis(&root, None, None, None, None, &all_files)
        .expect("text-locator analysis succeeds");
    assert!(analysis.edges.edges.iter().any(|edge| matches!(
        edge,
        crate::playwright::analysis::types::Edge::LocatorText { app_file, .. }
            if app_file.as_str() == "web/app/components/discuss-button.tsx"
    )));
    let locator_test_files: std::collections::BTreeSet<_> = analysis
        .edges
        .edges
        .iter()
        .filter_map(|edge| match edge {
            crate::playwright::analysis::types::Edge::LocatorText { test_file, .. } => {
                Some(test_file.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        locator_test_files,
        std::collections::BTreeSet::from(["tests/e2e/app.spec.ts", "tests/e2e/secondary.spec.ts",])
    );
    let mut expected = selector_edges_from_analysis(&root, &all_files, &analysis);
    let mut with_default =
        collect_playwright_selector_edges(&root, None, &all_files, Some(&default_facts));
    let mut with_sparse =
        collect_playwright_selector_edges(&root, None, &all_files, Some(&sparse_facts));
    expected.sort();
    with_default.sort();
    with_sparse.sort();

    assert_selector_edge(&expected, &root, "web/app/components/discuss-button.tsx");
    assert_eq!(with_default, expected);
    assert_eq!(with_sparse, expected);
}

struct CountingFacts {
    facts: TsFactMap,
    graph_files: Vec<PathBuf>,
    lookups: AtomicUsize,
    route_scans: AtomicUsize,
    text_scans: AtomicUsize,
    reachability_scans: AtomicUsize,
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

    fn get_or_compute_playwright_routes(
        &self,
        compute: &dyn Fn() -> Vec<crate::routes::Route>,
    ) -> Arc<Vec<crate::routes::Route>> {
        self.route_scans.fetch_add(1, Ordering::Relaxed);
        Arc::new(compute())
    }

    fn get_or_compute_app_text_targets(
        &self,
        compute: &dyn Fn() -> anyhow::Result<
            Vec<crate::playwright::analysis::text_types::AppTextTarget>,
        >,
    ) -> anyhow::Result<Arc<Vec<crate::playwright::analysis::text_types::AppTextTarget>>> {
        self.text_scans.fetch_add(1, Ordering::Relaxed);
        compute().map(Arc::new)
    }

    fn get_or_compute_route_reachable_files(
        &self,
        compute: &dyn Fn() -> anyhow::Result<RouteReachableFiles>,
    ) -> anyhow::Result<Arc<RouteReachableFiles>> {
        self.reachability_scans.fetch_add(1, Ordering::Relaxed);
        compute().map(Arc::new)
    }
}

#[test]
fn multiple_scoped_locators_build_route_reachability_once() {
    let root = repository_fixture("selector-text-sparse-universe");
    let graph_files = GraphFiles::discover(&root);
    let facts = CountingFacts {
        facts: collect_ts_facts(graph_files.indexable(), TsFactPlan::imports()),
        graph_files: graph_files.all().to_vec(),
        lookups: AtomicUsize::new(0),
        route_scans: AtomicUsize::new(0),
        text_scans: AtomicUsize::new(0),
        reachability_scans: AtomicUsize::new(0),
    };

    run_playwright_selector_analysis(&root, None, Some(&facts), None, None, graph_files.all())
        .expect("scoped text locator analysis succeeds");

    assert_eq!(facts.route_scans.load(Ordering::Relaxed), 1);
    assert_eq!(facts.text_scans.load(Ordering::Relaxed), 1);
    assert_eq!(facts.reachability_scans.load(Ordering::Relaxed), 1);
}

#[test]
fn selector_only_graph_skips_route_import_second_pass() {
    let root = repository_fixture("selector-only-malformed-tsconfig");
    let graph_files = GraphFiles::discover(&root);
    let facts = CountingFacts {
        facts: collect_ts_facts(graph_files.indexable(), TsFactPlan::imports()),
        graph_files: graph_files.all().to_vec(),
        lookups: AtomicUsize::new(0),
        route_scans: AtomicUsize::new(0),
        text_scans: AtomicUsize::new(0),
        reachability_scans: AtomicUsize::new(0),
    };
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..TsConfig::default()
    };

    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            playwright_selectors: true,
            ..GraphBuildPlan::default()
        },
        &graph_files,
        None,
        Some(&facts),
    )
    .expect("selector-only graph builds");

    assert_eq!(facts.lookups.load(Ordering::Relaxed), 0);
    assert_eq!(facts.route_scans.load(Ordering::Relaxed), 0);
    assert_eq!(facts.text_scans.load(Ordering::Relaxed), 0);
    assert_eq!(facts.reachability_scans.load(Ordering::Relaxed), 0);
    assert_graph_selector_edge(&graph, &root, "web/components/save-button.tsx");
    assert!(graph
        .forward
        .values()
        .flatten()
        .all(|(_, kind)| *kind != EdgeKind::RouteImport));
}

#[test]
fn malformed_frontend_tsconfig_does_not_drop_direct_selector_edges() {
    let root = repository_fixture("selector-only-malformed-tsconfig");
    let settings = crate::playwright::config::load_settings(&root, None, &[], None)
        .expect("Playwright settings load");
    let route_tsconfig =
        crate::playwright::analysis::pipeline_text_setup::load_route_import_tsconfig(
            &root, &settings,
        );
    assert!(route_tsconfig.is_err());
    let graph_files = GraphFiles::discover(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..TsConfig::default()
    };

    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            playwright_selectors: true,
            ..GraphBuildPlan::default()
        },
        &graph_files,
        None,
        None,
    )
    .expect("selector graph builds without text demand");

    assert_graph_selector_edge(&graph, &root, "web/components/save-button.tsx");
}

#[test]
fn demanded_app_selector_scan_surfaces_parse_errors() {
    let root = repository_fixture("selector-malformed-app-source");
    let graph_files = GraphFiles::discover(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..TsConfig::default()
    };

    let result = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            playwright_selectors: true,
            ..GraphBuildPlan::default()
        },
        &graph_files,
        None,
        None,
    );
    let error = match result {
        Ok(_) => panic!("demanded app selector scan must surface parse errors"),
        Err(error) => error,
    };

    assert!(
        format!("{error:#}").contains("save-button.tsx"),
        "expected app source parse diagnostic, got: {error:#}"
    );
}

#[test]
fn demanded_app_text_scan_surfaces_parse_errors() {
    let root = repository_fixture("selector-malformed-app-source");
    let settings = crate::playwright::config::load_settings(&root, None, &[], None)
        .expect("Playwright settings load");

    let result = crate::playwright::analysis::pipeline_text_setup::build_text_resolution_setup(
        &root,
        &settings,
        crate::playwright::analysis::pipeline_text_setup::TextResolutionInputs {
            facts: None,
            graph_file_universe: None,
            route_import_candidate: None,
            routes: &[],
            has_eligible_text_locator: true,
            has_text_candidate: &|_, _| false,
            has_route_reachability_demand: &|_, _| false,
        },
    );
    let error = match result {
        Ok(_) => panic!("demanded app text scan must surface parse errors"),
        Err(error) => error,
    };

    assert!(
        format!("{error:#}").contains("save-button.tsx"),
        "expected app source parse diagnostic, got: {error:#}"
    );
}

#[test]
fn unrelated_route_does_not_demand_text_reachability_for_adjacent_selector() {
    let root = repository_fixture("selector-text-unrelated-route");
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);

    let analysis = run_playwright_selector_analysis(&root, None, None, None, None, &all_files)
        .expect("unrelated navigation must not load the malformed route tsconfig");
    let locator = analysis.edges.edges.iter().find(|edge| {
        matches!(
            edge,
            crate::playwright::analysis::types::Edge::LocatorText { test_file, .. }
                if test_file.as_str() == "tests/e2e/selector.spec.ts"
        )
    });
    assert!(matches!(
        locator,
        Some(crate::playwright::analysis::types::Edge::LocatorText { reasons, .. })
            if reasons == &["adjacent-selector".to_string()]
    ));
}

#[test]
fn same_scope_route_and_text_locator_surface_malformed_route_tsconfig() {
    let root = repository_fixture("selector-text-malformed-tsconfig");
    let graph_files = GraphFiles::discover(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..TsConfig::default()
    };

    let result = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            playwright_selectors: true,
            ..GraphBuildPlan::default()
        },
        &graph_files,
        None,
        None,
    );
    let error = match result {
        Ok(_) => panic!("eligible navigated text locator must demand route reachability"),
        Err(error) => error,
    };

    assert!(
        format!("{error:#}").contains("tsconfig.json"),
        "expected malformed route tsconfig diagnostic, got: {error:#}"
    );
}

#[test]
fn lazy_route_graph_prefers_outer_graph_universe_over_fact_cache_universe() {
    let root = repository_fixture("selector-text-sparse-universe");
    let settings = crate::playwright::config::load_settings(&root, None, &[], None)
        .expect("Playwright settings load");
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        &root,
        Vec::new(),
        Vec::new(),
        crate::codebase::check_facts::CheckFactPlan::default(),
        None,
    );
    let outer_file = root.join("web/app/components/discuss-button.tsx");

    let graph = crate::playwright::analysis::pipeline_text_setup::build_route_import_graph(
        &root,
        &settings,
        Some(&facts),
        Some(std::slice::from_ref(&outer_file)),
        &[],
    )
    .expect("lazy route graph builds from the outer graph universe");

    assert!(graph.contains_file(&outer_file));
}

#[test]
fn lazy_route_graph_discovers_files_without_a_supplied_universe() {
    let root = repository_fixture("selector-text-sparse-universe");
    let settings = crate::playwright::config::load_settings(&root, None, &[], None)
        .expect("Playwright settings load");
    let component = root.join("web/app/components/discuss-button.tsx");

    let graph = crate::playwright::analysis::pipeline_text_setup::build_route_import_graph(
        &root,
        &settings,
        None,
        None,
        &[],
    )
    .expect("lazy route graph discovers its file universe");

    assert!(graph.contains_file(&component));
}
