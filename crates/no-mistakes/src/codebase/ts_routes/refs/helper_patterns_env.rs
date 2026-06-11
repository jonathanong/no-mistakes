fn helper_param_env<'a>(
    params: &'a oxc::ast::ast::FormalParameters<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    provided: &HashMap<String, Vec<String>>,
    depth: usize,
) -> HashMap<String, Vec<String>> {
    let mut env = HashMap::new();
    for param in &params.items {
        let Some(name) = binding_identifier_name(&param.pattern) else {
            continue;
        };
        let value = provided
            .get(name)
            .cloned()
            .or_else(|| {
                param.initializer.as_ref().map(|expr| {
                    evaluate_route_expression(expr, defs, imported_helpers, &env, depth + 1)
                })
            })
            .unwrap_or_else(|| vec!["*".to_string()]);
        env.insert(name.to_string(), value);
    }
    env
}
