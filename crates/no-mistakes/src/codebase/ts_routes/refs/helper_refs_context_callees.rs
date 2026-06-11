#[derive(Clone)]
struct RouteHelperRefCandidate {
    callee: String,
    wrapper_pattern: String,
}

#[derive(Default)]
struct RouteHelperContextSummary {
    patterns: Vec<String>,
    refs: Vec<RouteHelperRefCandidate>,
}

fn collect_route_helper_callee_names(
    expr: &Expression,
    helper_bindings: &RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRefCandidate>,
) {
    refs.extend(route_helper_context_summary(expr, helper_bindings, local_helpers).refs);
}

fn route_helper_context_summary(
    expr: &Expression,
    helper_bindings: &RouteHelperBindings,
    local_helpers: &HashSet<String>,
) -> RouteHelperContextSummary {
    match expr {
        Expression::StringLiteral(s) => RouteHelperContextSummary {
            patterns: vec![normalize_next_pathname_pattern(s.value.as_str())],
            refs: Vec::new(),
        },
        Expression::CallExpression(call) => {
            route_helper_call_context_summary(call, helper_bindings, local_helpers)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc::ast::ast::ChainElement::CallExpression(call) => {
                route_helper_call_context_summary(call, helper_bindings, local_helpers)
            }
            other => match other
                .as_member_expression()
                .and_then(route_helper_callee_name_from_member)
                .and_then(|callee| bound_helper_callee_name(&callee, helper_bindings))
            {
                Some(callee) => helper_ref_summary(callee),
                None => dynamic_route_helper_context_summary(),
            },
        },
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = route_helper_context_summary(&binary.left, helper_bindings, local_helpers);
            let right = route_helper_context_summary(&binary.right, helper_bindings, local_helpers);
            concat_route_helper_context_summaries(left, right)
        }
        Expression::TemplateLiteral(tpl) => {
            route_helper_template_context_summary(tpl, helper_bindings, local_helpers)
        }
        Expression::ParenthesizedExpression(paren) => {
            route_helper_context_summary(&paren.expression, helper_bindings, local_helpers)
        }
        Expression::TSAsExpression(ts_as) => {
            route_helper_context_summary(&ts_as.expression, helper_bindings, local_helpers)
        }
        Expression::TSTypeAssertion(ts_assertion) => {
            route_helper_context_summary(&ts_assertion.expression, helper_bindings, local_helpers)
        }
        Expression::TSNonNullExpression(ts_nn) => {
            route_helper_context_summary(&ts_nn.expression, helper_bindings, local_helpers)
        }
        Expression::TSSatisfiesExpression(ts_sat) => {
            route_helper_context_summary(&ts_sat.expression, helper_bindings, local_helpers)
        }
        Expression::AwaitExpression(await_expr) => {
            route_helper_context_summary(&await_expr.argument, helper_bindings, local_helpers)
        }
        Expression::ConditionalExpression(cond) => {
            let consequent =
                route_helper_context_summary(&cond.consequent, helper_bindings, local_helpers);
            let alternate =
                route_helper_context_summary(&cond.alternate, helper_bindings, local_helpers);
            merge_route_helper_context_summaries(consequent, alternate)
        }
        Expression::LogicalExpression(logical) => {
            let left = route_helper_context_summary(&logical.left, helper_bindings, local_helpers);
            let right = route_helper_context_summary(&logical.right, helper_bindings, local_helpers);
            merge_route_helper_context_summaries(left, right)
        }
        Expression::ObjectExpression(obj) => {
            route_helper_object_context_summary(obj, helper_bindings, local_helpers)
        }
        _ => dynamic_route_helper_context_summary(),
    }
}

fn route_helper_call_context_summary(
    call: &oxc::ast::ast::CallExpression<'_>,
    helper_bindings: &RouteHelperBindings,
    local_helpers: &HashSet<String>,
) -> RouteHelperContextSummary {
    let mut argument_summary = RouteHelperContextSummary::default();
    for arg in &call.arguments {
        if let Some(expr) = arg.as_expression() {
            argument_summary = merge_route_helper_context_summaries(
                argument_summary,
                route_helper_context_summary(expr, helper_bindings, local_helpers),
            );
        }
    }
    if let Some(member) = call.callee.as_member_expression() {
        argument_summary = merge_route_helper_context_summaries(
            argument_summary,
            route_helper_context_summary(member.object(), helper_bindings, local_helpers),
        );
    }

    let callee = route_helper_callee_name_from_callee(&call.callee)
        .and_then(|callee| bound_helper_callee_name(&callee, helper_bindings));
    match callee {
        Some(callee)
            if local_helpers.contains(&callee)
                || route_helper_callee_name_looks_like_helper(&callee)
                || argument_summary.refs.is_empty() =>
        {
            helper_ref_summary(callee)
        }
        Some(_) | None => argument_summary,
    }
}

fn route_helper_callee_name_looks_like_helper(callee: &str) -> bool {
    let name = callee.rsplit_once('.').map_or(callee, |(_, name)| name);
    name.ends_with("Href")
        || name.ends_with("Path")
        || name.ends_with("Pathname")
        || name.ends_with("Url")
}

fn route_helper_template_context_summary(
    tpl: &TemplateLiteral<'_>,
    helper_bindings: &RouteHelperBindings,
    local_helpers: &HashSet<String>,
) -> RouteHelperContextSummary {
    let mut summary = RouteHelperContextSummary {
        patterns: vec![String::new()],
        refs: Vec::new(),
    };
    for (index, quasi) in tpl.quasis.iter().enumerate() {
        let cooked = quasi
            .value
            .cooked
            .map(|value| value.as_str())
            .unwrap_or("");
        summary = concat_route_helper_context_summaries(
            summary,
            RouteHelperContextSummary {
                patterns: vec![cooked.to_string()],
                refs: Vec::new(),
            },
        );
        if let Some(expr) = tpl.expressions.get(index) {
            summary = concat_route_helper_context_summaries(
                summary,
                route_helper_context_summary(expr, helper_bindings, local_helpers),
            );
        }
    }
    summary.patterns = summary
        .patterns
        .into_iter()
        .map(|pattern| normalize_next_pathname_pattern(&pattern))
        .collect();
    summary
}

fn route_helper_object_context_summary(
    obj: &oxc::ast::ast::ObjectExpression<'_>,
    helper_bindings: &RouteHelperBindings,
    local_helpers: &HashSet<String>,
) -> RouteHelperContextSummary {
    let mut summary = RouteHelperContextSummary::default();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if property_key_is_pathname(&prop.key) {
            summary = merge_route_helper_context_summaries(
                summary,
                route_helper_context_summary(&prop.value, helper_bindings, local_helpers),
            );
        }
    }
    summary
}
