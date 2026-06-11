fn collect_helper_refs_from_conditional_expression<'a>(
    cond: &'a oxc::ast::ast::ConditionalExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_expression(
        &cond.test,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    collect_helper_refs_from_expression(
        &cond.consequent,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    collect_helper_refs_from_expression(
        &cond.alternate,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
}

fn collect_helper_refs_from_logical_expression<'a>(
    logical: &'a oxc::ast::ast::LogicalExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_expression(
        &logical.left,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    collect_helper_refs_from_expression(
        &logical.right,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
}

fn collect_helper_refs_from_unary_wrapper<'a>(
    expr: &'a Expression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_expression(
        expr,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
}
