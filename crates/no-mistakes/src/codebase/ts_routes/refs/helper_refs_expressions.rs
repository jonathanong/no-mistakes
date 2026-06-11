fn collect_helper_refs_from_expression<'a>(
    expr: &'a Expression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match expr {
        Expression::JSXElement(jsx_elem) => {
            collect_helper_refs_from_jsx_element(jsx_elem, source, file, router_bindings, refs);
        }
        Expression::JSXFragment(frag) => {
            for child in &frag.children {
                collect_helper_refs_from_jsx_child(child, source, file, router_bindings, refs);
            }
        }
        Expression::CallExpression(call) => {
            collect_route_context_helper_ref(call, source, file, router_bindings, refs);
            collect_helper_refs_from_expression(&call.callee, source, file, router_bindings, refs);
            for arg in &call.arguments {
                collect_helper_refs_from_argument(arg, source, file, router_bindings, refs);
            }
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc::ast::ast::ChainElement::CallExpression(call) => {
                collect_route_context_helper_ref(call, source, file, router_bindings, refs);
                collect_helper_refs_from_expression(
                    &call.callee,
                    source,
                    file,
                    router_bindings,
                    refs,
                );
                for arg in &call.arguments {
                    collect_helper_refs_from_argument(arg, source, file, router_bindings, refs);
                }
            }
            other => {
                if let Some(member) = other.as_member_expression() {
                    collect_helper_refs_from_expression(
                        member.object(),
                        source,
                        file,
                        router_bindings,
                        refs,
                    );
                }
            }
        },
        Expression::ArrowFunctionExpression(arrow) => {
            let mut scoped_bindings = router_bindings.clone();
            remove_shadowed_parameters(&arrow.params, &mut scoped_bindings);
            collect_router_bindings_for_scope(&arrow.body.statements, &mut scoped_bindings);
            for stmt in &arrow.body.statements {
                collect_helper_refs_from_statement(stmt, source, file, &mut scoped_bindings, refs);
            }
        }
        Expression::FunctionExpression(func) => {
            collect_helper_refs_from_function_body(func, source, file, router_bindings, refs);
        }
        Expression::ConditionalExpression(cond) => {
            collect_helper_refs_from_expression(&cond.test, source, file, router_bindings, refs);
            collect_helper_refs_from_expression(
                &cond.consequent,
                source,
                file,
                router_bindings,
                refs,
            );
            collect_helper_refs_from_expression(
                &cond.alternate,
                source,
                file,
                router_bindings,
                refs,
            );
        }
        Expression::LogicalExpression(logical) => {
            collect_helper_refs_from_expression(&logical.left, source, file, router_bindings, refs);
            collect_helper_refs_from_expression(&logical.right, source, file, router_bindings, refs);
        }
        Expression::SequenceExpression(seq) => {
            for expr in &seq.expressions {
                collect_helper_refs_from_expression(expr, source, file, router_bindings, refs);
            }
        }
        Expression::AssignmentExpression(assign) => {
            collect_helper_refs_from_expression(&assign.right, source, file, router_bindings, refs);
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_helper_refs_from_expression(
                &paren.expression,
                source,
                file,
                router_bindings,
                refs,
            );
        }
        Expression::TSAsExpression(ts_as) => {
            collect_helper_refs_from_expression(
                &ts_as.expression,
                source,
                file,
                router_bindings,
                refs,
            );
        }
        Expression::TSTypeAssertion(ts_assertion) => {
            collect_helper_refs_from_expression(
                &ts_assertion.expression,
                source,
                file,
                router_bindings,
                refs,
            );
        }
        Expression::TSNonNullExpression(ts_nn) => {
            collect_helper_refs_from_expression(
                &ts_nn.expression,
                source,
                file,
                router_bindings,
                refs,
            );
        }
        Expression::TSSatisfiesExpression(ts_sat) => {
            collect_helper_refs_from_expression(
                &ts_sat.expression,
                source,
                file,
                router_bindings,
                refs,
            );
        }
        _ => {}
    }
}

fn collect_helper_refs_from_argument<'a>(
    arg: &'a Argument<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match arg {
        Argument::SpreadElement(spread) => {
            collect_helper_refs_from_expression(
                &spread.argument,
                source,
                file,
                router_bindings,
                refs,
            );
        }
        _ => {
            if let Some(expr) = arg.as_expression() {
                collect_helper_refs_from_expression(expr, source, file, router_bindings, refs);
            }
        }
    }
}
