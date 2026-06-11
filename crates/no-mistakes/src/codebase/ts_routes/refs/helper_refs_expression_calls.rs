fn collect_helper_refs_from_call_expression<'a>(
    call: &'a oxc::ast::ast::CallExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_route_context_helper_ref(call, source, file, router_bindings, helper_bindings, refs);
    collect_helper_refs_from_expression(
        &call.callee,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    for arg in &call.arguments {
        collect_helper_refs_from_argument(
            arg,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        );
    }
}

fn collect_helper_refs_from_chain_expression<'a>(
    chain: &'a oxc::ast::ast::ChainExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match &chain.expression {
        oxc::ast::ast::ChainElement::CallExpression(call) => {
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
        other => {
            if let Some(member) = other.as_member_expression() {
                collect_helper_refs_from_expression(
                    member.object(),
                    source,
                    file,
                    router_bindings,
                    helper_bindings,
                    local_helpers,
                    refs,
                );
            }
        }
    }
}

fn collect_helper_refs_from_argument<'a>(
    arg: &'a Argument<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match arg {
        Argument::SpreadElement(spread) => {
            collect_helper_refs_from_expression(
                &spread.argument,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        _ => {
            if let Some(expr) = arg.as_expression() {
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
    }
}
