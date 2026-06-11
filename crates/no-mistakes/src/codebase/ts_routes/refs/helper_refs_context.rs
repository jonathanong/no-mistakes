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
        bound_helper_callee_name(&callee, helper_bindings).map(|callee| RouteHelperRef {
            file: file.to_string(),
            line,
            callee,
        })
    }));
}

fn collect_route_helper_callee_names(expr: &Expression, callees: &mut Vec<String>) {
    match expr {
        Expression::CallExpression(call) => {
            if let Some(callee) = route_helper_callee_name_from_callee(&call.callee) {
                callees.push(callee);
            }
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    collect_route_helper_callee_names(expr, callees);
                }
            }
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc::ast::ast::ChainElement::CallExpression(call) => {
                if let Some(callee) = route_helper_callee_name_from_callee(&call.callee) {
                    callees.push(callee);
                }
                for arg in &call.arguments {
                    if let Some(expr) = arg.as_expression() {
                        collect_route_helper_callee_names(expr, callees);
                    }
                }
            }
            other => {
                if let Some(member) = other.as_member_expression() {
                    if let Some(callee) = route_helper_callee_name_from_member(member) {
                        callees.push(callee);
                    }
                }
            }
        },
        Expression::BinaryExpression(binary) => {
            collect_route_helper_callee_names(&binary.left, callees);
            collect_route_helper_callee_names(&binary.right, callees);
        }
        Expression::TemplateLiteral(tpl) => {
            for expr in &tpl.expressions {
                collect_route_helper_callee_names(expr, callees);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_route_helper_callee_names(&paren.expression, callees);
        }
        Expression::TSAsExpression(ts_as) => {
            collect_route_helper_callee_names(&ts_as.expression, callees);
        }
        Expression::TSTypeAssertion(ts_assertion) => {
            collect_route_helper_callee_names(&ts_assertion.expression, callees);
        }
        Expression::TSNonNullExpression(ts_nn) => {
            collect_route_helper_callee_names(&ts_nn.expression, callees);
        }
        Expression::TSSatisfiesExpression(ts_sat) => {
            collect_route_helper_callee_names(&ts_sat.expression, callees);
        }
        Expression::AwaitExpression(await_expr) => {
            collect_route_helper_callee_names(&await_expr.argument, callees);
        }
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
