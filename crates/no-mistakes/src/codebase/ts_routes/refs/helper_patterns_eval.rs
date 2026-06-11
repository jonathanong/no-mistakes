fn evaluate_helper_def<'a>(
    def: &HelperDef<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    provided: &HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    if depth > 4 {
        return Vec::new();
    }
    let mut env = helper_param_env(def.params, defs, imported_helpers, provided, depth);
    let patterns = evaluate_helper_body(
        def.body,
        def.expression_body,
        defs,
        imported_helpers,
        &mut env,
        depth,
    );
    normalize_helper_patterns(patterns)
}

fn evaluate_route_expression<'a>(
    expr: &'a Expression<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    if depth > 8 {
        return vec!["*".to_string()];
    }
    match expr {
        Expression::StringLiteral(s) => vec![normalize_next_pathname_pattern(s.value.as_str())],
        Expression::TemplateLiteral(tpl) => {
            evaluate_template_literal(tpl, defs, imported_helpers, env, depth)
        }
        Expression::Identifier(id) => env
            .get(id.name.as_str())
            .cloned()
            .unwrap_or_else(|| vec!["*".to_string()]),
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left =
                evaluate_route_expression(&binary.left, defs, imported_helpers, env, depth + 1);
            let right =
                evaluate_route_expression(&binary.right, defs, imported_helpers, env, depth + 1);
            concat_candidates(&left, &right)
        }
        Expression::LogicalExpression(logical) => {
            let mut values =
                evaluate_route_expression(&logical.left, defs, imported_helpers, env, depth + 1);
            values.extend(evaluate_route_expression(
                &logical.right,
                defs,
                imported_helpers,
                env,
                depth + 1,
            ));
            dedupe_candidates(values)
        }
        Expression::ConditionalExpression(cond) => {
            let mut values =
                evaluate_route_expression(&cond.consequent, defs, imported_helpers, env, depth + 1);
            values.extend(evaluate_route_expression(
                &cond.alternate,
                defs,
                imported_helpers,
                env,
                depth + 1,
            ));
            dedupe_candidates(values)
        }
        Expression::ObjectExpression(obj) => {
            evaluate_url_object_expression(obj, defs, imported_helpers, env, depth + 1)
        }
        Expression::CallExpression(call) => {
            evaluate_helper_call(call, defs, imported_helpers, env, depth + 1)
        }
        Expression::ParenthesizedExpression(paren) => {
            evaluate_route_expression(&paren.expression, defs, imported_helpers, env, depth + 1)
        }
        Expression::TSAsExpression(ts_as) => {
            evaluate_route_expression(&ts_as.expression, defs, imported_helpers, env, depth + 1)
        }
        Expression::TSTypeAssertion(ts_assertion) => {
            evaluate_route_expression(
                &ts_assertion.expression,
                defs,
                imported_helpers,
                env,
                depth + 1,
            )
        }
        Expression::TSNonNullExpression(ts_nn) => {
            evaluate_route_expression(&ts_nn.expression, defs, imported_helpers, env, depth + 1)
        }
        Expression::TSSatisfiesExpression(ts_sat) => {
            evaluate_route_expression(&ts_sat.expression, defs, imported_helpers, env, depth + 1)
        }
        _ => vec!["*".to_string()],
    }
}
