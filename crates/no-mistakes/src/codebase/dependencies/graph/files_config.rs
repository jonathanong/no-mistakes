pub(crate) struct GraphFiles {
    all: Vec<PathBuf>,
    indexable: Vec<PathBuf>,
    visible: HashSet<PathBuf>,
}

impl GraphFiles {
    pub(crate) fn discover(root: &Path) -> Self {
        let all = crate::codebase::ts_source::discover_files(root, &[]);
        Self::from_files(all)
    }

    pub(crate) fn from_files(all: Vec<PathBuf>) -> Self {
        let visible = all.iter().cloned().collect();
        let indexable = all.iter().filter(|p| is_indexable(p)).cloned().collect();
        Self {
            all,
            indexable,
            visible,
        }
    }

    fn is_visible(&self, path: &Path) -> bool {
        self.visible.contains(path)
    }

    pub(crate) fn indexable(&self) -> &[PathBuf] {
        &self.indexable
    }

    pub(crate) fn all(&self) -> &[PathBuf] {
        &self.all
    }

    pub(crate) fn visible(&self) -> &HashSet<PathBuf> {
        &self.visible
    }
}

pub(crate) fn ts_fact_context_for_plan(root: &Path, plan: GraphBuildPlan) -> TsFactContext {
    let options = graph_config_options_for_plan(root, plan);
    ts_fact_context_from_options(root, plan, options.as_ref())
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
}

fn graph_config_options(root: &Path) -> Option<GraphConfigOptions> {
    let config = crate::codebase::config::load_config(root).ok()?;
    let v2_config = load_v2_config(root, None).ok();
    let project_route_globs = v2_config
        .as_ref()
        .map(|config| ConfigView::new(config).server_route_globs())
        .unwrap_or_default();
    let project_route_globset = compile_project_route_globset(&project_route_globs);
    let test_filter = v2_config
        .as_ref()
        .map(|config| crate::codebase::test_filter::TestFileFilter::new(root, config));
    let rewrites = v2_config
        .as_ref()
        .map(|c| ConfigView::new(c).nextjs_rewrites().to_vec())
        .unwrap_or_default();
    Some(GraphConfigOptions {
        route: config.rule_options("route-consistency"),
        queue: config.rule_options("queue-dashboard-reachability"),
        http_route: config.rule_options("http-route-static-paths"),
        http_call: config.rule_options("http-call-static-paths"),
        project_route_globset,
        test_filter,
        rewrites,
        queue_project_factory_names: v2_config.as_ref().map(|c| c.queues.factories.clone()).unwrap_or_default(),
    })
}

fn graph_config_options_for_plan(root: &Path, plan: GraphBuildPlan) -> Option<GraphConfigOptions> {
    if graph_plan_needs_config(plan) {
        graph_config_options(root)
    } else {
        None
    }
}

fn graph_plan_needs_config(plan: GraphBuildPlan) -> bool { plan.routes || plan.queues || plan.http }

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

fn resolved_backend_pattern(options: &GraphConfigOptions) -> Option<String> {
    if !options.http_route.backend_pattern.is_empty() {
        Some(options.http_route.backend_pattern.clone())
    } else {
        route_backend_pattern(options)
    }
}

fn resolved_backend_register_object(options: &GraphConfigOptions) -> Option<String> {
    if !options.http_route.register_object.is_empty() {
        Some(options.http_route.register_object.clone())
    } else {
        route_backend_register_object(options)
    }
}

fn resolved_backend_prefixes(options: &GraphConfigOptions) -> Vec<String> {
    if !options.http_call.backend_prefixes.is_empty() {
        options.http_call.backend_prefixes.clone()
    } else {
        route_backend_prefixes(options)
    }
}

fn route_backend_prefixes(options: &GraphConfigOptions) -> Vec<String> {
    options.route.backend_prefixes.clone()
}

fn route_backend_pattern(options: &GraphConfigOptions) -> Option<String> {
    (!options.route.backend_pattern.is_empty()).then(|| options.route.backend_pattern.clone())
}

fn route_backend_register_object(options: &GraphConfigOptions) -> Option<String> {
    (!options.route.backend_register_object.is_empty())
        .then(|| options.route.backend_register_object.clone())
}
