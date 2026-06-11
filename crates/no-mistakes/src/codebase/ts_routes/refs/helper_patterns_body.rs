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
                returns.extend(evaluate_helper_try_statement(
                    try_stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                ));
            }
            Statement::BlockStatement(block) => {
                for stmt in &block.body {
                    returns.extend(evaluate_helper_return_statement(
                        stmt,
                        defs,
                        imported_helpers,
                        env,
                        depth + 1,
                    ));
                }
                if !returns.is_empty() {
                    break;
                }
            }
            _ => {}
        }
    }
    returns
}

