fn evaluate_helper_try_statement<'a>(
    try_stmt: &'a oxc::ast::ast::TryStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    let base_env = env.clone();
    let mut values = Vec::new();
    let mut block_env = base_env.clone();
    values.extend(evaluate_helper_block_returns(
        &try_stmt.block,
        defs,
        imported_helpers,
        &mut block_env,
        depth + 1,
    ));
    let mut branch_envs = vec![block_env];
    if let Some(handler) = &try_stmt.handler {
        let mut handler_env = base_env.clone();
        values.extend(evaluate_helper_block_returns(
            &handler.body,
            defs,
            imported_helpers,
            &mut handler_env,
            depth + 1,
        ));
        branch_envs.push(handler_env);
    }
    replace_helper_env_with_branches(env, branch_envs);
    if let Some(finalizer) = &try_stmt.finalizer {
        let finalizer_values = evaluate_helper_block_returns(
            finalizer,
            defs,
            imported_helpers,
            env,
            depth + 1,
        );
        if !finalizer_values.is_empty() {
            return finalizer_values;
        }
    }
    values
}

fn evaluate_helper_block_returns<'a>(
    block: &'a oxc::ast::ast::BlockStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
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
