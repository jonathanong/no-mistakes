fn collect_route_context_helper_ref<'a>(
    call: &'a oxc::ast::ast::CallExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &RouterBindings<'a>,
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
    push_helper_ref_from_expression(expr, source, file, call.span.start as usize, refs);
}

fn callee_is_route_context(expr: &Expression, router_bindings: &RouterBindings<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            router_bindings.redirects.contains(name)
                || router_bindings.methods.contains(name)
                || name == "fetch"
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc::ast::ast::ChainElement::CallExpression(call) => {
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
    member: &oxc::ast::ast::MemberExpression<'_>,
    router_bindings: &RouterBindings<'_>,
) -> bool {
    let Some(property) = member.static_property_name() else {
        return false;
    };
    if property == "fetch" {
        return true;
    }
    if property != "push" && property != "replace" && property != "prefetch" {
        return false;
    }
    matches!(
        member.object(),
        Expression::Identifier(ident) if router_bindings.objects.contains(ident.name.as_str())
    )
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
        Expression::CallExpression(call) => route_helper_callee_name_from_callee(&call.callee),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc::ast::ast::ChainElement::CallExpression(call) => {
                route_helper_callee_name_from_callee(&call.callee)
            }
            other => route_helper_callee_name_from_member(other.as_member_expression()?),
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

fn route_helper_callee_name_from_callee(callee: &Expression) -> Option<String> {
    match callee {
        Expression::Identifier(id) => Some(id.name.as_str().to_string()),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc::ast::ast::ChainElement::CallExpression(call) => {
                route_helper_callee_name_from_callee(&call.callee)
            }
            other => route_helper_callee_name_from_member(other.as_member_expression()?),
        },
        other => route_helper_callee_name_from_member(other.as_member_expression()?),
    }
}

fn route_helper_callee_name_from_member(
    member: &oxc::ast::ast::MemberExpression<'_>,
) -> Option<String> {
    let object = match member.object() {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return None,
    };
    let property = member.static_property_name()?;
    Some(format!("{object}.{property}"))
}
