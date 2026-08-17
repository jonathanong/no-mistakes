use super::*;

#[test]
fn graph_collectors_cover_malformed_and_invalid_config_branches() {
    let source_root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&source_root.join("tsconfig.json")).unwrap();
    let files = vec![source_root.join("packages/api/src/index.mts")];
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);

    let malformed =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-malformed-config"));
    let invalid = crate::codebase::ts_resolver::normalize_path(&fixture("graph-invalid-globs"));
    let empty = crate::codebase::ts_resolver::normalize_path(&fixture("graph-empty-route-config"));
    let frontend_only =
        crate::codebase::ts_resolver::normalize_path(&fixture("playwright-coverage"));
    let frontend_files = GraphFiles::discover(&frontend_only).all;
    let malformed_options = graph_config_options(&malformed);
    let invalid_options = graph_config_options(&invalid);
    let empty_options = graph_config_options(&empty);
    let frontend_options = graph_config_options(&frontend_only);

    assert!(collect_route_edges(
        &malformed,
        &tsconfig,
        &resolver,
        &files,
        None,
        malformed_options.as_ref()
    )
    .is_empty());
    assert!(collect_route_edges(
        &invalid,
        &tsconfig,
        &resolver,
        &files,
        None,
        invalid_options.as_ref(),
    )
    .is_empty());
    assert!(collect_route_edges(
        &empty,
        &tsconfig,
        &resolver,
        &files,
        None,
        empty_options.as_ref(),
    )
    .is_empty());
    assert!(collect_route_edges(
        &frontend_only,
        &tsconfig,
        &resolver,
        &frontend_files,
        None,
        frontend_options.as_ref(),
    )
    .is_empty());

    let mut forward = EdgeMap::new();
    let mut reverse = EdgeMap::new();
    test_support::add_queue_edges(
        &malformed,
        &resolver,
        &files,
        None,
        malformed_options.as_ref(),
        &mut forward,
        &mut reverse,
    );
    test_support::add_queue_edges(
        &invalid,
        &resolver,
        &files,
        None,
        invalid_options.as_ref(),
        &mut forward,
        &mut reverse,
    );
    test_support::add_queue_edges(
        &empty,
        &resolver,
        &files,
        None,
        empty_options.as_ref(),
        &mut forward,
        &mut reverse,
    );
    assert!(forward.is_empty());

    let sources = vec![(files[0].clone(), "fetch('/api/users')".to_string())];
    assert!(collect_http_call_edges(
        &malformed,
        None,
        &sources,
        &files,
        &files,
        malformed_options.as_ref(),
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
    assert!(collect_http_call_edges(
        &invalid,
        None,
        &sources,
        &files,
        &files,
        invalid_options.as_ref(),
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
}

#[test]
fn graph_config_options_use_explicit_config_for_legacy_rule_options() {
    let empty = crate::codebase::ts_resolver::normalize_path(&fixture("graph-empty-route-config"));
    let explicit = crate::codebase::ts_resolver::normalize_path(
        &fixture("graph-default-route-config").join(".no-mistakes.yml"),
    );

    let default_options = graph_config_options_with_config(&empty, None).unwrap();
    let explicit_options = graph_config_options_with_config(&empty, Some(&explicit)).unwrap();

    assert!(!route_backend_facts_configured(&default_options));
    assert!(route_backend_facts_configured(&explicit_options));
}

#[test]
fn route_collectors_cover_configured_prefixes_and_scan_globs() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let client = root.join("src/client.ts");
    let route = root.join("backend/api/users.mts");
    let entity_route = root.join("backend/api/entity.mts");
    let admin_route = root.join("backend/api/admin.mts");
    let fake_route = root.join("src/fake-backend.mts");
    let config_options = graph_config_options(&root);
    let fact_plan = effective_ts_fact_plan(
        GraphBuildPlan {
            routes: true,
            ..GraphBuildPlan::default()
        },
        config_options.as_ref(),
    );
    let fact_context = ts_fact_context_for_plan(
        &root,
        GraphBuildPlan {
            routes: true,
            ..GraphBuildPlan::default()
        },
    );
    let facts = collect_ts_facts_with_context(&all_files, fact_plan, &fact_context);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);

    let route_edges = collect_route_edges(
        &root,
        &tsconfig,
        &resolver,
        &all_files,
        Some(&facts),
        config_options.as_ref(),
    );
    assert!(route_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(route.as_path())
    }));
    assert!(route_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(entity_route.as_path())
    }));

    let sources = vec![(client.clone(), std::fs::read_to_string(&client).unwrap())];
    let http_edges = collect_http_call_edges(
        &root,
        None,
        &sources,
        &all_files,
        &all_files,
        config_options.as_ref(),
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(http_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::HttpCall
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(route.as_path())
    }));
    let route_sources = vec![(route.clone(), std::fs::read_to_string(&route).unwrap())];
    let route_http_edges = collect_http_call_edges(
        &root,
        None,
        &route_sources,
        &all_files,
        &all_files,
        config_options.as_ref(),
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(route_http_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::HttpCall
            && from.as_file() == Some(route.as_path())
            && to.as_file() == Some(admin_route.as_path())
    }));

    let fact_plan = GraphBuildPlan {
        routes: true,
        http: true,
        ..GraphBuildPlan::default()
    };
    let fact_context = ts_fact_context_for_plan(&root, fact_plan);
    let facts = collect_ts_facts_with_context(&all_files, fact_plan.ts_fact_plan(), &fact_context);
    assert!(facts
        .get(&fake_route)
        .expect("fake route source should be parsed")
        .backend_routes
        .is_empty());

    let http_edges_with_facts = collect_http_call_edges(
        &root,
        Some(&facts),
        &[],
        &all_files,
        &all_files,
        config_options.as_ref(),
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(http_edges_with_facts.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::HttpCall
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(route.as_path())
    }));
    assert!(http_edges_with_facts.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::HttpCall
            && from.as_file() == Some(route.as_path())
            && to.as_file() == Some(admin_route.as_path())
    }));
}

