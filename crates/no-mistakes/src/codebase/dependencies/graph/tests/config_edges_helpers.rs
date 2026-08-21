use super::*;

#[test]
fn graph_config_helpers_require_explicit_prefixes_and_valid_globs() {
    let empty = crate::codebase::ts_resolver::normalize_path(&fixture("graph-empty-route-config"));
    let empty_options = graph_config_options(&empty).unwrap();
    assert!(resolved_backend_prefixes(&empty_options).is_empty());
    assert!(route_backend_prefixes(&empty_options).is_empty());

    let plan = GraphBuildPlan {
        routes: true,
        queues: true,
        http: true,
        ..GraphBuildPlan::default()
    };
    let context = ts_fact_context_from_options(&empty, plan, Some(&empty_options));
    assert!(context.backend_route_extractors.is_empty());
    assert!(context.queue_factory_glob.is_none());
    assert!(context.http_prefixes.is_empty());
    let context_without_options = ts_fact_context_from_options(&empty, plan, None);
    assert!(context_without_options.backend_route_extractors.is_empty());

    let mut manual_context = TsFactContext::new(&empty);
    add_backend_route_extractor(
        &mut manual_context,
        None,
        Some("backend/**/*.mts".to_string()),
    );
    add_backend_route_extractor(&mut manual_context, Some("app".to_string()), None);
    add_backend_route_extractor(
        &mut manual_context,
        Some("app".to_string()),
        Some("[".to_string()),
    );
    assert!(manual_context.backend_route_extractors.is_empty());

    assert!(compile_graph_glob("").is_none());
    assert!(compile_graph_glob("[").is_none());
    assert!(compile_graph_glob("backend/**/*.mts")
        .expect("valid graph glob should compile")
        .is_match(Path::new("backend/api/users.mts")));

    let explicit =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let explicit_options = graph_config_options(&explicit).unwrap();
    let explicit_route_prefixes = vec![
        "/api/".to_string(),
        "/prefix/".to_string(),
        "/crawler/".to_string(),
    ];
    assert_eq!(
        resolved_backend_prefixes(&explicit_options),
        vec!["/api/".to_string()]
    );
    assert_eq!(
        route_backend_prefixes(&explicit_options),
        explicit_route_prefixes
    );

    let missing_register_options = GraphConfigOptions {
        route: crate::codebase::config::RouteOptions::default(),
        queue: crate::codebase::config::QueueOptions::default(),
        http_route: crate::codebase::config::HttpRouteOptions {
            backend_pattern: "backend/**/*.mts".to_string(),
            register_object: String::new(),
        },
        http_call: crate::codebase::config::HttpCallOptions {
            backend_prefixes: vec!["/api/".to_string()],
        },
        project_route_globset: None,
        test_filter: None,
        rewrites: vec![],
        queue_project_factory_names: vec![],
        dotnet_projects: vec![],
        swift_packages: vec![],
        python_packages: vec![],
        go_modules: vec![],
        rust_packages: vec![],
        rails_apps: vec![],
        php_apps: vec![],
        php_framework: None,
        queue_enqueues: vec![],
        queue_workers: vec![],
        queue_cluster: None,
        queue_glob_clusters: HashMap::new(),
        terraform: Default::default(),
        ci: crate::config::v2::schema::CiConfig::default(),
    };
    let invalid_glob_options = GraphConfigOptions {
        route: crate::codebase::config::RouteOptions::default(),
        queue: crate::codebase::config::QueueOptions::default(),
        http_route: crate::codebase::config::HttpRouteOptions {
            backend_pattern: "[".to_string(),
            register_object: "app".to_string(),
        },
        http_call: crate::codebase::config::HttpCallOptions {
            backend_prefixes: vec!["/api/".to_string()],
        },
        project_route_globset: None,
        test_filter: None,
        rewrites: vec![],
        queue_project_factory_names: vec![],
        dotnet_projects: vec![],
        swift_packages: vec![],
        python_packages: vec![],
        go_modules: vec![],
        rust_packages: vec![],
        rails_apps: vec![],
        php_apps: vec![],
        php_framework: None,
        queue_enqueues: vec![],
        queue_workers: vec![],
        queue_cluster: None,
        queue_glob_clusters: HashMap::new(),
        terraform: Default::default(),
        ci: crate::config::v2::schema::CiConfig::default(),
    };
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&explicit.join("tsconfig.json")).unwrap();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    assert!(collect_route_edges(
        &explicit,
        &tsconfig,
        &resolver,
        &[],
        None,
        Some(&explicit_options),
    )
    .is_empty());
    assert!(collect_http_call_edges(
        &explicit,
        None,
        &[],
        &[],
        &[],
        Some(&explicit_options),
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());

    let queue_options = GraphConfigOptions {
        route: crate::codebase::config::RouteOptions::default(),
        queue: crate::codebase::config::QueueOptions {
            queue_pattern: "src/**/*.ts".to_string(),
            factory_specifier: "@app/queue".to_string(),
            factory_function: "createQueue".to_string(),
        },
        http_route: crate::codebase::config::HttpRouteOptions::default(),
        http_call: crate::codebase::config::HttpCallOptions::default(),
        project_route_globset: None,
        test_filter: None,
        rewrites: vec![],
        queue_project_factory_names: vec![],
        dotnet_projects: vec![],
        swift_packages: vec![],
        python_packages: vec![],
        go_modules: vec![],
        rust_packages: vec![],
        rails_apps: vec![],
        php_apps: vec![],
        php_framework: None,
        queue_enqueues: vec![],
        queue_workers: vec![],
        queue_cluster: None,
        queue_glob_clusters: HashMap::new(),
        terraform: Default::default(),
        ci: crate::config::v2::schema::CiConfig::default(),
    };
    let mut forward = EdgeMap::default();
    let mut reverse = EdgeMap::default();
    test_support::add_queue_edges(
        &explicit,
        &resolver,
        &[],
        None,
        Some(&queue_options),
        &mut forward,
        &mut reverse,
    );
    assert!(forward.is_empty());

    assert!(collect_http_call_edges(
        &explicit,
        None,
        &[],
        &[],
        &[],
        Some(&missing_register_options),
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
    assert!(collect_http_call_edges(
        &explicit,
        None,
        &[],
        &[],
        &[],
        Some(&invalid_glob_options),
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
}
