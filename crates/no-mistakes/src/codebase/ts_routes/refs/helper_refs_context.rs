fn collect_route_context_helper_ref<'a>(
    call: &'a oxc::ast::ast::CallExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &RouterBindings<'a>,
    helper_bindings: &RouteHelperBindings,
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
        refs,
    );
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
    refs: &mut Vec<RouteHelperRef>,
) {
    let mut callees = Vec::new();
    collect_route_helper_callee_names(expr, &mut callees);
    let line = byte_offset_to_line(source, offset);
    refs.extend(callees.into_iter().filter_map(|callee| {
        helper_callee_is_bound(&callee, helper_bindings).then(|| RouteHelperRef {
            callee,
            file: file.to_string(),
            line,
        })
    }));
}

fn collect_route_helper_callee_names(expr: &Expression, callees: &mut Vec<String>) {
    if let Some(callee) = route_helper_callee_name(expr) {
        callees.push(callee);
        return;
    }
    match expr {
        Expression::ConditionalExpression(cond) => {
            collect_route_helper_callee_names(&cond.consequent, callees);
            collect_route_helper_callee_names(&cond.alternate, callees);
        }
        Expression::LogicalExpression(logical) => {
            collect_route_helper_callee_names(&logical.left, callees);
            collect_route_helper_callee_names(&logical.right, callees);
        }
        Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                    continue;
                };
                if property_key_is_pathname(&prop.key) {
                    collect_route_helper_callee_names(&prop.value, callees);
                }
            }
        }
        _ => {}
    }
}

fn property_key_is_pathname(key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(id) => id.name == "pathname",
        PropertyKey::StringLiteral(s) => s.value == "pathname",
        _ => false,
    }
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
