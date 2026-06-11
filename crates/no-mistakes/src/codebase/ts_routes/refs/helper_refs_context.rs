fn collect_route_context_helper_ref<'a>(
    call: &'a oxc::ast::ast::CallExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &RouterBindings<'a>,
    refs: &mut Vec<RouteHelperRef>,
) {
    let mut route_context = false;
    if let Some(member) = call.callee.as_member_expression() {
        let is_router_method = member
            .static_property_name()
            .is_some_and(|prop| prop == "push" || prop == "replace" || prop == "prefetch");
        if is_router_method {
            if let Expression::Identifier(ident) = member.object() {
                route_context = router_bindings.objects.contains(ident.name.as_str());
            }
        }
    }

    if let Expression::Identifier(id) = &call.callee {
        let name = id.name.as_str();
        route_context = route_context
            || router_bindings.redirects.contains(name)
            || router_bindings.methods.contains(name)
            || name == "fetch";
    } else if call
        .callee
        .as_member_expression()
        .and_then(|member| member.static_property_name())
        .is_some_and(|name| name == "fetch")
    {
        route_context = true;
    }

    if !route_context {
        return;
    }

    let Some(first) = call.arguments.first() else {
        return;
    };
    let Some(expr) = first.as_expression() else {
        return;
    };
    push_helper_ref_from_expression(expr, source, file, call.span.start as usize, refs);
}

fn push_helper_ref_from_expression(
    expr: &Expression,
    source: &str,
    file: &str,
    offset: usize,
    refs: &mut Vec<RouteHelperRef>,
) {
    let Some(callee) = route_helper_callee_name(expr) else {
        return;
    };
    refs.push(RouteHelperRef {
        callee,
        file: file.to_string(),
        line: byte_offset_to_line(source, offset),
    });
}

fn route_helper_callee_name(expr: &Expression) -> Option<String> {
    match expr {
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(id) => Some(id.name.as_str().to_string()),
            other => {
                let member = other.as_member_expression()?;
                let object = match member.object() {
                    Expression::Identifier(id) => id.name.as_str(),
                    _ => return None,
                };
                let property = member.static_property_name()?;
                Some(format!("{object}.{property}"))
            }
        },
        Expression::BinaryExpression(binary) => route_helper_callee_name(&binary.left)
            .or_else(|| route_helper_callee_name(&binary.right)),
        Expression::TemplateLiteral(tpl) => tpl
            .expressions
            .iter()
            .find_map(|expr| route_helper_callee_name(expr)),
        Expression::ParenthesizedExpression(paren) => route_helper_callee_name(&paren.expression),
        Expression::TSAsExpression(ts_as) => route_helper_callee_name(&ts_as.expression),
        Expression::TSTypeAssertion(ts_assertion) => {
            route_helper_callee_name(&ts_assertion.expression)
        }
        Expression::TSNonNullExpression(ts_nn) => route_helper_callee_name(&ts_nn.expression),
        Expression::TSSatisfiesExpression(ts_sat) => route_helper_callee_name(&ts_sat.expression),
        _ => None,
    }
}
