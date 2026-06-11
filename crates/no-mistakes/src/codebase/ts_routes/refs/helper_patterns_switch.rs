fn evaluate_helper_switch_statement<'a>(
    switch_stmt: &'a oxc::ast::ast::SwitchStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    let mut values = Vec::new();
    let base_env = env.clone();
    let mut case_envs = Vec::new();
    let mut fallthrough_envs = Vec::new();
    let has_default = switch_stmt.cases.iter().any(|case| case.test.is_none());
    for case in &switch_stmt.cases {
        let mut entry_envs = vec![base_env.clone()];
        entry_envs.append(&mut fallthrough_envs);
        let mut next_fallthrough_envs = Vec::new();
        for mut case_env in entry_envs {
            let mut stops_fallthrough = false;
            for stmt in &case.consequent {
                values.extend(evaluate_helper_return_statement(
                    stmt,
                    defs,
                    imported_helpers,
                    &mut case_env,
                    depth + 1,
                ));
                if helper_statement_stops_switch_fallthrough(stmt) {
                    stops_fallthrough = true;
                    break;
                }
            }
            case_envs.push(case_env.clone());
            if !stops_fallthrough {
                next_fallthrough_envs.push(case_env);
            }
        }
        fallthrough_envs = next_fallthrough_envs;
    }
    if !case_envs.is_empty() {
        if has_default {
            replace_helper_env_with_branches(env, case_envs);
        } else {
            let mut merged_env = base_env;
            for case_env in case_envs {
                merge_helper_env(&mut merged_env, case_env);
            }
            *env = merged_env;
        }
    }
    values
}

fn helper_statement_stops_switch_fallthrough(stmt: &Statement<'_>) -> bool {
    matches!(stmt, Statement::BreakStatement(_) | Statement::ReturnStatement(_))
}
