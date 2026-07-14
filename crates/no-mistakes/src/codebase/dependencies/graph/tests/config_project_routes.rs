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
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);

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

    let route_edges = collect_route_edges(
        &root,
        &tsconfig,
        &resolver,
        &all_files,
        Some(&facts),
        Some(&config_options),
    );
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
        &resolver,
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

#[test]
fn configured_project_routes_reuse_prepared_server_facts() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/project-server-routes"),
    );
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let options = graph_config_options(&root).unwrap();
    let route_globset = options.project_route_globset.as_ref().unwrap();
    let plan = GraphBuildPlan {
        routes: true,
        ..GraphBuildPlan::default()
    };
    let fact_plan = effective_ts_fact_plan(plan, Some(&options));
    assert!(fact_plan.server_routes);
    let facts = collect_ts_facts_with_context(
        &all_files,
        fact_plan,
        &ts_fact_context_from_options(&root, plan, Some(&options)),
    );
    assert!(facts[&root.join("backend/api/users.ts")].server_routes.is_some());
    assert!(facts[&root.join("src/client.ts")].server_routes.is_none());
    assert!(facts[&root.join("backend/api/ignored.test.ts")].server_routes.is_none());

    let standalone = collect_project_server_route_defs(
        &root,
        &all_files,
        &tsconfig,
        route_globset,
        None,
        options.test_filter.as_ref(),
    );
    let reused = collect_project_server_route_defs(
        &root,
        &all_files,
        &tsconfig,
        route_globset,
        Some(&facts),
        options.test_filter.as_ref(),
    );
    assert_eq!(reused, standalone);

    crate::ast::begin_parse_count(&root);
    let graph = DepGraph::build_with_plan(&root, &tsconfig, plan).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(graph
        .dependencies_of_node(&NodeId::File(root.join("src/client.ts")))
        .is_some_and(|edges| edges.iter().any(|(to, kind)| {
            *kind == EdgeKind::RouteRef
                && to.as_file() == Some(root.join("backend/api/users.ts").as_path())
        })));
    assert_eq!(counts.get(&root.join("src/client.ts")), Some(&1));
    assert_eq!(counts.get(&root.join("backend/api/users.ts")), Some(&1));
    assert!(
        counts.values().all(|count| *count == 1),
        "configured graph sources must be parsed once: {counts:#?}"
    );
}

#[test]
fn prepared_project_route_facts_preserve_imported_mounts_and_test_exclusions() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/project-server-routes-mounted"),
    );
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let options = graph_config_options(&root).unwrap();
    let route_globset = options.project_route_globset.as_ref().unwrap();
    let plan = GraphBuildPlan {
        routes: true,
        ..GraphBuildPlan::default()
    };
    let facts = collect_ts_facts_with_context(
        &all_files,
        effective_ts_fact_plan(plan, Some(&options)),
        &ts_fact_context_from_options(&root, plan, Some(&options)),
    );

    let standalone = collect_project_server_route_defs(
        &root,
        &all_files,
        &tsconfig,
        route_globset,
        None,
        options.test_filter.as_ref(),
    );
    let reused = collect_project_server_route_defs(
        &root,
        &all_files,
        &tsconfig,
        route_globset,
        Some(&facts),
        options.test_filter.as_ref(),
    );

    assert_eq!(reused, standalone);
    assert!(reused.contains(&(root.join("backend/api/admin-router.ts"), "/api/admin/*".into())));
    assert!(reused.iter().all(|(file, route)| {
        file != &root.join("backend/api/ignored.test.ts") && route != "/api/test-only"
    }));
}
