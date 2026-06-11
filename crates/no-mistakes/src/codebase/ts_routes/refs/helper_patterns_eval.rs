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

fn evaluate_helper_body<'a>(
    body: &'a oxc::ast::ast::FunctionBody<'a>,
    expression_body: bool,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    let mut returns = Vec::new();
    for stmt in &body.statements {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    let (Some(name), Some(init)) = (binding_identifier_name(&decl.id), &decl.init)
                    else {
                        continue;
                    };
                    let value =
                        evaluate_route_expression(init, defs, imported_helpers, env, depth + 1);
                    env.insert(name.to_string(), value);
                }
            }
            Statement::ReturnStatement(ret) => {
                if let Some(expr) = &ret.argument {
                    returns.extend(evaluate_route_expression(
                        expr,
                        defs,
                        imported_helpers,
                        env,
                        depth + 1,
                    ));
                }
                break;
            }
            Statement::ExpressionStatement(expr_stmt) if expression_body => {
                returns.extend(evaluate_route_expression(
                    &expr_stmt.expression,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                ));
                break;
            }
            Statement::ExpressionStatement(expr_stmt) => {
                if let Expression::AssignmentExpression(assignment) = &expr_stmt.expression {
                    apply_helper_assignment_expression(
                        assignment,
                        defs,
                        imported_helpers,
                        env,
                        depth + 1,
                    );
                }
            }
            Statement::IfStatement(if_stmt) => {
                let base_env = env.clone();
                let mut consequent_env = base_env.clone();
                returns.extend(evaluate_helper_return_statement(
                    &if_stmt.consequent,
                    defs,
                    imported_helpers,
                    &mut consequent_env,
                    depth + 1,
                ));
                if let Some(alternate) = &if_stmt.alternate {
                    let mut alternate_env = base_env.clone();
                    returns.extend(evaluate_helper_return_statement(
                        alternate,
                        defs,
                        imported_helpers,
                        &mut alternate_env,
                        depth + 1,
                    ));
                    replace_helper_env_with_branches(env, vec![consequent_env, alternate_env]);
                } else {
                    merge_helper_env(env, consequent_env);
                }
            }
            Statement::SwitchStatement(switch_stmt) => {
                returns.extend(evaluate_helper_switch_statement(
                    switch_stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                ));
            }
            Statement::TryStatement(try_stmt) => {
                returns.extend(evaluate_helper_try_statement(try_stmt, defs, imported_helpers, env, depth + 1));
            }
            _ => {}
        }
    }
    returns
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
