fn evaluate_helper_try_statement<'a>(
    try_stmt: &'a oxc::ast::ast::TryStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> HelperStatementEvaluation {
    let base_env = env.clone();
    let mut values = Vec::new();
    let mut block_env = base_env.clone();
    let block_evaluation = evaluate_helper_block_returns(
        &try_stmt.block,
        defs,
        imported_helpers,
        &mut block_env,
        depth + 1,
    );
    values.extend(block_evaluation.values);
    let mut branch_envs = Vec::new();
    if block_evaluation.can_continue {
        branch_envs.push(block_env);
    }
    if let Some(handler) = &try_stmt.handler {
        let mut handler_env = base_env.clone();
        if let Some(param) = &handler.param {
            if let Some(name) = binding_identifier_name(&param.pattern) {
                handler_env.remove(name);
            }
        }
        let handler_evaluation = evaluate_helper_block_returns(
            &handler.body,
            defs,
            imported_helpers,
            &mut handler_env,
            depth + 1,
        );
        values.extend(handler_evaluation.values);
        if handler_evaluation.can_continue {
            branch_envs.push(handler_env);
        }
    }
    let mut can_continue = !branch_envs.is_empty();
    if can_continue {
        replace_helper_env_with_branches(env, branch_envs);
    }
    if let Some(finalizer) = &try_stmt.finalizer {
        let finalizer_evaluation = evaluate_helper_block_returns(
            finalizer,
            defs,
            imported_helpers,
            env,
            depth + 1,
        );
        if !finalizer_evaluation.values.is_empty() {
            return HelperStatementEvaluation::returning(finalizer_evaluation.values);
        }
        can_continue = can_continue && finalizer_evaluation.can_continue;
    }
    HelperStatementEvaluation {
        values,
        can_continue,
    }
}

fn evaluate_helper_block_returns<'a>(
    block: &'a oxc::ast::ast::BlockStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> HelperStatementEvaluation {
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
