use super::*;

/// Regression test: `DepGraph::build_with_plan_file_list_config_and_check_facts`
/// (the entrypoint `forbidden_dependencies::check_with_facts` uses) must
/// resolve `GraphConfigOptions` from the given `config_path`, not silently
/// fall back to default discovery — the same class of bug already fixed for
/// `collect_playwright_selector_edges`, found by a reviewer one layer up:
/// `check_with_facts` built its graph via an entrypoint that hardcoded
/// `config_path: None` before this fix, so passing `--config` to `check`
/// never reached any `no-mistakes check`-shared `DepGraph` build.
///
/// Uses the same two fixtures as `graph_config_options_use_explicit_config_for_legacy_rule_options`:
/// `graph-default-route-config`'s own `.no-mistakes.yml` configures a real
/// `backendPattern`, while `graph-empty-route-config`'s configures an empty
/// one. Building the graph for `graph-default-route-config`'s files without
/// an explicit `config_path` (default discovery finds its own config) must
/// produce the `RouteRef` edge; passing the empty-pattern config explicitly
/// must suppress it.
#[test]
fn build_with_plan_file_list_config_and_check_facts_uses_explicit_config_path() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let empty_config = crate::codebase::ts_resolver::normalize_path(
        &fixture("graph-empty-route-config").join(".no-mistakes.yml"),
    );
    let all_files = GraphFiles::discover(&root).all;
    let plan = GraphBuildPlan {
        routes: true,
        ..GraphBuildPlan::default()
    };
    let (fact_plan, fact_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(&root, plan);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        all_files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            graph: fact_plan,
            graph_context: fact_context,
            ..Default::default()
        },
    );

    let client = NodeId::File(root.join("src/client.ts"));
    let route = NodeId::File(root.join("backend/api/users.mts"));
    let has_route_ref = |graph: &DepGraph| {
        graph.dependents_of_node(&route).is_some_and(|deps| {
            deps.iter()
                .any(|(from, kind)| *from == client && *kind == EdgeKind::RouteRef)
        })
    };

    let default_graph = DepGraph::build_with_plan_file_list_config_and_check_facts(
        &root,
        &tsconfig,
        plan,
        all_files.clone(),
        None,
        &shared,
    )
    .expect("default graph builds");
    assert!(
        has_route_ref(&default_graph),
        "default-discovered config (this fixture's own .no-mistakes.yml) should produce the RouteRef edge"
    );

    let explicit_graph = DepGraph::build_with_plan_file_list_config_and_check_facts(
        &root,
        &tsconfig,
        plan,
        all_files,
        Some(&empty_config),
        &shared,
    )
    .expect("explicit-config graph builds");
    assert!(
        !has_route_ref(&explicit_graph),
        "passing the explicit empty-pattern config must be honored, not silently ignored in favor of default discovery"
    );
}

/// Regression test, one layer earlier than the one above:
/// `ts_fact_plan_and_context_for_plan_with_config` (used by `check_runner`
/// and `forbidden_dependencies::check_with_facts` to decide *what to parse*
/// before any `DepGraph` is built) must also resolve `GraphConfigOptions`
/// from the given `config_path`. If it didn't, the `TsFactContext` used to
/// collect shared facts could disagree with the `GraphConfigOptions` the
/// `DepGraph` build resolves later from the same `config_path` — the facts
/// collector would never even attempt to recognize a custom
/// `backendRegisterObject`/`backendPattern`, silently missing backend route
/// facts regardless of how correctly the graph build itself honors
/// `config_path`.
#[test]
fn ts_fact_plan_and_context_for_plan_with_config_uses_explicit_config_path() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let empty_config = crate::codebase::ts_resolver::normalize_path(
        &fixture("graph-empty-route-config").join(".no-mistakes.yml"),
    );
    let plan = GraphBuildPlan {
        routes: true,
        ..GraphBuildPlan::default()
    };

    let (_, default_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_config(
            &root, plan, None,
        );
    assert!(
        !default_context.backend_route_extractors.is_empty(),
        "default-discovered config (this fixture's own .no-mistakes.yml) should register a backend route extractor"
    );

    let (_, explicit_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_config(
            &root,
            plan,
            Some(&empty_config),
        );
    assert!(
        explicit_context.backend_route_extractors.is_empty(),
        "passing the explicit empty-pattern config must be honored, not silently ignored in favor of default discovery"
    );
}

#[test]
fn prepared_graph_playwright_edges_use_explicit_loaded_config() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "playwright-config-path-graph",
    ));
    let all_files = GraphFiles::discover(&root).all;
    let plan = GraphBuildPlan {
        playwright_routes: true,
        playwright_selectors: true,
        ..GraphBuildPlan::default()
    };
    let custom_config = root.join("custom.no-mistakes.yml");
    let loaded = crate::config::v2::load_v2_config(&root, Some(&custom_config)).unwrap();
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, Some(&custom_config), &loaded);
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let prepared = prepare_graph_config(&root, plan, &codebase_config, &loaded, &visible).unwrap();
    let (graph_fact_plan, graph_context) =
        ts_fact_plan_and_context_for_plan_with_prepared(&root, plan, &prepared);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        all_files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            graph: graph_fact_plan,
            graph_context,
            ..Default::default()
        },
    );
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan_file_list_prepared_config_and_check_facts(
        &root,
        &tsconfig,
        plan,
        all_files,
        Some(&custom_config),
        &facts,
        &prepared,
    )
    .unwrap();

    let test = NodeId::File(root.join("tests/e2e/app.spec.ts"));
    let page = NodeId::File(root.join("web/app/page.tsx"));
    let layout = NodeId::File(root.join("web/app/layout.tsx"));
    let test_dependencies = graph
        .dependencies_of_node(&test)
        .expect("test file is present in prepared graph");
    assert!(test_dependencies.contains(&(page.clone(), EdgeKind::RouteTest)));
    assert!(test_dependencies.contains(&(page.clone(), EdgeKind::Selector)));
    assert!(graph
        .dependencies_of_node(&page)
        .is_some_and(|edges| edges.contains(&(layout, EdgeKind::Layout))));
}

#[test]
fn playwright_route_edges_use_explicit_config_path() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "playwright-config-path-graph",
    ));
    let all_files = GraphFiles::discover(&root).all;

    assert!(collect_playwright_route_edges(&root, None, &all_files, None).is_empty());

    let custom_config = root.join("custom.no-mistakes.yml");
    let edges = collect_playwright_route_edges(
        &root,
        Some(&custom_config),
        &all_files,
        None,
    );
    let test = NodeId::File(root.join("tests/e2e/app.spec.ts"));
    let page = NodeId::File(root.join("web/app/page.tsx"));
    let layout = NodeId::File(root.join("web/app/layout.tsx"));
    assert!(edges.contains(&(test, page.clone(), EdgeKind::RouteTest)));
    assert!(edges.contains(&(page, layout, EdgeKind::Layout)));
}
