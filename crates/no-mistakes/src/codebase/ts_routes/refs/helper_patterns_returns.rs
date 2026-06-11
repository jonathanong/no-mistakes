fn evaluate_helper_return_statement<'a>(
    stmt: &'a Statement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    match stmt {
        Statement::ReturnStatement(ret) => ret
            .argument
            .as_ref()
            .map(|expr| evaluate_route_expression(expr, defs, imported_helpers, env, depth + 1))
            .unwrap_or_default(),
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                let (Some(name), Some(init)) = (binding_identifier_name(&decl.id), &decl.init)
                else {
                    continue;
                };
                let value = evaluate_route_expression(init, defs, imported_helpers, env, depth + 1);
                env.insert(name.to_string(), value);
            }
            Vec::new()
        }
        Statement::ExpressionStatement(expr_stmt) => {
            if let Expression::AssignmentExpression(assignment) = &expr_stmt.expression {
                apply_helper_assignment_expression(assignment, defs, imported_helpers, env, depth + 1);
            }
            Vec::new()
        }
        Statement::BlockStatement(block) => {
            let mut values = Vec::new();
            for stmt in &block.body {
                values.extend(evaluate_helper_return_statement(
                    stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                ));
            }
            values
        }
        Statement::IfStatement(if_stmt) => {
            let mut values = evaluate_helper_return_statement(
                &if_stmt.consequent,
                defs,
                imported_helpers,
                env,
                depth + 1,
            );
            if let Some(alternate) = &if_stmt.alternate {
                values.extend(evaluate_helper_return_statement(
                    alternate,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                ));
            }
            values
        }
        Statement::SwitchStatement(switch_stmt) => {
            evaluate_helper_switch_statement(switch_stmt, defs, imported_helpers, env, depth + 1)
        }
        _ => Vec::new(),
    }
}
