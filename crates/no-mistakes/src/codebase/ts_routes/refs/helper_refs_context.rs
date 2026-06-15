fn collect_route_context_helper_ref<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &RouterBindings<'a>,
    helper_bindings: &RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    if !callee_is_route_context(&call.callee, router_bindings) {
        return;
    }

    let Some(first) = call.arguments.first() else {
        return;
    };
    let Some(expr) = first.as_expression() else {
        return;
    };
    push_helper_refs_from_expression(
        expr,
        source,
        file,
        call.span.start as usize,
        helper_bindings,
        local_helpers,
        refs,
    );
}

fn callee_is_route_context(expr: &Expression, router_bindings: &RouterBindings<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            router_bindings.redirects.contains(name)
                || router_bindings.methods.contains(name)
                || (name == "fetch" && !router_bindings.fetch_shadowed)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                callee_is_route_context(&call.callee, router_bindings)
            }
            other => other
                .as_member_expression()
                .is_some_and(|member| member_is_route_context(member, router_bindings)),
        },
        other => other
            .as_member_expression()
            .is_some_and(|member| member_is_route_context(member, router_bindings)),
    }
}

fn member_is_route_context(
    member: &oxc_ast::ast::MemberExpression<'_>,
    router_bindings: &RouterBindings<'_>,
) -> bool {
    let Some(property) = member.static_property_name() else {
        return false;
    };
    if property == "fetch" {
        return matches!(
            member.object(),
            Expression::Identifier(ident)
                if matches!(ident.name.as_str(), "globalThis" | "window" | "self")
        );
    }
    if property != "push" && property != "replace" && property != "prefetch" {
        return false;
    }
    matches!(
        member.object(),
        Expression::Identifier(ident) if router_bindings.objects.contains(ident.name.as_str())
    )
}

fn push_helper_refs_from_expression(
    expr: &Expression,
    source: &str,
    file: &str,
    offset: usize,
    helper_bindings: &RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    let mut candidates = Vec::new();
    collect_route_helper_callee_names(expr, helper_bindings, local_helpers, &mut candidates);
    let line = byte_offset_to_line(source, offset);
    refs.extend(candidates.into_iter().filter_map(|candidate| {
        bound_helper_callee_name(&candidate.callee, helper_bindings).map(|callee| RouteHelperRef {
            file: file.to_string(),
            line,
            callee,
            wrapper_pattern: (candidate.wrapper_pattern != ROUTE_HELPER_REF_PATTERN_MARKER)
                .then_some(candidate.wrapper_pattern),
        })
    }));
}

fn property_key_is_pathname(key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(id) => id.name == "pathname",
        PropertyKey::StringLiteral(s) => s.value == "pathname",
        _ => false,
    }
}

fn route_helper_callee_name_from_callee(callee: &Expression) -> Option<String> {
    match callee {
        Expression::Identifier(id) => Some(id.name.as_str().to_string()),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                route_helper_callee_name_from_callee(&call.callee)
            }
            other => route_helper_callee_name_from_member(other.as_member_expression()?),
        },
        other => route_helper_callee_name_from_member(other.as_member_expression()?),
    }
}

fn route_helper_callee_name_from_member(
    member: &oxc_ast::ast::MemberExpression<'_>,
) -> Option<String> {
    let object = match member.object() {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return None,
    };
    let property = member.static_property_name()?;
    Some(format!("{object}.{property}"))
}