#[test]
fn project_route_def_collection_returns_empty_for_invalid_config_globs() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-invalid-project-route-glob"));
    let config_options = graph_config_options(&root).unwrap();
    let invalid_globs = vec!["backend/[".to_string()];
    let fact_plan = effective_ts_fact_plan(
        GraphBuildPlan {
            routes: true,
            ..GraphBuildPlan::default()
        },
        Some(&config_options),
    );

    assert!(config_options.project_route_globset.is_none());
    assert!(!fact_plan.route_refs);
    assert!(compile_project_route_globset(&invalid_globs).is_none());
}

#[test]
fn route_and_http_fact_context_keep_separate_backend_matchers() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-split-route-http-config"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let client = root.join("src/client.ts");
    let route_def = root.join("routes/users.mts");
    let http_def = root.join("http/users.mts");
    let config_options = graph_config_options(&root);
    let plan = GraphBuildPlan {
        routes: true,
        http: true,
        ..GraphBuildPlan::default()
    };
    let context = ts_fact_context_for_plan(&root, plan);
    assert_eq!(context.backend_route_extractors.len(), 2);

    let facts = collect_ts_facts_with_context(&all_files, plan.ts_fact_plan(), &context);
    assert!(facts[&route_def]
        .backend_routes
        .iter()
        .any(|route| { route.register_object == "routeApp" && route.route == "/route/users/:id" }));
    assert!(facts[&http_def]
        .backend_routes
        .iter()
        .any(|route| { route.register_object == "httpApp" && route.route == "/http/users/:id" }));
    assert!(facts[&route_def]
        .backend_routes
        .iter()
        .all(|route| route.register_object != "httpApp"));
    assert!(facts[&http_def]
        .backend_routes
        .iter()
        .all(|route| route.register_object != "routeApp"));
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);

    let route_edges = collect_route_edges(
        &root,
        &tsconfig,
        &resolver,
        &all_files,
        Some(&facts),
        config_options.as_ref(),
    );
    assert!(route_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(route_def.as_path())
    }));
    assert!(route_edges.iter().all(|(_from, to, kind)| {
        *kind != EdgeKind::RouteRef || to.as_file() != Some(http_def.as_path())
    }));

    let http_edges = collect_http_call_edges(
        &root,
        Some(&facts),
        &[],
        &all_files,
        &all_files,
        config_options.as_ref(),
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(http_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::HttpCall
            && from.as_file() == Some(client.as_path())
            && to.as_file() == Some(http_def.as_path())
    }));
    assert!(http_edges.iter().all(|(_from, to, kind)| {
        *kind != EdgeKind::HttpCall || to.as_file() != Some(route_def.as_path())
    }));
}
