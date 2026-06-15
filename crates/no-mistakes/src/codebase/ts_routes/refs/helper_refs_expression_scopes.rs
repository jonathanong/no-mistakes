fn collect_helper_refs_from_arrow_body<'a>(
    arrow: &'a oxc_ast::ast::ArrowFunctionExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    let mut scoped_bindings = router_bindings.clone();
    let mut scoped_helper_bindings = helper_bindings.clone();
    remove_shadowed_parameters(&arrow.params, &mut scoped_bindings);
    remove_shadowed_helper_parameters(&arrow.params, &mut scoped_helper_bindings);
    collect_router_bindings_for_scope(&arrow.body.statements, &mut scoped_bindings);
    collect_scope_helper_bindings(
        &arrow.body.statements,
        &mut scoped_helper_bindings,
        local_helpers,
    );
    for stmt in &arrow.body.statements {
        collect_helper_refs_from_statement(
            stmt,
            source,
            file,
            &mut scoped_bindings,
            &mut scoped_helper_bindings,
            local_helpers,
            refs,
        );
    }
}

fn collect_helper_refs_from_function_expression<'a>(
    func: &'a oxc_ast::ast::Function<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_function_body(
        func,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
}
