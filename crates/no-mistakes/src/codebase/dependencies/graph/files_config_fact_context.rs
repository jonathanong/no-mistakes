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
