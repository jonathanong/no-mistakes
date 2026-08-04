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
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("playwright-config-path-graph"));
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
    assert!(
        prepared
            .playwright_fact_plan(&root, &tsconfig, &visible)
            .unwrap()
            .is_some(),
        "prepared graph settings must build a reusable Playwright fact plan"
    );
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

/// Regression test for a review finding on this change: `no-mistakes graph`
/// previously resolved Playwright settings via a single, unbound
/// `settings_from_loaded_v2(..., None, None, ...)` call, which — once #624's
/// fix made an unbound ambiguous app an error instead of a silent guess —
/// meant the entire graph build failed outright for any repository with two
/// or more `type: nextjs` projects (there is no Playwright-project/rule
/// context at this call site to bind against, so no config change could
/// route around it). `PreparedGraphConfig` now resolves one `Settings` per
/// app instead, so both apps' route/selector edges are present.
#[test]
fn prepared_graph_playwright_edges_cover_every_frontend_app() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-multi-frontend-apps"));
    let all_files = GraphFiles::discover(&root).all;
    let plan = GraphBuildPlan {
        playwright_routes: true,
        playwright_selectors: true,
        ..GraphBuildPlan::default()
    };
    let loaded = crate::config::v2::load_v2_config(&root, None).unwrap();
    let codebase_config = crate::codebase::config::config_from_loaded_v2(&root, None, &loaded);
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
        None,
        &facts,
        &prepared,
    )
    .unwrap();

    let control_test = NodeId::File(root.join("tests/e2e/control.spec.ts"));
    let control_page = NodeId::File(root.join("control-web/app/control/page.tsx"));
    let agent_test = NodeId::File(root.join("tests/e2e/agent.spec.ts"));
    let agent_page = NodeId::File(root.join("agent-web/app/agent/page.tsx"));

    let control_dependencies = graph
        .dependencies_of_node(&control_test)
        .expect("control spec is present in prepared graph");
    assert!(control_dependencies.contains(&(control_page.clone(), EdgeKind::RouteTest)));
    assert!(control_dependencies.contains(&(control_page, EdgeKind::Selector)));

    let agent_dependencies = graph
        .dependencies_of_node(&agent_test)
        .expect("agent spec is present in prepared graph");
    assert!(agent_dependencies.contains(&(agent_page.clone(), EdgeKind::RouteTest)));
    assert!(agent_dependencies.contains(&(agent_page, EdgeKind::Selector)));
}

/// `PreparedGraphConfig::playwright_fact_plan` builds one fact plan per
/// resolved frontend app and propagates the first one that fails to load —
/// here, a `tests.playwright.configs` path that doesn't exist on disk. Both
/// apps in this fixture share the same (missing) config, so the very first
/// app in the per-app loop already fails.
#[test]
fn playwright_fact_plan_propagates_a_missing_playwright_config() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-multi-frontend-apps"));
    let plan = GraphBuildPlan {
        playwright_routes: true,
        ..GraphBuildPlan::default()
    };
    let missing_config = root.join("missing-config.no-mistakes.yml");
    let loaded = crate::config::v2::load_v2_config(&root, Some(&missing_config)).unwrap();
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, Some(&missing_config), &loaded);
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let prepared =
        prepare_graph_config(&root, plan, &codebase_config, &loaded, &visible).unwrap();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };

    let result = prepared.playwright_fact_plan(&root, &tsconfig, &visible);
    let error = match result {
        Ok(_) => panic!("expected a missing Playwright config to fail fact-plan construction"),
        Err(error) => error,
    };
    assert!(
        format!("{error:#}").contains("does-not-exist.playwright.config.mts"),
        "{error:#}"
    );
}

/// A `type: nextjs` project whose root can't be inferred (no `root:`, no
/// discoverable `next.config.*`) must fail `prepare_graph_config` outright
/// when Playwright edges are requested — the graph's per-app settings
/// resolution surfaces the same actionable error a Playwright rule would,
/// rather than silently building zero Playwright edges.
#[test]
fn prepare_graph_config_surfaces_an_unresolvable_frontend_app() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "graph-playwright-app-resolution-error",
    ));
    let plan = GraphBuildPlan {
        playwright_routes: true,
        ..GraphBuildPlan::default()
    };
    let loaded = crate::config::v2::load_v2_config(&root, None).unwrap();
    let codebase_config = crate::codebase::config::config_from_loaded_v2(&root, None, &loaded);
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);

    let result = prepare_graph_config(&root, plan, &codebase_config, &loaded, &visible);
    let error = match result {
        Ok(_) => panic!("expected prepare_graph_config to fail for an unresolvable frontend app"),
        Err(error) => error,
    };

    let message = format!("{error:#}");
    assert!(message.contains("web"), "{message}");
    assert!(message.contains("projects.web.root"), "{message}");
}

#[test]
fn playwright_route_edges_use_explicit_config_path() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("playwright-config-path-graph"));
    let all_files = GraphFiles::discover(&root).all;

    assert!(collect_playwright_route_edges(&root, None, &all_files, None).is_empty());

    let custom_config = root.join("custom.no-mistakes.yml");
    let edges = collect_playwright_route_edges(&root, Some(&custom_config), &all_files, None);
    let test = NodeId::File(root.join("tests/e2e/app.spec.ts"));
    let page = NodeId::File(root.join("web/app/page.tsx"));
    let layout = NodeId::File(root.join("web/app/layout.tsx"));
    assert!(edges.contains(&(test, page.clone(), EdgeKind::RouteTest)));
    assert!(edges.contains(&(page, layout, EdgeKind::Layout)));
}
