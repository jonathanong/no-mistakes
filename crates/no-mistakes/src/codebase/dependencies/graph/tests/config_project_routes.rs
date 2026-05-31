use super::*;

#[test]
fn project_route_globs_drive_graph_route_edges_without_guardrails() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-project-route-config"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let client = root.join("src/client.ts");
    let route = root.join("backend/api/users.mts");
    let client_route = root.join("backend/api/client.mts");
    let test_route = root.join("backend/api/users.test.mts");
    let ignored_route = root.join("backend/services/ignored.mts");
    let config_options = graph_config_options(&root).unwrap();

    let project_route_globset = config_options.project_route_globset.as_ref().unwrap();
    assert!(project_route_globset.is_match("backend/api/users.mts"));
    assert!(!project_route_globset.is_match("backend/services/ignored.mts"));

    let fact_plan = effective_ts_fact_plan(
        GraphBuildPlan {
            routes: true,
            ..GraphBuildPlan::default()
        },
        Some(&config_options),
    );
    assert!(fact_plan.route_refs);
    assert!(!fact_plan.backend_routes);
    let fact_context = ts_fact_context_for_plan(
        &root,
        GraphBuildPlan {
            routes: true,
            ..GraphBuildPlan::default()
        },
    );
    let facts = collect_ts_facts_with_context(&all_files, fact_plan, &fact_context);

    let route_edges =
        collect_route_edges(&root, &tsconfig, &all_files, Some(&facts), Some(&config_options));
    assert!(route_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(route.as_path())
    }));
    assert!(route_edges.iter().all(|(_from, to, kind)| {
        *kind != EdgeKind::RouteRef
            || (to.as_file() != Some(test_route.as_path())
                && to.as_file() != Some(client_route.as_path())
                && to.as_file() != Some(ignored_route.as_path()))
    }));

    let mut invalid_legacy_options = config_options.clone();
    invalid_legacy_options.route.backend_pattern = "[".to_string();
    invalid_legacy_options.route.backend_register_object = "app".to_string();
    let invalid_legacy_route_edges = collect_route_edges(
        &root,
        &tsconfig,
        &all_files,
        Some(&facts),
        Some(&invalid_legacy_options),
    );
    assert!(invalid_legacy_route_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(route.as_path())
    }));
}
