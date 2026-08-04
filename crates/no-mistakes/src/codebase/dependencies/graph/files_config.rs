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
    ci: crate::config::v2::schema::CiConfig,
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
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    Some(graph_config_options_from_loaded(
        root,
        &config,
        v2_config.as_ref()?,
        &visible_paths,
    ))
}

fn graph_config_options_from_loaded(
    root: &Path,
    config: &crate::codebase::config::Config,
    v2_config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> GraphConfigOptions {
    graph_config_options_from_loaded_with_test_filter(root, config, v2_config, visible_paths, None)
}

fn graph_config_options_from_loaded_with_test_filter(
    root: &Path,
    config: &crate::codebase::config::Config,
    v2_config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &[PathBuf],
    test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
) -> GraphConfigOptions {
    let project_route_globs = ConfigView::new(v2_config).server_route_globs();
    let test_filter = Some(test_filter.unwrap_or_else(|| {
        crate::codebase::test_filter::TestFileFilter::new(root, v2_config)
    }));
    // Union of every configured frontend app's rewrites. #624 was caused by
    // this list being sourced from whichever `type: nextjs` project sorted
    // first. `frontend_apps_lenient` resolves each `type: nextjs` project
    // independently and skips only the ones whose root can't be inferred,
    // so one app's ambiguous root no longer discards every other app's
    // rewrites too — this is a best-effort convenience list, not a
    // Playwright rule execution, so per-app skip-on-error is intentional.
    let rewrites = crate::config::v2::frontend_apps_lenient(root, v2_config, visible_paths)
        .into_iter()
        .flat_map(|app| app.rewrites)
        .collect::<Vec<_>>();
    let rewrites = dedup_rewrites(rewrites);
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
        ci: v2_config.ci.clone(),
    }
}

fn dedup_rewrites(
    mut rewrites: Vec<crate::config::v2::schema::RewriteRule>,
) -> Vec<crate::config::v2::schema::RewriteRule> {
    rewrites.sort_by(|a, b| (&a.source, &a.destination).cmp(&(&b.source, &b.destination)));
    rewrites.dedup_by(|a, b| a.source == b.source && a.destination == b.destination);
    rewrites
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
