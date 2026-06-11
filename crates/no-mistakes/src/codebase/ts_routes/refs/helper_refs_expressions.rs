fn collect_helper_refs_from_expression<'a>(
    expr: &'a Expression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    if collect_helper_refs_from_wrapper_expression(
        expr,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    ) {
        return;
    }
    match expr {
        Expression::JSXElement(jsx_elem) => {
            collect_helper_refs_from_jsx_element(
                jsx_elem,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Expression::JSXFragment(frag) => {
            for child in &frag.children {
                collect_helper_refs_from_jsx_child(
                    child,
                    source,
                    file,
                    router_bindings,
                    helper_bindings,
                    local_helpers,
                    refs,
                );
            }
        }
        Expression::CallExpression(call) => {
            collect_helper_refs_from_call_expression(
                call,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Expression::ChainExpression(chain) => collect_helper_refs_from_chain_expression(
            chain,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        ),
        Expression::ArrowFunctionExpression(arrow) => {
            collect_helper_refs_from_arrow_body(
                arrow,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Expression::FunctionExpression(func) => {
            collect_helper_refs_from_function_expression(
                func,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Expression::ConditionalExpression(cond) => collect_helper_refs_from_conditional_expression(
            cond,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        ),
        Expression::LogicalExpression(logical) => collect_helper_refs_from_logical_expression(
            logical,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        ),
        Expression::SequenceExpression(seq) => {
            for expr in &seq.expressions {
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
        }
        Expression::ObjectExpression(obj) => collect_helper_refs_from_object_expression(
            obj,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        ),
        _ => {}
    }
}
