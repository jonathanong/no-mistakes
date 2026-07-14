pub fn ts_fact_plan_and_context_for_plan(
    root: &Path,
    plan: GraphBuildPlan,
) -> (TsFactPlan, TsFactContext) {
    ts_fact_plan_and_context_for_plan_with_config(root, plan, None)
}

/// Same as [`ts_fact_plan_and_context_for_plan`], but resolves
/// `GraphConfigOptions` from an explicit `--config` path instead of always
/// falling back to default discovery. Callers that already have a
/// `config_path` in scope (e.g. `check_runner`, `forbidden_dependencies`'s
/// shared-facts path) must use this, not the config-insensitive variant —
/// otherwise the `TsFactPlan`/`TsFactContext` used to *collect* facts can
/// disagree with the `GraphConfigOptions` the `DepGraph` build itself later
/// resolves from the same `config_path`, silently missing backend
/// route/http/queue facts a custom config enables.
pub fn ts_fact_plan_and_context_for_plan_with_config(
    root: &Path,
    plan: GraphBuildPlan,
    config_path: Option<&Path>,
) -> (TsFactPlan, TsFactContext) {
    let options = graph_config_options_for_plan_with_config(root, plan, config_path);
    (
        effective_ts_fact_plan(plan, options.as_ref()),
        ts_fact_context_from_options(root, plan, options.as_ref()),
    )
}

#[derive(Clone)]
struct GraphConfigOptions {
    route: crate::codebase::config::RouteOptions,
    queue: crate::codebase::config::QueueOptions,
    http_route: crate::codebase::config::HttpRouteOptions,
    http_call: crate::codebase::config::HttpCallOptions,
    project_route_globset: Option<GlobSet>,
    test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
    rewrites: Vec<crate::config::v2::schema::RewriteRule>,
    queue_project_factory_names: Vec<String>,
    dotnet_projects: Vec<crate::codebase::dotnet::DotnetConfigProject>,
    swift_packages: Vec<String>,
    terraform: crate::config::v2::schema::TerraformConfig,
}

fn graph_config_options(root: &Path) -> Option<GraphConfigOptions> {
    graph_config_options_with_config(root, None)
}

fn graph_config_options_with_config(
    root: &Path,
    config_path: Option<&Path>,
) -> Option<GraphConfigOptions> {
    let config = match config_path {
        Some(path) => crate::codebase::config::load_config_with_path(root, Some(path)),
        None => crate::codebase::config::load_config(root),
    }
    .ok()?;
    let v2_config = load_v2_config(root, config_path).ok();
    Some(graph_config_options_from_loaded(
        root,
        &config,
        v2_config.as_ref()?,
    ))
}

fn graph_config_options_from_loaded(
    root: &Path,
    config: &crate::codebase::config::Config,
    v2_config: &crate::config::v2::NoMistakesConfig,
) -> GraphConfigOptions {
    graph_config_options_from_loaded_with_test_filter(root, config, v2_config, None)
}

fn graph_config_options_from_loaded_with_test_filter(
    root: &Path,
    config: &crate::codebase::config::Config,
    v2_config: &crate::config::v2::NoMistakesConfig,
    test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
) -> GraphConfigOptions {
    let project_route_globs = ConfigView::new(v2_config).server_route_globs();
    let test_filter = Some(test_filter.unwrap_or_else(|| {
        crate::codebase::test_filter::TestFileFilter::new(root, v2_config)
    }));
    let rewrites = ConfigView::new(v2_config).nextjs_rewrites().to_vec();
    GraphConfigOptions {
        route: config.rule_options("route-consistency"),
        queue: config.rule_options("queue-dashboard-reachability"),
        http_route: config.rule_options("http-route-static-paths"),
        http_call: config.rule_options("http-call-static-paths"),
        project_route_globset: compile_project_route_globset(&project_route_globs),
        test_filter,
        rewrites,
        queue_project_factory_names: v2_config.queues.factories.clone(),
        dotnet_projects: crate::codebase::dotnet::configured_projects(root, &v2_config.tests.dotnet),
        swift_packages: v2_config.tests.swift.packages.clone(),
        terraform: v2_config.infra.terraform.clone(),
    }
}

fn graph_config_options_for_plan_with_config(
    root: &Path,
    plan: GraphBuildPlan,
    config_path: Option<&Path>,
) -> Option<GraphConfigOptions> {
    if graph_plan_needs_config(plan) {
        match config_path {
            Some(_) => graph_config_options_with_config(root, config_path),
            None => graph_config_options(root),
        }
    } else {
        None
    }
}

fn ts_fact_context_from_options(
    root: &Path,
    plan: GraphBuildPlan,
    options: Option<&GraphConfigOptions>,
) -> TsFactContext {
    let mut context = TsFactContext::new(root);
    let Some(options) = options else {
        return context;
    };
    if plan.routes {
        add_backend_route_extractor(
            &mut context,
            route_backend_register_object(options),
            route_backend_pattern(options),
        );
    }
    if plan.http {
        add_backend_route_extractor(
            &mut context,
            resolved_backend_register_object(options),
            resolved_backend_pattern(options),
        );
        context.http_prefixes = resolved_backend_prefixes(options);
    }
    if plan.queues
        && !options.queue.factory_specifier.is_empty()
        && !options.queue.factory_function.is_empty()
    {
        context.queue_factory_specifier = Some(options.queue.factory_specifier.clone());
        context.queue_factory_function = Some(options.queue.factory_function.clone());
        context.queue_factory_glob = compile_graph_glob(&options.queue.queue_pattern);
        context.queue_project_factory_names = options.queue_project_factory_names.clone();
    }
    context
}

fn route_ref_facts_configured(options: &GraphConfigOptions) -> bool {
    route_backend_facts_configured(options)
        || !options.route.frontend_root.is_empty()
        || options.project_route_globset.is_some()
}

fn route_backend_facts_configured(options: &GraphConfigOptions) -> bool {
    let pattern = route_backend_pattern(options);
    let has_register_object = route_backend_register_object(options).is_some();
    let has_valid_glob = pattern.as_deref().and_then(compile_graph_glob).is_some();
    has_register_object && has_valid_glob
}

fn http_facts_configured(options: &GraphConfigOptions) -> bool {
    let pattern = resolved_backend_pattern(options);
    let has_register_object = resolved_backend_register_object(options).is_some();
    let has_prefixes = !resolved_backend_prefixes(options).is_empty();
    let has_valid_glob = pattern.as_deref().and_then(compile_graph_glob).is_some();
    let has_next_route_handlers = !options.route.frontend_root.is_empty();
    has_prefixes && ((has_register_object && has_valid_glob) || has_next_route_handlers)
}

fn queue_facts_configured(options: &GraphConfigOptions) -> bool {
    !options.queue.factory_specifier.is_empty()
        && !options.queue.factory_function.is_empty()
        && compile_graph_glob(&options.queue.queue_pattern).is_some()
}

fn add_backend_route_extractor(
    context: &mut TsFactContext,
    register_object: Option<String>,
    pattern: Option<String>,
) {
    let (Some(register_object), Some(pattern)) = (register_object, pattern) else {
        return;
    };
    let Some(glob) = compile_graph_glob(&pattern) else {
        return;
    };
    context.add_backend_route_extractor(register_object, pattern, glob);
}

fn compile_graph_glob(pattern: &str) -> Option<GlobSet> {
    if pattern.is_empty() {
        return None;
    }
    let glob = GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .ok()?;
    let mut builder = GlobSetBuilder::new();
    builder.add(glob);
    builder.build().ok()
}
