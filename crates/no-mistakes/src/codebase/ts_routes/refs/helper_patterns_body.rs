fn evaluate_helper_body<'a>(
    body: &'a oxc_ast::ast::FunctionBody<'a>,
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
            Statement::IfStatement(_) => {
                let evaluation = evaluate_helper_return_statement(
                    stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                );
                returns.extend(evaluation.values);
                if !evaluation.can_continue {
                    break;
                }
            }
            Statement::SwitchStatement(switch_stmt) => {
                let evaluation = evaluate_helper_switch_statement(
                    switch_stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                );
                returns.extend(evaluation.values);
                if !evaluation.can_continue {
                    break;
                }
            }
            Statement::TryStatement(try_stmt) => {
                let evaluation = evaluate_helper_try_statement(
                    try_stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                );
                returns.extend(evaluation.values);
                if !evaluation.can_continue {
                    break;
                }
            }
            Statement::BlockStatement(_) => {
                let evaluation = evaluate_helper_return_statement(
                    stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                );
                returns.extend(evaluation.values);
                if !evaluation.can_continue {
                    break;
                }
            }
            _ => {}
        }
    }
    returns
}
