fn collect_route_helper_callee_names(
    expr: &Expression,
    helper_bindings: &RouteHelperBindings,
    callees: &mut Vec<String>,
) {
    match expr {
        Expression::CallExpression(call) => {
            collect_call_route_helper_callee_names(call, helper_bindings, callees);
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc::ast::ast::ChainElement::CallExpression(call) => {
                collect_call_route_helper_callee_names(call, helper_bindings, callees);
            }
            other => {
                if let Some(callee) = other
                    .as_member_expression()
                    .and_then(route_helper_callee_name_from_member)
                {
                    callees.push(callee);
                }
            }
        },
        Expression::BinaryExpression(binary) => {
            collect_route_helper_callee_names(&binary.left, helper_bindings, callees);
            collect_route_helper_callee_names(&binary.right, helper_bindings, callees);
        }
        Expression::TemplateLiteral(tpl) => {
            for expr in &tpl.expressions {
                collect_route_helper_callee_names(expr, helper_bindings, callees);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_route_helper_callee_names(&paren.expression, helper_bindings, callees);
        }
        Expression::TSAsExpression(ts_as) => {
            collect_route_helper_callee_names(&ts_as.expression, helper_bindings, callees);
        }
        Expression::TSTypeAssertion(ts_assertion) => {
            collect_route_helper_callee_names(&ts_assertion.expression, helper_bindings, callees);
        }
        Expression::TSNonNullExpression(ts_nn) => {
            collect_route_helper_callee_names(&ts_nn.expression, helper_bindings, callees);
        }
        Expression::TSSatisfiesExpression(ts_sat) => {
            collect_route_helper_callee_names(&ts_sat.expression, helper_bindings, callees);
        }
        Expression::AwaitExpression(await_expr) => {
            collect_route_helper_callee_names(&await_expr.argument, helper_bindings, callees);
        }
        Expression::ConditionalExpression(cond) => {
            collect_route_helper_callee_names(&cond.consequent, helper_bindings, callees);
            collect_route_helper_callee_names(&cond.alternate, helper_bindings, callees);
        }
        Expression::LogicalExpression(logical) => {
            collect_route_helper_callee_names(&logical.left, helper_bindings, callees);
            collect_route_helper_callee_names(&logical.right, helper_bindings, callees);
        }
        Expression::ObjectExpression(obj) => {
            collect_object_route_helper_callee_names(obj, helper_bindings, callees);
        }
        _ => {}
    }
}

fn collect_call_route_helper_callee_names(
    call: &oxc::ast::ast::CallExpression<'_>,
    helper_bindings: &RouteHelperBindings,
    callees: &mut Vec<String>,
) {
    if let Some(callee) = bound_route_helper_callee_name(&call.callee, helper_bindings) {
        callees.push(callee);
        return;
    }
    for arg in &call.arguments {
        if let Some(expr) = arg.as_expression() {
            collect_route_helper_callee_names(expr, helper_bindings, callees);
        }
    }
}

fn collect_object_route_helper_callee_names(
    obj: &oxc::ast::ast::ObjectExpression<'_>,
    helper_bindings: &RouteHelperBindings,
    callees: &mut Vec<String>,
) {
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if property_key_is_pathname(&prop.key) {
            collect_route_helper_callee_names(&prop.value, helper_bindings, callees);
        }
    }
}

fn bound_route_helper_callee_name(
    callee: &Expression,
    helper_bindings: &RouteHelperBindings,
) -> Option<String> {
    let name = route_helper_callee_name_from_callee(callee)?;
    bound_helper_callee_name(&name, helper_bindings).map(|_| name)
}
