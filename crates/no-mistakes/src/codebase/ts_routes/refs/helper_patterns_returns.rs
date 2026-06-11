#[derive(Default)]
struct HelperStatementEvaluation {
    values: Vec<String>,
    can_continue: bool,
}

impl HelperStatementEvaluation {
    fn continuing() -> Self {
        Self {
            values: Vec::new(),
            can_continue: true,
        }
    }

    fn returning(values: Vec<String>) -> Self {
        Self {
            values,
            can_continue: false,
        }
    }
}

fn evaluate_helper_return_statement<'a>(
    stmt: &'a Statement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> HelperStatementEvaluation {
    match stmt {
        Statement::ReturnStatement(ret) => HelperStatementEvaluation::returning(ret
            .argument
            .as_ref()
            .map(|expr| evaluate_route_expression(expr, defs, imported_helpers, env, depth + 1))
            .unwrap_or_default()),
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                let (Some(name), Some(init)) = (binding_identifier_name(&decl.id), &decl.init)
                else {
                    continue;
                };
                let value = evaluate_route_expression(init, defs, imported_helpers, env, depth + 1);
                env.insert(name.to_string(), value);
            }
            HelperStatementEvaluation::continuing()
        }
        Statement::ExpressionStatement(expr_stmt) => {
            if let Expression::AssignmentExpression(assignment) = &expr_stmt.expression {
                apply_helper_assignment_expression(assignment, defs, imported_helpers, env, depth + 1);
            }
            HelperStatementEvaluation::continuing()
        }
        Statement::BlockStatement(block) => {
            let mut values = Vec::new();
            for stmt in &block.body {
                let evaluation = evaluate_helper_return_statement(
                    stmt,
                    defs,
                    imported_helpers,
                    env,
                    depth + 1,
                );
                values.extend(evaluation.values);
                if !evaluation.can_continue {
                    return HelperStatementEvaluation::returning(values);
                }
            }
            HelperStatementEvaluation {
                values,
                can_continue: true,
            }
        }
        Statement::IfStatement(if_stmt) => {
            let base_env = env.clone();
            let mut consequent_env = base_env.clone();
            let consequent = evaluate_helper_return_statement(
                &if_stmt.consequent,
                defs,
                imported_helpers,
                &mut consequent_env,
                depth + 1,
            );
            let mut values = consequent.values;
            if let Some(alternate) = &if_stmt.alternate {
                let mut alternate_env = base_env;
                let alternate = evaluate_helper_return_statement(
                    alternate,
                    defs,
                    imported_helpers,
                    &mut alternate_env,
                    depth + 1,
                );
                values.extend(alternate.values);
                let mut branch_envs = Vec::new();
                if consequent.can_continue {
                    branch_envs.push(consequent_env);
                }
                if alternate.can_continue {
                    branch_envs.push(alternate_env);
                }
                if !branch_envs.is_empty() {
                    replace_helper_env_with_branches(env, branch_envs);
                }
                return HelperStatementEvaluation {
                    values,
                    can_continue: consequent.can_continue || alternate.can_continue,
                };
            }
            if consequent.can_continue {
                merge_helper_env(env, consequent_env);
            }
            HelperStatementEvaluation {
                values,
                can_continue: true,
            }
        }
        Statement::SwitchStatement(switch_stmt) => {
            evaluate_helper_switch_statement(switch_stmt, defs, imported_helpers, env, depth + 1)
        }
        Statement::TryStatement(try_stmt) => {
            evaluate_helper_try_statement(try_stmt, defs, imported_helpers, env, depth + 1)
        }
        _ => HelperStatementEvaluation::continuing(),
    }
}
