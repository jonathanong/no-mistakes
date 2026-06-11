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

fn collect_helper_refs_from_jsx_element<'a>(
    jsx_elem: &'a JSXElement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    refs: &mut Vec<RouteHelperRef>,
) {
    for attr_item in &jsx_elem.opening_element.attributes {
        let JSXAttributeItem::Attribute(attr) = attr_item else {
            continue;
        };
        let attr_name = match &attr.name {
            JSXAttributeName::Identifier(id) => id.name.as_str(),
            JSXAttributeName::NamespacedName(_) => continue,
        };
        if attr_name != "href" && attr_name != "to" {
            continue;
        }
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            continue;
        };
        let Some(expr) = container.expression.as_expression() else {
            continue;
        };
        push_helper_ref_from_expression(expr, source, file, attr.span.start as usize, refs);
    }

    for child in &jsx_elem.children {
        collect_helper_refs_from_jsx_child(child, source, file, router_bindings, refs);
    }
}

fn collect_helper_refs_from_jsx_child<'a>(
    child: &'a JSXChild<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match child {
        JSXChild::Element(elem) => {
            collect_helper_refs_from_jsx_element(elem, source, file, router_bindings, refs);
        }
        JSXChild::Fragment(frag) => {
            for child in &frag.children {
                collect_helper_refs_from_jsx_child(child, source, file, router_bindings, refs);
            }
        }
        JSXChild::ExpressionContainer(container) => {
            if let Some(expr) = container.expression.as_expression() {
                collect_helper_refs_from_expression(expr, source, file, router_bindings, refs);
            }
        }
        _ => {}
    }
}
