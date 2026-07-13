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
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
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
        graph
            .dependents_of_node(&route)
            .is_some_and(|deps| deps.iter().any(|(from, kind)| *from == client && *kind == EdgeKind::RouteRef))
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
