use super::*;

#[test]
fn effective_fact_plan_skips_config_dependent_domains_without_required_config() {
    let requested = GraphBuildPlan {
        routes: true,
        queues: true,
        http: true,
        ..GraphBuildPlan::default()
    };
    assert!(effective_ts_fact_plan(requested, None).is_empty());

    let empty = crate::codebase::ts_resolver::normalize_path(&fixture("graph-empty-route-config"));
    let empty_options = graph_config_options(&empty).unwrap();
    assert!(effective_ts_fact_plan(requested, Some(&empty_options)).is_empty());

    let explicit =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let explicit_options = graph_config_options(&explicit).unwrap();
    let route_and_http = effective_ts_fact_plan(requested, Some(&explicit_options));
    assert!(route_and_http.route_refs);
    assert!(route_and_http.backend_routes);
    assert!(route_and_http.http_calls);
    assert!(!route_and_http.symbols);
    assert!(!route_and_http.queue_usage);
    assert!(!route_and_http.queue_factory);

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
        queue_project_factory_names: vec!["createQueue".to_string()],
        dotnet_projects: vec![],
        swift_packages: vec![],
        terraform: Default::default(),
        ci: crate::config::v2::schema::CiConfig::default(),
    };
    let queue_only = effective_ts_fact_plan(
        GraphBuildPlan {
            queues: true,
            ..GraphBuildPlan::default()
        },
        Some(&queue_options),
    );
    assert!(queue_only.symbols);
    assert!(queue_only.queue_usage);
    assert!(queue_only.queue_factory);
    assert!(queue_only.queue_project);
}
